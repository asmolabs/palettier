package be.asmolabs.palettier.ui.drying

import androidx.lifecycle.ViewModel
import be.asmolabs.palettier.domain.drying.DryingContext
import be.asmolabs.palettier.domain.drying.DryingTimeService
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Combien de temps pour une couche a venir.
 *
 * <p>Tout y est immediat : le modele de sechage est une poignee de multiplications, et
 * l'interet de l'ecran est justement de voir bouger les jalons pendant qu'on deplace un
 * curseur. Rien ne justifierait d'attendre.</p>
 */
class DryingViewModel(private val service: DryingTimeService) : ViewModel() {

    private val _state = MutableStateFlow(DryingUiState())
    val state: StateFlow<DryingUiState> = _state.asStateFlow()

    init {
        recompute()
    }

    fun onIntent(intent: DryingIntent) {
        _state.value = when (intent) {
            is DryingIntent.SetWorkshop -> _state.value.copy(workshop = intent.workshop)

            // Changer de technique repositionne ses reglages usuels : c'est ce qu'on
            // attend en la choisissant, et rien n'empeche de les ajuster ensuite.
            is DryingIntent.SetTechnique -> _state.value.copy(
                technique = intent.technique,
                medium = intent.technique.defaultMedium,
                mediumRatio = intent.technique.defaultRatio,
                thickness = intent.technique.typicalThickness,
            )

            is DryingIntent.SetDryingClass -> _state.value.copy(dryingClass = intent.dryingClass)
            is DryingIntent.SetMedium -> _state.value.copy(medium = intent.medium)
            is DryingIntent.SetRatio -> _state.value.copy(mediumRatio = intent.ratio.coerceIn(0.0, 1.0))
            is DryingIntent.SetThickness -> _state.value.copy(thickness = intent.thickness)
        }
        recompute()
    }

    private fun recompute() {
        val current = _state.value
        _state.value = current.copy(
            estimate = service.estimate(
                DryingContext(
                    current.dryingClass, current.medium, current.mediumRatio,
                    current.thickness, current.workshop,
                )
            )
        )
    }
}
