package be.asmolabs.palettier.ui.projects

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.plan.PaintingPlan
import be.asmolabs.palettier.domain.plan.PlanDryingService
import be.asmolabs.palettier.domain.plan.ProjectPlanner
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.image.ImagePalette
import be.asmolabs.palettier.domain.project.ProgressCheckService
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.plan.uniqueName
import be.asmolabs.palettier.domain.project.SubstituteService
import be.asmolabs.palettier.image.ImageDecoder
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlin.time.Clock

/**
 * Les pieces en cours et leur plan.
 *
 * <p>Deux rythmes cohabitent. La liste des projets vient du flux de la base et se met a
 * jour toute seule. Le plan de la piece choisie, lui, se recalcule a la demande : il
 * parcourt la palette pour chaque couche, et ce n'est pas quelque chose qu'on refait a
 * chaque respiration de la base.</p>
 *
 * <p>Les couches sont designees par leur rang dans le projet, jamais par leur rang
 * d'affichage. Cote JavaFX les deux differaient pour les variations locales, et cliquer
 * "Modifier" sur l'une d'elles levait une erreur. Ici l'ecran montre les couches telles
 * qu'elles sont rangees, et retrouve leur melange par le role.</p>
 */
class ProjectsViewModel(
    private val projects: ProjectRepository,
    private val catalog: PaintCatalogRepository,
    private val planner: ProjectPlanner,
    private val fatOverLean: FatOverLeanService,
    private val substitutes: SubstituteService,
    private val decoder: ImageDecoder,
    private val progress: ProgressCheckService,
    private val clock: Clock = Clock.System,
    private val workshop: () -> Workshop = { Workshop.standard() },
) : ViewModel() {

    private val selectedId = MutableStateFlow<Long?>(null)
    private val detail = MutableStateFlow(Detail())

    private data class Detail(
        val computing: Boolean = false,
        val zones: List<ZoneRow> = emptyList(),
        val risks: List<FatOverLeanService.Risk> = emptyList(),
        val missing: List<SubstituteService.Missing> = emptyList(),
        val photos: List<ProjectPhoto> = emptyList(),
        val comparison: List<ProgressCheckService.Observed> = emptyList(),
        val comparing: Boolean = false,
        val busy: Boolean = false,
        val message: String? = null,
    )

    val state: StateFlow<ProjectsUiState> =
        combine(projects.observeAll(), selectedId, detail) { pieces, id, detail ->
            val selected = pieces.firstOrNull { it.id == id } ?: pieces.firstOrNull()
            ProjectsUiState(
                loading = false,
                projects = pieces,
                selected = selected,
                zones = detail.zones,
                computing = detail.computing,
                risks = detail.risks,
                missing = detail.missing,
                photos = detail.photos,
                comparison = detail.comparison,
                comparing = detail.comparing,
                paletteDiverges = selected?.divergesFromPalette() == true,
                busy = detail.busy,
                message = detail.message,
            )
        }
            .catch { emit(ProjectsUiState(loading = false, error = it.message ?: "Lecture impossible")) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), ProjectsUiState())

    fun onIntent(intent: ProjectsIntent) {
        when (intent) {
            is ProjectsIntent.Select -> {
                selectedId.value = intent.project.id
                refreshDetail(intent.project)
            }

            is ProjectsIntent.ToggleApplied -> viewModelScope.launch {
                val project = state.value.selected ?: return@launch
                toggle(project, intent.zoneIndex, intent.layerIndex)
            }

            is ProjectsIntent.Rename -> viewModelScope.launch {
                val project = state.value.selected ?: return@launch
                val wanted = intent.name.trim()
                if (wanted.isEmpty()) return@launch
                // Les autres projets, pas celui-ci : se renommer en soi-meme ne doit pas
                // faire apparaitre un " 2" au bout du nom.
                val taken = projects.all().filterNot { it.id == project.id }.map { it.name }
                refreshDetail(projects.save(project.copy(name = uniqueName(wanted, taken))))
            }

            ProjectsIntent.Delete -> viewModelScope.launch {
                val project = state.value.selected ?: return@launch
                projects.delete(project)
                selectedId.value = null
                detail.value = Detail(message = "Projet \"${project.name}\" supprime.")
            }

            ProjectsIntent.SyncPalette -> viewModelScope.launch {
                val project = state.value.selected ?: return@launch
                val palette = project.palette ?: return@launch
                // Les couleurs visees ne bougent pas : seuls les tubes avec lesquels on
                // calcule les melanges sont repris.
                refreshDetail(projects.save(project.copy(paints = palette.paints)))
            }

            is ProjectsIntent.AddPhoto -> viewModelScope.launch { addPhoto(intent.bytes, intent.role) }

            is ProjectsIntent.RemovePhoto -> viewModelScope.launch {
                val project = state.value.selected ?: return@launch
                refreshDetail(projects.removePhoto(project, intent.id))
            }

            is ProjectsIntent.EditLayer -> viewModelScope.launch { editLayer(intent) }

            is ProjectsIntent.CompareWithPlan -> viewModelScope.launch { compare(intent.bytes) }

            ProjectsIntent.ForgetComparison ->
                detail.value = detail.value.copy(comparison = emptyList(), message = null)
        }
    }

    /**
     * Coche ou decoche une couche.
     *
     * <p>La vitesse de sechage n'est pas demandee au peintre : elle se lit dans le
     * melange calcule pour cette couche, donc dans les tubes du projet. Elle est figee
     * avec la pose, au meme titre que les conditions -- remanier la palette ensuite ne
     * doit pas reecrire ce qui a deja seche.</p>
     */
    private suspend fun toggle(project: Project, zoneIndex: Int, layerIndex: Int) {
        val zone = project.zones.getOrNull(zoneIndex) ?: return
        val layer = zone.layers.getOrNull(layerIndex) ?: return

        val updated = if (layer.isApplied) layer.clearApplied() else {
            val mix = state.value.zones.getOrNull(zoneIndex)?.layers?.getOrNull(layerIndex)?.mix
            layer.markApplied(clock.now(), workshop(), PlanDryingService.dryingClassOf(mix ?: return))
        }

        val saved = projects.save(
            project.copy(
                zones = project.zones.mapIndexed { z, existing ->
                    if (z != zoneIndex) existing
                    else existing.copy(layers = existing.layers.mapIndexed { l, it -> if (l == layerIndex) updated else it })
                }
            )
        )
        refreshDetail(saved)
    }

    /**
     * Range une photo avec le projet.
     *
     * <p>Elle est reduite ici et non chez l'appelant : une base locale qui accueille des
     * cliches de telephone doit garantir elle-meme qu'ils restent d'une taille
     * raisonnable, quel que soit l'ecran qui les lui envoie.</p>
     */
    private suspend fun addPhoto(bytes: ByteArray, role: ProjectPhoto.Role) {
        val project = state.value.selected ?: return
        detail.value = detail.value.copy(busy = true, message = null)
        try {
            val reduced = decoder.scaleTo(bytes, STORED_EDGE)
            refreshDetail(projects.addPhoto(project, ProjectPhoto(data = reduced, role = role)))
        } catch (e: Exception) {
            detail.value = detail.value.copy(
                busy = false,
                message = e.message ?: "Cette image n'a pas pu etre lue.",
            )
        }
    }

    /**
     * Corrige une couche, sans toucher a ce qui a deja seche.
     *
     * <p>La pose n'est pas effacee : une couche deja posee garde la date, l'atelier et la
     * vitesse de sechage de ce jour-la. Corriger la couleur visee pour la suite ne doit
     * pas reecrire ce qui est sur la piece.</p>
     */
    private suspend fun editLayer(intent: ProjectsIntent.EditLayer) {
        val project = state.value.selected ?: return
        val hex = runCatching { Rgb.ofHex(intent.targetHex) }.getOrNull()?.toHex() ?: return

        val updated = project.copy(
            zones = project.zones.mapIndexed { z, zone ->
                if (z != intent.zoneIndex) {
                    zone
                } else {
                    zone.copy(
                        layers = zone.layers.mapIndexed { l, layer ->
                            if (l != intent.layerIndex) layer
                            else layer.copy(
                                targetHex = hex,
                                technique = intent.technique,
                                note = intent.note,
                            )
                        }
                    )
                }
            }
        )
        refreshDetail(projects.save(updated))
    }

