package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.service.ColorMixService;
import be.asmolabs.palettier.core.service.MixModels.MixComponent;
import be.asmolabs.palettier.core.service.MixModels.MixResult;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.core.service.PaletteService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import be.asmolabs.palettier.ui.component.Formats;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;
import javafx.beans.property.SimpleStringProperty;
import javafx.concurrent.Task;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.ColorPicker;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.SplitPane;
import javafx.scene.control.Spinner;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.util.StringConverter;
import org.springframework.stereotype.Component;

/**
 * Simulateur de melange : on compose une recette en parts, on voit la couleur obtenue,
 * et on peut demander a l'inverse quelles huiles combiner pour atteindre une teinte.
 */
@Component
public class MixerView implements AppView {

    private static final int SUGGESTION_COUNT = 6;

    private final PaintCatalogService catalog;
    private final ColorMixService mixer;
    private final PaletteService palettes;

    private final ObservableList<PaintPart> parts = FXCollections.observableArrayList();
    private final TableView<PaintPart> partsTable = new TableView<>(parts);
    private final ObservableList<MixSuggestion> suggestions = FXCollections.observableArrayList();
    private final ComboBox<String> brandChoice = new ComboBox<>();
    private final ComboBox<OilPaint> paintChoice = new ComboBox<>();
    private final Button searchButton = new Button("Proposer des recettes");
    private final ComboBox<Integer> maxPaints = new ComboBox<>();
    private final Spinner<Double> partCount = new Spinner<>(0.5, 20.0, 1.0, 0.5);
    private final ColorSwatch resultSwatch = new ColorSwatch(190, 140);
    private final Label resultHex = new Label();
    private final Label resultDrying = new Label();
    private final Label deltaToTarget = new Label();
    private final ListView<String> warnings = new ListView<>();
    private final ColorPicker targetPicker = new ColorPicker(Color.web("#9A8F80"));
    private final ListView<MixSuggestion> suggestionList = new ListView<>(suggestions);
    private final ComboBox<Scope> scopeChoice = new ComboBox<>();

    /** Dernier melange calcule, partage par les colonnes de repartition. */
    private MixResult currentResult;

    public MixerView(PaintCatalogService catalog, ColorMixService mixer, PaletteService palettes) {
        this.catalog = catalog;
        this.mixer = mixer;
        this.palettes = palettes;
    }

    /**
     * Ensemble de tubes dans lequel chercher. Une palette donne une reponse realisable
     * avec ce qu'on a sorti sur la table ; le catalogue entier donne une reponse
     * theorique, souvent avec des tubes qu'on ne possede pas.
     */
    private record Scope(String label, Supplier<List<OilPaint>> candidates) {
        @Override
        public String toString() {
            return label;
        }
    }

    @Override
    public String title() {
        return "Melangeur";
    }

    @Override
    public String subtitle() {
        return "Simulez un melange en parts, ou partez d'une teinte et laissez le logiciel proposer la recette.";
    }

    @Override
    public int order() {
        // Trouver une couleur : la composer soi-meme.
        return 50;
    }

    @Override
    public Node create() {
        SplitPane split = new SplitPane(mixPanel(), suggestionPanel());
        split.setDividerPositions(0.58);
        recompute();
        return split;
    }

    // --- Composition du melange -------------------------------------------

    private Node mixPanel() {
        brandChoice.getItems().setAll(catalog.findAll().stream()
                .map(OilPaint::getBrand).distinct().sorted().toList());
        brandChoice.setPrefWidth(200);
        brandChoice.valueProperty().addListener((obs, old, brand) -> fillPaintChoice(brand));
        brandChoice.setValue(brandChoice.getItems().isEmpty() ? null : brandChoice.getItems().getFirst());

        paintChoice.getItems().setAll(catalog.findAll());
        paintChoice.setConverter(paintConverter());
        paintChoice.setCellFactory(view -> new PaintCell());
        paintChoice.setButtonCell(new PaintCell());
        paintChoice.setPrefWidth(300);
        fillPaintChoice(brandChoice.getValue());

        partCount.setEditable(true);
        partCount.setPrefWidth(90);

        Button add = new Button("Ajouter au melange");
        add.setDefaultButton(true);
        add.setOnAction(event -> addSelectedPaint());

        Button clear = new Button("Vider");
        clear.setOnAction(event -> {
            parts.clear();
            recompute();
        });

        HBox controls = new HBox(8, brandChoice, paintChoice, new Label("parts"), partCount, add, clear);
        controls.setAlignment(Pos.CENTER_LEFT);
        controls.setPadding(new Insets(10));

        BorderPane composition = new BorderPane(partsTable());
        composition.setTop(controls);

        VBox panel = new VBox(14, new Card("Composition", composition), resultPanel());
        VBox.setVgrow(panel.getChildren().getFirst(), Priority.ALWAYS);
        return panel;
    }

