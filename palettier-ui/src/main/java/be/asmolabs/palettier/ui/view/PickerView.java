package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.service.ColorMixService;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.PaletteService;
import be.asmolabs.palettier.core.service.ProjectService;
import be.asmolabs.palettier.core.service.SampledPlanService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.SampledColor;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.Dialogs;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;
import javafx.beans.binding.Bindings;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Cursor;
import javafx.scene.Group;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Tooltip;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.SplitPane;
import javafx.scene.control.Spinner;
import javafx.scene.control.TextField;
import javafx.scene.control.TextInputDialog;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToolBar;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.image.PixelReader;
import javafx.scene.input.DragEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.input.ScrollEvent;
import javafx.scene.input.TransferMode;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.shape.Circle;
import javafx.stage.FileChooser;
import org.springframework.stereotype.Component;

/**
 * Pipette : relever une teinte sur une photo, ou la saisir en hexadecimal, et obtenir
 * le melange qui l'approche avec les tubes d'une palette donnee.
 *
 * <p>C'est le chemin inverse du melangeur : on part du resultat voulu, vu sur une
 * reference ou une piece existante, et on remonte aux tubes.</p>
 */
@Component
public class PickerView implements AppView {

    private static final int SUGGESTION_COUNT = 8;
    private static final double ZOOM_STEP = 1.25;
    private static final double MIN_SCALE = 0.05;
    private static final double MAX_SCALE = 16.0;

    private final PaintCatalogService catalog;
    private final PaletteService palettes;
    private final ColorMixService mixer;
    private final SampledColor sampled;
    private final SampledPlanService sampledPlans;
    private final ProjectService projects;

    private final ImageView imageView = new ImageView();
    private final Circle marker = new Circle(6);
    private final Group imageLayer = new Group(imageView, marker);
    private final StackPane imageHolder = new StackPane(imageLayer);
    private final ScrollPane imageScroll = new ScrollPane(imageHolder);
    private final Label imageInfo = new Label("Aucune image");
    private final Spinner<Integer> radius = new Spinner<>(0, 30, 3, 1);
    private final ToggleButton pickNeutral = new ToggleButton("Definir le point neutre");
    private final Label neutralInfo = new Label();
    private final Button zoomOut = new Button("−");
    private final Button zoomIn = new Button("+");
    private final Button zoomFit = new Button("Ajuster");
    private final Button zoomActual = new Button("100 %");
    private final Label zoomLabel = new Label();

    private final ColorSwatch targetSwatch = new ColorSwatch(120, 90);
    private final TextField hexField = new TextField("#9A8F80");
    private final Label sampleDetail = new Label();
    private final ComboBox<Scope> scopeChoice = new ComboBox<>();
    private final Button searchButton = new Button("Trouver le melange");
    private final ComboBox<Integer> maxPaints = new ComboBox<>();
    private final ObservableList<MixSuggestion> suggestions = FXCollections.observableArrayList();
    private final ListView<MixSuggestion> suggestionList = new ListView<>(suggestions);

    // --- Groupes de relevés : le chemin mesuré vers un projet ---
    private final ObservableList<SampleGroup> groups = FXCollections.observableArrayList();
    private final ListView<SampleGroup> groupList = new ListView<>(groups);
    private final TextField groupName = new TextField();
    private final ComboBox<Palette> projectPalette = new ComboBox<>();
    private final FlowPane groupSamples = new FlowPane(8, 8);
    private final Label groupState = new Label();

    /**
     * Une partie du sujet en cours de relevé : un nom, et les couleurs prélevées dessus.
     *
     * <p>Mutable et non persistant : c'est un brouillon de travail, qui ne prend corps
     * qu'au moment ou l'on en fait un projet.</p>
     */
    private static final class SampleGroup {

        private String name;
        private final List<Rgb> samples = new ArrayList<>();

        SampleGroup(String name) {
            this.name = name;
        }

        @Override
        public String toString() {
            return "%s (%d point%s)".formatted(name, samples.size(), samples.size() > 1 ? "s" : "");
        }
    }

    /** Facteur d'affichage courant, 1 signifiant un pixel d'image par pixel d'ecran. */
    private double scale = 1.0;

