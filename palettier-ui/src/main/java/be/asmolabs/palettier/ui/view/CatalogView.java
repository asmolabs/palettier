package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.service.MixModels.PaintMatch;
import be.asmolabs.palettier.ai.TubeRecognitionService;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.SampledColor;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.Formats;
import be.asmolabs.palettier.ui.component.PaintEditor;
import be.asmolabs.palettier.ui.component.Pill;
import java.io.File;
import java.nio.file.Files;
import java.util.List;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.scene.Node;
import javafx.geometry.Pos;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.Tooltip;
import javafx.scene.control.ColorPicker;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.SplitPane;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.control.TextField;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToolBar;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.stage.FileChooser;
import org.springframework.stereotype.Component;

/** Consultation du catalogue d'huiles et recherche du tube le plus proche d'une teinte. */
@Component
public class CatalogView implements AppView {

    private static final int MATCH_COUNT = 8;
    private static final String ALL_BRANDS = "Toutes les marques";

    /**
     * Largeur commune aux deux colonnes de pastilles. Elles se lisent par comparaison,
     * l'une au-dessus de l'autre : des largeurs differentes fausseraient la lecture.
     */
    private static final double SWATCH_COLUMN_WIDTH = 110;

    private final PaintCatalogService catalog;
    private final TubeRecognitionService recognition;
    private final SampledColor sampled;

    private final ObservableList<OilPaint> paints = FXCollections.observableArrayList();
    private final ObservableList<PaintMatch> matches = FXCollections.observableArrayList();
    private final TextField searchField = new TextField();
    private final CheckBox inStockOnly = new CheckBox("En stock uniquement");
    private final ComboBox<String> brandFilter = new ComboBox<>();
    private final Label countLabel = new Label();
    private final Label ownedCount = new Label();
    private final Label recognitionState = new Label();
    private final Label editState = new Label();
    private final ListView<TubeRecognitionService.Identification> recognised = new ListView<>();
    private final ColorPicker targetPicker = new ColorPicker(Color.web("#8A6A4A"));
    private final TableView<OilPaint> table = new TableView<>(paints);

    // Calibrage du tube selectionne dans le tableau.
    private final Label calibrationName = new Label();
    private final ColorSwatch calibrationMasstone = new ColorSwatch(46, 32);
    private final ColorSwatch calibrationTint = new ColorSwatch(46, 32);
    private final ColorPicker calibrationPicker = new ColorPicker(Color.web("#9A8F80"));
    private final Button useSampled = new Button();
    private final Label calibrationState = new Label();

    public CatalogView(PaintCatalogService catalog, TubeRecognitionService recognition,
                       SampledColor sampled) {
        this.catalog = catalog;
        this.recognition = recognition;
        this.sampled = sampled;
    }

    @Override
    public String title() {
        return "Catalogue";
    }

    @Override
    public String subtitle() {
        return "Vos tubes, leurs pigments, et lequel se rapproche le plus d'une teinte visee.";
    }

    @Override
    public int order() {
        // La reference : tout ce qui existe.
        return 60;
    }

