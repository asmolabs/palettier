package be.asmolabs.palettier.domain.recipe

import be.asmolabs.palettier.domain.drying.DryingContext
import be.asmolabs.palettier.domain.drying.DryingEstimate
import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import kotlin.time.Duration

/**
 * Deroule une recette dans le temps.
 *
 * <p>L'interet a l'huile est de voir d'un coup d'oeil combien de seances une recette
 * represente reellement : l'essentiel du planning y est fait d'attente, et une suite
 * d'etapes qui se lit en trente secondes peut demander trois semaines.</p>
 */
class RecipeTimelineService(private val dryingTimeService: DryingTimeService = DryingTimeService()) {

    /**
     * Une etape replacee dans le planning.
     *
     * @param startOffset delai entre le debut de la recette et le debut de cette etape
     * @param waitAfter   attente imposee avant l'etape suivante
     */
    data class TimelineEntry(
        val position: Int,
        val step: RecipeStep,
        val startOffset: Duration,
        val drying: DryingEstimate,
        val waitAfter: Duration,
    )

    /** Planning complet d'une recette. */
    data class Timeline(
        val recipe: Recipe,
        val entries: List<TimelineEntry>,
        val totalActiveSpan: Duration,
        val untilVarnish: Duration,
    )

    /**
     * @param dryingClass classe de sechage retenue pour toutes les etapes, faute de
     *                    connaitre le tube exact employe a chacune
     */
    fun plan(recipe: Recipe, dryingClass: DryingClass, workshop: Workshop): Timeline {
        val entries = ArrayList<TimelineEntry>(recipe.steps.size)
        var offset = Duration.ZERO
        var last: DryingEstimate? = null

        for (i in recipe.steps.indices) {
            val step = recipe.steps[i]
            val estimate = dryingTimeService.estimate(
                DryingContext(dryingClass, step.medium, step.mediumRatio, step.thickness, workshop)
            )

            // Une technique qui demande un support sec a coeur attend le sechage complet
            // de la couche precedente ; les autres se contentent du recouvrement.
            val wait = if (i == recipe.steps.lastIndex) Duration.ZERO
            else if (recipe.steps[i + 1].technique.requiresCuredBase) estimate.throughDry
            else estimate.recoat

            entries += TimelineEntry(i + 1, step, offset, estimate, wait)
            offset += wait
            last = estimate
        }

        val untilVarnish = last?.let { offset + it.fullCure } ?: Duration.ZERO
        return Timeline(recipe, entries, offset, untilVarnish)
    }
}