    /** Vrai tant que l'image doit tenir entiere dans la zone visible. */
    private boolean fitToViewport = true;
    /** Couleur brute du point designe comme neutre, avant toute correction. */
    private Rgb rawNeutral;
    private Rgb target = Rgb.ofHex("#9A8F80");
    /** Evite de rejouer la mise a jour quand le champ hexadecimal se reecrit lui-meme. */
    private boolean syncing;

    public PickerView(PaintCatalogService catalog, PaletteService palettes, ColorMixService mixer,
                      SampledColor sampled, SampledPlanService sampledPlans, ProjectService projects) {
        this.catalog = catalog;
        this.palettes = palettes;
        this.mixer = mixer;
        this.sampled = sampled;
        this.sampledPlans = sampledPlans;
        this.projects = projects;
    }

    /** Ensemble de tubes dans lequel chercher le melange. */
    private record Scope(String label, Supplier<List<OilPaint>> candidates) {
        @Override
        public String toString() {
            return label;
        }
    }

    @Override
    public String title() {
        return "Pipette";
    }

    @Override
    public String subtitle() {
        return "Relevez une teinte sur une photo ou saisissez-la, et obtenez le melange a faire avec votre palette.";
    }

    @Override
    public int order() {
        // Trouver une couleur : la relever.
        return 30;
    }

    @Override
    public Node create() {
        SplitPane split = new SplitPane(imagePanel(), analysisPanel());
        split.setDividerPositions(0.56);
        applyTarget(target);
        return split;
    }

    // --- Photo -------------------------------------------------------------

    private Node imagePanel() {
        marker.setVisible(false);
        marker.setFill(Color.TRANSPARENT);
        marker.setStroke(Color.WHITE);
        marker.setStrokeWidth(1.5);
        marker.setMouseTransparent(true);

        imageView.setPreserveRatio(true);
        imageView.setSmooth(false);
        imageView.setCursor(Cursor.CROSSHAIR);
        imageView.setOnMouseClicked(this::onImageClicked);
        imageView.setOnMouseDragged(this::onImageClicked);

        // Le conteneur fait au moins la taille de la zone visible : l'image y est centree
        // quand elle est petite, et fait apparaitre les ascenseurs quand elle deborde.
        imageHolder.minWidthProperty().bind(Bindings.createDoubleBinding(
                () -> imageScroll.getViewportBounds().getWidth(), imageScroll.viewportBoundsProperty()));
        imageHolder.minHeightProperty().bind(Bindings.createDoubleBinding(
                () -> imageScroll.getViewportBounds().getHeight(), imageScroll.viewportBoundsProperty()));

        imageScroll.setPannable(true);
        imageScroll.setFitToWidth(false);
        imageScroll.setFitToHeight(false);
        imageScroll.setOnDragOver(this::onDragOver);
        imageScroll.setOnDragDropped(this::onDragDropped);
        // Tant que l'image doit tenir entiere, elle suit les redimensionnements de la fenetre.
        imageScroll.viewportBoundsProperty().addListener((obs, old, bounds) -> {
            if (fitToViewport) {
                applyFit();
            }
        });
        // Molette avec la touche de commande, et pincement sur pave tactile.
        imageScroll.addEventFilter(ScrollEvent.SCROLL, event -> {
            if (event.isShortcutDown()) {
                zoomBy(event.getDeltaY() > 0 ? ZOOM_STEP : 1 / ZOOM_STEP);
                event.consume();
            }
        });
        imageScroll.setOnZoom(event -> {
            zoomBy(event.getZoomFactor());
            event.consume();
        });
        VBox.setVgrow(imageScroll, Priority.ALWAYS);

        Button open = new Button("Ouvrir une image...");
        // Pas de bouton par defaut ici : la touche Entree ouvrirait un selecteur de
        // fichiers alors qu'on est en train de saisir un code hexadecimal.
        open.setOnAction(event -> chooseFile());

        radius.setPrefWidth(70);
        zoomOut.setOnAction(event -> zoomBy(1 / ZOOM_STEP));
        zoomIn.setOnAction(event -> zoomBy(ZOOM_STEP));
        zoomFit.setOnAction(event -> applyFit());
        zoomActual.setOnAction(event -> setScale(1.0));
        zoomLabel.getStyleClass().add("hint");
        zoomLabel.setMinWidth(110);

        pickNeutral.setOnAction(event -> updateNeutralInfo());
        Button clearNeutral = new Button("Retirer");
        clearNeutral.setOnAction(event -> {
            rawNeutral = null;
            pickNeutral.setSelected(false);
            updateNeutralInfo();
        });
        neutralInfo.getStyleClass().add("hint");

        ToolBar bar = new ToolBar(open, new Label("Rayon"), radius,
                zoomOut, zoomLabel, zoomIn, zoomFit, zoomActual);
        HBox neutralRow = new HBox(8, pickNeutral, clearNeutral, neutralInfo);
        neutralRow.setAlignment(Pos.CENTER_LEFT);

        imageInfo.getStyleClass().add("hint");

        Label hint = new Label("""
                Glissez une photo ici, ou ouvrez-en une. Elle s'affiche entiere ; zoomez avec \
                les boutons, la molette touche commande enfoncee, ou le pincement du pave \
                tactile. Cliquez pour relever la teinte : \
                un rayon de quelques pixels moyenne le bruit du capteur, ce qui vaut toujours \
                mieux qu'un pixel isole.

                Si la photo tire au jaune ou au bleu, designez d'abord un point cense etre \
                gris ou blanc neutre : la dominante sera retiree de tous les relevés suivants.""");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        BorderPane body = new BorderPane(imageScroll);
        body.setTop(new VBox(6, bar, neutralRow));
        body.setBottom(new VBox(6, imageInfo, hint));
        BorderPane.setMargin(body.getBottom(), new Insets(10, 0, 0, 0));

        radius.valueProperty().addListener((obs, old, value) -> applyScale());
        updateZoomLabel();
        updateNeutralInfo();
        return new Card("Photo de reference", body);
    }

