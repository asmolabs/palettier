package be.asmolabs.palettier.ui.recipes

import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.recipe.Recipe
import be.asmolabs.palettier.domain.recipe.RecipeStep
import be.asmolabs.palettier.domain.recipe.RecipeTimelineService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class RecipesViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private val casque = Recipe(
        id = 1, name = "Casque allemand", subject = "Metal",
        steps = listOf(
            RecipeStep.of(Technique.BASE_LAYER, "Vert olive"),
            RecipeStep.of(Technique.OIL_WASH, "Terre d'ombre"),
            RecipeStep.of(Technique.HIGHLIGHT, "Ocre"),
        ),
    )

    private class FakeRecipes(recipes: List<Recipe>) : RecipeRepository {
        val recipes = MutableStateFlow(recipes)
        override fun observeAll(): Flow<List<Recipe>> = this.recipes
        override suspend fun all() = recipes.value
        override suspend fun exists(name: String) = recipes.value.any { it.name.equals(name, true) }
        override suspend fun save(recipe: Recipe) = recipe
        override suspend fun delete(recipe: Recipe) {}
    }

    private suspend fun kotlinx.coroutines.CoroutineScope.model(): RecipesViewModel {
        val model = RecipesViewModel(FakeRecipes(listOf(casque)), RecipeTimelineService())
        launch { model.state.collect { } }
        model.state.first { !it.loading }
        return model
    }

    @Test
    fun `la premiere recette est deroulee d'emblee`() = runTest(dispatcher) {
        val model = backgroundScope.model()

        val state = model.state.value
        assertEquals("Casque allemand", state.selected?.name)
        assertNotNull(state.timeline)
        assertEquals(3, state.timeline.entries.size)
    }

    @Test
    fun `un pigment plus lent allonge tout le planning`() = runTest(dispatcher) {
        val model = backgroundScope.model()
        val medium = model.state.value.timeline!!.totalActiveSpan

        model.onIntent(RecipesIntent.SetDryingClass(DryingClass.VERY_SLOW))
        advanceUntilIdle()

        assertTrue(model.state.value.timeline!!.totalActiveSpan > medium)
    }

    @Test
    fun `l'atelier compte autant que la recette`() = runTest(dispatcher) {
        val model = backgroundScope.model()
        val warm = model.state.value.timeline!!.untilVarnish

        model.onIntent(RecipesIntent.SetWorkshop(Workshop(10.0, 80.0, Ventilation.CONFINED)))
        advanceUntilIdle()

        assertTrue(model.state.value.timeline!!.untilVarnish > warm)
    }

    @Test
    fun `l'attente cumulee est bien faite d'attente, pas de travail`() = runTest(dispatcher) {
        val model = backgroundScope.model()
        val timeline = model.state.value.timeline!!

        // Le vernis vient apres la derniere attente, plus la polymerisation complete.
        assertTrue(timeline.untilVarnish > timeline.totalActiveSpan)
        assertEquals(kotlin.time.Duration.ZERO, timeline.entries.last().waitAfter)
    }
}
