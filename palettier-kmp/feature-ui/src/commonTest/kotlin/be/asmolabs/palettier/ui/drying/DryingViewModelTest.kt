package be.asmolabs.palettier.ui.drying

import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.paint.Ventilation
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class DryingViewModelTest {

    private fun model() = DryingViewModel(DryingTimeService())

    @Test
    fun `les jalons sont calcules des l'ouverture`() {
        val estimate = model().state.value.estimate
        assertNotNull(estimate)
        assertTrue(estimate.openTime < estimate.touchDry)
        assertTrue(estimate.recoat < estimate.throughDry)
    }

    @Test
    fun `choisir une technique repositionne ses reglages usuels`() {
        val model = model()

        model.onIntent(DryingIntent.SetTechnique(Technique.HIGHLIGHT))

        val state = model.state.value
        assertEquals(Medium.NONE, state.medium, "un point lumineux se pose pur")
        assertEquals(LayerThickness.THIN, state.thickness)
        assertEquals(Technique.HIGHLIGHT.defaultRatio, state.mediumRatio)
    }

    @Test
    fun `on peut ajuster un reglage apres avoir choisi la technique`() {
        val model = model()
        model.onIntent(DryingIntent.SetTechnique(Technique.GLAZE))
        val asChosen = model.state.value.estimate!!.touchDry

        model.onIntent(DryingIntent.SetThickness(LayerThickness.IMPASTO))

        assertEquals(LayerThickness.IMPASTO, model.state.value.thickness)
        assertTrue(model.state.value.estimate!!.touchDry > asChosen)
    }

    @Test
    fun `un atelier froid double le temps de sechage`() {
        val model = model()
        val warm = model.state.value.estimate!!.touchDry

        model.onIntent(DryingIntent.SetWorkshop(Workshop(10.0, 50.0, Ventilation.NORMAL)))

        val cold = model.state.value.estimate!!.touchDry
        assertTrue(kotlin.math.abs(cold.inWholeMinutes - warm.inWholeMinutes * 2) <= 2)
    }

    @Test
    fun `un pigment plus lent allonge tout`() {
        val model = model()
        val medium = model.state.value.estimate!!.fullCure

        model.onIntent(DryingIntent.SetDryingClass(DryingClass.VERY_SLOW))

        assertTrue(model.state.value.estimate!!.fullCure > medium)
    }
}