    private void chooseFile() {
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Choisir une photo");
        chooser.getExtensionFilters().add(
                new FileChooser.ExtensionFilter("Images", "*.png", "*.jpg", "*.jpeg", "*.gif", "*.bmp"));
        File file = chooser.showOpenDialog(imageScroll.getScene().getWindow());
        if (file != null) {
            loadImage(file);
        }
    }

    private void onDragOver(DragEvent event) {
        if (event.getDragboard().hasFiles()) {
            event.acceptTransferModes(TransferMode.COPY);
        }
        event.consume();
    }

    private void onDragDropped(DragEvent event) {
        List<File> files = event.getDragboard().getFiles();
        boolean loaded = false;
        if (files != null && !files.isEmpty()) {
            loaded = loadImage(files.getFirst());
        }
        event.setDropCompleted(loaded);
        event.consume();
    }

    private boolean loadImage(File file) {
        Image image = new Image(file.toURI().toString(), false);
        if (image.isError()) {
            imageInfo.setText("Format d'image non reconnu : " + file.getName());
            return false;
        }
        imageView.setImage(image);
        marker.setVisible(false);
        // A l'ouverture, on montre la photo entiere : c'est la vue dont on a besoin pour
        // reperer la zone a prelever, avant d'y zoomer.
        applyFit();
        imageInfo.setText("%s  -  %.0f x %.0f pixels"
                .formatted(file.getName(), image.getWidth(), image.getHeight()));
        return true;
    }

    /** Ajuste l'image pour qu'elle tienne entiere dans la zone visible. */
    private void applyFit() {
        fitToViewport = true;
        Image image = imageView.getImage();
        double width = imageScroll.getViewportBounds().getWidth();
        double height = imageScroll.getViewportBounds().getHeight();
        if (image == null || width <= 0 || height <= 0) {
            updateZoomLabel();
            return;
        }
        scale = Math.clamp(Math.min(width / image.getWidth(), height / image.getHeight()),
                MIN_SCALE, MAX_SCALE);
        applyScale();
    }

    private void zoomBy(double factor) {
        setScale(scale * factor);
    }

    private void setScale(double wanted) {
        fitToViewport = false;
        scale = Math.clamp(wanted, MIN_SCALE, MAX_SCALE);
        applyScale();
    }

