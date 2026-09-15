package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.color.Rgb;
import javafx.scene.control.Label;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Color;

/**
 * Pastille de couleur. Le libelle bascule du noir au blanc selon la luminance du fond,
 * pour rester lisible sur un blanc de titane comme sur un noir de mars.
 */
public class ColorSwatch extends StackPane {

    private final Label label = new Label();

    public ColorSwatch(double size) {
        this(size, size);
    }

    public ColorSwatch(double width, double height) {
        getStyleClass().add("color-swatch");
        setMinSize(width, height);
        setPrefSize(width, height);
        label.getStyleClass().add("color-swatch-label");
        getChildren().add(label);
    }

    public void setColor(Rgb color) {
        setColor(color, null);
    }

    /** @param caption texte affiche par-dessus la pastille, ou {@code null} pour aucun */
    public void setColor(Rgb color, String caption) {
        setStyle("-fx-background-color: %s;".formatted(color.toHex()));
        label.setText(caption == null ? "" : caption);
        label.setTextFill(color.relativeLuminance() > 0.35 ? Color.web("#1A1A1A") : Color.web("#F2F2F2"));
    }
}
