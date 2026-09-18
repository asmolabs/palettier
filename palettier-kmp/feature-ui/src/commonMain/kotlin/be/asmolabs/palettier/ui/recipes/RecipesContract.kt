package be.asmolabs.palettier.ui.recipes

import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.recipe.Recipe
import be.asmolabs.palettier.domain.recipe.RecipeTimelineService

data class RecipesUiState(
    val loading: Boolean = true,
    val recipes: List<Recipe> = emptyList(),
    val selected: Recipe? = null,
    val timeline: RecipeTimelineService.Timeline? = null,
    val dryingClass: DryingClass = DryingClass.MEDIUM,
    val workshop: Workshop = Workshop.standard(),
    val error: String? = null,
)

sealed interface RecipesIntent {
    data class Select(val recipe: Recipe) : RecipesIntent
    data class SetDryingClass(val dryingClass: DryingClass) : RecipesIntent
    data class SetWorkshop(val workshop: Workshop) : RecipesIntent
}