    private void applyScale() {
        Image image = imageView.getImage();
        if (image != null) {
            imageView.setFitWidth(image.getWidth() * scale);
            // Au-dela de la taille reelle, on veut voir les pixels, pas un flou d'interpolation.
            imageView.setSmooth(scale < 1.0);
            marker.setRadius(Math.max(4, radius.getValue() * scale));
        }
        updateZoomLabel();
    }

    private void updateZoomLabel() {
        zoomLabel.setText(imageView.getImage() == null
                ? "-"
                : "%.0f %%%s".formatted(scale * 100, fitToViewport ? " (ajuste)" : ""));
    }

    // --- Echantillonnage ---------------------------------------------------

    private void onImageClicked(MouseEvent event) {
        Image image = imageView.getImage();
        if (image == null) {
            return;
        }

        // Des coordonnees a l'ecran vers les coordonnees de l'image, quel que soit le zoom.
        double displayedWidth = imageView.getBoundsInLocal().getWidth();
        double toImage = image.getWidth() / displayedWidth;
        int centreX = (int) Math.floor(event.getX() * toImage);
        int centreY = (int) Math.floor(event.getY() * toImage);

        List<Rgb> samples = sampleAround(image, centreX, centreY, radius.getValue());
        if (samples.isEmpty()) {
            return;
        }
        Rgb raw = Colors.average(samples);

        marker.setCenterX(event.getX());
        marker.setCenterY(event.getY());
        marker.setRadius(Math.max(4, radius.getValue() / toImage));
        marker.setVisible(true);

        if (pickNeutral.isSelected()) {
            rawNeutral = raw;
            pickNeutral.setSelected(false);
            updateNeutralInfo();
            return;
        }
        applyTarget(rawNeutral == null ? raw : Colors.neutralise(raw, rawNeutral));
    }

    /** Moyenne un carre de pixels autour du point clique, en restant dans l'image. */
    private static List<Rgb> sampleAround(Image image, int centreX, int centreY, int radius) {
        PixelReader reader = image.getPixelReader();
        if (reader == null) {
            return List.of();
        }
        int width = (int) image.getWidth();
        int height = (int) image.getHeight();

        List<Rgb> samples = new ArrayList<>();
        for (int y = centreY - radius; y <= centreY + radius; y++) {
            for (int x = centreX - radius; x <= centreX + radius; x++) {
                if (x >= 0 && x < width && y >= 0 && y < height) {
                    Color pixel = reader.getColor(x, y);
                    samples.add(new Rgb(pixel.getRed(), pixel.getGreen(), pixel.getBlue()));
                }
            }
        }
        return samples;
    }

    private void updateNeutralInfo() {
        if (rawNeutral != null) {
            neutralInfo.setText("Dominante corrigee d'apres %s".formatted(rawNeutral.toHex()));
        } else if (pickNeutral.isSelected()) {
            neutralInfo.setText("Cliquez sur un point de l'image cense etre gris ou blanc neutre.");
        } else {
            neutralInfo.setText("Aucune correction de dominante.");
        }
    }

    // --- Teinte visee et recherche -----------------------------------------

