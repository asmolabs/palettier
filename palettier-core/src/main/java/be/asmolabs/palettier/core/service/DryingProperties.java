package be.asmolabs.palettier.core.service;

import org.springframework.boot.context.properties.ConfigurationProperties;

/**
 * Parametres du modele de sechage, ajustables dans {@code application.yaml} sous
 * {@code palettier.drying}. Les valeurs par defaut correspondent a un atelier a 20 degres
 * et 50 % d'humidite relative.
 *
 * @param referenceTemperature   temperature de reference, en degres Celsius
 * @param temperatureHalvingStep ecart de temperature qui divise le temps de sechage par deux
 * @param referenceHumidity      humidite relative de reference, en pourcentage
 * @param humiditySensitivity    sensibilite a l'ecart d'humidite (0 = aucun effet)
 * @param openTimeShare          part du temps de sechage au toucher pendant laquelle la couche reste travaillable
 * @param recoatFactor           multiplicateur du sechage au toucher pour pouvoir recouvrir
 * @param throughDryFactor       multiplicateur du sechage au toucher pour un sechage a coeur
 * @param fullCureFactor         multiplicateur du sechage au toucher pour une polymerisation complete
 */
@ConfigurationProperties(prefix = "palettier.drying")
public record DryingProperties(double referenceTemperature,
                               double temperatureHalvingStep,
                               double referenceHumidity,
                               double humiditySensitivity,
                               double openTimeShare,
                               double recoatFactor,
                               double throughDryFactor,
                               double fullCureFactor) {

    public DryingProperties() {
        this(20.0, 10.0, 50.0, 0.6, 0.22, 1.8, 5.0, 22.0);
    }
}
