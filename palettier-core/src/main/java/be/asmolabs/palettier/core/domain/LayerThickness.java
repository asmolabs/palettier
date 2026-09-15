package be.asmolabs.palettier.core.domain;

/** Epaisseur de la couche deposee : premier facteur du temps de sechage apres le pigment. */
public enum LayerThickness {

    GLAZE("Glacis / voile", 0.30),
    THIN("Couche fine", 0.60),
    NORMAL("Couche normale", 1.00),
    THICK("Couche chargee", 1.90),
    IMPASTO("Empatement", 3.40);

    private final String label;
    private final double factor;

    LayerThickness(String label, double factor) {
        this.label = label;
        this.factor = factor;
    }

    public String label() {
        return label;
    }

    public double factor() {
        return factor;
    }

    @Override
    public String toString() {
        return label;
    }
}