    private Node analysisPanel() {
        hexField.setPrefWidth(140);
        hexField.getStyleClass().add("hex-field");
        hexField.textProperty().addListener((obs, old, value) -> onHexTyped(value));

        sampleDetail.getStyleClass().add("hint");
        sampleDetail.setWrapText(true);

        HBox targetRow = new HBox(14, targetSwatch,
                new VBox(8, new Label("Code hexadecimal"), hexField, sampleDetail));
        targetRow.setAlignment(Pos.TOP_LEFT);

        scopeChoice.setPrefWidth(280);
        scopeChoice.setOnShowing(event -> rebuildScopes());
        rebuildScopes();

        maxPaints.getItems().setAll(1, 2, 3, 4, 5);
        maxPaints.setValue(3);
        maxPaints.setPrefWidth(150);
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

        searchButton.setOnAction(event -> search());

        HBox controls = new HBox(10, new Label("Chercher dans"), scopeChoice,
                maxPaints, searchButton);
        controls.setAlignment(Pos.CENTER_LEFT);

        Label parsimony = new Label(
                "C'est un maximum, pas un objectif : a resultat perceptuellement identique, la "
                + "recherche prefere toujours le melange le plus simple, et les tubes "
                + "supplementaires n'apparaissent que s'ils servent. Sur tout le catalogue, deux "
                + "ou trois suffisent presque toujours. C'est sur une palette courte qu'il faut "
                + "monter : certaines teintes ne s'obtiennent qu'avec les trois primaires plus un "
                + "blanc et une terre. Au-dela de trois tubes la recherche devient sensiblement "
                + "plus longue sur le catalogue entier ; sur une palette, elle reste immediate.");
        parsimony.setWrapText(true);
        parsimony.getStyleClass().add("hint");

        suggestionList.setCellFactory(view -> new SuggestionCell());
        suggestionList.setPlaceholder(new Label("Relevez une teinte, puis lancez la recherche."));
        VBox.setVgrow(suggestionList, Priority.ALWAYS);

        Card targetCard = new Card("Teinte relevee", targetRow);
        Card mixCard = new Card("Melange a faire",
                "Chaque proposition indique les parts a doser et l'ecart avec la teinte visee.",
                new VBox(12, controls, parsimony, suggestionList));
        suggestionList.setMinHeight(180);
        VBox.setVgrow(mixCard, Priority.ALWAYS);

        VBox panel = new VBox(14, targetCard, groupsCard(), mixCard);

        // Trois cartes dont une haute : sans defilement, la derniere se fait rogner des que
        ScrollPane scroll = new ScrollPane(panel);
        scroll.setFitToWidth(true);
        scroll.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        panel.setMinHeight(Region.USE_PREF_SIZE);
        return scroll;
    }

    // --- Groupes ------------------------------------------------------------

    /**
     * Composer un projet a partir de relevés plutot que d'une proposition.
     *
     * <p>On nomme une partie du sujet, on y ajoute autant de points qu'on veut -- le creux
     * de l'orbite, la joue, l'arete du nez -- et l'application en deduit le degrade : le
     * point median tient la base, les plus sombres deviennent les ombres, les plus clairs
     * les lumieres. Rien n'est devine, chaque couche vient d'une mesure.</p>
     */
    private Node groupsCard() {
        groupName.setPromptText("nom du groupe, par exemple Peau");
        groupName.setOnAction(event -> createGroup());
        HBox.setHgrow(groupName, Priority.ALWAYS);

        Button create = new Button("Nouveau groupe");
        create.setOnAction(event -> createGroup());

        groupList.setPrefHeight(110);
        groupList.setPlaceholder(new Label("Aucun groupe. Nommez la premiere partie du sujet."));
        groupList.getSelectionModel().selectedItemProperty()
                .addListener((obs, old, group) -> showGroup(group));

        Button add = new Button("Ajouter le releve au groupe");
        add.setOnAction(event -> addSampleToGroup());

        Button removeGroup = new Button("Supprimer le groupe");
        removeGroup.setOnAction(event -> {
            SampleGroup selected = groupList.getSelectionModel().getSelectedItem();
            if (selected != null) {
                groups.remove(selected);
                showGroup(null);
            }
        });

        projectPalette.setPrefWidth(220);
        projectPalette.setConverter(new javafx.util.StringConverter<>() {
            @Override
            public String toString(Palette palette) {
                return palette == null ? "" : "%s (%d tubes)".formatted(
                        palette.getName(), palette.getPaints().size());
            }

            @Override
            public Palette fromString(String value) {
                return null;
            }
        });
        projectPalette.setOnShowing(event -> projectPalette.getItems().setAll(palettes.findAll()));
        projectPalette.getItems().setAll(palettes.findAll());
        if (!projectPalette.getItems().isEmpty()) {
            projectPalette.setValue(projectPalette.getItems().getFirst());
        }

        Button toProject = new Button("Creer un projet");
        toProject.setDefaultButton(false);
        toProject.setOnAction(event -> createProject());

        groupState.getStyleClass().add("hint");
        groupState.setWrapText(true);

        Label hint = new Label(
                "Relevez plusieurs points par groupe : le plus sombre deviendra l'ombre profonde, "
                + "le median la base, le plus clair le point lumineux. Trois points suffisent a "
                + "obtenir un degrade, cinq donnent un modele complet.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        VBox content = new VBox(10,
                new HBox(8, groupName, create),
                groupList,
                new HBox(8, add, removeGroup),
                groupSamples,
                new HBox(8, new Label("Palette"), projectPalette, toProject),
                groupState, hint);

        showGroup(null);
        return new Card("Groupes de releves", content);
    }

