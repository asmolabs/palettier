package be.asmolabs.palettier.ui.picker

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.neutralise
import be.asmolabs.palettier.domain.image.ImagePalette
import be.asmolabs.palettier.domain.paint.findClosest
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.image.ImageDecoder
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * La pipette : relever une teinte sur une photo, et voir avec quoi la faire.
 *
 * <p>Deux precautions y sont incorporees, parce que sans elles le releve ment.</p>
 *
 * <p>La mesure se fait sur le fichier d'origine, jamais sur une image reduite : reduire
 * reencode, et reencoder deplace les couleurs.</p>
 *
 * <p>Et une photo prise sous lampe de bureau tire au jaune, sous LED froide au bleu.
 * Designer un point cense etre gris donne le gain a appliquer a chaque canal pour annuler
 * cette dominante -- la balance des blancs du photographe, reduite a son necessaire.</p>
 */
class PickerViewModel(
    private val decoder: ImageDecoder,
    private val catalog: PaintCatalogRepository,
) : ViewModel() {

    private val _state = MutableStateFlow(PickerUiState())
    val state: StateFlow<PickerUiState> = _state.asStateFlow()

    fun onIntent(intent: PickerIntent) {
        when (intent) {
            is PickerIntent.Load -> viewModelScope.launch { load(intent.bytes) }

            // Les propositions sont videes avant de recalculer : les garder afficherait,
            // le temps du calcul, des tubes choisis pour une autre teinte. Bref, mais faux.
            is PickerIntent.Select -> viewModelScope.launch {
                _state.value = _state.value.copy(selectedHex = intent.hex, closest = emptyList())
                recompute()
            }

            is PickerIntent.UseAsGrey -> viewModelScope.launch {
                _state.value = _state.value.copy(greyPointHex = intent.hex, closest = emptyList())
                recompute()
            }

            is PickerIntent.OnlyInStock -> viewModelScope.launch {
                _state.value = _state.value.copy(onlyInStock = intent.only, closest = emptyList())
                recompute()
            }
        }
    }

    private suspend fun load(bytes: ByteArray) {
        _state.value = PickerUiState(loading = true, onlyInStock = _state.value.onlyInStock)
        try {
            val dominant = ImagePalette.dominant(decoder.decode(bytes), COLOURS)
            _state.value = _state.value.copy(
                loading = false,
                dominant = dominant,
                selectedHex = dominant.firstOrNull()?.color?.toHex(),
            )
            recompute()
        } catch (e: Exception) {
            // Large a dessein : un fichier douteux ne doit pas faire tomber l'ecran, et
            // les facons d'echouer a lire une image ne se laissent pas enumerer.
            _state.value = PickerUiState(error = e.message ?: "Image illisible")
        }
    }

    /**
     * Applique la correction de dominante s'il y en a une, puis cherche les tubes les
     * plus proches.
     */
    private suspend fun recompute() {
        val current = _state.value
        val selected = current.selectedHex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() }
            ?: return

        val grey = current.greyPointHex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() }
        val working = grey?.let { neutralise(selected, it) } ?: selected

        val candidates = if (current.onlyInStock) catalog.inStock() else catalog.all()

        _state.value = current.copy(
            correctedHex = grey?.let { working.toHex() },
            closest = findClosest(working, candidates, CLOSEST),
        )
    }

    companion object {
        /** Teintes relevees sur la photo : assez pour couvrir une piece, pas pour noyer. */
        private const val COLOURS = 8

        /** Tubes proposes : au-dela, on ne compare plus, on parcourt. */
        private const val CLOSEST = 6
    }
}