    @Override
    public Node create() {
        searchField.setPromptText("Rechercher une marque, un nom ou une reference");
        searchField.setPrefWidth(360);
        searchField.textProperty().addListener((obs, old, value) -> refresh());
        inStockOnly.selectedProperty().addListener((obs, old, value) -> refresh());

        brandFilter.getItems().add(ALL_BRANDS);
        catalog.findAll().stream().map(OilPaint::getBrand).distinct().sorted()
                .forEach(brandFilter.getItems()::add);
        brandFilter.setValue(ALL_BRANDS);
        brandFilter.setPrefWidth(210);
        brandFilter.valueProperty().addListener((obs, old, value) -> refresh());

        ToolBar toolBar = new ToolBar(searchField, brandFilter, inStockOnly);

        countLabel.getStyleClass().add("hint");
        toolBar.getItems().add(countLabel);

        // Repliee par defaut : l'explication est utile une fois, le tableau sert tout le
        // temps. Elle ne doit pas lui prendre huit lignes de hauteur en permanence.
        Label legend = new Label(
                "La colonne \u00AB Coupe de blanc \u00BB montre la teinte du tube mele de blanc de "
                + "titane, une part pour neuf. C'est elle qui revele le vrai caractere d'un "
                + "pigment : un noir d'ivoire sort chaud du tube et donne pourtant un gris "
                + "bleute. Quand elle est connue, les melanges de ce tube sont calcules avec le "
                + "modele de Kubelka-Munk a deux constantes ; un tiret signale les tubes qui "
                + "restent sur le modele simple. Vous pouvez relever la votre sur vos propres "
                + "ecouvillons : onglet Pipette pour mesurer, carte ci-contre pour attribuer.\n\n"
                + "La colonne \u00AB Source \u00BB dit d'ou viennent les pigments. \u00AB fabricant \u00BB : "
                + "releves dans le catalogue officiel de la marque. \u00AB a verifier \u00BB : reconstitues, "
                + "donc possiblement faux. Ce sont eux qui donnent la vitesse de sechage, et donc les "
                + "delais annonces : sur un tube a verifier, lisez l'etiquette et corrigez la fiche.");
        legend.setWrapText(true);
        legend.getStyleClass().add("hint");
        legend.setManaged(false);
        legend.setVisible(false);

        ToggleButton explain = new ToggleButton("Que signifie \u00AB Coupe de blanc \u00BB ?");
        explain.getStyleClass().add("link-button");
        explain.selectedProperty().addListener((obs, old, shown) -> {
            legend.setManaged(shown);
            legend.setVisible(shown);
        });

        VBox footer = new VBox(6, explain, legend);

        BorderPane left = new BorderPane(paintTable());
        left.setTop(toolBar);
        left.setBottom(footer);
        BorderPane.setMargin(footer, new javafx.geometry.Insets(10, 0, 0, 0));
        Card shelf = new Card("Etagere", left);
        paints.addListener((javafx.collections.ListChangeListener<OilPaint>) change ->
                countLabel.setText("%d huiles".formatted(paints.size())));
        table.getSelectionModel().selectedItemProperty()
                .addListener((obs, old, paint) -> showCalibration(paint));

        VBox right = new VBox(14, inventoryPanel(), matchPanel(), calibrationPanel());
        SplitPane split = new SplitPane(shelf, right);
        split.setDividerPositions(0.62);

        refresh();
        findClosest();
        return split;
    }

    private TableView<OilPaint> paintTable() {
        table.setPlaceholder(new Label("Aucune huile ne correspond a cette recherche."));

        TableColumn<OilPaint, OilPaint> swatch = new TableColumn<>("Tube");
        swatch.setCellValueFactory(cell -> new javafx.beans.property.SimpleObjectProperty<>(cell.getValue()));
        swatch.setCellFactory(column -> new TableCellWithSwatch(OilPaint::color));
        fixWidth(swatch);
        swatch.setSortable(false);

        // La teinte coupee de blanc : c'est elle qui revele le vrai caractere d'un pigment.
        // Un noir d'ivoire sort chaud du tube et donne pourtant un gris bleute.
        TableColumn<OilPaint, OilPaint> tint = new TableColumn<>("Coupe de blanc");
        tint.setCellValueFactory(cell -> new javafx.beans.property.SimpleObjectProperty<>(cell.getValue()));
        tint.setCellFactory(column -> new TableCellWithSwatch(paint ->
                paint.getTintHex() == null ? null : be.asmolabs.palettier.core.color.Rgb.ofHex(paint.getTintHex())));
        fixWidth(tint);
        tint.setSortable(false);

        table.getColumns().addAll(
                swatch,
                tint,
                column("Marque", 140, OilPaint::getBrand),
                column("Nom", 220, OilPaint::getName),
                column("Ref.", 70, OilPaint::getCode),
                column("Pigments", 120, p -> String.join(", ", p.getPigments())),
                pillColumn("Source", 110, paint -> paint.isPigmentsVerified()
                        ? Pill.of("fabricant", "pill-fast")
                        : Pill.of("a verifier", "pill-ghost")),
                pillColumn("Opacite", 140, paint -> Pill.forOpacity(paint.getOpacity())),
                pillColumn("Sechage", 110, paint -> Pill.forDrying(paint.getDryingClass())),
                column("Pouvoir colorant", 130, p -> Formats.percent(p.getTintingStrength())),
                ownedColumn());

        // Dix colonnes ne tiennent pas dans la moitie d'une fenetre. Une politique
        // "contrainte" les y forcerait en les ecrasant toutes, sans jamais proposer de
        // defilement horizontal : les pigments et le pouvoir colorant deviennent alors
        // illisibles. On laisse donc les colonnes a leur largeur utile et le tableau
        // defile, dans les deux sens.
        table.setColumnResizePolicy(TableView.UNCONSTRAINED_RESIZE_POLICY);
        table.setMinHeight(240);
        return table;
    }

