package be.asmolabs.palettier.ui.mixer

import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.ui.FakeCatalogRepository
import be.asmolabs.palettier.ui.testPaint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
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
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class MixerViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private val white = testPaint(1, "Titanium White", "#F4F2EC", drying = DryingClass.SLOW,
        opacity = Opacity.OPAQUE, pigments = setOf("PW6"))
    private val umber = testPaint(2, "Burnt Umber", "#4A3427", drying = DryingClass.FAST,
        pigments = setOf("PBr7"))
    private val cadmium = testPaint(3, "Cadmium Yellow", "#F5B800", drying = DryingClass.VERY_SLOW,
        opacity = Opacity.OPAQUE, pigments = setOf("PY35"))
    private val lake = testPaint(4, "Alizarin", "#8A1538", opacity = Opacity.TRANSPARENT,
        pigments = setOf("PR83"))

    private fun model(paints: List<be.asmolabs.palettier.domain.paint.Paint> = listOf(white, umber, cadmium, lake)) =
        MixerViewModel(FakeCatalogRepository(paints), ColorMixService())

    @Test
    fun `melanger deux tubes donne une couleur et la vitesse du plus lent`() = runTest(dispatcher) {
        val model = model()
        advanceUntilIdle()

        model.onIntent(MixerIntent.Add(umber))
        model.onIntent(MixerIntent.Add(white))
        advanceUntilIdle()

        val result = model.state.value.result
        assertNotNull(result)
        assertEquals(2, result.components.size)
        // Le blanc de titane est lent : c'est lui qui commande, quelle que soit sa dose.
        assertEquals(DryingClass.SLOW, result.dryingClass)
        assertEquals(setOf("PBr7", "PW6"), result.pigments)
    }

    @Test
    fun `changer les doses change la couleur, sans changer les tubes`() = runTest(dispatcher) {
        val model = model()
        advanceUntilIdle()
        model.onIntent(MixerIntent.Add(umber))
        model.onIntent(MixerIntent.Add(white))
        advanceUntilIdle()
        val balanced = model.state.value.result!!.color

        model.onIntent(MixerIntent.SetParts(white, 9))
        advanceUntilIdle()
        val pale = model.state.value.result!!

        assertEquals(2, pale.components.size)
        assertTrue(pale.color.relativeLuminance() > balanced.relativeLuminance(), "neuf parts de blanc eclaircissent")
    }

    @Test
    fun `le melange trop riche et l'opacite melangee sont signales`() = runTest(dispatcher) {
        val model = model()
        advanceUntilIdle()

        listOf(white, umber, cadmium, lake).forEach { model.onIntent(MixerIntent.Add(it)) }
        advanceUntilIdle()

        val warnings = model.state.value.result!!.warnings
        assertTrue(warnings.any { "gris" in it }, warnings.toString())
        // Une transparente avec une opaque : le glacis perd sa profondeur.
        assertTrue(warnings.any { "profondeur" in it }, warnings.toString())
    }

    @Test
    fun `retirer le dernier tube efface le resultat`() = runTest(dispatcher) {
        val model = model()
        advanceUntilIdle()
        model.onIntent(MixerIntent.Add(umber))
        advanceUntilIdle()
        assertNotNull(model.state.value.result)

        model.onIntent(MixerIntent.Remove(umber))
        advanceUntilIdle()
        assertNull(model.state.value.result)
    }

    @Test
    fun `le sens inverse propose des recettes pour une teinte visee`() = runTest(dispatcher) {
        val model = model()
        advanceUntilIdle()

        model.onIntent(MixerIntent.AimAt("#4A3427"))
        advanceUntilIdle()

        val suggestions = model.state.value.suggestions
        assertTrue(suggestions.isNotEmpty())
        assertTrue(!model.state.value.searching)
        // La terre d'ombre est dans la palette : un seul tube suffit.
        assertEquals("Burnt Umber", suggestions.first().parts.single().paint.name)
    }

    @Test
    fun `une teinte mal formee ne lance aucune recherche`() = runTest(dispatcher) {
        val model = model()
        advanceUntilIdle()

        // Attention : "#4A3" serait valide, c'est la forme courte que Rgb.ofHex developpe.
        // Il faut quelque chose de reellement illisible.
        model.onIntent(MixerIntent.AimAt("brun chaud"))
        advanceUntilIdle()

        assertTrue(model.state.value.suggestions.isEmpty())
        assertTrue(!model.state.value.searching)

        // Et la forme courte, elle, est bien acceptee.
        model.onIntent(MixerIntent.AimAt("#4A3"))
        advanceUntilIdle()
        assertTrue(model.state.value.suggestions.isNotEmpty())
    }
}
