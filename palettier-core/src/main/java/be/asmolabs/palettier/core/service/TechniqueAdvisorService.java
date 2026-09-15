package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.util.List;
import org.springframework.stereotype.Service;

/** Reglages de depart et rappels pratiques pour une technique donnee. */
@Service
public class TechniqueAdvisorService {

    private final DryingTimeService dryingTimeService;

    public TechniqueAdvisorService(DryingTimeService dryingTimeService) {
        this.dryingTimeService = dryingTimeService;
    }

    /**
     * Fiche complete d'une technique.
     *
     * @param ratioAdvice formulation lisible de la dilution conseillee
     */
    public record TechniqueGuide(Technique technique,
                                 String ratioAdvice,
                                 DryingEstimate drying,
                                 List<String> tips) {
    }

    public TechniqueGuide guide(Technique technique, DryingClass dryingClass, Workshop workshop) {
        DryingContext context = DryingContext.forTechnique(technique, dryingClass, workshop);
        return new TechniqueGuide(technique, ratioAdvice(technique), dryingTimeService.estimate(context), technique.tips());
    }

    private String ratioAdvice(Technique technique) {
        if (technique.maxRatio() <= 0.0) {
            return "Huile pure, sans diluant.";
        }
        return "%s : de %.0f a %.0f %% du melange."
                .formatted(technique.defaultMedium().label(), technique.minRatio() * 100, technique.maxRatio() * 100);
    }
}
