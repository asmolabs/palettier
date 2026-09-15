package be.asmolabs.palettier.core.domain;

import java.time.Duration;

/**
 * Vitesse de sechage propre au pigment. A l'huile, c'est le pigment qui commande :
 * les terres et les siccatifs naturels oxydent vite, les cadmiums et les blancs de
 * titane restent ouverts tres longtemps.
 */
public enum DryingClass {

    FAST("Rapide", Duration.ofHours(10), "Terres d'ombre, bleu de Prusse, siccatifs naturels."),
    MEDIUM("Moyen", Duration.ofHours(20), "Terres de Sienne, ocres, bleus et verts courants."),
    SLOW("Lent", Duration.ofHours(38), "Noirs d'ivoire, blancs de titane, la plupart des rouges."),
    VERY_SLOW("Tres lent", Duration.ofHours(64), "Cadmiums, blanc de zinc, laques organiques.");

    private final String label;
    private final Duration referenceTouchDry;
    private final String typicalPigments;

    DryingClass(String label, Duration referenceTouchDry, String typicalPigments) {
        this.label = label;
        this.referenceTouchDry = referenceTouchDry;
        this.typicalPigments = typicalPigments;
    }

    public String label() {
        return label;
    }

    /** Sechage au toucher d'une couche normale, a 20 degres et 50 % d'humidite, sans medium. */
    public Duration referenceTouchDry() {
        return referenceTouchDry;
    }

    public String typicalPigments() {
        return typicalPigments;
    }

    /** La classe la plus lente commande le sechage d'un melange. */
    public DryingClass slowest(DryingClass other) {
        return compareTo(other) >= 0 ? this : other;
    }

    @Override
    public String toString() {
        return label;
    }
}
