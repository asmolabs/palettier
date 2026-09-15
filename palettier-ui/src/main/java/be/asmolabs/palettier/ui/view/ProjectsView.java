package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.ProjectService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import be.asmolabs.palettier.ui.component.Dialogs;
import be.asmolabs.palettier.ui.component.LayerEditor;
import be.asmolabs.palettier.ui.component.PlanRenderer;
import be.asmolabs.palettier.ui.export.PlanPdf;
import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Locale;
import java.util.Optional;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.scene.Node;
import javafx.scene.control.Alert;
import javafx.scene.control.Button;
import javafx.scene.control.Tooltip;
import javafx.scene.control.ButtonType;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.SplitPane;
import javafx.scene.control.TextInputDialog;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.FileChooser;
import org.springframework.stereotype.Component;

/**
 * Les pieces en cours et le plan retenu pour chacune.
 *
 * <p>A l'ouverture, les dosages sont recalcules avec la palette du projet telle qu'elle
 * est aujourd'hui : un ecart qui se resserre signale que la palette s'est enrichie
 * depuis, un ecart qui s'ouvre qu'un tube en a disparu.</p>
 */
@Component
public class ProjectsView implements AppView {

    private static final DateTimeFormatter CREATED =
            DateTimeFormatter.ofPattern("d MMMM yyyy", Locale.FRENCH).withZone(ZoneId.systemDefault());

    private final ProjectService projects;
    private final PlanPdf pdf;
    private final PlanRenderer renderer = new PlanRenderer();

    private final ObservableList<Project> items = FXCollections.observableArrayList();
    private final ListView<Project> list = new ListView<>(items);
    private final VBox detail = new VBox(14);
    private final FlowPane photoStrip = new FlowPane(10, 10);
    private final FlowPane paintStrip = new FlowPane(6, 6);
    private final Label paletteState = new Label();
    private final Button syncPalette = new Button("Reprendre la palette actuelle");
    private final Label summary = new Label();
    private final Button exportPdf = new Button("Exporter en PDF");

    private PaintingPlan currentPlan;
    private Project current;

    public ProjectsView(ProjectService projects, PlanPdf pdf) {
        this.projects = projects;
        this.pdf = pdf;
    }

    @Override
    public String title() {
        return "Projets";
    }

    @Override
    public String subtitle() {
        return "Les pieces en cours et leur plan de peinture, dosages recalcules avec votre palette du moment.";
    }

    @Override
    public int order() {
        // Ce qui m'appartient : les pieces en cours.
        return 10;
    }

    @Override
    public Node create() {
        SplitPane split = new SplitPane(listPanel(), detailPanel());
        split.setDividerPositions(0.3);
        reload();
        return split;
    }

    private Node listPanel() {
        list.setCellFactory(view -> new ProjectCell());
        list.getSelectionModel().selectedItemProperty().addListener((obs, old, project) -> show(project));
        VBox.setVgrow(list, Priority.ALWAYS);

        Button rename = new Button("Renommer");
        rename.setOnAction(event -> renameSelected());

        Button delete = new Button("Supprimer");
        delete.setOnAction(event -> deleteSelected());

        Button refresh = new Button("Rafraichir");
        refresh.setOnAction(event -> reload());

        Card card = new Card("Mes projets", new VBox(12, list, new HBox(8, rename, delete, refresh)));
        VBox.setVgrow(card, Priority.ALWAYS);
        return card;
    }

