package be.asmolabs.palettier.ui.mixer

import be.asmolabs.palettier.domain.mix.MixResult
import be.asmolabs.palettier.domain.mix.MixSuggestion
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.ui.SearchScope

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
    /** La derniere teinte relevee a la pipette, quand elle differe de la teinte visee. */
    val sampledHex: String? = null,
    val suggestions: List<MixSuggestion> = emptyList(),
    val searching: Boolean = false,
    /** Ou chercher, et de quoi se composer un melange a la main. */
    val scope: SearchScope = SearchScope.Shelf,
    val scopes: List<SearchScope> = listOf(SearchScope.Shelf, SearchScope.Catalogue),
    /** Nombre de tubes que la recherche s'autorise par melange. */
    val maxPaints: Int = DEFAULT_MAX_PAINTS,
    val error: String? = null,
)

/** Trois tubes : au-dela, un melange vire au gris et ne se refait pas a l'identique. */
const val DEFAULT_MAX_PAINTS = 3

sealed interface MixerIntent {
    data class Add(val paint: Paint) : MixerIntent
    data class SetParts(val paint: Paint, val parts: Int) : MixerIntent
    data class Remove(val paint: Paint) : MixerIntent
    data object Clear : MixerIntent

    /** Sens inverse : quelle recette donne cette couleur. */
    data class AimAt(val hex: String) : MixerIntent
    /**
     * Change l'ensemble dans lequel chercher, et donc celui qu'on a sous la main.
     *
     * <p>La portee commande les deux sens du melangeur : les tubes proposes au clic et
     * ceux que la recherche explore. Chercher dans la palette sortie sur la table, c'est
     * la difference entre une recette theorique et une recette a faire ce soir.</p>
     */
    data class ChooseScope(val scope: SearchScope) : MixerIntent

    data class SetMaxPaints(val count: Int) : MixerIntent
}
