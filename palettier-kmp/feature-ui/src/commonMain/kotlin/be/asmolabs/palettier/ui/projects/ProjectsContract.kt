package be.asmolabs.palettier.ui.projects

import be.asmolabs.palettier.domain.plan.PaintingPlan
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.SubstituteService

/** Une couche telle que l'ecran la montre : la decision, et le melange qui en decoule. */
data class LayerRow(
    val zoneIndex: Int,
    val layerIndex: Int,
    val role: String,
    val targetHex: String,
    val technique: String,
    val isApplied: Boolean,
    /** Ce que la palette permet d'atteindre, ou null tant que le calcul n'est pas fini. */
    val mix: PaintingPlan.Layer?,
)

data class ZoneRow(val name: String, val material: String, val layers: List<LayerRow>)

data class ProjectsUiState(
    val loading: Boolean = true,
    val projects: List<Project> = emptyList(),
    val selected: Project? = null,
    val zones: List<ZoneRow> = emptyList(),
    /** Vrai pendant le recalcul des dosages, qui parcourt la palette pour chaque couche. */
    val computing: Boolean = false,
    val risks: List<FatOverLeanService.Risk> = emptyList(),
    val missing: List<SubstituteService.Missing> = emptyList(),
    val error: String? = null,
)

sealed interface ProjectsIntent {
    data class Select(val project: Project) : ProjectsIntent

    /** Coche ou decoche une couche. La pose est un fait, pas une decision revocable. */
    data class ToggleApplied(val zoneIndex: Int, val layerIndex: Int) : ProjectsIntent
}
