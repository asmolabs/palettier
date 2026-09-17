package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.service.PaletteService;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import be.asmolabs.palettier.ui.component.Dialogs;
import be.asmolabs.palettier.ui.component.Pill;
import java.util.List;
import java.util.Optional;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Alert;
import javafx.scene.control.Button;
import javafx.scene.control.ButtonType;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.SplitPane;
import javafx.scene.control.TextField;
import javafx.scene.control.TextInputDialog;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import org.springframework.stereotype.Component;

/**
 * Composition des palettes : une selection nommee de tubes, preparee pour un sujet.
 *
 * <p>C'est aussi le filtre des autres ecrans : une fois une palette definie, le
 * melangeur peut ne chercher que dedans, ce qui transforme une reponse theorique sur
 * quatre cents tubes en une recette realisable avec ce qu'on a sorti.</p>
 */
@Component
public class PaletteView implements AppView {

    private static final String ALL_BRANDS = "Toutes les marques";

    private final PaletteService palettes;
    private final PaintCatalogService catalog;

    private final ObservableList<Palette> paletteItems = FXCollections.observableArrayList();
    private final ListView<Palette> paletteList = new ListView<>(paletteItems);
    private final ComboBox<String> brandChoice = new ComboBox<>();
    private final ComboBox<OilPaint> paintChoice = new ComboBox<>();
    private final TextField paintSearch = new TextField();
    private final Label matchCount = new Label();

    /** Tubes de la gamme choisie, avant filtrage textuel. Evite de relire la base a chaque frappe. */
    private List<OilPaint> brandPaints = List.of();
    private final FlowPane tiles = new FlowPane();
    private final Label summary = new Label();
    private final Label notes = new Label();
    private final Label dryingNote = new Label();

    public PaletteView(PaletteService palettes, PaintCatalogService catalog) {
        this.palettes = palettes;
        this.catalog = catalog;
    }

    @Override
    public String title() {
        return "Palettes";
    }

    @Override
    public String subtitle() {
        return "Preparez une selection de tubes par sujet, et servez-vous en comme filtre dans le melangeur.";
    }

    @Override
    public int order() {
        // Ce qui m'appartient : les tubes prepares.
        return 20;
    }

    @Override
    public int shortcut() {
        return 2;
    }

    @Override
    public Node create() {
        SplitPane split = new SplitPane(listPanel(), contentPanel());
        split.setDividerPositions(0.28);

        reloadPalettes();
        return split;
    }

    // --- Liste des palettes ------------------------------------------------

    private Node listPanel() {
        paletteList.setCellFactory(view -> new PaletteCell());
        paletteList.getSelectionModel().selectedItemProperty()
                .addListener((obs, old, palette) -> showPalette(palette));
        VBox.setVgrow(paletteList, Priority.ALWAYS);

        Button create = new Button("Nouvelle");
        create.setDefaultButton(true);
        create.setOnAction(event -> createPalette());

        Button rename = new Button("Renommer");
        rename.setOnAction(event -> renameSelected());

        Button delete = new Button("Supprimer");
        delete.setOnAction(event -> deleteSelected());

        HBox buttons = new HBox(8, create, rename, delete);

        Card card = new Card("Mes palettes", new VBox(12, paletteList, buttons));
        VBox.setVgrow(card, Priority.ALWAYS);
        return card;
    }

    private void createPalette() {
        prompt("Nouvelle palette", "Nom de la palette", "").ifPresent(name -> {
            Palette created = palettes.create(name, "");
            reloadPalettes();
            paletteList.getSelectionModel().select(created);
        });
    }

    private void renameSelected() {
        Palette selected = selected();
        if (selected == null) {
            return;
        }
        prompt("Renommer la palette", "Nouveau nom", selected.getName()).ifPresent(name -> {
            selected.setName(name);
            palettes.save(selected);
            reloadPalettes();
            paletteList.getSelectionModel().select(selected);
        });
    }

    private void deleteSelected() {
        Palette selected = selected();
        if (selected == null) {
            return;
        }
        Alert confirm = new Alert(Alert.AlertType.CONFIRMATION,
                "Supprimer la palette \"%s\" ? Les tubes restent dans le catalogue."
                        .formatted(selected.getName()),
                ButtonType.CANCEL, ButtonType.OK);
        Dialogs.themed(confirm);
        confirm.showAndWait()
                .filter(ButtonType.OK::equals)
                .ifPresent(button -> {
                    palettes.delete(selected);
                    reloadPalettes();
                });
    }

