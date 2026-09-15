package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.springframework.stereotype.Service;

/**
 * Estime les jalons de sechage d'une couche d'huile.
 *
 * <p>Le modele part du temps de reference du pigment le plus lent, puis applique
 * successivement l'effet du medium, de l'epaisseur, de la temperature, de l'humidite
 * et de la ventilation. Ce sont des ordres de grandeur destines a planifier une seance,
 * pas une mesure : sur une piece reelle, on verifie toujours du doigt sur le socle.</p>
 */
@Service
public class DryingTimeService {

    private final DryingProperties properties;

    public DryingTimeService(DryingProperties properties) {
        this.properties = properties;
    }

    public DryingEstimate estimate(DryingContext context) {
        double hours = context.dryingClass().referenceTouchDry().toMinutes() / 60.0
                * context.medium().dryingFactor(context.mediumRatio())
                * context.thickness().factor()
                * temperatureFactor(context.workshop().temperatureCelsius())
                * humidityFactor(context.workshop().relativeHumidity())
                * context.workshop().ventilation().factor();

        Duration touchDry = hoursToDuration(hours);

        return new DryingEstimate(
                hoursToDuration(hours * properties.openTimeShare()),
                touchDry,
                hoursToDuration(hours * properties.recoatFactor()),
                hoursToDuration(hours * properties.throughDryFactor()),
                hoursToDuration(hours * properties.fullCureFactor()),
                advice(context));
    }

    /** La vitesse d'oxydation double environ a chaque palier de temperature gagne. */
    private double temperatureFactor(double celsius) {
        double delta = properties.referenceTemperature() - celsius;
        return Math.pow(2, delta / properties.temperatureHalvingStep());
    }

    /** Une atmosphere humide ralentit la prise de l'huile, une atmosphere seche l'accelere. */
    private double humidityFactor(double relativeHumidity) {
        double delta = relativeHumidity - properties.referenceHumidity();
        return Math.max(0.5, 1.0 + delta / 100.0 * properties.humiditySensitivity());
    }

    private List<String> advice(DryingContext context) {
        List<String> advice = new ArrayList<>();
        Workshop workshop = context.workshop();

        if (context.mediumRatio() > context.medium().maxUsefulRatio() && context.medium() != Medium.NONE) {
            advice.add("Proportion de %s au-dela de la limite utile (%.0f %%) : le liant ne tient plus le pigment."
                    .formatted(context.medium().label().toLowerCase(), context.medium().maxUsefulRatio() * 100));
        }
        if (workshop.temperatureCelsius() < 15) {
            advice.add("Sous 15 degres, l'huile prend tres lentement et peut rester collante plusieurs jours.");
        }
        if (workshop.temperatureCelsius() > 28) {
            advice.add("Au-dela de 28 degres, le temps ouvert se reduit fortement : travaillez par petites zones.");
        }
        if (workshop.relativeHumidity() > 70) {
            advice.add("Humidite elevee : le sechage tire en longueur, evitez de vernir juste apres.");
        }
        if (context.thickness().compareTo(LayerThickness.THICK) >= 0) {
            advice.add("Couche epaisse : la peau se forme avant le coeur. Attendez le sechage a coeur avant toute manipulation.");
        }
        if (context.medium() == Medium.COBALT_DRIER && context.mediumRatio() > 0.03) {
            advice.add("Le siccatif au cobalt s'emploie a la goutte : au-dela, la couche craquelle en vieillissant.");
        }
        if (context.dryingClass() == DryingClass.VERY_SLOW) {
            advice.add(context.dryingClass().typicalPigments()
                    + " Ces pigments restent ouverts longtemps : parfait pour les fondus, penible pour les couches suivantes.");
        }

        advice.add(context.medium().advice());
        return List.copyOf(advice);
    }

    private static Duration hoursToDuration(double hours) {
        return Duration.ofMinutes(Math.round(hours * 60));
    }
}