/**
     * Confronte une photo d'avancement aux couleurs visees par le plan.
     *
     * <p>La mesure porte sur le fichier d'origine, jamais sur la photo rangee avec le
     * projet : celle-ci est reduite et reencodee, et ses couleurs ont bouge. Meme ainsi,
     * une photo reste prise sous une lumiere quelconque -- l'ecart se lit comme une
     * tendance, pas comme un verdict.</p>
     *
     * <p>Rien n'est enregistre. C'est un constat a un instant, et la photo comparee n'est
     * pas rangee : on compare, on regarde, on referme.</p>
     */
    private suspend fun compare(bytes: ByteArray) {
        val project = state.value.selected ?: return
        detail.value = detail.value.copy(comparing = true, comparison = emptyList(), message = null)
        try {
            val dominant = ImagePalette.dominant(decoder.decode(bytes), OBSERVED)
            detail.value = detail.value.copy(
                comparing = false,
                comparison = progress.compare(project, dominant),
            )
        } catch (e: Exception) {
            detail.value = detail.value.copy(
                comparing = false,
                message = "Image illisible : ${e.message ?: "format non reconnu"}",
            )
        }
    }

    /** Recalcule le plan de la piece choisie, hors du fil d'affichage. */
    private fun refreshDetail(project: Project) {
        selectedId.value = project.id
        detail.value = detail.value.copy(computing = true)
        viewModelScope.launch {
            val plan = planner.plan(project)
            val owned = catalog.inStock()
            // Les photos se relisent a part : le projet que l'on tient a pu etre charge
            // sans elles, et les afficher d'apres lui les montrerait absentes.
            val photos = project.id?.let { projects.findWithPhotos(it)?.photos }.orEmpty()
            detail.value = Detail(
                computing = false,
                zones = rows(project, plan),
                risks = fatOverLean.inspect(project),
                missing = substitutes.missingFrom(project, owned),
                photos = photos,
            )
        }
    }

    /**
     * Relie chaque couche rangee a son melange calcule, par le role.
     *
     * <p>C'est volontairement plus laborieux qu'un rang commun : les deux ordres ont
     * diverge une fois deja, et un melange attribue a la mauvaise couche ne se voit pas.</p>
     */
    private fun rows(project: Project, plan: PaintingPlan): List<ZoneRow> =
        project.zones.mapIndexed { zoneIndex, zone ->
            val planned = plan.zones.getOrNull(zoneIndex)
            val byRole = ((planned?.layers() ?: emptyList()) + (planned?.accents ?: emptyList()))
                .associateBy { it.role }

            ZoneRow(
                name = zone.name,
                material = zone.material,
                layers = zone.layers.mapIndexed { layerIndex, layer ->
                    LayerRow(
                        zoneIndex = zoneIndex,
                        layerIndex = layerIndex,
                        role = layer.role,
                        targetHex = layer.targetHex,
                        technique = layer.technique,
                        isApplied = layer.isApplied,
                        note = layer.note,
                        mix = byRole[layer.role],
                    )
                },
            )
        }

    private companion object {
        /** Cote maximal d'une photo rangee : assez pour rouvrir sa piece dans six mois. */
        const val STORED_EDGE = 1024

        /** Teintes relevees sur la photo : assez pour couvrir une piece, pas pour noyer. */
        const val OBSERVED = 8
    }
}
