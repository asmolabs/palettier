package be.asmolabs.palettier.ui.recipes

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.recipe.RecipeTimelineService
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

/**
 * Le deroule reel d'une recette.
 *
 * <p>L'essentiel du planning est fait d'attente : une suite d'etapes qui se lit en trente
 * secondes peut demander trois semaines, et c'est la seule chose que cet ecran a a
 * montrer.</p>
 */
class RecipesViewModel(
    recipes: RecipeRepository,
    private val timelines: RecipeTimelineService,
) : ViewModel() {

    private val selectedId = MutableStateFlow<Long?>(null)
    private val dryingClass = MutableStateFlow(DryingClass.MEDIUM)
    private val workshop = MutableStateFlow(Workshop.standard())

    val state: StateFlow<RecipesUiState> =
        combine(recipes.observeAll(), selectedId, dryingClass, workshop) { all, id, drying, atelier ->
            val selected = all.firstOrNull { it.id == id } ?: all.firstOrNull()
            RecipesUiState(
                loading = false,
                recipes = all,
                selected = selected,
                timeline = selected?.let { timelines.plan(it, drying, atelier) },
                dryingClass = drying,
                workshop = atelier,
            )
        }
            .catch { emit(RecipesUiState(loading = false, error = it.message ?: "Lecture impossible")) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), RecipesUiState())

    fun onIntent(intent: RecipesIntent) {
        when (intent) {
            is RecipesIntent.Select -> selectedId.value = intent.recipe.id
            is RecipesIntent.SetDryingClass -> dryingClass.value = intent.dryingClass
            is RecipesIntent.SetWorkshop -> workshop.value = intent.workshop
        }
    }
}
