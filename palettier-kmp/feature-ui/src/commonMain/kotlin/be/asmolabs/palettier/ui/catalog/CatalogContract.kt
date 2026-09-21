package be.asmolabs.palettier.ui.catalog

import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.PaintMatchByColour

/** Une ligne du catalogue : le tube, et son ecart a la teinte visee s'il y en a une. */
data class CatalogRow(val paint: Paint, val deltaE: Double?)

data class CatalogUiState(
    val loading: Boolean = true,
    val rows: List<CatalogRow> = emptyList(),
    val total: Int = 0,
    val query: String = "",
    val onlyInStock: Boolean = false,
    /** Teinte visee, quand le peintre cherche "ce qui s'en rapproche le plus". */
    val targetHex: String? = null,
    /** La derniere teinte relevee a la pipette, quand elle differe de la teinte visee. */
    val sampledHex: String? = null,
    /** Le tube en cours de calibration, choisi dans le tableau. */
    val selected: Paint? = null,
    /** Ce que la derniere attribution a donne. */
    val calibrationMessage: String? = null,
    val error: String? = null,
) {
    /** Vrai quand le classement se fait par ecart plutot que par marque et nom. */
    val sortedByDistance: Boolean get() = targetHex != null
}

sealed interface CatalogIntent {
    data class Search(val query: String) : CatalogIntent
    data class OnlyInStock(val only: Boolean) : CatalogIntent

    /** Classe le catalogue par ecart a une teinte, ou revient a l'ordre alphabetique. */
    data class AimAt(val hex: String?) : CatalogIntent

    /** Declare qu'un tube est, ou n'est plus, sur l'etagere. */
    data class SetOwned(val paint: Paint, val owned: Boolean) : CatalogIntent

    /** Choisit le tube a calibrer. Le meme tube deux fois le deselectionne. */
    data class Select(val paint: Paint) : CatalogIntent

    /**
     * Attribue une couleur mesuree au tube choisi.
     *
     * <p>Le partage est deliberé : la pipette mesure, le catalogue attribue. C'est dans le
     * tableau qu'on voit ce qui manque, et c'est la qu'on dispose deja de la recherche et
     * des filtres.</p>
     *
     * @param asTint vrai pour la teinte coupee de blanc, faux pour le ton de masse
     */
    data class Calibrate(val hex: String, val asTint: Boolean) : CatalogIntent
}
