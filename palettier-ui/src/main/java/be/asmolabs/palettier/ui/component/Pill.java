package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Opacity;
import javafx.scene.control.Label;

/** Etiquette arrondie, utilisee dans les tableaux la ou une couleur vaut mieux qu'un mot. */
public final class Pill {

    private Pill() {
    }

    public static Label of(String text, String modifier) {
        Label label = new Label(text);
        label.getStyleClass().addAll("pill", modifier);
        return label;
    }

    /** Du transparent a l'opaque, la pastille se remplit. */
    public static Label forOpacity(Opacity opacity) {
        String modifier = switch (opacity) {
            case TRANSPARENT -> "pill-ghost";
            case SEMI_TRANSPARENT -> "pill-soft";
            case SEMI_OPAQUE -> "pill-mid";
            case OPAQUE -> "pill-solid";
        };
        return of(opacity.label(), modifier);
    }

    /** Du vert au rouge : plus c'est lent, plus cela contraint le planning. */
    public static Label forDrying(DryingClass dryingClass) {
        String modifier = switch (dryingClass) {
            case FAST -> "pill-fast";
            case MEDIUM -> "pill-medium";
            case SLOW -> "pill-slow";
            case VERY_SLOW -> "pill-very-slow";
        };
        return of(dryingClass.label(), modifier);
    }
}
