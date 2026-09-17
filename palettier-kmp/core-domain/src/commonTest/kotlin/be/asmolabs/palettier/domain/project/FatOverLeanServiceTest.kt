package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.recipe.Recipe
import be.asmolabs.palettier.domain.recipe.RecipeStep
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * La regle doit se taire sur ce qui se pratique tous les jours, et parler quand la piece
 * est reellement en jeu. Une alerte qui crie a chaque glacis serait desactivee le premier
 * soir, et ne servirait plus a rien le jour ou elle aurait raison.
 */
class FatOverLeanServiceTest {

    private val service = FatOverLeanService()

    private fun pieceWith(vararg techniques: String) = project(
        "Piece",
        zone("Visage", *techniques.mapIndexed { i, t -> layer("Couche ${i + 1}", technique = t) }.toTypedArray()),
    )

    @Test
    fun `un plan tout en glacis ne declenche rien`() {
        assertTrue(
            service.inspect(
                pieceWith(Technique.GLAZE.label, Technique.GLAZE.label, Technique.GLAZE.label)
            ).isEmpty()
        )
    }

    @Test
    fun `un glacis sur un aplat de base ne declenche rien, un voile ne tire pas`() {
        // Le glacis est pourtant bien plus maigre que l'aplat : c'est l'epaisseur qui
        // decide, et c'est la pratique courante sur figurine.
        assertTrue(service.fatnessOf(Technique.GLAZE.label) < service.fatnessOf(Technique.BASE_LAYER.label))
        assertTrue(service.isVeil(Technique.GLAZE.label))

        assertTrue(service.inspect(pieceWith(Technique.BASE_LAYER.label, Technique.GLAZE.label)).isEmpty())
    }

    @Test
    fun `des coulures maigres sur un fondu gras, a epaisseur egale, sont signalees`() {
        val risks = service.inspect(pieceWith(Technique.BLENDING.label, Technique.STREAKING_GRIME.label))

        assertEquals(1, risks.size)
        val risk = risks.first()
        assertEquals("Visage", risk.where)
        assertEquals("Couche 1", risk.under)
        assertEquals("Couche 2", risk.over)
        assertTrue(risk.drop > 0.4)
        assertTrue(Technique.STREAKING_GRIME.label in risk.explanation)
        assertTrue(Technique.BLENDING.label in risk.explanation)
        assertTrue("tirera" in risk.explanation)
    }

    @Test
    fun `l'ordre inverse, du maigre vers le gras, est la bonne facon de faire`() {
        assertTrue(
            service.inspect(pieceWith(Technique.STREAKING_GRIME.label, Technique.BLENDING.label)).isEmpty()
        )
    }

    @Test
    fun `une technique inconnue ne fait pas echouer la verification`() {
        assertTrue(service.inspect(pieceWith("Sfumato maison", Technique.GLAZE.label)).isEmpty())
    }

    @Test
    fun `une recette est verifiee sur ses valeurs saisies, pas sur des moyennes`() {
        // Deux etapes au meme medium et a la meme epaisseur : c'est la dilution saisie,
        // et elle seule, qui fait passer la seconde du gras au maigre.
        val recipe = Recipe(name = "Cape rouge", subject = "Tissu").
            withStep(RecipeStep(technique = Technique.BLENDING, paintMix = "Rouge + noir",
                medium = Medium.LINSEED_OIL, mediumRatio = 0.30, thickness = LayerThickness.NORMAL)).
            withStep(RecipeStep(technique = Technique.BLENDING, paintMix = "Rouge pur",
                medium = Medium.ODORLESS_THINNER, mediumRatio = 0.80, thickness = LayerThickness.NORMAL))

        val risks = service.inspect(recipe)

        assertEquals(1, risks.size)
        assertEquals("Cape rouge", risks.first().where)
        assertEquals("Etape 1", risks.first().under)
        assertEquals("Etape 2", risks.first().over)
    }

    @Test
    fun `la meme recette dans le bon ordre ne declenche rien`() {
        val recipe = Recipe(name = "Cape rouge", subject = "Tissu").
            withStep(RecipeStep(technique = Technique.BLENDING, paintMix = "Rouge pur",
                medium = Medium.ODORLESS_THINNER, mediumRatio = 0.80, thickness = LayerThickness.NORMAL)).
            withStep(RecipeStep(technique = Technique.BLENDING, paintMix = "Rouge + noir",
                medium = Medium.LINSEED_OIL, mediumRatio = 0.30, thickness = LayerThickness.NORMAL))

        assertTrue(service.inspect(recipe).isEmpty())
    }
}
