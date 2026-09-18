package be.asmolabs.palettier.domain.recipe

import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration

class RecipeTimelineServiceTest {

    private val service = RecipeTimelineService()

    private fun recipe(vararg steps: RecipeStep) =
        Recipe(name = "Casque allemand", subject = "Metal", steps = steps.toList())

    @Test
    fun `chaque etape recoit son rang, son depart et son attente`() {
        val timeline = service.plan(
            recipe(
                RecipeStep.of(Technique.BASE_LAYER, "Vert olive"),
                RecipeStep.of(Technique.OIL_WASH, "Terre d'ombre"),
                RecipeStep.of(Technique.HIGHLIGHT, "Ocre"),
            ),
            DryingClass.MEDIUM, Workshop.standard(),
        )

        assertEquals(listOf(1, 2, 3), timeline.entries.map { it.position })
        assertEquals(Duration.ZERO, timeline.entries.first().startOffset)
        // La derniere n'attend plus rien : il n'y a rien apres elle.
        assertEquals(Duration.ZERO, timeline.entries.last().waitAfter)
        // Les departs se suivent dans l'ordre.
        assertTrue(timeline.entries[1].startOffset > timeline.entries[0].startOffset)
    }

    @Test
    fun `une technique qui exige un support sec a coeur allonge l'attente`() {
        // L'aplat de base est la seule technique qui se contente d'un support recouvrable.
        val lenient = service.plan(
            recipe(RecipeStep.of(Technique.BASE_LAYER, "A"), RecipeStep.of(Technique.BASE_LAYER, "B")),
            DryingClass.MEDIUM, Workshop.standard(),
        )
        val demanding = service.plan(
            recipe(RecipeStep.of(Technique.BASE_LAYER, "A"), RecipeStep.of(Technique.GLAZE, "B")),
            DryingClass.MEDIUM, Workshop.standard(),
        )

        assertTrue(demanding.entries.first().waitAfter > lenient.entries.first().waitAfter)
    }

    @Test
    fun `le vernis final vient longtemps apres la derniere etape`() {
        val timeline = service.plan(
            recipe(RecipeStep.of(Technique.GLAZE, "Jus")),
            DryingClass.MEDIUM, Workshop.standard(),
        )

        assertEquals(Duration.ZERO, timeline.totalActiveSpan, "une seule etape, aucune attente entre")
        assertTrue(timeline.untilVarnish > Duration.ZERO, "la polymerisation, elle, prend du temps")
    }

    @Test
    fun `un atelier froid allonge tout le planning`() {
        val steps = arrayOf(RecipeStep.of(Technique.BASE_LAYER, "A"), RecipeStep.of(Technique.GLAZE, "B"))
        val warm = service.plan(recipe(*steps), DryingClass.MEDIUM, Workshop.standard())
        val cold = service.plan(
            recipe(*steps), DryingClass.MEDIUM,
            Workshop(10.0, 80.0, be.asmolabs.palettier.domain.paint.Ventilation.CONFINED),
        )

        assertTrue(cold.totalActiveSpan > warm.totalActiveSpan)
        assertTrue(cold.untilVarnish > warm.untilVarnish)
    }

    @Test
    fun `une recette vide ne produit aucun planning`() {
        val timeline = service.plan(recipe(), DryingClass.MEDIUM, Workshop.standard())

        assertTrue(timeline.entries.isEmpty())
        assertEquals(Duration.ZERO, timeline.totalActiveSpan)
        assertEquals(Duration.ZERO, timeline.untilVarnish)
    }

    @Test
    fun `l'epaisseur saisie compte, pas seulement la technique`() {
        val thin = service.plan(
            recipe(
                RecipeStep(technique = Technique.BASE_LAYER, medium = Medium.NONE, mediumRatio = 0.0,
                    thickness = LayerThickness.GLAZE),
                RecipeStep.of(Technique.GLAZE, "B"),
            ),
            DryingClass.MEDIUM, Workshop.standard(),
        )
        val thick = service.plan(
            recipe(
                RecipeStep(technique = Technique.BASE_LAYER, medium = Medium.NONE, mediumRatio = 0.0,
                    thickness = LayerThickness.IMPASTO),
                RecipeStep.of(Technique.GLAZE, "B"),
            ),
            DryingClass.MEDIUM, Workshop.standard(),
        )

        assertTrue(thick.entries.first().waitAfter > thin.entries.first().waitAfter * 5)
    }
}
