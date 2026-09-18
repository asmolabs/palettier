package be.asmolabs.palettier.ui.picker

import be.asmolabs.palettier.domain.paint.PaintMatchByColour
import be.asmolabs.palettier.domain.project.DominantColour

data class PickerUiState(
    val loading: Boolean = false,
    /** Teintes relevees sur la photo, de la plus presente a la moins presente. */
    val dominant: List<DominantColour> = emptyList(),
    val selectedHex: String? = null,
    /** Teinte du point choisi comme gris neutre, quand le peintre en a designe un. */
    val greyPointHex: String? = null,
    /** La teinte retenue une fois la dominante de la photo annulee. */
    val correctedHex: String? = null,
    /** Les tubes du catalogue les plus proches de la teinte retenue. */
    val closest: List<PaintMatchByColour> = emptyList(),
    val onlyInStock: Boolean = true,
    val error: String? = null,
) {
    /** La teinte sur laquelle tout le reste travaille : corrigee si elle l'a ete. */
    val workingHex: String? get() = correctedHex ?: selectedHex
}

sealed interface PickerIntent {
    /** Une photo choisie par la plateforme : le domaine ne connait que ses octets. */
    data class Load(val bytes: ByteArray) : PickerIntent

    data class Select(val hex: String) : PickerIntent

    /** Designe une teinte censee etre grise, pour annuler la dominante de la photo. */
    data class UseAsGrey(val hex: String?) : PickerIntent

    data class OnlyInStock(val only: Boolean) : PickerIntent
}
