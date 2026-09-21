package be.asmolabs.palettier.data.backup

import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.project.Project
import kotlinx.serialization.json.Json
import kotlin.time.Clock

/**
 * Ce qu'une archive doit contenir, avant de savoir comment l'ecrire.
 *
 * <p>Le ZIP appartient a la plateforme ; la composition de l'archive, non. La separation
 * permet d'ecrire le contenu une fois et de ne refaire que l'emballage.</p>
 */
interface BackupWriter {
    /** Le document principal. */
    suspend fun writeDocument(json: String)

    /** Une image rangee a cote, sous le chemin annonce par le document. */
    suspend fun writeFile(path: String, bytes: ByteArray)
}

/** Ce qu'une sauvegarde a emporte. */
data class ExportReport(
    val paints: Int,
    val palettes: Int,
    val projects: Int,
    val recipes: Int,
    val photos: Int,
)

/**
 * Ecrit une sauvegarde au format que l'application Java sait relire.
 *
 * <p>Sans cet export, la migration serait un aller simple : les donnees entreraient dans
 * la nouvelle application sans pouvoir en ressortir. Le format n'est pas repris par
 * commodite mais par prudence -- tant que les deux applications parlent la meme langue,
 * revenir en arriere reste possible.</p>
 *
 * <p>Les tubes sont designes par leur marque et leur nom, jamais par un identifiant : un
 * numero de ligne ne veut rien dire dans une autre installation.</p>
 */
class BackupExporter(
    private val catalog: PaintCatalogRepository,
    private val palettes: PaletteRepository,
    private val projects: ProjectRepository,
    private val recipes: RecipeRepository,
) {

    private val json = Json { prettyPrint = true; encodeDefaults = true }

    suspend fun export(writer: BackupWriter): ExportReport {
        val paints = catalog.all()
        val allPalettes = palettes.all()
        val allRecipes = recipes.all()

        // Les photos ne sont pas dans la liste : il faut les demander projet par projet.
        val pieces = projects.all().mapNotNull { it.id?.let { id -> projects.findWithPhotos(id) } }

        var photos = 0
        val documents = pieces.map { project ->
            val entries = project.photos.mapIndexed { index, photo ->
                val path = "$IMAGES/${slug(project.name)}-${(index + 1).toString().padStart(2, '0')}.jpg"
                writer.writeFile(path, photo.data)
                photos++
                BackupPhoto(path, photo.role.name, photo.caption, photo.addedAt.toString())
            }
            describe(project, entries)
        }

        val document = BackupDocument(
            formatVersion = BackupImporter.SUPPORTED_FORMAT,
            exportedAt = Clock.System.now().toString(),
            application = APPLICATION,
            paints = paints.map {
                BackupPaint(
                    brand = it.brand, name = it.name, code = it.code,
                    pigments = it.pigments.toList(), hex = it.hexColor, tintHex = it.tintHex,
                    opacity = it.opacity.name, dryingClass = it.dryingClass.name,
                    tintingStrength = it.tintingStrength, owned = it.inStock,
                    colorDerived = it.colorDerived, notes = it.notes,
                )
            },
            palettes = allPalettes.map { palette ->
                BackupPalette(
                    palette.name, palette.purpose, palette.notes,
                    palette.paints.map { BackupPaintRef(it.brand, it.name) },
                )
            },
            projects = documents,
            recipes = allRecipes.map { recipe ->
                BackupRecipe(
                    recipe.name, recipe.subject, recipe.notes,
                    recipe.steps.map {
                        BackupStep(
                            it.technique.name, it.paintMix, it.medium.name,
                            it.mediumRatio, it.thickness.name, it.notes,
                        )
                    },
                )
            },
        )

        writer.writeDocument(json.encodeToString(document))
        return ExportReport(paints.size, allPalettes.size, pieces.size, allRecipes.size, photos)
    }

    private fun describe(project: Project, photos: List<BackupPhoto>) = BackupProject(
        name = project.name,
        subject = project.subject,
        approach = project.approach,
        notes = project.notes,
        paletteName = project.palette?.name,
        createdAt = project.createdAt.toString(),
        paints = project.paints.map { BackupPaintRef(it.brand, it.name) },
        zones = project.zones.map { zone ->
            BackupZone(
                zone.name, zone.material, zone.note,
                zone.layers.map { layer ->
                    BackupLayer(
                        role = layer.role,
                        targetHex = layer.targetHex,
                        technique = layer.technique,
                        note = layer.note,
                        kind = layer.kind.name,
                        appliedAt = layer.applied?.at?.toString(),
                        appliedTemperature = layer.applied?.workshop?.temperatureCelsius,
                        appliedHumidity = layer.applied?.workshop?.relativeHumidity,
                        appliedVentilation = layer.applied?.workshop?.ventilation?.name,
                        appliedDryingClass = layer.applied?.dryingClass?.name,
                    )
                },
            )
        },
        photos = photos,
    )

    /** Un nom de fichier lisible : on doit pouvoir ranger l'archive a la main. */
    private fun slug(name: String): String =
        name.lowercase()
            .map { if (it in 'a'..'z' || it in '0'..'9') it else '-' }
            .joinToString("")
            .trim('-')
            .replace(Regex("-+"), "-")
            .ifBlank { "projet" }

    companion object {
        const val DOCUMENT = "palettier.json"
        const val IMAGES = "images"
        private const val APPLICATION = "Palettier"
    }
}
