package be.asmolabs.palettier.ui.workbench

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.palette.PaletteMixService.MixState
import be.asmolabs.palettier.domain.workbench.Bench

/**
 * Ce que l'ecran Aujourd'hui montre, a un instant donne.
 *
 * <p>Un seul objet, immuable, et non une collection de proprietes observables. La
 * difference se voit a l'usage : avec JavaFX, afficher un etat coherent demandait de
 * mettre a jour six champs dans le bon ordre, et un oubli laissait l'ecran a moitie
 * rafraichi. Ici un etat partiel n'est pas representable.</p>
 */
data class WorkbenchUiState(
    val loading: Boolean = true,
    val bench: Bench = Bench(emptyList()),
    val mixes: List<MixState> = emptyList(),
    /** La derniere teinte relevee a la pipette, si le peintre en a releve une. */
    val sampledHex: String? = null,
    val error: String? = null,
) {
    val isEmpty: Boolean get() = !loading && bench.pieces.isEmpty()
}

/** Ce que le peintre peut demander depuis cet ecran. */
sealed interface WorkbenchIntent {

    /** Pose un melange sur la palette, avec la teinte relevee et les conditions du moment. */
    data class RecordMix(
        val name: String,
        val recipe: String,
        val dryingClass: DryingClass,
    ) : WorkbenchIntent

    data class ForgetMix(val id: Long) : WorkbenchIntent

    /** Retire de la palette ce qui a pris. */
    data object CleanSpentMixes : WorkbenchIntent
}
