package be.asmolabs.palettier.ui.component;

import java.util.Locale;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;

/**
 * Bloc de contenu titre. Remplace {@code TitledPane}, dont l'aspect "panneau
 * repliable" fait tres client lourd : ici le titre est une simple etiquette et le
 * cadre est porte par la feuille de style.
 */
public class Card extends VBox {

    public Card(String title, Node content) {
        this(title, null, content);
    }

    /** @param hint phrase d'explication sous le titre, ou {@code null} */
    public Card(String title, String hint, Node content) {
        this(title, hint, content, null);
    }

    /**
     * @param action commande placee a droite du titre -- modifier, supprimer -- ou
     *               {@code null}. Elle appartient a l'en-tete plutot qu'au contenu :
     *               elle agit sur le bloc entier.
     */
    public Card(String title, String hint, Node content, Node action) {
        getStyleClass().add("card");
        setSpacing(12);

        Label heading = new Label(title.toUpperCase(Locale.FRENCH));
        heading.getStyleClass().add("card-title");
        if (action == null) {
            getChildren().add(heading);
        } else {
            javafx.scene.layout.Region spacer = new javafx.scene.layout.Region();
            javafx.scene.layout.HBox.setHgrow(spacer, Priority.ALWAYS);
            javafx.scene.layout.HBox header = new javafx.scene.layout.HBox(8, heading, spacer, action);
            header.setAlignment(javafx.geometry.Pos.CENTER_LEFT);
            getChildren().add(header);
        }

        if (hint != null) {
            Label hintLabel = new Label(hint);
            hintLabel.getStyleClass().add("hint");
            hintLabel.setWrapText(true);
            setMargin(getChildren().getFirst(), new javafx.geometry.Insets(0, 0, -8, 0));
            getChildren().add(hintLabel);
        }

        VBox.setVgrow(content, Priority.ALWAYS);
        getChildren().add(content);
    }
}
