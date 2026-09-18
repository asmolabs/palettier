package be.asmolabs.palettier.ui.mixer

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.mix.PaintPart
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * Le melangeur, dans les deux sens.
 *
 * <p>Sens direct : quelle couleur donne ce melange. Le calcul est immediat -- c'est du
 * Kubelka-Munk sur trois canaux -- et se refait a chaque changement de dose.</p>
 *
 * <p>Sens inverse : quel melange donne cette couleur. Celui-la coute, puisqu'il explore
 * les dosages realisables : il tourne dans sa propre coroutine, qu'une nouvelle demande
 * annule. Sans cela, taper une teinte caractere par caractere lancerait six recherches
 * dont cinq pour rien.</p>
 */
class MixerViewModel(
    private val catalog: PaintCatalogRepository,
    private val mixer: ColorMixService,
) : ViewModel() {

    private val _state = MutableStateFlow(MixerUiState())
    val state: StateFlow<MixerUiState> = _state.asStateFlow()

    private var search: Job? = null

    init {
        viewModelScope.launch { reloadAvailable() }
    }

    fun onIntent(intent: MixerIntent) {
        when (intent) {
            is MixerIntent.Add -> {
                if (_state.value.doses.none { it.paint.id == intent.paint.id }) {
                    _state.value = _state.value.copy(doses = _state.value.doses + Dose(intent.paint, 1))
                    recompute()
                }
            }

            is MixerIntent.SetParts -> {
                val parts = intent.parts.coerceIn(1, 30)
                _state.value = _state.value.copy(
                    doses = _state.value.doses.map {
                        if (it.paint.id == intent.paint.id) it.copy(parts = parts) else it
                    }
                )
                recompute()
            }

            is MixerIntent.Remove -> {
                _state.value = _state.value.copy(doses = _state.value.doses.filterNot { it.paint.id == intent.paint.id })
                recompute()
            }

            MixerIntent.Clear -> _state.value = _state.value.copy(doses = emptyList(), result = null)

            is MixerIntent.AimAt -> aimAt(intent.hex)

            is MixerIntent.OnlyInStock -> viewModelScope.launch {
                _state.value = _state.value.copy(onlyInStock = intent.only)
                reloadAvailable()
                aimAt(_state.value.targetHex)
            }
        }
    }

    private suspend fun reloadAvailable() {
        val paints = if (_state.value.onlyInStock) catalog.inStock() else catalog.all()
        _state.value = _state.value.copy(loading = false, available = paints)
    }

    /** Sens direct : immediat, refait a chaque changement. */
    private fun recompute() {
        val doses = _state.value.doses
        if (doses.isEmpty()) {
            _state.value = _state.value.copy(result = null)
            return
        }
        _state.value = _state.value.copy(
            result = mixer.mix(doses.map { PaintPart.of(it.paint, it.parts) }),
        )
    }

    /** Sens inverse : coute, donc annulable. */
    private fun aimAt(hex: String) {
        search?.cancel()
        val target = runCatching { Rgb.ofHex(hex) }.getOrNull()

        _state.value = _state.value.copy(targetHex = hex, suggestions = emptyList(), searching = target != null)
        if (target == null) return

        search = viewModelScope.launch {
            val candidates = _state.value.available
            val found = mixer.suggestMixes(target, candidates, maxResults = 5)
            _state.value = _state.value.copy(suggestions = found, searching = false)
        }
    }
}
