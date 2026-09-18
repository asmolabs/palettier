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
}
