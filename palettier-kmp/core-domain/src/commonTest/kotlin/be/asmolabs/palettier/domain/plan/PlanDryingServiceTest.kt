package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.mix.MixSuggestion
import be.asmolabs.palettier.domain.mix.PaintPart
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Technique
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Traduction fidele de PlanDryingServiceTest cote Java. */
class PlanDryingServiceTest {

    private val service = PlanDryingService()

    private val umber = Paint(
        id = 1, brand = "W&N", name = "Burnt Umber", code = "076", hexColor = "#4A3427",
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.FAST, tintingStrength = 0.8,
        pigments = setOf("PBr7"),
    )
    private val cadmium = Paint(
        id = 2, brand = "W&N", name = "Cadmium Yellow", code = "108", hexColor = "#F5B800",
        opacity = Opacity.OPAQUE, dryingClass = DryingClass.VERY_SLOW, tintingStrength = 0.8,
        pigments = setOf("PY35"),
    )

    private fun layer(role: String, technique: Technique, paint: Paint): PaintingPlan.Layer {
        val colour = paint.color
        val recipe = MixSuggestion(listOf(PaintPart.of(paint, 1)), colour, 0.5)
        return PaintingPlan.Layer(role, colour, technique.label, "", recipe, colour, 0.5)
    }

    private fun zone(name: String, highlightPaint: Paint) = PaintingPlan.Zone(
        name, "", "",
        base = layer("Base", Technique.BASE_LAYER, umber),
        shadows = listOf(layer("Ombre 1", Technique.GLAZE, umber)),
        highlights = listOf(layer("Lumiere 1", Technique.HIGHLIGHT, highlightPaint)),
    )

    @Test
    fun `la vitesse de sechage vient des pigments du melange, on ne la demande pas`() {
        val schedule = service.schedule(
            PaintingPlan("Buste", "Zorn", "", listOf(zone("Peau", cadmium))),
            Workshop.standard(),
        )

        assertEquals(
            listOf(DryingClass.FAST, DryingClass.FAST, DryingClass.VERY_SLOW),
            schedule.zones.first().layers.map { it.dryingClass },
        )
    }

    @Test
    fun `un cadmium dans une lumiere allonge le planning de la zone`() {
        val fast = service.schedule(
            PaintingPlan("B", "Z", "", listOf(zone("Peau", umber))), Workshop.standard()
        )
        val slow = service.schedule(
            PaintingPlan("B", "Z", "", listOf(zone("Peau", cadmium))), Workshop.standard()
        )

        // Le cadmium est en derniere couche : il ne rallonge pas l'enchainement, mais la
        // zone porte bien sa classe de sechage.
        assertEquals(DryingClass.FAST, fast.zones.first().slowest())
        assertEquals(DryingClass.VERY_SLOW, slow.zones.first().slowest())
    }

    @Test
    fun `mener les zones de front est plus court que les enchainer`() {
        val plan = PaintingPlan("Buste", "Zorn", "", listOf(zone("Peau", umber), zone("Cape", umber)))

        val schedule = service.schedule(plan, Workshop.standard())

        assertTrue(schedule.parallel < schedule.sequential)
        assertTrue(schedule.advice.any { "de front" in it })
    }

    @Test
    fun `une zone au pigment lent est signalee`() {
        val schedule = service.schedule(
            PaintingPlan("Buste", "Zorn", "", listOf(zone("Peau", cadmium))), Workshop.standard()
        )

        assertTrue(schedule.advice.any { "tres lent" in it }, schedule.advice.toString())
    }

    @Test
    fun `un plan vide ne produit ni planning ni conseil`() {
        val schedule = service.schedule(PaintingPlan("Rien", "", "", emptyList()), Workshop.standard())

        assertTrue(schedule.zones.isEmpty())
        assertTrue(schedule.advice.isEmpty())
        assertEquals(kotlin.time.Duration.ZERO, schedule.parallel)
    }
}
