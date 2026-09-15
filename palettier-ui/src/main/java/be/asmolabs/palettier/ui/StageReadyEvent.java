package be.asmolabs.palettier.ui;

import javafx.stage.Stage;
import org.springframework.context.ApplicationEvent;

/** Publie quand JavaFX a fourni la fenetre principale, encore vide. */
public class StageReadyEvent extends ApplicationEvent {

    public StageReadyEvent(Stage stage) {
        super(stage);
    }

    public Stage stage() {
        return (Stage) getSource();
    }
}