    /** Le catalogue compte plusieurs centaines de tubes : on filtre par gamme avant de choisir. */
    private void fillPaintChoice(String brand) {
        List<OilPaint> available = catalog.findAll().stream()
                .filter(paint -> brand == null || brand.equals(paint.getBrand()))
                .toList();
        paintChoice.getItems().setAll(available);
        paintChoice.setValue(available.isEmpty() ? null : available.getFirst());
    }

    private TableView<PaintPart> partsTable() {
        partsTable.setPlaceholder(new Label("Ajoutez une premiere huile pour commencer le melange."));

        TableColumn<PaintPart, String> paint = new TableColumn<>("Huile");
        paint.setCellValueFactory(cell -> new SimpleStringProperty(cell.getValue().paint().displayName()));
        paint.setPrefWidth(260);

        TableColumn<PaintPart, String> dose = new TableColumn<>("Parts");
        dose.setCellValueFactory(cell -> new SimpleStringProperty("%.1f".formatted(cell.getValue().parts())));
        dose.setPrefWidth(70);

        TableColumn<PaintPart, Void> volume = new TableColumn<>("Volume");
        volume.setCellFactory(column -> new ShareCell(MixComponent::volumeShare));
        volume.setPrefWidth(80);
        volume.setSortable(false);

        TableColumn<PaintPart, Void> pigment = new TableColumn<>("Poids dans la couleur");
        pigment.setCellFactory(column -> new ShareCell(MixComponent::pigmentShare));
        pigment.setPrefWidth(160);
        pigment.setSortable(false);

        TableColumn<PaintPart, Void> remove = new TableColumn<>("");
        remove.setCellFactory(column -> new RemoveCell());
        remove.setPrefWidth(40);
        remove.setSortable(false);

        partsTable.getColumns().addAll(paint, dose, volume, pigment, remove);
        partsTable.setColumnResizePolicy(TableView.CONSTRAINED_RESIZE_POLICY_FLEX_LAST_COLUMN);
        return partsTable;
    }

    private Node resultPanel() {
        resultHex.getStyleClass().add("result-hex");
        warnings.setPrefHeight(110);
        warnings.setPlaceholder(new Label("Rien a signaler sur ce melange."));
        warnings.setCellFactory(view -> new WrappingCell());

        VBox texts = new VBox(6, resultHex, resultDrying, deltaToTarget, warnings);
        VBox.setVgrow(warnings, Priority.ALWAYS);
        HBox.setHgrow(texts, Priority.ALWAYS);

        HBox layout = new HBox(14, resultSwatch, texts);
        layout.setPadding(new Insets(10));

        return new Card("Resultat du melange", layout);
    }

    private void addSelectedPaint() {
        OilPaint selected = paintChoice.getValue();
        if (selected != null) {
            parts.add(PaintPart.of(selected, partCount.getValue()));
            recompute();
        }
    }

    private void recompute() {
        if (parts.isEmpty()) {
            currentResult = null;
            partsTable.refresh();
            resultSwatch.setColor(new Rgb(0.9, 0.9, 0.9), "?");
            resultHex.setText("Aucun melange");
            resultDrying.setText("");
            deltaToTarget.setText("");
            warnings.getItems().clear();
            return;
        }

        MixResult result = mixer.mix(List.copyOf(parts));
        currentResult = result;
        partsTable.refresh();
        resultSwatch.setColor(result.color(), result.hex());
        resultHex.setText(result.hex());
        resultDrying.setText("Sechage impose : %s  -  pigments : %s"
                .formatted(result.dryingClass().label(), String.join(", ", result.pigments())));
        deltaToTarget.setText("Ecart avec la teinte visee : %.1f"
                .formatted(Colors.deltaE2000(Formats.fromFx(targetPicker.getValue()), result.color())));
        warnings.getItems().setAll(result.warnings());
    }

