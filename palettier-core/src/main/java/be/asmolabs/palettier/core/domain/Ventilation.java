package be.asmolabs.palettier.core.domain;

/** Renouvellement d'air autour de la piece pendant le sechage. */
public enum Ventilation {

    CONFINED("Boite fermee / vitrine", 1.30),
    NORMAL("Piece normale", 1.00),
    GOOD("Bien aere ou courant d'air", 0.82);

    private final String label;
    private final double factor;

    Ventilation(String label, double factor) {
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
