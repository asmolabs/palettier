package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.domain.Ventilation;
import java.time.Duration;
import java.util.List;

/** Types de valeur echanges autour du sechage. */
public final class DryingModels {

    private DryingModels() {
    }

    /** Conditions de l'atelier. */
    public record Workshop(double temperatureCelsius, double relativeHumidity, Ventilation ventilation) {

        public static Workshop standard() {
            return new Workshop(20, 50, Ventilation.NORMAL);
        }
    }

    /** Tout ce qui determine le sechage d'une couche. */
    public record DryingContext(DryingClass dryingClass,
                                Medium medium,
                                double mediumRatio,
                                LayerThickness thickness,
                                Workshop workshop) {

        public DryingContext {
            mediumRatio = Math.clamp(mediumRatio, 0.0, 1.0);
        }

        /** Contexte pre-rempli avec les reglages usuels de la technique. */
        public static DryingContext forTechnique(Technique technique, DryingClass dryingClass, Workshop workshop) {
            return new DryingContext(dryingClass, technique.defaultMedium(),
                    technique.defaultRatio(), technique.typicalThickness(), workshop);
        }
    }

    /**
     * Jalons de sechage d'une couche.
     *
     * @param openTime   duree pendant laquelle la couche reste travaillable (fondus, dot fading)
     * @param touchDry   sec au toucher, la couche ne marque plus
     * @param recoat     on peut poser la couche suivante sans relever celle-ci
     * @param throughDry sec a coeur, manipulation et masquage possibles
     * @param fullCure   polymerisation complete, avant vernis final
     */
    public record DryingEstimate(Duration openTime,
                                 Duration touchDry,
                                 Duration recoat,
                                 Duration throughDry,
                                 Duration fullCure,
                                 List<String> advice) {
    }
}