    private void createGroup() {
        String name = groupName.getText() == null ? "" : groupName.getText().trim();
        if (name.isEmpty()) {
            groupState.setText("Donnez un nom au groupe.");
            return;
        }
        SampleGroup group = new SampleGroup(name);
        groups.add(group);
        groupName.clear();
        groupList.getSelectionModel().select(group);
    }

    private void addSampleToGroup() {
        SampleGroup group = groupList.getSelectionModel().getSelectedItem();
        if (group == null) {
            groupState.setText("Choisissez d'abord un groupe.");
            return;
        }
        group.samples.add(target);
        groupList.refresh();
        showGroup(group);
    }

    private void showGroup(SampleGroup group) {
        groupSamples.getChildren().clear();
        if (group == null) {
            groupState.setText("Selectionnez un groupe pour y ajouter les teintes relevees.");
            return;
        }
        if (group.samples.isEmpty()) {
            groupState.setText("Groupe vide : relevez une teinte sur la photo, puis ajoutez-la.");
            return;
        }
        // Presentes du sombre au clair : c'est l'ordre dans lequel elles deviendront des couches.
        group.samples.stream()
                .sorted(java.util.Comparator.comparingDouble(Rgb::relativeLuminance))
                .forEach(sample -> groupSamples.getChildren().add(sampleTile(group, sample)));
        groupState.setText("%d point%s releve%s.".formatted(group.samples.size(),
                group.samples.size() > 1 ? "s" : "", group.samples.size() > 1 ? "s" : ""));
    }

    private Node sampleTile(SampleGroup group, Rgb sample) {
        ColorSwatch swatch = new ColorSwatch(44, 30);
        swatch.setColor(sample);

        Button remove = new Button("\u2715");
        remove.getStyleClass().add("icon-button");
        remove.setTooltip(new Tooltip("Retirer ce releve du groupe"));
        remove.setOnAction(event -> {
            group.samples.remove(sample);
            groupList.refresh();
            showGroup(group);
        });

        VBox tile = new VBox(2, swatch, remove);
        tile.setAlignment(Pos.CENTER);
        return tile;
    }

    private void createProject() {
        Palette palette = projectPalette.getValue();
        if (palette == null) {
            groupState.setText("Choisissez une palette.");
            return;
        }
        List<SampledPlanService.ColourGroup> definition = groups.stream()
                .map(group -> new SampledPlanService.ColourGroup(group.name, "", List.copyOf(group.samples)))
                .toList();

        try {
            TextInputDialog dialog = new TextInputDialog("Releve du " + java.time.LocalDate.now());
            dialog.setTitle("Creer un projet");
            dialog.setContentText("Nom du projet");
            Dialogs.themed(dialog);

            dialog.showAndWait().map(String::trim).filter(name -> !name.isEmpty()).ifPresent(name -> {
                PaintingPlan plan = sampledPlans.plan(name, palette, definition, 3);
                var project = projects.save(plan, palette, name);
                groupState.setText("Projet \"%s\" cree a partir de %d groupe%s. Retrouvez-le dans l'onglet Projets."
                        .formatted(project.getName(), plan.zones().size(), plan.zones().size() > 1 ? "s" : ""));
            });
        } catch (IllegalArgumentException e) {
            groupState.setText(e.getMessage());
        }
    }

    private void onHexTyped(String value) {
        if (syncing) {
            return;
        }
        String candidate = value.startsWith("#") ? value : "#" + value;
        if (!candidate.matches("#[0-9A-Fa-f]{6}")) {
            hexField.setStyle("-fx-border-color: #d9705f;");
            return;
        }
        hexField.setStyle("");
        applyTarget(Rgb.ofHex(candidate));
    }

