package be.asmolabs.palettier.domain.drying

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Ventilation
import kotlin.math.abs
import kotlin.test.Test
import kotlin.test.assertTrue
import kotlin.time.Duration

/** Traduction fidele de DryingTimeServiceTest cote Java, valeurs comprises. */
class DryingTimeServiceTest {

    private val service = DryingTimeService()

    private fun context(
        medium: Medium = Medium.NONE,
        ratio: Double = 0.0,
        thickness: LayerThickness = LayerThickness.NORMAL,
        workshop: Workshop = Workshop.standard(),
    ) = DryingContext(DryingClass.MEDIUM, medium, ratio, thickness, workshop)

    private fun touchDry(
        medium: Medium = Medium.NONE,
        ratio: Double = 0.0,
        thickness: LayerThickness = LayerThickness.NORMAL,
        workshop: Workshop = Workshop.standard(),
    ): Duration = service.estimate(context(medium, ratio, thickness, workshop)).touchDry

    @Test
    fun `les jalons de sechage sont ordonnes du temps ouvert a la polymerisation`() {
        val estimate = service.estimate(
            context(Medium.ODORLESS_THINNER, 0.8, LayerThickness.GLAZE)
        )

        assertTrue(estimate.openTime < estimate.touchDry)
        assertTrue(estimate.touchDry < estimate.recoat)
        assertTrue(estimate.recoat < estimate.throughDry)
        assertTrue(estimate.throughDry < estimate.fullCure)
    }

    @Test
    fun `le diluant accelere le sechage, l'huile de lin le ralentit`() {
        val pure = touchDry(Medium.NONE, 0.0)
        val thinned = touchDry(Medium.ODORLESS_THINNER, 0.8)
        val oiled = touchDry(Medium.LINSEED_OIL, 0.4)

        assertTrue(thinned < pure)
        assertTrue(oiled > pure)
    }

    @Test
    fun `une couche epaisse seche nettement plus lentement qu'un glacis`() {
        val glaze = touchDry(thickness = LayerThickness.GLAZE)
        val impasto = touchDry(thickness = LayerThickness.IMPASTO)

        assertTrue(impasto > glaze * 5)
    }

    @Test
    fun `dix degres de moins doublent le temps de sechage`() {
        val warm = touchDry(workshop = Workshop(20.0, 50.0, Ventilation.NORMAL))
        val cold = touchDry(workshop = Workshop(10.0, 50.0, Ventilation.NORMAL))

        assertTrue(abs(cold.inWholeMinutes - warm.inWholeMinutes * 2) <= 2)
    }

    @Test
    fun `une dilution excessive declenche un avertissement`() {
        val estimate = service.estimate(context(Medium.ODORLESS_THINNER, 0.98, LayerThickness.GLAZE))
        assertTrue(estimate.advice.any { "limite utile" in it }, estimate.advice.toString())
    }

    @Test
    fun `un atelier froid est signale au peintre`() {
        val estimate = service.estimate(
            context(workshop = Workshop(12.0, 50.0, Ventilation.NORMAL))
        )
        assertTrue(estimate.advice.any { "15 degres" in it }, estimate.advice.toString())
    }

    @Test
    fun `la regle du gras sur maigre lit la proportion de medium`() {
        // Un diluant maigrit, une huile engraisse, la peinture pure est la reference.
        assertTrue(Medium.ODORLESS_THINNER.fatness(0.8) < Medium.NONE.fatness(0.0))
        assertTrue(Medium.LINSEED_OIL.fatness(0.4) > Medium.NONE.fatness(0.0))
    }
}
