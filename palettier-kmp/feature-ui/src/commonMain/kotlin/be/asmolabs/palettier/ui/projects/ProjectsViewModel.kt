package be.asmolabs.palettier.ui.projects

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.plan.PaintingPlan
import be.asmolabs.palettier.domain.plan.PlanDryingService
import be.asmolabs.palettier.domain.plan.ProjectPlanner
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.SubstituteService
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

    /** Recalcule le plan de la piece choisie, hors du fil d'affichage. */
    private fun refreshDetail(project: Project) {
        detail.value = detail.value.copy(computing = true)
        viewModelScope.launch {
            val plan = planner.plan(project)
            val owned = catalog.inStock()
            detail.value = Detail(
                computing = false,
                zones = rows(project, plan),
                risks = fatOverLean.inspect(project),
                missing = substitutes.missingFrom(project, owned),
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
                        mix = byRole[layer.role],
                    )
                },
            )
        }
}