    private void applyTarget(Rgb colour) {
        target = colour;
        // Mise a disposition du Catalogue, ou elle pourra etre attribuee a un tube.
        sampled.set(colour);
        targetSwatch.setColor(colour, colour.toHex());

        syncing = true;
        if (!hexField.getText().equalsIgnoreCase(colour.toHex())) {
            hexField.setText(colour.toHex());
        }
        syncing = false;

        sampleDetail.setText((rawNeutral == null
                ? "Teinte telle que relevee."
                : "Teinte corrigee de la dominante de la photo.")
                + " Disponible dans le Catalogue pour calibrer un tube.");
    }

    private void rebuildScopes() {
        Scope previous = scopeChoice.getValue();

        List<Scope> scopes = new ArrayList<>();
        for (Palette palette : palettes.findAll()) {
            scopes.add(new Scope("Palette : " + palette.getName(), palette::getPaints));
        }
        scopes.add(new Scope("Mes tubes en stock", catalog::findInStock));
        scopes.add(new Scope("Tout le catalogue", catalog::findAll));

        scopeChoice.getItems().setAll(scopes);
        scopeChoice.setValue(scopes.stream()
                .filter(scope -> previous != null && scope.label().equals(previous.label()))
                .findFirst()
                .orElse(scopes.getFirst()));
    }

    private void search() {
        Scope scope = scopeChoice.getValue();
        List<OilPaint> candidates = scope == null ? catalog.findAll() : scope.candidates().get();
        Rgb wanted = target;
        int limit = maxPaints.getValue() == null ? 3 : maxPaints.getValue();

        suggestions.clear();
        if (candidates.isEmpty()) {
            suggestionList.setPlaceholder(new Label("Cet ensemble ne contient aucun tube."));
            return;
        }

        suggestionList.setPlaceholder(new Label("Recherche parmi %d tubes...".formatted(candidates.size())));
        searchButton.setDisable(true);

        Task<List<MixSuggestion>> task = new Task<>() {
            @Override
            protected List<MixSuggestion> call() {
                return mixer.suggestMixes(wanted, candidates, SUGGESTION_COUNT, limit);
            }
        };
        task.setOnSucceeded(event -> {
            suggestions.setAll(task.getValue());
            searchButton.setDisable(false);
        });
        task.setOnFailed(event -> {
            suggestionList.setPlaceholder(new Label("La recherche a echoue."));
            searchButton.setDisable(false);
        });
        // Calcul purement processeur : un fil plateforme, pas un fil virtuel.
        Thread.ofPlatform().daemon().name("picker-search").start(task);
    }

    /** Une proposition : la teinte visee et le melange cote a cote, puis les doses. */
    private class SuggestionCell extends ListCell<MixSuggestion> {

        private final ColorSwatch wanted = new ColorSwatch(26, 42);
        private final ColorSwatch obtained = new ColorSwatch(26, 42);
        private final Label recipe = new Label();
        private final Label detail = new Label();
        private final HBox layout;

        SuggestionCell() {
            recipe.setWrapText(true);
            detail.getStyleClass().add("hint");
            VBox text = new VBox(3, recipe, detail);
            HBox.setHgrow(text, Priority.ALWAYS);
            HBox pair = new HBox(2, wanted, obtained);
            layout = new HBox(12, pair, text);
            layout.setAlignment(Pos.CENTER_LEFT);
            layout.setPadding(new Insets(5));
        }

        @Override
        protected void updateItem(MixSuggestion suggestion, boolean empty) {
            super.updateItem(suggestion, empty);
            if (empty || suggestion == null) {
                setGraphic(null);
                return;
            }
            wanted.setColor(target);
            obtained.setColor(suggestion.color());
            recipe.setText(suggestion.describe());
            detail.setText("%s  -  ecart %.1f  -  %s".formatted(
                    suggestion.color().toHex(), suggestion.deltaE(), verdict(suggestion.deltaE())));
            setGraphic(layout);
        }

        private String verdict(double deltaE) {
            if (deltaE < 2) {
                return "indiscernable a l'oeil";
            }
            if (deltaE < 5) {
                return "ecart visible seulement cote a cote";
            }
            if (deltaE < 10) {
                return "meme famille, a rattraper au glacis";
            }
            return "teinte differente";
        }
    }
}