    private Node detailPanel() {
        summary.getStyleClass().add("result-summary");
        summary.setWrapText(true);

        // Le plan d'un projet est modifiable : c'est une proposition qu'on corrige a
        // l'usage, pas un verdict.
        renderer.editable(new PlanRenderer.Edits() {
            @Override
            public void editZone(int zoneIndex) {
                ProjectsView.this.editZone(zoneIndex);
            }

            @Override
            public void editLayer(int zoneIndex, int layerIndex) {
                ProjectsView.this.editLayer(zoneIndex, layerIndex);
            }
        });

        exportPdf.setDisable(true);
        exportPdf.setOnAction(event -> exportToPdf());

        detail.setPadding(new Insets(2));

        // Tout descend dans le meme defilement. Auparavant les tubes et les photos
        // occupaient une hauteur fixe au-dessus du plan, qui se retrouvait ecrase dans
        // une fenetre courte -- alors que c'est lui le contenu principal.
        VBox everything = new VBox(14, summary, new HBox(8, exportPdf),
                paintsCard(), photosCard(), detail);
        everything.setPadding(new Insets(2));

        ScrollPane scroll = new ScrollPane(everything);
        scroll.setFitToWidth(true);
        VBox.setVgrow(scroll, Priority.ALWAYS);

        Card card = new Card("Plan du projet", scroll);
        VBox.setVgrow(card, Priority.ALWAYS);
        return card;
    }

    // --- Tubes figes avec le projet ----------------------------------------

    /**
     * Les tubes avec lesquels le plan a ete etabli.
     *
     * <p>Ils appartiennent au projet, pas a la palette : c'est ce qui permet de reprendre
     * une piece des mois plus tard a l'identique. Quand la palette d'origine a change
     * depuis, on le signale et on propose d'en profiter -- sans jamais l'imposer.</p>
     */
    private Node paintsCard() {
        paletteState.getStyleClass().add("hint");
        paletteState.setWrapText(true);

        syncPalette.setVisible(false);
        syncPalette.setManaged(false);
        syncPalette.setOnAction(event -> {
            if (current != null) {
                current = projects.syncWithPalette(current);
                show(current);
            }
        });

        paintStrip.setPrefWrapLength(560);
        return new Card("Tubes du projet", new VBox(10, paintStrip, paletteState, syncPalette));
    }

    private void refreshPaints() {
        paintStrip.getChildren().clear();
        syncPalette.setVisible(false);
        syncPalette.setManaged(false);

        if (current == null) {
            paletteState.setText("");
            return;
        }

        var paints = current.effectivePaints();
        if (paints.isEmpty()) {
            paletteState.setText("Aucun tube conserve avec ce projet : les melanges ne peuvent pas etre calcules.");
            return;
        }
        paints.forEach(paint -> paintStrip.getChildren().add(paintTile(paint)));

        if (current.divergesFromPalette()) {
            paletteState.setText(("La palette %s a change depuis l'enregistrement. Le projet garde ses "
                    + "propres tubes, donc son plan reste reproductible. Vous pouvez reprendre la "
                    + "composition actuelle : les couleurs visees ne bougeront pas, seuls les melanges "
                    + "seront recalcules.").formatted(current.getPalette().getName()));
            syncPalette.setVisible(true);
            syncPalette.setManaged(true);
        } else {
            paletteState.setText("%d tubes, figes avec le projet.".formatted(paints.size()));
        }
    }

    private Node paintTile(be.asmolabs.palettier.core.domain.OilPaint paint) {
        ColorSwatch swatch = new ColorSwatch(30, 22);
        swatch.setColor(paint.color());

        Label name = new Label(paint.getName());
        name.getStyleClass().add("hint");
        name.setMaxWidth(130);

        HBox tile = new HBox(6, swatch, name);
        tile.setAlignment(javafx.geometry.Pos.CENTER_LEFT);
        return tile;
    }

    // --- Photos ------------------------------------------------------------

    /** Les photos du projet : celle qui a servi au plan, les references, l'avancement. */
    private Node photosCard() {
        Button addPiece = new Button("Photo de la piece");
        addPiece.setOnAction(event -> addPhoto(ProjectPhoto.Role.PIECE));

        Button addReference = new Button("Reference");
        addReference.setOnAction(event -> addPhoto(ProjectPhoto.Role.REFERENCE));

        Button addProgress = new Button("Avancement");
        addProgress.setOnAction(event -> addPhoto(ProjectPhoto.Role.PROGRESS));

        photoStrip.setPrefWrapLength(560);

        return new Card("Photos", new VBox(10,
                new HBox(8, addPiece, addReference, addProgress), photoStrip));
    }

