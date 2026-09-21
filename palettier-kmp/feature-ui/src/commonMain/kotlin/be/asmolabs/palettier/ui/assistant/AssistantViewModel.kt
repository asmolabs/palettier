package be.asmolabs.palettier.ui.assistant

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.ai.AiSettings
import be.asmolabs.palettier.ai.AiSettingsStore
import be.asmolabs.palettier.ai.PaintingPlanService
import be.asmolabs.palettier.ai.PhotoInput
import be.asmolabs.palettier.ai.PlanUnavailable
import be.asmolabs.palettier.domain.port.PaletteRepository
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * L'assistant.
 *
 * <p>Une demande de plan prend des minutes -- un modele local qui lit une photo n'est pas
 * rapide -- alors elle tourne dans sa propre coroutine et l'ecran reste vivant pendant ce
 * temps. Une seule demande a la fois : relancer annule la precedente, qui n'interesse
 * plus personne.</p>
 *
 * <p>Un echec est une information, pas un incident : le peintre voit ce que le moteur a
 * repondu et decide. Rien d'autre dans l'application n'en depend.</p>
 */
class AssistantViewModel(
    private val plans: PaintingPlanService,
    private val store: AiSettingsStore,
    private val palettes: PaletteRepository,
) : ViewModel() {

    private val _state = MutableStateFlow(AssistantUiState())
    val state: StateFlow<AssistantUiState> = _state.asStateFlow()

    private var asking: Job? = null

    init {
        viewModelScope.launch {
            val known = palettes.all()
            _state.value = _state.value.copy(
                settings = store.current(),
                palettes = known,
                paletteId = _state.value.paletteId ?: known.firstOrNull()?.id,
            )
        }
    }

    fun onIntent(intent: AssistantIntent) {
        when (intent) {
            is AssistantIntent.SetSubject -> _state.value = _state.value.copy(subject = intent.subject)

            is AssistantIntent.ChoosePalette -> _state.value = _state.value.copy(paletteId = intent.id)

            is AssistantIntent.SetMaxPaints ->
                _state.value = _state.value.copy(maxPaints = intent.count.coerceIn(1, 5))

            is AssistantIntent.AttachFigurine -> _state.value = _state.value.copy(
                figurine = AttachedPhoto("la piece telle qu'elle est", intent.bytes),
            )

            is AssistantIntent.AttachReference -> _state.value = _state.value.copy(
                references = _state.value.references +
                    AttachedPhoto("reference ${_state.value.references.size + 1}", intent.bytes),
            )

            AssistantIntent.ClearPhotos ->
                _state.value = _state.value.copy(figurine = null, references = emptyList())

            AssistantIntent.Ask -> ask()

            is AssistantIntent.SaveSettings -> save(intent.settings)
        }
    }

    private fun save(settings: AiSettings) {
        viewModelScope.launch {
            store.save(settings)
            _state.value = _state.value.copy(settings = store.current(), error = null)
        }
    }

    private fun ask() {
        val current = _state.value
        val palette = current.palette ?: return

        asking?.cancel()
        _state.value = current.copy(working = true, error = null, plan = null)
        asking = viewModelScope.launch {
            try {
                val plan = plans.plan(
                    subject = current.subject,
                    palette = palette,
                    maxPaints = current.maxPaints,
                    figurine = current.figurine?.let { PhotoInput(it.bytes, it.caption) },
                    references = current.references.map { PhotoInput(it.bytes, it.caption) },
                )
                _state.value = _state.value.copy(working = false, plan = plan)
            } catch (e: PlanUnavailable) {
                _state.value = _state.value.copy(working = false, error = e.message)
            } catch (e: Exception) {
                // Une panne de reseau, un serveur eteint : le peintre doit le lire, et
                // l'ecran doit rester utilisable.
                _state.value = _state.value.copy(
                    working = false,
                    error = "Le moteur n'a pas repondu : ${e.message ?: "cause inconnue"}",
                )
            }
        }
    }
}
