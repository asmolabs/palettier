package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Recipe;
import be.asmolabs.palettier.core.domain.RecipeStep;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.springframework.stereotype.Service;

/**
 * Deroule une recette dans le temps : a l'huile, l'essentiel du planning est fait
 * d'attente, et c'est cette attente qu'il faut voir avant de commencer.
 */
@Service
public class RecipeTimelineService {

    private final DryingTimeService dryingTimeService;

    public RecipeTimelineService(DryingTimeService dryingTimeService) {
        this.dryingTimeService = dryingTimeService;
    }

    /**
     * Une etape replacee dans le planning.
     *
     * @param startOffset delai entre le debut de la recette et le debut de cette etape
     * @param waitAfter   attente imposee avant l'etape suivante
     */
    public record TimelineEntry(int position,
                                RecipeStep step,
                                Duration startOffset,
                                DryingEstimate drying,
                                Duration waitAfter) {
    }

    /** Planning complet d'une recette. */
    public record Timeline(Recipe recipe, List<TimelineEntry> entries, Duration totalActiveSpan, Duration untilVarnish) {
    }

    /**
     * @param dryingClass classe de sechage retenue pour toutes les etapes, faute de connaitre
     *                    le tube exact utilise a chacune
     */
    public Timeline plan(Recipe recipe, DryingClass dryingClass, Workshop workshop) {
        List<TimelineEntry> entries = new ArrayList<>();
        Duration offset = Duration.ZERO;
        DryingEstimate last = null;

        List<RecipeStep> steps = recipe.getSteps();
        for (int i = 0; i < steps.size(); i++) {
            RecipeStep step = steps.get(i);
            DryingEstimate estimate = dryingTimeService.estimate(
                    new DryingContext(dryingClass, step.getMedium(), step.getMediumRatio(), step.getThickness(), workshop));

            boolean isLast = i == steps.size() - 1;
            Duration wait = isLast
                    ? Duration.ZERO
                    : waitBefore(steps.get(i + 1), estimate);

            entries.add(new TimelineEntry(i + 1, step, offset, estimate, wait));
            offset = offset.plus(wait);
            last = estimate;
        }

        Duration untilVarnish = last == null ? Duration.ZERO : offset.plus(last.fullCure());
        return new Timeline(recipe, List.copyOf(entries), offset, untilVarnish);
    }

    /**
     * Une technique qui demande un support sec a coeur attend le sechage complet de la
     * couche precedente ; les autres se contentent du delai de recouvrement.
     */
    private Duration waitBefore(RecipeStep next, DryingEstimate previous) {
        return next.getTechnique().requiresCuredBase() ? previous.throughDry() : previous.recoat();
    }
}
