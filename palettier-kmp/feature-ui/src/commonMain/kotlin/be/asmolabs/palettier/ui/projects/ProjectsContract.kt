package be.asmolabs.palettier.ui.projects

import be.asmolabs.palettier.domain.plan.PaintingPlan
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.ProgressCheckService
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.project.SubstituteService

/** Une couche telle que l'ecran la montre : la decision, et le melange qui en decoule. */
data class LayerRow(
    val zoneIndex: Int,
    val layerIndex: Int,
    val role: String,
    val targetHex: String,
    val technique: String,
    val isApplied: Boolean,
    val note: String = "",
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
    /**
     * Les photos de la piece choisie.
     *
     * <p>A part du projet, parce qu'elles se lisent a part : ce sont des images entieres,
     * et la liste des pieces n'en a que faire.</p>
     */
    val photos: List<ProjectPhoto> = emptyList(),
    /** Vrai quand la palette a ete remaniee depuis que le plan a ete etabli. */
    val paletteDiverges: Boolean = false,
    /**
     * Ce que la derniere photo d'avancement a montre, confronte au plan.
     *
     * <p>Rien n'est enregistre : c'est un constat a un instant, pas une decision. La
     * photo elle-meme n'est pas rangee non plus -- on compare, on regarde, on referme.</p>
     */
    val comparison: List<ProgressCheckService.Observed> = emptyList(),
    val comparing: Boolean = false,
    /** Le plan calcule de la piece choisie, celui que l'export met sur papier. */
    val plan: PaintingPlan? = null,
    val busy: Boolean = false,
    val message: String? = null,
    val error: String? = null,
)

sealed interface ProjectsIntent {
    data class Select(val project: Project) : ProjectsIntent

    /** Coche ou decoche une couche. La pose est un fait, pas une decision revocable. */
    data class ToggleApplied(val zoneIndex: Int, val layerIndex: Int) : ProjectsIntent

    data class Rename(val name: String) : ProjectsIntent

    data object Delete : ProjectsIntent

    /**
     * Reprend les tubes de la palette telle qu'elle est aujourd'hui.
     *
     * <p>Les couleurs visees ne bougent pas : seuls les melanges qui y menent sont
     * recalcules. C'est ce qui permet de profiter d'un tube achete depuis, sans refaire
     * le plan.</p>
     */
    data object SyncPalette : ProjectsIntent

    /** La plateforme a lu un fichier ; le ViewModel ne connait que des octets. */
    data class AddPhoto(val bytes: ByteArray, val role: ProjectPhoto.Role) : ProjectsIntent

    data class RemovePhoto(val id: Long) : ProjectsIntent

    /**
     * Confronte une photo d'avancement aux couleurs visees par le plan.
     *
     * <p>La lecture va de la piece vers le plan et non l'inverse : partir des couleurs
     * visees pour chercher la plus proche sur la photo trouverait toujours quelque chose,
     * une photo contenant des milliers de teintes.</p>
     */
    data class CompareWithPlan(val bytes: ByteArray) : ProjectsIntent

    data object ForgetComparison : ProjectsIntent

    /**
     * Corrige une couche : la couleur visee, la technique, la note.
     *
     * <p>Seule la decision se modifie. Le dosage n'est pas la parce qu'il ne se decide
     * pas : il se calcule a partir de la couleur retenue et des tubes du projet.</p>
     */
    data class EditLayer(
        val zoneIndex: Int,
        val layerIndex: Int,
        val targetHex: String,
        val technique: String,
        val note: String,
    ) : ProjectsIntent
}
