package be.asmolabs.palettier.core.domain;

/**
 * Diluant ou medium ajoute a l'huile.
 *
 * <p>{@code dryingFactorAtFullRatio} est le multiplicateur applique au temps de sechage
 * quand le melange contient la proportion maximale utile de ce medium. Entre 0 et cette
 * valeur, l'effet est interpole lineairement sur le ratio.</p>
 */
public enum Medium {

    NONE("Pur (sortie de tube)", 1.00, 0.00, "Pate epaisse : reserve aux empatements et aux textures."),
    ODORLESS_THINNER("Diluant inodore", 0.55, 0.85, "Le standard pour les jus et les fondus sur figurine."),
    WHITE_SPIRIT("White spirit", 0.48, 0.85, "Plus agressif : verifier la tenue du vernis en dessous."),
    TURPENTINE("Essence de terebenthine", 0.50, 0.85, "Evapore vite, odeur forte, bonne accroche."),
    ALKYD_MEDIUM("Medium alkyde (type Liquin)", 0.35, 0.50, "Accelere fortement et lisse le fondu."),
    LINSEED_OIL("Huile de lin", 1.90, 0.40, "Rallonge le temps ouvert, jaunit legerement en couche epaisse."),
    WALNUT_OIL("Huile de noix", 2.20, 0.40, "Temps ouvert tres long, ne jaunit presque pas."),
    STAND_OIL("Huile stand", 2.60, 0.30, "Tres lent, tres lissant : pour les fondus longs."),
    COBALT_DRIER("Siccatif au cobalt", 0.28, 0.05, "Quelques gouttes suffisent ; au-dela, la couche craquelle.");

    private final String label;
    private final double dryingFactorAtFullRatio;
    private final double maxUsefulRatio;
    private final String advice;

    Medium(String label, double dryingFactorAtFullRatio, double maxUsefulRatio, String advice) {
        this.label = label;
        this.dryingFactorAtFullRatio = dryingFactorAtFullRatio;
        this.maxUsefulRatio = maxUsefulRatio;
        this.advice = advice;
    }

    public String label() {
        return label;
    }

    /** Proportion de medium, entre 0 et 1, au-dela de laquelle la couche devient instable. */
    public double maxUsefulRatio() {
        return maxUsefulRatio;
    }

    public String advice() {
        return advice;
    }

    /**
     * Multiplicateur de sechage pour une proportion de medium donnee.
     *
     * @param ratio part de medium dans le melange, entre 0 et 1
     */
    public double dryingFactor(double ratio) {
        double r = Math.clamp(ratio, 0.0, 1.0);
        double reference = Math.max(maxUsefulRatio, 1e-6);
        double progress = Math.min(r / reference, 1.0);
        return 1.0 + (dryingFactorAtFullRatio - 1.0) * progress;
    }

    @Override
    public String toString() {
        return label;
    }
}
