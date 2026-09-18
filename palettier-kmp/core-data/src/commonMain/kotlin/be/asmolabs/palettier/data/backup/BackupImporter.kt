package be.asmolabs.palettier.data.backup

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.project.AppliedCoat
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.project.ProjectZone
import be.asmolabs.palettier.domain.recipe.Recipe
import be.asmolabs.palettier.domain.recipe.RecipeStep
import kotlinx.serialization.json.Json
import kotlin.time.Clock
import kotlin.time.Instant

/**
 * Une archive de sauvegarde, ouverte.
 *
 * <p>Une interface plutot qu'un expect/actual : lire un ZIP n'a rien de commun d'une
 * plateforme a l'autre, mais le besoin, lui, tient en deux fonctions.</p>
 */
interface BackupArchive {
    /** Le document JSON principal. */
    fun document(): ByteArray

    /** Un fichier range a cote, typiquement une image. Null s'il manque. */
    fun file(path: String): ByteArray?
}

/** Ce qu'une restauration a fait, dans le detail. */
data class ImportReport(
    val paintsUpdated: Int = 0,
    val paintsCreated: Int = 0,
    val palettesAdded: Int = 0,
    val palettesSkipped: Int = 0,
    val projectsAdded: Int = 0,
    val projectsSkipped: Int = 0,
    val recipesAdded: Int = 0,
    val recipesSkipped: Int = 0,
    val photos: Int = 0,
    val warnings: List<String> = emptyList(),
) {
    val skipped: Int get() = palettesSkipped + projectsSkipped + recipesSkipped
}

/**
 * Restaure une sauvegarde de l'application Java dans la base SQLite.
 *
 * <p>Une restauration n'ecrase jamais un travail en cours : une palette, un projet ou une
 * recette dont le nom existe deja est laisse de cote, et compte dans le rapport. Le
 * catalogue, lui, se met a jour -- ce sont des corrections du peintre sur des fiches
 * livrees, pas un travail qu'on risquerait de perdre.</p>
 */
