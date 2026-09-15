package be.asmolabs.palettier.core.domain;

/** Pouvoir couvrant d'une huile, tel qu'indique par les fabricants sur le tube. */
public enum Opacity {

    TRANSPARENT("Transparent", "Ideal pour les glacis et les filtres : la couche du dessous reste visible."),
    SEMI_TRANSPARENT("Semi-transparent", "Bon compromis pour un jus ou un fondu leger."),
    SEMI_OPAQUE("Semi-opaque", "Couvre en deux passes, garde un peu de profondeur."),
    OPAQUE("Opaque", "Couvre en une passe : pour les aplats et les points lumineux.");

    private final String label;
    private final String advice;

    Opacity(String label, String advice) {
        this.label = label;
        this.advice = advice;
    }

    public String label() {
        return label;
    }

    public String advice() {
        return advice;
    }

    @Override
    public String toString() {
        return label;
    }
}
