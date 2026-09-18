package be.asmolabs.palettier.ui.palettes

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.ui.FakeCatalogRepository
import be.asmolabs.palettier.ui.FakePaletteRepository
import be.asmolabs.palettier.ui.testPaint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class PalettesViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private val white = testPaint(1, "Titanium White", "#F4F2EC", drying = DryingClass.SLOW, pigments = setOf("PW6"))
    private val umber = testPaint(2, "Burnt Umber", "#4A3427", drying = DryingClass.FAST, pigments = setOf("PBr7"))
    private val cadmium = testPaint(3, "Cadmium Yellow", "#F5B800", drying = DryingClass.VERY_SLOW, pigments = setOf("PY35"))
    private val extras = (4L..12L).map { testPaint(it, "Tube $it", "#40506$it", pigments = setOf("PX$it")) }

    private fun catalog() = FakeCatalogRepository(listOf(white, umber, cadmium) + extras)

    private suspend fun kotlinx.coroutines.CoroutineScope.model(
        palettes: FakePaletteRepository = FakePaletteRepository(),
    ): Pair<PalettesViewModel, FakePaletteRepository> {
        val model = PalettesViewModel(palettes, catalog())
        launch { model.state.collect { } }
        model.state.first { !it.loading }
        return model to palettes
    }

    @Test
    fun `creer une palette la selectionne aussitot`() = runTest(dispatcher) {
        val (model, _) = backgroundScope.model()

        model.onIntent(PalettesIntent.Create("Carnations 1/10"))
        advanceUntilIdle()

        assertEquals("Carnations 1/10", model.state.value.selected?.name)
        assertEquals(1, model.state.value.palettes.size)
    }

    @Test
    fun `ajouter un tube le retire des candidats`() = runTest(dispatcher) {
        val (model, _) = backgroundScope.model()
        model.onIntent(PalettesIntent.Create("Zorn"))
        advanceUntilIdle()

        assertTrue(model.state.value.candidates.any { it.id == umber.id })

        model.onIntent(PalettesIntent.AddPaint(umber))
        advanceUntilIdle()

        assertEquals(listOf("Burnt Umber"), model.state.value.selected!!.paints.map { it.name })
        assertTrue(model.state.value.candidates.none { it.id == umber.id }, "un tube deja pris ne se propose plus")
    }

    @Test
    fun `la palette annonce la vitesse de son tube le plus lent`() = runTest(dispatcher) {
        val (model, _) = backgroundScope.model()
        model.onIntent(PalettesIntent.Create("Zorn"))
        advanceUntilIdle()

        model.onIntent(PalettesIntent.AddPaint(umber))
        advanceUntilIdle()
        assertEquals(DryingClass.FAST, model.state.value.slowest)

        // Un cadmium dans la palette, et c'est tout le planning qui change.
        model.onIntent(PalettesIntent.AddPaint(cadmium))
        advanceUntilIdle()
        assertEquals(DryingClass.VERY_SLOW, model.state.value.slowest)
    }

    @Test
    fun `au-dela de six pigments, la palette le signale`() = runTest(dispatcher) {
        val (model, _) = backgroundScope.model()
        model.onIntent(PalettesIntent.Create("Trop riche"))
        advanceUntilIdle()

        (listOf(white, umber, cadmium) + extras.take(3)).forEach {
            model.onIntent(PalettesIntent.AddPaint(it))
            advanceUntilIdle()
        }
        assertEquals(6, model.state.value.pigments.size)
        assertTrue(!model.state.value.tooManyPigments)

        model.onIntent(PalettesIntent.AddPaint(extras[3]))
        advanceUntilIdle()
        assertTrue(model.state.value.tooManyPigments, "sept pigments : les melanges vont griser")
    }

    @Test
    fun `retirer un tube le rend aux candidats`() = runTest(dispatcher) {
        val (model, _) = backgroundScope.model()
        model.onIntent(PalettesIntent.Create("Zorn"))
        advanceUntilIdle()
        model.onIntent(PalettesIntent.AddPaint(umber))
        advanceUntilIdle()

        model.onIntent(PalettesIntent.RemovePaint(umber))
        advanceUntilIdle()

        assertTrue(model.state.value.selected!!.paints.isEmpty())
        assertTrue(model.state.value.candidates.any { it.id == umber.id })
    }

    @Test
    fun `la recherche restreint les candidats sans toucher a la palette`() = runTest(dispatcher) {
        val (model, _) = backgroundScope.model()
        model.onIntent(PalettesIntent.Create("Zorn"))
        advanceUntilIdle()

        model.onIntent(PalettesIntent.Search("PBr7"))
        advanceUntilIdle()

        assertEquals(listOf("Burnt Umber"), model.state.value.candidates.map { it.name })
    }
}
