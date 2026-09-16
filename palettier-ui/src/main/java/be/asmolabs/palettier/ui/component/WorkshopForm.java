package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.domain.Ventilation;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.Spinner;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;

/**
 * Saisie des conditions de l'atelier. Partagee par les ecrans qui calculent un
 * sechage, pour que le peintre ne saisisse ses conditions qu'une fois par ecran.
 */
public class WorkshopForm extends VBox {

    private final Spinner<Double> temperature = new Spinner<>(5.0, 40.0, 20.0, 0.5);
    private final Spinner<Double> humidity = new Spinner<>(10.0, 95.0, 50.0, 5.0);
    private final ComboBox<Ventilation> ventilation = new ComboBox<>();

    public WorkshopForm() {
        this(null);
    }

    /** @param hint phrase expliquant a quoi ces conditions servent ici, ou {@code null} */
    public WorkshopForm(String hint) {
        getStyleClass().addAll("card", "workshop-form");
        setSpacing(12);

        Label heading = new Label("CONDITIONS DE L'ATELIER");
        heading.getStyleClass().add("card-title");

        temperature.setEditable(true);
        temperature.setMaxWidth(Double.MAX_VALUE);
        humidity.setEditable(true);
        humidity.setMaxWidth(Double.MAX_VALUE);
        ventilation.getItems().setAll(Ventilation.values());
        ventilation.setValue(Ventilation.NORMAL);
        ventilation.setMaxWidth(Double.MAX_VALUE);

        GridPane grid = new GridPane();
        grid.setHgap(12);
        grid.setVgap(9);
        ColumnConstraints labels = new ColumnConstraints();
        labels.setMinWidth(140);
        ColumnConstraints fields = new ColumnConstraints();
        fields.setHgrow(Priority.ALWAYS);
        grid.getColumnConstraints().addAll(labels, fields);
        grid.addRow(0, fieldLabel("Temperature (°C)"), temperature);
        grid.addRow(1, fieldLabel("Humidite relative (%)"), humidity);
        grid.addRow(2, fieldLabel("Ventilation"), ventilation);

        getChildren().addAll(heading, grid);

        if (hint != null) {
            Label explanation = new Label(hint);
            explanation.getStyleClass().add("hint");
            explanation.setWrapText(true);
            getChildren().add(explanation);
        }
    }

    private static Label fieldLabel(String text) {
        Label label = new Label(text);
        label.getStyleClass().add("field-label");
        return label;
    }

    public Workshop current() {
        return new Workshop(temperature.getValue(), humidity.getValue(), ventilation.getValue());
    }

    /** Rebranche le calcul a chaque modification d'un champ. */
    public void onChange(Runnable listener) {
        temperature.valueProperty().addListener((obs, old, value) -> listener.run());
        humidity.valueProperty().addListener((obs, old, value) -> listener.run());
        ventilation.valueProperty().addListener((obs, old, value) -> listener.run());
    }
}