    /** Fige une colonne a la largeur commune des pastilles, que le tableau ne l'etire pas. */
    private static void fixWidth(TableColumn<OilPaint, ?> column) {
        column.setMinWidth(SWATCH_COLUMN_WIDTH);
        column.setPrefWidth(SWATCH_COLUMN_WIDTH);
        column.setMaxWidth(SWATCH_COLUMN_WIDTH);
    }

    private static TableColumn<OilPaint, String> column(String title, double width,
                                                        java.util.function.Function<OilPaint, String> extractor) {
        TableColumn<OilPaint, String> column = new TableColumn<>(title);
        column.setCellValueFactory(cell ->
                new javafx.beans.property.SimpleStringProperty(extractor.apply(cell.getValue())));
        column.setPrefWidth(width);
        return column;
    }

    // --- Inventaire --------------------------------------------------------

    /**
     * Declarer ce que l'on possede reellement.
     *
     * <p>Sans cet inventaire, le filtre "mes tubes" et les recherches de melange
     * restreintes ne servent a rien : tout le catalogue est marque comme possede a la
     * livraison.</p>
     */
    private Node inventoryPanel() {
        ownedCount.getStyleClass().add("result-summary");

        Button none = new Button("Je ne possede rien");
        none.setTooltip(new Tooltip("Decoche tout le catalogue, pour partir de zero"));
        none.setOnAction(event -> {
            catalog.declareNothingOwned();
            table.refresh();
            refreshOwnedCount();
        });

        Button markSelection = new Button("J'ai ceux-ci");
        markSelection.setOnAction(event -> applyToSelection(true));

        Button unmarkSelection = new Button("Je ne les ai pas");
        unmarkSelection.setOnAction(event -> applyToSelection(false));

        Label hint = new Label(
                "Cochez la colonne \u00AB J'ai \u00BB tube par tube, ou selectionnez plusieurs "
                + "lignes du tableau et employez les boutons. La selection multiple se fait avec "
                + "la touche majuscule ou commande.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        Button fromPhoto = new Button("Reconnaitre depuis une photo...");
        fromPhoto.setDisable(!recognition.isAvailable());
        fromPhoto.setTooltip(new Tooltip(recognition.isAvailable()
                ? "Photographiez votre boite : les etiquettes seront lues et rapprochees du catalogue"
                : "Demande un assistant configure (voir Parametres)"));
        fromPhoto.setOnAction(event -> recogniseFromPhoto());

        editState.getStyleClass().add("hint");
        editState.setWrapText(true);
        recognitionState.getStyleClass().add("hint");
        recognitionState.setWrapText(true);
        recognised.setPlaceholder(new Label("Aucune lecture."));
        recognised.setPrefHeight(150);
        recognised.setCellFactory(view -> new IdentificationCell());

        Button applyRecognised = new Button("Ajouter les tubes reconnus");
        applyRecognised.setOnAction(event -> applyRecognised());

        Button addPaint = new Button("Ajouter un tube...");
        addPaint.setTooltip(new Tooltip("Pour un tube que vous possedez et qui n'est pas au catalogue"));
        addPaint.setOnAction(event -> addPaint());

        Button editPaint = new Button("Corriger le tube selectionne...");
        editPaint.setOnAction(event -> editSelectedPaint());

        VBox content = new VBox(10, ownedCount,
                new HBox(8, markSelection, unmarkSelection), none, hint,
                new javafx.scene.control.Separator(),
                new HBox(8, addPaint, editPaint), editState,
                new javafx.scene.control.Separator(),
                fromPhoto, recognitionState, recognised, applyRecognised);

        table.getSelectionModel().setSelectionMode(javafx.scene.control.SelectionMode.MULTIPLE);
        refreshOwnedCount();
        return new Card("Mes tubes", content);
    }

    /** Lit une photo de tubes et propose les rapprochements, sans rien decider. */
    private void recogniseFromPhoto() {
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Photo de vos tubes");
        chooser.getExtensionFilters().add(
                new FileChooser.ExtensionFilter("Images", "*.png", "*.jpg", "*.jpeg", "*.heic"));
        File file = chooser.showOpenDialog(table.getScene().getWindow());
        if (file == null) {
            return;
        }

        recognised.getItems().clear();
        recognitionState.setText("Lecture de la photo en cours...");

        Task<List<TubeRecognitionService.Identification>> task = new Task<>() {
            @Override
            protected List<TubeRecognitionService.Identification> call() throws Exception {
                return recognition.identify(Files.readAllBytes(file.toPath()), catalog.findAll(), null);
            }
        };
        task.setOnSucceeded(event -> {
            var found = task.getValue();
            recognised.getItems().setAll(found);
            long matched = found.stream().filter(i -> i.match().isPresent()).count();
            recognitionState.setText(found.isEmpty()
                    ? "Aucune etiquette lisible sur cette photo."
                    : "%d tubes lus, %d rapproches du catalogue. Verifiez avant d'ajouter."
                            .formatted(found.size(), matched));
        });
        task.setOnFailed(event -> {
            Throwable error = task.getException();
            recognitionState.setText(error == null ? "La lecture a echoue." : error.getMessage());
        });
        Thread.ofPlatform().daemon().name("tube-recognition").start(task);
    }

    /** N'ajoute que les rapprochements surs : un tube mal reconnu fausserait tout le reste. */
    private void applyRecognised() {
        List<OilPaint> reliable = recognised.getItems().stream()
                .map(TubeRecognitionService.Identification::match)
                .flatMap(java.util.Optional::stream)
                .filter(match -> match.isReliable())
                .map(match -> match.paint())
                .distinct()
                .toList();

        if (reliable.isEmpty()) {
            recognitionState.setText("Aucun rapprochement assez sur pour etre ajoute automatiquement. "
                    + "Cochez-les a la main dans le tableau.");
            return;
        }
        catalog.setOwned(reliable, true);
        table.refresh();
        refreshOwnedCount();
        recognitionState.setText("%d tubes ajoutes a votre inventaire.".formatted(reliable.size()));
    }

    /** Saisir un tube absent du catalogue : aucun catalogue livre n'est complet. */
    private void addPaint() {
        String brand = brandFilter.getValue();
        PaintEditor.show(null, ALL_BRANDS.equals(brand) ? null : brand).ifPresent(result -> {
            try {
                var created = catalog.add(result.brand(), result.name(), result.code(),
                        result.pigments(), result.color(), result.opacity(),
                        result.drying(), result.tintingStrength());
                rebuildBrandFilter();
                refresh();
                refreshOwnedCount();
                table.getSelectionModel().select(created);
                editState.setText("%s ajoute et marque comme possede.".formatted(created.displayName()));
            } catch (IllegalArgumentException e) {
                editState.setText(e.getMessage());
            }
        });
    }

    private void editSelectedPaint() {
        OilPaint selected = table.getSelectionModel().getSelectedItem();
        if (selected == null) {
            editState.setText("Selectionnez d'abord un tube dans le tableau.");
            return;
        }
        PaintEditor.show(selected, null).ifPresent(result -> {
            try {
                catalog.update(selected, result.code(), result.pigments(),
                        result.opacity(), result.drying(), result.tintingStrength());
                catalog.recordMasstone(selected, result.color());
                table.refresh();
                findClosest();
                editState.setText("%s corrige.".formatted(selected.displayName()));
            } catch (IllegalArgumentException e) {
                editState.setText(e.getMessage());
            }
        });
    }

    /** La liste des marques peut s'enrichir d'un tube ajoute a la main. */
    private void rebuildBrandFilter() {
        String current = brandFilter.getValue();
        brandFilter.getItems().setAll(ALL_BRANDS);
        catalog.findAll().stream().map(OilPaint::getBrand).distinct().sorted()
                .forEach(brandFilter.getItems()::add);
        brandFilter.setValue(brandFilter.getItems().contains(current) ? current : ALL_BRANDS);
    }

    private void applyToSelection(boolean owned) {
        var selected = List.copyOf(table.getSelectionModel().getSelectedItems());
        if (selected.isEmpty()) {
            ownedCount.setText("Selectionnez d'abord des lignes dans le tableau.");
            return;
        }
        catalog.setOwned(selected, owned);
        table.refresh();
        refreshOwnedCount();
    }

    private void refreshOwnedCount() {
        long owned = catalog.countOwned();
        long total = catalog.findAll().size();
        ownedCount.setText(owned == total
                ? "%d tubes, soit tout le catalogue : l'inventaire n'a pas encore ete fait.".formatted(total)
                : "%d tubes sur %d declares".formatted(owned, total));
    }

    // --- Calibrage du tube selectionne -------------------------------------

    /**
     * Attribuer a un tube une couleur mesuree sur ses propres ecouvillons.
     *
     * <p>Sa place est ici, sur la fiche du tube, et non dans la Pipette : c'est dans ce
     * tableau qu'on voit ce qui manque, et c'est ici qu'on dispose deja de la recherche
     * et des filtres pour trouver le bon tube. La Pipette mesure, le Catalogue attribue.</p>
     */
    private Node calibrationPanel() {
        calibrationName.getStyleClass().add("result-summary");
        calibrationName.setWrapText(true);
        calibrationState.getStyleClass().add("hint");
        calibrationState.setWrapText(true);

        calibrationPicker.setPrefWidth(150);

        useSampled.setOnAction(event -> {
            if (sampled.get() != null) {
                calibrationPicker.setValue(Formats.toFx(sampled.get()));
            }
        });
        sampled.valueProperty().addListener((obs, old, colour) -> updateSampledButton());
        updateSampledButton();

        Button asMasstone = new Button("Ton de masse");
        asMasstone.setOnAction(event -> record(false));

        Button asTint = new Button("Teinte diluee 1:9");
        asTint.setOnAction(event -> record(true));

        HBox current = new HBox(10, new Label("Actuel"), calibrationMasstone,
                new Label("coupe de blanc"), calibrationTint);
        current.setAlignment(Pos.CENTER_LEFT);

        HBox source = new HBox(10, new Label("Couleur mesuree"), calibrationPicker, useSampled);
        source.setAlignment(Pos.CENTER_LEFT);

        HBox actions = new HBox(10, new Label("Enregistrer comme"), asMasstone, asTint);
        actions.setAlignment(Pos.CENTER_LEFT);

        Label hint = new Label(
                "Peignez un ecouvillon de couleur pure et un autre d'une part de couleur pour neuf "
                + "de blanc de titane, photographiez-les sous lumiere neutre, relevez-les dans "
                + "l'onglet Pipette puis revenez ici les attribuer. C'est la seconde mesure qui "
                + "fait passer le tube au modele a deux constantes, et elle vaut mieux que toute "
                + "valeur de catalogue : elle vient de votre peinture, sous votre lumiere.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        VBox content = new VBox(12, calibrationName, current, source, actions, calibrationState, hint);
        showCalibration(null);
        return new Card("Calibrer le tube selectionne", content);
    }

    private void updateSampledButton() {
        Rgb colour = sampled.get();
        useSampled.setDisable(colour == null);
        useSampled.setText(colour == null
                ? "Aucune teinte relevee"
                : "Utiliser la teinte relevee " + colour.toHex());
    }

    private void showCalibration(OilPaint paint) {
        if (paint == null) {
            calibrationName.setText("Aucun tube selectionne");
            calibrationState.setText("Choisissez une ligne dans le tableau pour la calibrer.");
            calibrationMasstone.setColor(new Rgb(0.15, 0.14, 0.13), "?");
            calibrationTint.setColor(new Rgb(0.15, 0.14, 0.13), "?");
            return;
        }
        calibrationName.setText(paint.displayName());
        calibrationMasstone.setColor(paint.color());
        if (paint.getTintHex() == null) {
            calibrationTint.setColor(new Rgb(0.15, 0.14, 0.13), "?");
            calibrationState.setText("Teinte diluee inconnue : ce tube se melange encore "
                    + "avec le modele a une seule constante.");
        } else {
            calibrationTint.setColor(Rgb.ofHex(paint.getTintHex()));
            calibrationState.setText("Teinte diluee connue : melange a deux constantes actif.");
        }
    }

    private void record(boolean asTint) {
        OilPaint selected = table.getSelectionModel().getSelectedItem();
        if (selected == null) {
            return;
        }
        Rgb measured = Formats.fromFx(calibrationPicker.getValue());
        OilPaint saved = asTint ? catalog.recordTint(selected, measured)
                : catalog.recordMasstone(selected, measured);

        showCalibration(saved);
        table.refresh();
        findClosest();
        calibrationState.setText("%s de %s enregistre : %s.%s".formatted(
                asTint ? "Teinte diluee" : "Ton de masse", saved.getName(), measured.toHex(),
                asTint ? " Ce tube passe au melange a deux constantes." : ""));
    }

    /**
     * Colonne de possession, cochable directement.
     *
     * <p>Le catalogue livre tout coche, ce qui ne veut rien dire tant que le peintre n'a
     * pas fait son inventaire. Le bouton "Je ne possede rien" sert a partir de zero.</p>
     */
    private TableColumn<OilPaint, Void> ownedColumn() {
        TableColumn<OilPaint, Void> column = new TableColumn<>("J'ai");
        column.setPrefWidth(60);
        column.setSortable(false);
        column.setCellFactory(c -> new javafx.scene.control.TableCell<>() {

            private final CheckBox box = new CheckBox();

            {
                box.setOnAction(event -> {
                    OilPaint paint = paintAt(getIndex());
                    if (paint != null) {
                        catalog.setOwned(paint, box.isSelected());
                        refreshOwnedCount();
                    }
                });
            }

            @Override
            protected void updateItem(Void item, boolean empty) {
                super.updateItem(item, empty);
                OilPaint paint = empty ? null : paintAt(getIndex());
                if (paint == null) {
                    setGraphic(null);
                } else {
                    box.setSelected(paint.isInStock());
                    setGraphic(box);
                }
            }

            private OilPaint paintAt(int index) {
                return index >= 0 && index < getTableView().getItems().size()
                        ? getTableView().getItems().get(index) : null;
            }
        });
        return column;
    }

    /** Colonne dont la valeur est rendue par une etiquette arrondie plutot que par du texte. */
    private static TableColumn<OilPaint, Void> pillColumn(String title, double width,
                                                          java.util.function.Function<OilPaint, javafx.scene.control.Label> renderer) {
        TableColumn<OilPaint, Void> column = new TableColumn<>(title);
        column.setPrefWidth(width);
        column.setSortable(false);
        column.setCellFactory(c -> new javafx.scene.control.TableCell<>() {
            @Override
            protected void updateItem(Void item, boolean empty) {
                super.updateItem(item, empty);
                OilPaint paint = empty || getIndex() >= getTableView().getItems().size()
                        ? null : getTableView().getItems().get(getIndex());
                setGraphic(paint == null ? null : renderer.apply(paint));
            }
        });
        return column;
    }

    private Node matchPanel() {
        targetPicker.setPrefWidth(200);
        targetPicker.setOnAction(event -> findClosest());

        ListView<PaintMatch> list = new ListView<>(matches);
        list.setCellFactory(view -> new MatchCell());
        VBox.setVgrow(list, Priority.ALWAYS);

        Label explanation = new Label("""
                Choisissez une teinte a atteindre : le catalogue est classe par ecart percu \
                (CIEDE2000). Sous 2, l'oeil ne fait plus la difference sur une figurine.""");
        explanation.setWrapText(true);
        explanation.getStyleClass().add("hint");

        HBox picker = new HBox(10, new Label("Teinte visee"), targetPicker);
        picker.setAlignment(javafx.geometry.Pos.CENTER_LEFT);

        VBox content = new VBox(12, picker, explanation, list);
        VBox.setVgrow(list, Priority.ALWAYS);
        return new Card("Quel tube s'en approche le plus ?", content);
    }

    private void refresh() {
        String brand = brandFilter.getValue();
        paints.setAll(catalog.search(searchField.getText()).stream()
                .filter(paint -> !inStockOnly.isSelected() || paint.isInStock())
                .filter(paint -> brand == null || ALL_BRANDS.equals(brand) || brand.equals(paint.getBrand()))
                .toList());
    }

    private void findClosest() {
        matches.setAll(catalog.findClosest(Formats.fromFx(targetPicker.getValue()), inStockOnly.isSelected(), MATCH_COUNT));
    }

    /** Pastille de couleur dans un tableau, la couleur etant extraite du tube a l'affichage. */
    private static class TableCellWithSwatch extends javafx.scene.control.TableCell<OilPaint, OilPaint> {

        private final ColorSwatch swatch = new ColorSwatch(38, 22);
        private final java.util.function.Function<OilPaint, be.asmolabs.palettier.core.color.Rgb> extractor;

        TableCellWithSwatch(java.util.function.Function<OilPaint, be.asmolabs.palettier.core.color.Rgb> extractor) {
            this.extractor = extractor;
        }

        @Override
        protected void updateItem(OilPaint paint, boolean empty) {
            super.updateItem(paint, empty);
            var color = empty || paint == null ? null : extractor.apply(paint);
            if (color == null) {
                setGraphic(null);
                // Teinte diluee non relevee : ce tube reste sur le modele a constante unique.
                setText(empty || paint == null ? null : "\u2014");
            } else {
                swatch.setColor(color);
                setText(null);
                setGraphic(swatch);
            }
        }
    }

    /** Un tube lu sur la photo, et ce a quoi on l'a rapproche. */
    private static class IdentificationCell extends ListCell<TubeRecognitionService.Identification> {

        @Override
        protected void updateItem(TubeRecognitionService.Identification item, boolean empty) {
            super.updateItem(item, empty);
            if (empty || item == null) {
                setGraphic(null);
                return;
            }
            Label read = new Label(item.label());
            Label verdict = new Label(item.match()
                    .map(m -> "%s  -  confiance %.0f %%%s".formatted(m.paint().displayName(),
                            m.confidence() * 100, m.isReliable() ? "" : ", a verifier"))
                    .orElse("aucune fiche correspondante"));
            verdict.getStyleClass().add("hint");
            verdict.setWrapText(true);

            VBox layout = new VBox(2, read, verdict);
            if (item.match().filter(m -> m.isReliable()).isPresent()) {
                layout.getChildren().add(Pill.of("sera ajoute", "pill-fast"));
            }
            setGraphic(layout);
        }
    }

    private static class MatchCell extends ListCell<PaintMatch> {

        private final ColorSwatch swatch = new ColorSwatch(44, 44);
        private final Label title = new Label();
        private final Label detail = new Label();
        private final HBox layout;

        MatchCell() {
            detail.getStyleClass().add("hint");
            VBox text = new VBox(2, title, detail);
            layout = new HBox(10, swatch, text);
            layout.setPadding(new Insets(4));
        }

        @Override
        protected void updateItem(PaintMatch match, boolean empty) {
            super.updateItem(match, empty);
            if (empty || match == null) {
                setGraphic(null);
                return;
            }
            swatch.setColor(match.paint().color());
            title.setText(match.paint().displayName());
            detail.setText("%s  -  ecart %.1f".formatted(match.verdict(), match.deltaE()));
            setGraphic(layout);
        }
    }
}