    private void addPhoto(ProjectPhoto.Role role) {
        if (current == null) {
            return;
        }
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Ajouter une photo");
        chooser.getExtensionFilters().add(
                new FileChooser.ExtensionFilter("Images", "*.png", "*.jpg", "*.jpeg", "*.gif", "*.bmp"));
        File file = chooser.showOpenDialog(detail.getScene().getWindow());
        if (file == null) {
            return;
        }
        try {
            current = projects.addPhoto(current, Files.readAllBytes(file.toPath()), role, file.getName());
            refreshPhotos();
        } catch (IOException | RuntimeException e) {
            summary.setText("Image illisible : " + e.getMessage());
        }
    }

    private void refreshPhotos() {
        photoStrip.getChildren().clear();
        if (current == null || current.getPhotos().isEmpty()) {
            Label none = new Label("Aucune photo. Celles de l'assistant sont conservees a l'enregistrement.");
            none.getStyleClass().add("hint");
            photoStrip.getChildren().add(none);
            return;
        }
        current.getPhotos().forEach(photo -> photoStrip.getChildren().add(thumbnail(photo)));
    }

    private Node thumbnail(ProjectPhoto photo) {
        ImageView view = new ImageView(new Image(new ByteArrayInputStream(photo.getData()), 150, 150, true, true));
        view.setFitWidth(140);
        view.setFitHeight(140);
        view.setPreserveRatio(true);

        Label role = new Label(photo.getRole().label());
        role.getStyleClass().add("hint");

        Button remove = new Button("\u2715");
        remove.getStyleClass().add("icon-button");
        remove.setTooltip(new Tooltip("Retirer cette photo du projet"));
        remove.setOnAction(event -> {
            current = projects.removePhoto(current, photo);
            refreshPhotos();
        });

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        VBox tile = new VBox(6, view, new HBox(4, role, spacer, remove));
        tile.getStyleClass().add("palette-tile");
        tile.setPadding(new Insets(8));
        return tile;
    }

    // --- Modification du plan ----------------------------------------------

    private void editLayer(int zoneIndex, int layerIndex) {
        if (current == null || currentPlan == null) {
            return;
        }
        PaintingPlan.Layer layer = currentPlan.zones().get(zoneIndex).layers().get(layerIndex);
        LayerEditor.show(layer).ifPresent(result -> {
            current = projects.updateLayer(current, zoneIndex, layerIndex,
                    result.target(), result.technique(), result.note());
            show(current);
        });
    }

    private void editZone(int zoneIndex) {
        if (current == null) {
            return;
        }
        var zone = current.getZones().get(zoneIndex);
        TextInputDialog dialog = new TextInputDialog(zone.getName());
        dialog.setTitle("Modifier la zone");
        dialog.setContentText("Nom de la zone");
        Dialogs.themed(dialog);
        dialog.showAndWait().map(String::trim).filter(name -> !name.isEmpty()).ifPresent(name -> {
            current = projects.updateZone(current, zoneIndex, name, zone.getMaterial(), zone.getNote());
            show(current);
        });
    }

    private void reload() {
        Project previous = list.getSelectionModel().getSelectedItem();
        items.setAll(projects.findAll());
        if (previous != null && items.contains(previous)) {
            list.getSelectionModel().select(previous);
        } else if (!items.isEmpty()) {
            list.getSelectionModel().selectFirst();
        } else {
            show(null);
        }
    }

