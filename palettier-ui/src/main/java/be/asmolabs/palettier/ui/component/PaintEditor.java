package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import java.util.Arrays;
import java.util.LinkedHashSet;
import java.util.Optional;
import java.util.Set;
import javafx.scene.control.ButtonType;
import javafx.scene.control.ColorPicker;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Dialog;
import javafx.scene.control.Label;
import javafx.scene.control.Spinner;
import javafx.scene.control.TextField;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;

/**
 * Saisie d'un tube absent du catalogue, ou correction d'un tube existant.
 *
 * <p>Aucun catalogue livre n'est complet : le fabricant en publie une partie, et le
 * peintre possede toujours quelque chose qui n'y figure pas. Mieux vaut qu'il le saisisse
 * que d'attendre une mise a jour.</p>
 */
public final class PaintEditor {

    /** Ce qui a ete saisi. */
    public record Result(String brand, String name, String code, Set<String> pigments,
                         Rgb color, Opacity opacity, DryingClass drying, double tintingStrength) {
    }

    private PaintEditor() {
    }

    /** @param existing tube a corriger, ou {@code null} pour en creer un */
    public static Optional<Result> show(OilPaint existing, String suggestedBrand) {
        TextField brand = new TextField(existing == null
                ? (suggestedBrand == null ? "" : suggestedBrand) : existing.getBrand());
        TextField name = new TextField(existing == null ? "" : existing.getName());
        TextField code = new TextField(existing == null ? "" : existing.getCode());

        TextField pigments = new TextField(existing == null
                ? "" : String.join(" ", existing.getPigments()));
        pigments.setPromptText("PBr7  ou  PY42 PR101, separes par des espaces");

        ColorPicker color = new ColorPicker(Formats.toFx(
                existing == null ? Rgb.ofHex("#808080") : existing.color()));
        color.setMaxWidth(Double.MAX_VALUE);

        TextField hex = new TextField(existing == null ? "#808080" : existing.getHexColor());
        hex.getStyleClass().add("hex-field");
        hex.setPrefWidth(110);
        color.valueProperty().addListener((obs, old, value) -> hex.setText(Formats.fromFx(value).toHex()));
        hex.textProperty().addListener((obs, old, value) -> {
            String candidate = value.startsWith("#") ? value : "#" + value;
            if (candidate.matches("#[0-9A-Fa-f]{6}")) {
                hex.setStyle("");
                color.setValue(Formats.toFx(Rgb.ofHex(candidate)));
            } else {
                hex.setStyle("-fx-border-color: #d9705f;");
            }
        });

        ComboBox<Opacity> opacity = new ComboBox<>();
        opacity.getItems().setAll(Opacity.values());
        opacity.setValue(existing == null ? Opacity.SEMI_OPAQUE : existing.getOpacity());
        opacity.setMaxWidth(Double.MAX_VALUE);

        ComboBox<DryingClass> drying = new ComboBox<>();
        drying.getItems().setAll(DryingClass.values());
        drying.setValue(existing == null ? DryingClass.MEDIUM : existing.getDryingClass());
        drying.setMaxWidth(Double.MAX_VALUE);

        Spinner<Double> tinting = new Spinner<>(0.05, 1.0,
                existing == null ? 0.6 : existing.getTintingStrength(), 0.05);
        tinting.setEditable(true);
        tinting.setMaxWidth(Double.MAX_VALUE);

        // La marque et le nom forment la cle du tube : les changer reviendrait a en
        // designer un autre, et a rompre les palettes qui s'y referent.
        brand.setDisable(existing != null);
        name.setDisable(existing != null);

        GridPane form = new GridPane();
        form.setHgap(12);
        form.setVgap(10);
        ColumnConstraints labels = new ColumnConstraints();
        labels.setMinWidth(130);
        ColumnConstraints fields = new ColumnConstraints();
        fields.setHgrow(Priority.ALWAYS);
        form.getColumnConstraints().addAll(labels, fields);

        HBox colour = new HBox(8, color, hex);
        HBox.setHgrow(color, Priority.ALWAYS);

        form.addRow(0, new Label("Marque"), brand);
        form.addRow(1, new Label("Nom"), name);
        form.addRow(2, new Label("Reference"), code);
        form.addRow(3, new Label("Pigments"), pigments);
        form.addRow(4, new Label("Couleur"), colour);
        form.addRow(5, new Label("Opacite"), opacity);
        form.addRow(6, new Label("Sechage"), drying);
        form.addRow(7, new Label("Pouvoir colorant"), tinting);

        Label hint = new Label("Les pigments sont ecrits sur le tube, en petit, sous la forme "
                + "PBr7 ou PY42. Ce sont eux qui donnent la vitesse de sechage : sans eux, les "
                + "plannings de ce tube seront faux.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");
        form.add(hint, 0, 8, 2, 1);

        Dialog<ButtonType> dialog = new Dialog<>();
        dialog.setTitle(existing == null ? "Ajouter un tube" : "Corriger " + existing.displayName());
        dialog.getDialogPane().setContent(form);
        dialog.getDialogPane().getButtonTypes().addAll(ButtonType.CANCEL, ButtonType.OK);
        dialog.getDialogPane().setPrefWidth(520);
        Dialogs.themed(dialog);

        return dialog.showAndWait()
                .filter(ButtonType.OK::equals)
                .map(button -> new Result(brand.getText(), name.getText(), code.getText(),
                        parse(pigments.getText()), Formats.fromFx(color.getValue()),
                        opacity.getValue(), drying.getValue(), tinting.getValue()));
    }

    /** Les pigments se saisissent separes par des espaces ou des virgules. */
    private static Set<String> parse(String text) {
        if (text == null) {
            return Set.of();
        }
        return Arrays.stream(text.trim().split("[\\s,;]+"))
                .filter(word -> !word.isBlank())
                .map(word -> word.toUpperCase())
                .collect(LinkedHashSet::new, Set::add, Set::addAll);
    }
}
