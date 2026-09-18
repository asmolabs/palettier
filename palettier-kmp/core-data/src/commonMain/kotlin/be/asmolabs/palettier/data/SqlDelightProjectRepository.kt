package be.asmolabs.palettier.data

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.project.AppliedCoat
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.project.ProjectZone
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import kotlin.time.Instant
import be.asmolabs.palettier.db.Project as ProjectRow
import be.asmolabs.palettier.db.Project_layer as LayerRow
import be.asmolabs.palettier.db.Project_zone as ZoneRow

/**
 * Les projets : leurs zones, leurs couches, la pose de chacune.
 *
 * <p>L'arbre entier est relu en quatre requetes -- projets, tubes figes, zones, couches --
 * puis assemble en memoire. Une requete par zone et par couche ferait des centaines
 * d'allers-retours pour afficher une liste de pieces.</p>
 *
 * <p>Les photos n'en font jamais partie : ce sont des images entieres, et seul l'ecran
 * qui les montre les demande.</p>
 */
class SqlDelightProjectRepository(
    private val database: PalettierDatabase,
    private val catalog: PaintCatalogRepository,
    private val palettes: PaletteRepository,
    private val io: CoroutineDispatcher,
) : ProjectRepository {

    private val queries get() = database.projectQueries

    override fun observeAll(): Flow<List<Project>> =
        queries.selectAll().asFlow().mapToList(io).map { rows -> assemble(rows, emptyMap()) }

    override suspend fun all(): List<Project> = withContext(io) {
        assemble(queries.selectAll().executeAsList(), emptyMap())
    }

    override suspend fun findWithPhotos(id: Long): Project? = withContext(io) {
        val row = queries.selectById(id).executeAsOneOrNull() ?: return@withContext null
        val photos = queries.photosOf(id).executeAsList().map {
            ProjectPhoto(
                id = it.id,
                data = it.data_,
                role = ProjectPhoto.Role.valueOf(it.role),
                caption = it.caption,
                addedAt = Instant.fromEpochMilliseconds(it.added_at),
            )
        }
        assemble(listOf(row), mapOf(id to photos)).firstOrNull()
    }

    /**
     * Enregistre le projet, zones et couches comprises.
     *
     * <p>Les zones sont effacees puis reecrites : un arbre se remplace en bloc, et
     * chercher a reconcilier des rangs qui ont bouge coute plus cher que de tout
     * reposer. Les photos, elles, ne sont pas touchees -- un projet lu sans elles et
     * reenregistre les effacerait, et c'est exactement le defaut qu'il a fallu corriger
     * cote JPA.</p>
     */
    override suspend fun save(project: Project): Project = withContext(io) {
        database.transactionWithResult {
            val id = project.id?.also {
                queries.updateProject(project.name, project.subject, project.approach,
                    project.notes, project.palette?.id, it)
            } ?: run {
                queries.insertProject(uniqueName(project.name), project.subject, project.approach,
                    project.notes, project.palette?.id, project.createdAt.toEpochMilliseconds())
                queries.lastInsertedId().executeAsOne()
            }

            queries.clearProjectPaints(id)
            project.paints.forEachIndexed { position, paint ->
                paint.id?.let { queries.addProjectPaint(id, it, position.toLong()) }
            }

            queries.clearZones(id)
            project.zones.forEachIndexed { zonePosition, zone ->
                queries.insertZone(id, zone.name, zone.material, zone.note, zonePosition.toLong())
                val zoneId = queries.lastInsertedId().executeAsOne()
                zone.layers.forEachIndexed { layerPosition, layer ->
                    queries.insertLayer(
                        zone_id = zoneId,
                        role = layer.role,
                        target_hex = layer.targetHex,
                        technique = layer.technique,
                        note = layer.note,
                        kind = layer.kind.name,
                        position = layerPosition.toLong(),
                        applied_at = layer.applied?.at?.toEpochMilliseconds(),
                        applied_temperature = layer.applied?.workshop?.temperatureCelsius,
                        applied_humidity = layer.applied?.workshop?.relativeHumidity,
                        applied_ventilation = layer.applied?.workshop?.ventilation?.name,
                        applied_drying_class = layer.applied?.dryingClass?.name,
                    )
                }
            }
            project.copy(id = id)
        }
    }

    override suspend fun addPhoto(project: Project, photo: ProjectPhoto): Project = withContext(io) {
        val id = requireNotNull(project.id) { "Projet sans identifiant" }
        database.transaction {
            val position = queries.photosOf(id).executeAsList().size.toLong()
            queries.insertPhoto(id, photo.data, photo.role.name, photo.caption,
                photo.addedAt.toEpochMilliseconds(), position)
        }
        findWithPhotos(id) ?: project
    }

    override suspend fun removePhoto(project: Project, photoId: Long): Project = withContext(io) {
        queries.deletePhoto(photoId)
        findWithPhotos(requireNotNull(project.id) { "Projet sans identifiant" }) ?: project
    }

    override suspend fun delete(project: Project): Unit = withContext(io) {
        project.id?.let { queries.deleteProject(it) }
    }

    /**
     * Un nom deja pris recoit un suffixe. Deux pieces peuvent se ressembler, leurs plans
     * doivent rester distinguables.
     */
    private fun uniqueName(wanted: String): String {
        val base = wanted.trim().ifBlank { "Projet" }
        var candidate = base
        var suffix = 2
        while (queries.existsByName(candidate).executeAsOne() > 0) {
            candidate = "$base ${suffix++}"
        }
        return candidate
    }

    private suspend fun assemble(rows: List<ProjectRow>, photos: Map<Long, List<ProjectPhoto>>): List<Project> {
        if (rows.isEmpty()) return emptyList()

        val paintsById = catalog.all().associateBy { it.id }
        val palettesById = palettes.all().associateBy { it.id }
        val frozen = queries.allProjectPaints().executeAsList()
            .groupBy({ it.project_id }, { it.paint_id })
        val zonesByProject = queries.allZones().executeAsList().groupBy { it.project_id }
        val layersByZone = queries.allLayers().executeAsList().groupBy { it.zone_id }

        return rows.map { row ->
            Project(
                id = row.id,
                name = row.name,
                subject = row.subject,
                approach = row.approach,
                notes = row.notes,
                palette = row.palette_id?.let { palettesById[it] },
                paints = frozen[row.id].orEmpty().mapNotNull { paintsById[it] },
                zones = zonesByProject[row.id].orEmpty().map { it.toDomain(layersByZone) },
                photos = photos[row.id].orEmpty(),
                createdAt = Instant.fromEpochMilliseconds(row.created_at),
            )
        }
    }

    private fun ZoneRow.toDomain(layersByZone: Map<Long, List<LayerRow>>) = ProjectZone(
        id = id,
        name = name,
        material = material,
        note = note,
        layers = layersByZone[id].orEmpty().map { it.toDomain() },
    )

    private fun LayerRow.toDomain() = ProjectLayer(
        id = id,
        role = role,
        targetHex = target_hex,
        technique = technique,
        note = note,
        kind = ProjectLayer.Kind.valueOf(kind),
        // Les cinq colonnes de pose vont ensemble : sans date, il n'y a pas de pose.
        applied = applied_at?.let {
            AppliedCoat(
                at = Instant.fromEpochMilliseconds(it),
                workshop = Workshop(
                    applied_temperature ?: Workshop.standard().temperatureCelsius,
                    applied_humidity ?: Workshop.standard().relativeHumidity,
                    applied_ventilation?.let(Ventilation::valueOf) ?: Workshop.standard().ventilation,
                ),
                dryingClass = applied_drying_class?.let(DryingClass::valueOf) ?: DryingClass.MEDIUM,
            )
        },
    )
}