    private Optional<String> prompt(String title, String label, String initial) {
        TextInputDialog dialog = new TextInputDialog(initial);
        dialog.setTitle(title);
        dialog.setContentText(label);
        Dialogs.themed(dialog);
        return dialog.showAndWait().map(String::trim).filter(value -> !value.isEmpty());
    }

    // --- Contenu d'une palette ---------------------------------------------

    private Node contentPanel() {
        brandChoice.getItems().add(ALL_BRANDS);
        catalog.findAll().stream().map(OilPaint::getBrand).distinct().sorted()
                .forEach(brandChoice.getItems()::add);
        brandChoice.setValue(ALL_BRANDS);
        brandChoice.setPrefWidth(200);
        brandChoice.valueProperty().addListener((obs, old, brand) -> fillPaintChoice(brand));

        paintChoice.setPrefWidth(320);
        paintChoice.setCellFactory(view -> new PaintCell());
        paintChoice.setButtonCell(new PaintCell());

        // Quatre cent cinquante tubes ne se parcourent pas dans une liste deroulante. La
        // recherche porte aussi sur les codes pigments : taper "PBr7" sort toutes les terres.
        paintSearch.setPromptText("Rechercher un nom, une reference, un pigment...");
        paintSearch.setPrefWidth(250);
        paintSearch.textProperty().addListener((obs, old, value) -> applyPaintFilter());
        paintSearch.setOnAction(event -> addSelectedPaint());

        matchCount.getStyleClass().add("hint");
        matchCount.setMinWidth(80);

        fillPaintChoice(ALL_BRANDS);

        Button add = new Button("Ajouter a la palette");
        add.setDefaultButton(false);
        add.setOnAction(event -> addSelectedPaint());

        HBox filters = new HBox(8, brandChoice, paintSearch, matchCount);
        filters.setAlignment(Pos.CENTER_LEFT);
        HBox chooser = new HBox(8, paintChoice, add);
        chooser.setAlignment(Pos.CENTER_LEFT);
        VBox picker = new VBox(8, filters, chooser);

        tiles.setHgap(10);
        tiles.setVgap(10);

        ScrollPane scroll = new ScrollPane(tiles);
        scroll.setFitToWidth(true);
        VBox.setVgrow(scroll, Priority.ALWAYS);

        summary.getStyleClass().add("result-summary");
        notes.getStyleClass().add("hint");
        notes.setWrapText(true);
        dryingNote.getStyleClass().add("hint");
        dryingNote.setWrapText(true);

        VBox content = new VBox(12, summary, notes, picker, scroll, dryingNote);
        return new Card("Contenu de la palette", content);
    }

    private void fillPaintChoice(String brand) {
        brandPaints = catalog.findAll().stream()
                .filter(paint -> brand == null || ALL_BRANDS.equals(brand) || brand.equals(paint.getBrand()))
                .toList();
        applyPaintFilter();
    }

    /** Croise la gamme choisie et le texte saisi. Le filtrage se fait en memoire. */
    private void applyPaintFilter() {
        String term = paintSearch.getText() == null ? "" : paintSearch.getText().trim().toLowerCase();
        List<OilPaint> matching = term.isEmpty()
                ? brandPaints
                : brandPaints.stream().filter(paint -> matches(paint, term)).toList();

        paintChoice.getItems().setAll(matching);
        paintChoice.setValue(matching.isEmpty() ? null : matching.getFirst());
        paintChoice.setDisable(matching.isEmpty());
        matchCount.setText(matching.isEmpty()
                ? "aucun tube"
                : "%d tube%s".formatted(matching.size(), matching.size() > 1 ? "s" : ""));
    }

    private static boolean matches(OilPaint paint, String term) {
        return paint.getName().toLowerCase().contains(term)
                || paint.getBrand().toLowerCase().contains(term)
                || paint.getCode().toLowerCase().contains(term)
                || paint.getLegacyCode().toLowerCase().contains(term)
                || paint.getPigments().stream().anyMatch(code -> code.toLowerCase().contains(term));
    }

