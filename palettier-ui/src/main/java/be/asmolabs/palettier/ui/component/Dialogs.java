package be.asmolabs.palettier.ui.component;

import javafx.scene.control.Dialog;
import javafx.scene.control.DialogPane;

/**
 * Les boites de dialogue JavaFX ouvrent leur propre scene : sans cela elles
 * apparaissent en gris clair systeme au milieu d'une application sombre.
 */
public final class Dialogs {

    private static final String STYLESHEET = "/be/asmolabs/palettier/ui/app.css";

    private Dialogs() {
    }

    public static <T> Dialog<T> themed(Dialog<T> dialog) {
        DialogPane pane = dialog.getDialogPane();
        pane.getStylesheets().add(Dialogs.class.getResource(STYLESHEET).toExternalForm());
        pane.getStyleClass().add("pal-dialog");
        dialog.setHeaderText(null);
        return dialog;
    }
}
