package be.asmolabs.palettier.core.domain;

/**
 * Diluant ou medium ajoute a l'huile.
 *
 * <p>{@code dryingFactorAtFullRatio} est le multiplicateur applique au temps de sechage
 * quand le melange contient la proportion maximale utile de ce medium. Entre 0 et cette
 * valeur, l'effet est interpole lineairement sur le ratio.</p>
 */
public enum Medium {

    NONE("Pur (sortie de tube)", 1.00, 0.00, 0, "Pate epaisse : reserve aux empatements et aux textures."),
    ODORLESS_THINNER("Diluant inodore", 0.55, 0.85, -1, "Le standard pour les jus et les fondus sur figurine."),
    WHITE_SPIRIT("White spirit", 0.48, 0.85, -1, "Plus agressif : verifier la tenue du vernis en dessous."),
    TURPENTINE("Essence de terebenthine", 0.50, 0.85, -1, "Evapore vite, odeur forte, bonne accroche."),
    ALKYD_MEDIUM("Medium alkyde (type Liquin)", 0.35, 0.50, 1, "Accelere fortement et lisse le fondu."),
    LINSEED_OIL("Huile de lin", 1.90, 0.40, 1, "Rallonge le temps ouvert, jaunit legerement en couche epaisse."),
    WALNUT_OIL("Huile de noix", 2.20, 0.40, 1, "Temps ouvert tres long, ne jaunit presque pas."),
    STAND_OIL("Huile stand", 2.60, 0.30, 1, "Tres lent, tres lissant : pour les fondus longs."),
    COBALT_DRIER("Siccatif au cobalt", 0.28, 0.05, 0, "Quelques gouttes suffisent ; au-dela, la couche craquelle.");

    private final String label;
    private final double dryingFactorAtFullRatio;
    private final double maxUsefulRatio;
    private final int fatDirection;
    private final String advice;

    Medium(String label, double dryingFactorAtFullRatio, double maxUsefulRatio,
           int fatDirection, String advice) {
        this.label = label;
        this.dryingFactorAtFullRatio = dryingFactorAtFullRatio;
        this.maxUsefulRatio = maxUsefulRatio;
        this.fatDirection = fatDirection;
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

    /**
     * Part de liant de la couche, la peinture pure valant 1.
     *
     * <p>Un diluant evapore et laisse moins d'huile pour tenir le pigment : la couche
     * maigrit. Une huile ou un medium gras en apporte : elle engraisse. C'est le nombre
     * que compare la regle du gras sur maigre -- une couche ne doit pas etre plus maigre
     * que celle qu'elle recouvre, sous peine de secher plus vite qu'elle et de craqueler
     * en tirant dessus.</p>
     *
     * <p>Modele volontairement grossier : la proportion de medium suffit a ordonner deux
     * couches l'une par rapport a l'autre, ce qui est tout ce qu'on lui demande. Il ne
     * dit rien de la quantite reelle de liant, qui depend aussi du pigment.</p>
     *
     * @param ratio part de medium dans le melange, entre 0 et 1
     */
    public double fatness(double ratio) {
        return 1.0 + fatDirection * Math.clamp(ratio, 0.0, 1.0);
    }

    @Override
    public String toString() {
        return label;
    }
}