    // --- Recherche inverse -------------------------------------------------

    private Node suggestionPanel() {
        targetPicker.setPrefWidth(200);
        targetPicker.setOnAction(event -> recompute());

        searchButton.setDefaultButton(false);
        searchButton.setOnAction(event -> suggest());

        scopeChoice.setPrefWidth(260);
        // Les palettes changent pendant la session : on relit la liste a chaque ouverture.
        scopeChoice.setOnShowing(event -> rebuildScopes());
        rebuildScopes();

        ListView<MixSuggestion> list = suggestionList;
        list.setCellFactory(view -> new SuggestionCell());
        list.setPlaceholder(new Label("Choisissez une teinte, puis lancez la recherche."));
        VBox.setVgrow(list, Priority.ALWAYS);

        list.getSelectionModel().selectedItemProperty().addListener((obs, old, suggestion) -> {
            if (suggestion != null) {
                parts.setAll(suggestion.parts());
                recompute();
            }
        });

        Label hint = new Label("""
                La recherche essaie chaque tube seul, puis toutes les paires de 1:9 a 9:1. \
                Restreindre a une palette donne une recette realisable tout de suite ; le \
                catalogue entier propose souvent des tubes que vous n'avez pas. Le nombre de \
                tubes est un maximum : a resultat identique, le melange le plus simple gagne. \
                Cliquez sur une proposition pour la charger dans le melangeur.""");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        HBox controls = new HBox(10, new Label("Teinte visee"), targetPicker, searchButton);
        controls.setAlignment(Pos.CENTER_LEFT);

        maxPaints.getItems().setAll(1, 2, 3, 4, 5);
        maxPaints.setValue(3);
        maxPaints.setConverter(new javafx.util.StringConverter<>() {
            @Override
            public String toString(Integer count) {
                return count == null ? "" : count == 1 ? "un seul tube" : count + " tubes au plus";
            }

            @Override
            public Integer fromString(String value) {
                return null;
            }
        });

        HBox scopeRow = new HBox(10, new Label("Chercher dans"), scopeChoice, maxPaints);
        scopeRow.setAlignment(Pos.CENTER_LEFT);

        VBox content = new VBox(12, controls, scopeRow, hint, list);
        VBox.setVgrow(list, Priority.ALWAYS);
        return new Card("Comment obtenir cette couleur ?", content);
    }

    /**
     * La recherche explore toutes les paires du catalogue : environ une seconde sur
     * quatre cents tubes. Elle part donc sur un fil dedie, sinon l'interface se fige
     * pendant le calcul.
     */
    private void suggest() {
        Scope scope = scopeChoice.getValue();
        List<OilPaint> candidates = scope == null ? catalog.findAll() : scope.candidates().get();
        Rgb target = Formats.fromFx(targetPicker.getValue());

        if (candidates.isEmpty()) {
            suggestions.clear();
            suggestionList.setPlaceholder(new Label("Cet ensemble ne contient aucun tube."));
            return;
        }

        suggestions.clear();
        suggestionList.setPlaceholder(new Label("Recherche parmi %d tubes...".formatted(candidates.size())));
        searchButton.setDisable(true);

        Task<List<MixSuggestion>> search = new Task<>() {
            @Override
            protected List<MixSuggestion> call() {
                return mixer.suggestMixes(target, candidates, SUGGESTION_COUNT,
                        maxPaints.getValue() == null ? 3 : maxPaints.getValue());
            }
        };
        search.setOnSucceeded(event -> {
            suggestions.setAll(search.getValue());
            searchButton.setDisable(false);
        });
        search.setOnFailed(event -> {
            suggestionList.setPlaceholder(new Label("La recherche a echoue."));
            searchButton.setDisable(false);
        });
        // Calcul purement processeur : un fil plateforme, pas un fil virtuel.
        Thread.ofPlatform().daemon().name("mix-search").start(search);
    }

