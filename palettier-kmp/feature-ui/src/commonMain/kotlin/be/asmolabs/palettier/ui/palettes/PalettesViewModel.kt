package be.asmolabs.palettier.ui.palettes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/**
 * Les palettes : une selection nommee de tubes par sujet.
 *
 * <p>Sur une figurine on ne travaille jamais avec le catalogue entier mais avec six a
 * douze tubes choisis ensemble. La palette sert donc autant a s'organiser qu'a restreindre
 * les recherches : chercher un melange "dans ma palette" donne une reponse utilisable, la
 * meme recherche sur sept cents tubes donne une reponse theorique.</p>
 */
class PalettesViewModel(
    private val palettes: PaletteRepository,
    catalog: PaintCatalogRepository,
) : ViewModel() {

    private val selectedId = MutableStateFlow<Long?>(null)
    private val query = MutableStateFlow("")

    val state: StateFlow<PalettesUiState> =
        combine(palettes.observeAll(), catalog.observeAll(), selectedId, query) { all, paints, id, text ->
            val selected = all.firstOrNull { it.id == id } ?: all.firstOrNull()
            val already = selected?.paints?.mapNotNull { it.id }?.toSet().orEmpty()

            PalettesUiState(
                loading = false,
                palettes = all,
                selected = selected,
                candidates = paints
                    .filter { it.id !in already }
                    .filter { matches(it, text) }
                    .take(CANDIDATES),
                query = text,
            )
        }
            .catch { emit(PalettesUiState(loading = false, error = it.message ?: "Lecture impossible")) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), PalettesUiState())

    fun onIntent(intent: PalettesIntent) {
        when (intent) {
            is PalettesIntent.Select -> selectedId.value = intent.palette.id
            is PalettesIntent.Search -> query.value = intent.query

            is PalettesIntent.Create -> viewModelScope.launch {
                if (intent.name.isBlank()) return@launch
                selectedId.value = palettes.save(Palette(name = intent.name.trim())).id
            }

            is PalettesIntent.AddPaint -> viewModelScope.launch {
                val palette = state.value.selected ?: return@launch
                palettes.save(palette.withPaint(intent.paint))
            }

            is PalettesIntent.RemovePaint -> viewModelScope.launch {
                val palette = state.value.selected ?: return@launch
                palettes.save(palette.withoutPaint(intent.paint))
            }

            is PalettesIntent.Delete -> viewModelScope.launch {
                palettes.delete(intent.palette)
                selectedId.value = null
            }
        }
    }

    private fun matches(paint: Paint, text: String): Boolean {
        if (text.isBlank()) return true
        val needle = text.trim().lowercase()
        return paint.name.lowercase().contains(needle) ||
            paint.brand.lowercase().contains(needle) ||
            paint.pigments.any { it.lowercase().contains(needle) }
    }

    companion object {
        /** On propose de quoi choisir, pas le catalogue entier : c'est une palette. */
        private const val CANDIDATES = 40
    }
}
