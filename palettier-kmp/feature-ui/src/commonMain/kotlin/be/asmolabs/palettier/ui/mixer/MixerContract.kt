package be.asmolabs.palettier.ui.mixer

import be.asmolabs.palettier.domain.mix.MixResult
import be.asmolabs.palettier.domain.mix.MixSuggestion
import be.asmolabs.palettier.domain.paint.Paint

/** Une dose sur la palette : un tube et son nombre de parts. */
data class Dose(val paint: Paint, val parts: Int)

data class MixerUiState(
    val loading: Boolean = true,
    val available: List<Paint> = emptyList(),
    /** Le melange qu'on compose, dans le sens direct. */
    val doses: List<Dose> = emptyList(),
    val result: MixResult? = null,
    /** La teinte visee, dans le sens inverse. */
    val targetHex: String = "",
    val suggestions: List<MixSuggestion> = emptyList(),
    val searching: Boolean = false,
    val onlyInStock: Boolean = true,
    val error: String? = null,
)

sealed interface MixerIntent {
    data class Add(val paint: Paint) : MixerIntent
    data class SetParts(val paint: Paint, val parts: Int) : MixerIntent
    data class Remove(val paint: Paint) : MixerIntent
    data object Clear : MixerIntent

    /** Sens inverse : quelle recette donne cette couleur. */
    data class AimAt(val hex: String) : MixerIntent
    data class OnlyInStock(val only: Boolean) : MixerIntent
}