    private void addSelectedPaint() {
        Palette palette = selected();
        OilPaint paint = paintChoice.getValue();
        if (palette == null || paint == null) {
            return;
        }
        showPalette(palettes.addPaint(palette, paint));
        paletteList.refresh();
    }

    private void removePaint(OilPaint paint) {
        Palette palette = selected();
        if (palette != null) {
            showPalette(palettes.removePaint(palette, paint));
            paletteList.refresh();
        }
    }

    private void showPalette(Palette palette) {
        tiles.getChildren().clear();
        if (palette == null) {
            summary.setText("Aucune palette selectionnee");
            notes.setText("");
            dryingNote.setText("");
            return;
        }

        palette.getPaints().forEach(paint -> tiles.getChildren().add(tile(paint)));

        int pigmentCount = palette.pigments().size();
        summary.setText("%s  -  %d tube%s, %d pigment%s"
                .formatted(palette.getName(),
                        palette.getPaints().size(), palette.getPaints().size() > 1 ? "s" : "",
                        pigmentCount, pigmentCount > 1 ? "s" : ""));
        notes.setText(palette.getNotes());

        if (palette.getPaints().isEmpty()) {
            dryingNote.setText("Palette vide : choisissez une gamme puis ajoutez vos tubes.");
            return;
        }
        String drying = "Le tube le plus lent de la palette impose son rythme : sechage %s."
                .formatted(palette.slowestDryingClass().label().toLowerCase());
        String mud = pigmentCount > 6
                ? " Avec %d pigments differents, surveillez les melanges a trois tubes : ils vont griser.".formatted(pigmentCount)
                : "";
        dryingNote.setText(drying + mud);
    }

    /** Une vignette par tube : pastille, nom, opacite, et une croix pour l'oter. */
    private Node tile(OilPaint paint) {
        ColorSwatch swatch = new ColorSwatch(56, 56);
        swatch.setColor(paint.color());

        Label name = new Label(paint.getName());
        name.setWrapText(true);
        name.setMaxWidth(150);

        Label brand = new Label(paint.getBrand());
        brand.getStyleClass().add("hint");

        Button remove = new Button("✕");
        remove.getStyleClass().add("icon-button");
        remove.setOnAction(event -> removePaint(paint));

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        HBox header = new HBox(4, brand, spacer, remove);
        header.setAlignment(Pos.CENTER_LEFT);

        VBox tile = new VBox(6, header, swatch, name, Pill.forDrying(paint.getDryingClass()));
        tile.getStyleClass().add("palette-tile");
        tile.setPadding(new Insets(12));
        return tile;
    }

    private Palette selected() {
        return paletteList.getSelectionModel().getSelectedItem();
    }

    private void reloadPalettes() {
        Palette previous = selected();
        paletteItems.setAll(palettes.findAll());
        if (previous != null && paletteItems.contains(previous)) {
            paletteList.getSelectionModel().select(previous);
        } else if (!paletteItems.isEmpty()) {
            paletteList.getSelectionModel().selectFirst();
        } else {
            showPalette(null);
        }
    }

    // --- Cellules ----------------------------------------------------------

    private static class PaletteCell extends ListCell<Palette> {

        private final Label name = new Label();
        private final Label detail = new Label();
        private final VBox layout;

        PaletteCell() {
            detail.getStyleClass().add("hint");
            layout = new VBox(1, name, detail);
        }

        @Override
        protected void updateItem(Palette palette, boolean empty) {
            super.updateItem(palette, empty);
            if (empty || palette == null) {
                setGraphic(null);
                return;
            }
            name.setText(palette.getName());
            int count = palette.getPaints().size();
            detail.setText(palette.getPurpose().isBlank()
                    ? "%d tubes".formatted(count)
                    : "%s  -  %d tubes".formatted(palette.getPurpose(), count));
            setGraphic(layout);
        }
    }

    private static class PaintCell extends ListCell<OilPaint> {

        private final ColorSwatch swatch = new ColorSwatch(24, 16);
        private final Label label = new Label();
        private final HBox layout = new HBox(8, swatch, label);

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
            label.setText(paint.displayName());
            setGraphic(layout);
            setText(null);
        }
    }
}
