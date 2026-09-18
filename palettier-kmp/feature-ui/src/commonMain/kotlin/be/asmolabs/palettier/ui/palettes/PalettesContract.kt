package be.asmolabs.palettier.ui.palettes

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette

data class PalettesUiState(
    val loading: Boolean = true,
    val palettes: List<Palette> = emptyList(),
    val selected: Palette? = null,
    /** Les tubes qu'on peut encore ajouter, filtres par la recherche. */
    val candidates: List<Paint> = emptyList(),
    val query: String = "",
    val error: String? = null,
) {
    /** Classe de sechage imposee par la palette : celle de son tube le plus lent. */
    val slowest: DryingClass? get() = selected?.takeIf { it.paints.isNotEmpty() }?.slowestDryingClass()

    /** Pigments distincts : au-dela de six, les melanges grisent quoi qu'on fasse. */
    val pigments: Set<String> get() = selected?.pigments().orEmpty()

    val tooManyPigments: Boolean get() = pigments.size > 6
}

sealed interface PalettesIntent {
    data class Select(val palette: Palette) : PalettesIntent
    data class Create(val name: String) : PalettesIntent
    data class Search(val query: String) : PalettesIntent
    data class AddPaint(val paint: Paint) : PalettesIntent
    data class RemovePaint(val paint: Paint) : PalettesIntent
    data class Delete(val palette: Palette) : PalettesIntent
}
