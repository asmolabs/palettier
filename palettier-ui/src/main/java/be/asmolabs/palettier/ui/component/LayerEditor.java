package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import java.util.Optional;
import javafx.scene.control.ButtonType;
import javafx.scene.control.ColorPicker;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Dialog;
import javafx.scene.control.Label;
import javafx.scene.control.TextArea;
import javafx.scene.control.TextField;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;

/**
 * Correction d'une couche : la couleur visee, la technique, la note.
 *
 * <p>Seule la decision se modifie. Le dosage n'apparait pas dans ce formulaire parce
 * qu'il ne se decide pas : il se calcule a partir de la couleur retenue et de la palette
 * du projet.</p>
 */
public final class LayerEditor {

    /** Ce que le peintre a corrige. */
    public record Result(Rgb target, String technique, String note) {
    }

    private LayerEditor() {
    }

    public static Optional<Result> show(PaintingPlan.Layer layer) {
        ColorPicker picker = new ColorPicker(Formats.toFx(layer.target()));
        picker.setMaxWidth(Double.MAX_VALUE);

        TextField hex = new TextField(layer.target().toHex());
        hex.setPrefWidth(110);
        hex.getStyleClass().add("hex-field");

        // Les deux champs disent la meme chose : on les tient synchronises plutot que de
        // laisser le peintre se demander lequel fait foi.
        picker.valueProperty().addListener((obs, old, value) -> hex.setText(Formats.fromFx(value).toHex()));
        hex.textProperty().addListener((obs, old, value) -> {
            String candidate = value.startsWith("#") ? value : "#" + value;
            if (candidate.matches("#[0-9A-Fa-f]{6}")) {
                hex.setStyle("");
                picker.setValue(Formats.toFx(Rgb.ofHex(candidate)));
            } else {
                hex.setStyle("-fx-border-color: #d9705f;");
            }
        });

        ComboBox<String> technique = new ComboBox<>();
        technique.setEditable(true);
        technique.setMaxWidth(Double.MAX_VALUE);
        Technique[] known = Technique.values();
        for (Technique value : known) {
            technique.getItems().add(value.label());
        }
        technique.getEditor().setText(layer.technique() == null ? "" : layer.technique());

        TextArea note = new TextArea(layer.note() == null ? "" : layer.note());
        note.setWrapText(true);
        note.setPrefRowCount(3);

        GridPane form = new GridPane();
        form.setHgap(12);
        form.setVgap(10);
        ColumnConstraints labels = new ColumnConstraints();
        labels.setMinWidth(120);
        ColumnConstraints fields = new ColumnConstraints();
        fields.setHgrow(Priority.ALWAYS);
        form.getColumnConstraints().addAll(labels, fields);

        HBox colour = new HBox(8, picker, hex);
        HBox.setHgrow(picker, Priority.ALWAYS);

        form.addRow(0, new Label("Couleur visee"), colour);
        form.addRow(1, new Label("Technique"), technique);
        form.addRow(2, new Label("Note"), note);

        Dialog<ButtonType> dialog = new Dialog<>();
        dialog.setTitle("Modifier " + layer.role().toLowerCase());
        dialog.getDialogPane().setContent(form);
        dialog.getDialogPane().getButtonTypes().addAll(ButtonType.CANCEL, ButtonType.OK);
        dialog.getDialogPane().setPrefWidth(460);
        Dialogs.themed(dialog);

        return dialog.showAndWait()
                .filter(ButtonType.OK::equals)
                .map(button -> new Result(Formats.fromFx(picker.getValue()),
                        technique.getEditor().getText(), note.getText()));
    }
}