    /** Reconstruit la liste des ensembles de recherche : catalogue, stock, puis chaque palette. */
    private void rebuildScopes() {
        Scope previous = scopeChoice.getValue();

        List<Scope> scopes = new ArrayList<>();
        scopes.add(new Scope("Tout le catalogue", catalog::findAll));
        scopes.add(new Scope("Mes tubes en stock", catalog::findInStock));
        for (Palette palette : palettes.findAll()) {
            scopes.add(new Scope("Palette : " + palette.getName(), palette::getPaints));
        }

        scopeChoice.getItems().setAll(scopes);
        scopeChoice.setValue(scopes.stream()
                .filter(scope -> previous != null && scope.label().equals(previous.label()))
                .findFirst()
                .orElse(scopes.getFirst()));
    }

    // --- Cellules ----------------------------------------------------------

    private static StringConverter<OilPaint> paintConverter() {
        return new StringConverter<>() {
            @Override
            public String toString(OilPaint paint) {
                return paint == null ? "" : paint.displayName();
            }

            @Override
            public OilPaint fromString(String value) {
                return null;
            }
        };
    }

    private static class PaintCell extends ListCell<OilPaint> {

        private final ColorSwatch swatch = new ColorSwatch(24, 16);
        private final HBox layout = new HBox(8, swatch, new Label());

        PaintCell() {
            layout.setAlignment(Pos.CENTER_LEFT);
        }

        @Override
        protected void updateItem(OilPaint paint, boolean empty) {
            super.updateItem(paint, empty);
            if (empty || paint == null) {
                setGraphic(null);
                setText(null);
                return;
            }
            swatch.setColor(paint.color());
            ((Label) layout.getChildren().get(1)).setText(paint.displayName());
            setGraphic(layout);
            setText(null);
        }
    }

    /** Affiche la part d'une huile dans le melange courant, lue dans le dernier resultat calcule. */
    private class ShareCell extends javafx.scene.control.TableCell<PaintPart, Void> {

        private final java.util.function.ToDoubleFunction<MixComponent> extractor;

        ShareCell(java.util.function.ToDoubleFunction<MixComponent> extractor) {
            this.extractor = extractor;
        }

        @Override
        protected void updateItem(Void item, boolean empty) {
            super.updateItem(item, empty);
            boolean available = !empty && currentResult != null && getIndex() < currentResult.components().size();
            setText(available
                    ? Formats.percent(extractor.applyAsDouble(currentResult.components().get(getIndex())))
                    : null);
        }
    }

    private class RemoveCell extends javafx.scene.control.TableCell<PaintPart, Void> {

        private final Button button = new Button("✕");

        RemoveCell() {
            button.getStyleClass().add("icon-button");
            button.setOnAction(event -> {
                parts.remove(getIndex());
                recompute();
            });
        }

        @Override
        protected void updateItem(Void item, boolean empty) {
            super.updateItem(item, empty);
            setGraphic(empty ? null : button);
        }
    }

    private static class SuggestionCell extends ListCell<MixSuggestion> {

        private final ColorSwatch swatch = new ColorSwatch(44, 44);
        private final Label title = new Label();
        private final Label detail = new Label();
        private final HBox layout;

        SuggestionCell() {
            title.setWrapText(true);
            detail.getStyleClass().add("hint");
            VBox text = new VBox(2, title, detail);
            HBox.setHgrow(text, Priority.ALWAYS);
            layout = new HBox(10, swatch, text);
            layout.setPadding(new Insets(4));
        }

        @Override
        protected void updateItem(MixSuggestion suggestion, boolean empty) {
            super.updateItem(suggestion, empty);
            if (empty || suggestion == null) {
                setGraphic(null);
                return;
            }
            swatch.setColor(suggestion.color());
            title.setText(suggestion.describe());
            detail.setText("%s  -  ecart %.1f".formatted(suggestion.color().toHex(), suggestion.deltaE()));
            setGraphic(layout);
        }
    }

    private static class WrappingCell extends ListCell<String> {

        private final Label label = new Label();

        WrappingCell() {
            label.setWrapText(true);
            setGraphic(label);
        }

        @Override
        protected void updateItem(String text, boolean empty) {
            super.updateItem(text, empty);
            label.setText(empty ? "" : text);
            setGraphic(empty ? null : label);
        }
    }
}