class BackupImporter(
    private val catalog: PaintCatalogRepository,
    private val palettes: PaletteRepository,
    private val projects: ProjectRepository,
    private val recipes: RecipeRepository,
) {

    private val json = Json { ignoreUnknownKeys = true }

    suspend fun import(archive: BackupArchive): ImportReport {
        val document = json.decodeFromString<BackupDocument>(archive.document().decodeToString())
        require(document.formatVersion <= SUPPORTED_FORMAT) {
            "Sauvegarde ecrite par une version plus recente (format ${document.formatVersion}, " +
                "connu jusqu'a $SUPPORTED_FORMAT)."
        }

        val warnings = mutableListOf<String>()
        val paints = restorePaints(document.paints)
        val paletteCounts = restorePalettes(document.palettes, warnings)
        val recipeCounts = restoreRecipes(document.recipes)
        val projectCounts = restoreProjects(document.projects, archive, warnings)

        return ImportReport(
            paintsUpdated = paints.first,
            paintsCreated = paints.second,
            palettesAdded = paletteCounts.first,
            palettesSkipped = paletteCounts.second,
            projectsAdded = projectCounts.first,
            projectsSkipped = projectCounts.second,
            recipesAdded = recipeCounts.first,
            recipesSkipped = recipeCounts.second,
            photos = projectCounts.third,
            warnings = warnings.toList(),
        )
    }

    /**
     * Le catalogue se met a jour plutot que de se dupliquer.
     *
     * <p>La cle naturelle -- marque et nom -- est ce qui le permet : un identifiant de
     * base ne veut rien dire d'une installation a l'autre. Un tube deja connu recoit les
     * corrections du peintre, un tube inconnu est cree.</p>
     */
    private suspend fun restorePaints(entries: List<BackupPaint>): Pair<Int, Int> {
        var updated = 0
        var created = 0
        for (entry in entries) {
            val existing = catalog.findByNaturalKey(entry.brand, entry.name)
            if (existing == null) {
                catalog.save(entry.toDomain())
                created++
            } else {
                catalog.setOwned(existing, entry.owned)
                entry.tintHex?.let { catalog.recordTint(existing, Rgb.ofHex(it)) }
                updated++
            }
        }
        return updated to created
    }

    private suspend fun restorePalettes(entries: List<BackupPalette>, warnings: MutableList<String>): Pair<Int, Int> {
        val known = palettes.all().associateBy { it.name }
        var added = 0
        var skipped = 0
        for (entry in entries) {
            if (known.containsKey(entry.name)) {
                skipped++
                continue
            }
            palettes.save(
                Palette(
                    name = entry.name,
                    purpose = entry.purpose,
                    notes = entry.notes,
                    paints = resolve(entry.paints, "palette ${entry.name}", warnings),
                )
            )
            added++
        }
        return added to skipped
    }

    private suspend fun restoreRecipes(entries: List<BackupRecipe>): Pair<Int, Int> {
        var added = 0
        var skipped = 0
        for (entry in entries) {
            if (recipes.exists(entry.name)) {
                skipped++
                continue
            }
            recipes.save(
                Recipe(
                    name = entry.name,
                    subject = entry.subject,
                    notes = entry.notes,
                    steps = entry.steps.map {
                        RecipeStep(
                            technique = Technique.valueOf(it.technique),
                            paintMix = it.paintMix,
                            medium = Medium.valueOf(it.medium),
                            mediumRatio = it.mediumRatio,
                            thickness = LayerThickness.valueOf(it.thickness),
                            notes = it.note,
                        )
                    },
                )
            )
            added++
        }
        return added to skipped
    }

    private suspend fun restoreProjects(
        entries: List<BackupProject>,
        archive: BackupArchive,
        warnings: MutableList<String>,
    ): Triple<Int, Int, Int> {
        val existing = projects.all().map { it.name }.toSet()
        val palettesByName = palettes.all().associateBy { it.name }
        var added = 0
        var skipped = 0
        var photos = 0

        for (entry in entries) {
            if (entry.name in existing) {
                skipped++
                continue
            }
            var project = projects.save(
                Project(
                    name = entry.name,
                    subject = entry.subject,
                    approach = entry.approach,
                    notes = entry.notes,
                    palette = entry.paletteName?.let { palettesByName[it] },
                    paints = resolve(entry.paints, "projet ${entry.name}", warnings),
                    zones = entry.zones.map { zone ->
                        ProjectZone(
                            name = zone.name,
                            material = zone.material,
                            note = zone.note,
                            layers = zone.layers.map(::toLayer),
                        )
                    },
                    createdAt = entry.createdAt?.let(Instant::parse) ?: Clock.System.now(),
                )
            )

            for (photo in entry.photos) {
                val data = archive.file(photo.file)
                if (data == null) {
                    warnings += "Image annoncee mais absente de l'archive : ${photo.file}"
                    continue
                }
                project = projects.addPhoto(
                    project,
                    ProjectPhoto(
                        data = data,
                        role = ProjectPhoto.Role.valueOf(photo.role),
                        caption = photo.caption,
                        addedAt = photo.addedAt?.let(Instant::parse) ?: Clock.System.now(),
                    ),
                )
                photos++
            }
            added++
        }
        return Triple(added, skipped, photos)
    }

    /**
     * Rend une couche, avancement compris.
     *
     * <p>Ne pas reporter la pose ferait perdre a la restauration ce qu'une sauvegarde a
     * de plus precieux sur une piece en cours : ou elle en est.</p>
     */
    private fun toLayer(layer: BackupLayer) = ProjectLayer(
        role = layer.role,
        targetHex = layer.targetHex,
        technique = layer.technique,
        note = layer.note,
        kind = ProjectLayer.Kind.valueOf(layer.kind),
        applied = layer.appliedAt?.let {
            AppliedCoat.of(
                at = Instant.parse(it),
                temperature = layer.appliedTemperature,
                humidity = layer.appliedHumidity,
                ventilation = layer.appliedVentilation?.let(Ventilation::valueOf),
                drying = layer.appliedDryingClass?.let(DryingClass::valueOf),
            )
        },
    )

    private suspend fun resolve(
        refs: List<BackupPaintRef>,
        context: String,
        warnings: MutableList<String>,
    ): List<Paint> = refs.mapNotNull { ref ->
        catalog.findByNaturalKey(ref.brand, ref.name).also {
            if (it == null) warnings += "Tube introuvable pour $context : ${ref.brand} - ${ref.name}"
        }
    }

    private fun BackupPaint.toDomain() = Paint(
        brand = brand,
        name = name,
        code = code,
        pigments = pigments.toSet(),
        hexColor = hex,
        tintHex = tintHex,
        opacity = Opacity.valueOf(opacity),
        dryingClass = DryingClass.valueOf(dryingClass),
        tintingStrength = tintingStrength,
        colorDerived = colorDerived,
        inStock = owned,
        notes = notes,
    )

    companion object {
        /** Version de format lisible : au-dela, l'archive vient d'une version plus recente. */
        const val SUPPORTED_FORMAT = 1
    }
}