    private void show(Project project) {
        detail.getChildren().clear();
        currentPlan = null;
        current = project;
        exportPdf.setDisable(true);
        refreshPhotos();
        refreshPaints();

        if (project == null) {
            summary.setText("Aucun projet");
            Label hint = new Label("Les plans proposes par l'assistant peuvent y etre enregistres.");
            hint.getStyleClass().add("hint");
            detail.getChildren().add(hint);
            return;
        }

        summary.setText("%s  -  palette %s  -  %d tubes  -  cree le %s".formatted(
                project.getName(),
                project.getPalette() == null ? "supprimee" : project.getPalette().getName(),
                project.effectivePaints().size(),
                CREATED.format(project.getCreatedAt())));

        // Le recalcul des dosages parcourt la palette pour chaque couche : hors du fil
        // d'affichage, comme toute recherche de melange.
        Task<PaintingPlan> task = new Task<>() {
            @Override
            protected PaintingPlan call() {
                return projects.plan(project);
            }
        };
        task.setOnSucceeded(event -> {
            currentPlan = task.getValue();
            renderer.render(detail, currentPlan);
            exportPdf.setDisable(currentPlan.zones().isEmpty());
        });
        Thread.ofPlatform().daemon().name("project-plan").start(task);
    }

    private void renameSelected() {
        Project selected = list.getSelectionModel().getSelectedItem();
        if (selected == null) {
            return;
        }
        prompt("Renommer le projet", selected.getName()).ifPresent(name -> {
            projects.rename(selected, name);
            reload();
        });
    }

    private void deleteSelected() {
        Project selected = list.getSelectionModel().getSelectedItem();
        if (selected == null) {
            return;
        }
        Alert confirm = new Alert(Alert.AlertType.CONFIRMATION,
                "Supprimer le projet \"%s\" ? Le plan sera perdu ; la palette reste intacte."
                        .formatted(selected.getName()),
                ButtonType.CANCEL, ButtonType.OK);
        Dialogs.themed(confirm);
        confirm.showAndWait().filter(ButtonType.OK::equals).ifPresent(button -> {
            projects.delete(selected);
            reload();
        });
    }

    private Optional<String> prompt(String title, String initial) {
        TextInputDialog dialog = new TextInputDialog(initial);
        dialog.setTitle(title);
        dialog.setContentText("Nom");
        Dialogs.themed(dialog);
        return dialog.showAndWait().map(String::trim).filter(value -> !value.isEmpty());
    }

    private void exportToPdf() {
        Project selected = list.getSelectionModel().getSelectedItem();
        if (currentPlan == null || selected == null) {
            return;
        }
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Enregistrer le plan");
        chooser.getExtensionFilters().add(new FileChooser.ExtensionFilter("Document PDF", "*.pdf"));
        chooser.setInitialFileName(selected.getName().toLowerCase(Locale.FRENCH)
                .replaceAll("[^a-z0-9]+", "-").replaceAll("(^-|-$)", "") + ".pdf");
        File file = chooser.showSaveDialog(detail.getScene().getWindow());
        if (file == null) {
            return;
        }
        try {
            pdf.write(currentPlan, file.toPath());
            summary.setText(summary.getText() + "   -   enregistre : " + file.getName());
        } catch (IOException e) {
            summary.setText("Enregistrement impossible : " + e.getMessage());
        }
    }

    private static class ProjectCell extends ListCell<Project> {

        private final Label name = new Label();
        private final Label detail = new Label();
        private final VBox layout;

        ProjectCell() {
            detail.getStyleClass().add("hint");
            detail.setWrapText(true);
            layout = new VBox(1, name, detail);
        }

        @Override
        protected void updateItem(Project project, boolean empty) {
            super.updateItem(project, empty);
            if (empty || project == null) {
                setGraphic(null);
                return;
            }
            name.setText(project.getName());
            detail.setText("%d zones  -  %s".formatted(
                    project.getZones().size(), CREATED.format(project.getCreatedAt())));
            setGraphic(layout);
        }
    }
}
