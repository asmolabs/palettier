package be.asmolabs.palettier.core.domain;

import java.util.List;

/**
 * Technique a l'huile appliquee sur figurine, avec ses reglages de depart.
 *
 * <p>Les ratios sont des points de depart eprouves, pas des dogmes : ils servent a
 * pre-remplir le simulateur, l'utilisateur ajuste ensuite.</p>
 */
public enum Technique {

    OIL_WASH("Jus a l'huile",
            Medium.ODORLESS_THINNER, 0.80, 0.92, LayerThickness.GLAZE, true,
            List.of("Le vernis brillant en dessous doit etre sec depuis 24 h au minimum.",
                    "Charger le pinceau puis l'essuyer : le jus doit couler, pas s'etaler.",
                    "Retirer l'exces au coton-tige a peine humide de diluant avant la prise.")),

    PIN_WASH("Jus capillaire (pin wash)",
            Medium.ODORLESS_THINNER, 0.85, 0.94, LayerThickness.GLAZE, true,
            List.of("Poser la pointe du pinceau dans le creux et laisser la capillarite faire le trajet.",
                    "Un seul passage par ligne : repasser dessus casse le depot.")),

    FILTER("Filtre",
            Medium.ODORLESS_THINNER, 0.88, 0.96, LayerThickness.GLAZE, true,
            List.of("Tres dilue : le filtre unifie une teinte, il ne la remplace pas.",
                    "Deux filtres legers valent mieux qu'un seul charge.")),

    GLAZE("Glacis",
            Medium.ODORLESS_THINNER, 0.70, 0.88, LayerThickness.GLAZE, true,
            List.of("Privilegier une huile transparente : une opaque tue la profondeur.",
                    "Laisser secher entre deux glacis, sinon la couche precedente se releve.")),

    BLENDING("Fondu / degrade",
            Medium.NONE, 0.00, 0.25, LayerThickness.THIN, true,
            List.of("Travailler dans le temps ouvert : une fois la prise commencee, arreter.",
                    "Pinceau propre et sec pour tirer la transition, essuye a chaque passage.")),

    DOT_FADING("Dot fading",
            Medium.NONE, 0.00, 0.20, LayerThickness.THIN, true,
            List.of("Poser des points minuscules de plusieurs teintes, puis les tirer vers le bas.",
                    "Moins de peinture que ce que l'on croit : le pinceau doit etre presque sec.")),

    STREAKING_GRIME("Coulures et salissures",
            Medium.ODORLESS_THINNER, 0.60, 0.85, LayerThickness.THIN, true,
            List.of("Tracer le trait, laisser mordre une a deux minutes, puis tirer vers le bas.",
                    "Suivre le sens de l'eau : verticales sur les flancs, en eventail sous les rivets.")),

    OIL_RENDERING("Oil Paint Rendering (OPR)",
            Medium.ODORLESS_THINNER, 0.50, 0.80, LayerThickness.THIN, true,
            List.of("Poser clair et fonce cote a cote, puis fondre la frontiere.",
                    "Construire en plusieurs seances plutot qu'en une couche chargee.")),

    BASE_LAYER("Aplat de base",
            Medium.ODORLESS_THINNER, 0.10, 0.35, LayerThickness.NORMAL, false,
            List.of("A l'huile, un aplat de base reste rare : il seche lentement et bloque la suite.",
                    "Utile surtout sur les grandes surfaces lisses (bustes, 1/10).")),

    HIGHLIGHT("Eclaircis et points lumineux",
            Medium.NONE, 0.00, 0.20, LayerThickness.THIN, true,
            List.of("Une huile opaque tient mieux le point lumineux qu'une transparente.",
                    "Le blanc de titane seche tres lentement : prevoir la couche suivante a distance."));

    private final String label;
    private final Medium defaultMedium;
    private final double minRatio;
    private final double maxRatio;
    private final LayerThickness typicalThickness;
    private final boolean requiresCuredBase;
    private final List<String> tips;

    Technique(String label, Medium defaultMedium, double minRatio, double maxRatio,
              LayerThickness typicalThickness, boolean requiresCuredBase, List<String> tips) {
        this.label = label;
        this.defaultMedium = defaultMedium;
        this.minRatio = minRatio;
        this.maxRatio = maxRatio;
        this.typicalThickness = typicalThickness;
        this.requiresCuredBase = requiresCuredBase;
        this.tips = List.copyOf(tips);
    }

    public String label() {
        return label;
    }

    public Medium defaultMedium() {
        return defaultMedium;
    }

    public double minRatio() {
        return minRatio;
    }

    public double maxRatio() {
        return maxRatio;
    }

    public double defaultRatio() {
        return (minRatio + maxRatio) / 2.0;
    }

    public LayerThickness typicalThickness() {
        return typicalThickness;
    }

    /** Indique si la couche precedente doit etre completement seche avant d'appliquer celle-ci. */
    public boolean requiresCuredBase() {
        return requiresCuredBase;
    }

    public List<String> tips() {
        return tips;
    }

    /**
     * Retrouve une technique par son libelle.
     *
     * <p>Les plans conservent le nom de la technique en clair plutot que la constante :
     * il provient parfois d'un modele de langage ou d'une saisie libre, et doit rester
     * lisible meme lorsqu'il ne correspond a rien de connu.</p>
     *
     * @return la technique correspondante, ou {@code fallback} si le libelle est inconnu
     */
    public static Technique byLabel(String label, Technique fallback) {
        if (label == null || label.isBlank()) {
            return fallback;
        }
        for (Technique technique : values()) {
            if (technique.label.equalsIgnoreCase(label.trim())) {
                return technique;
            }
        }
        return fallback;
    }

    @Override
    public String toString() {
        return label;
    }
}
