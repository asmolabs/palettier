package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.FatOverLeanService;
import be.asmolabs.palettier.core.service.FatOverLeanService.Risk;
import be.asmolabs.palettier.core.service.PlanDryingService;
import be.asmolabs.palettier.core.service.ProgressCheckService;
import be.asmolabs.palettier.core.service.ProgressCheckService.Observed;
import be.asmolabs.palettier.core.service.ProjectService;
import be.asmolabs.palettier.core.service.SubstituteService;
import be.asmolabs.palettier.core.service.SubstituteService.Missing;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import be.asmolabs.palettier.ui.component.Dialogs;
import be.asmolabs.palettier.ui.component.LayerEditor;
import be.asmolabs.palettier.ui.component.PlanRenderer;
import be.asmolabs.palettier.ui.component.WorkshopForm;
import be.asmolabs.palettier.ui.export.PlanPdf;
import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.List;
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
    private final FatOverLeanService fatOverLean;
    private final SubstituteService substitutes;
    private final ProgressCheckService progress;
    private final PlanPdf pdf;
    private final PlanRenderer renderer = new PlanRenderer();

    private final ObservableList<Project> items = FXCollections.observableArrayList();
    private final ListView<Project> list = new ListView<>(items);
    private final VBox detail = new VBox(14);
    private final FlowPane photoStrip = new FlowPane(10, 10);
    private final FlowPane paintStrip = new FlowPane(6, 6);
    private final Label paletteState = new Label();
    private final Button syncPalette = new Button("Reprendre la palette actuelle");
    private final VBox shelf = new VBox(8);
    private final VBox comparison = new VBox(8);
    private final Label summary = new Label();
    private final Button exportPdf = new Button("Exporter en PDF");
    private final VBox cracking = new VBox(8);

    // Les conditions servent au moment ou l'on coche une couche : elles sont figees avec
    // la pose, puisque c'est dans cet atelier-la que l'huile va secher.
    private final WorkshopForm workshop = new WorkshopForm(
            "Ces conditions sont enregistrees avec chaque couche que vous marquez posee. "
            + "Ce sont elles qui donnent la date a laquelle la piece redeviendra reprenable, "
            + "dans Aujourd'hui.");

    private PaintingPlan currentPlan;
    private Project current;

    public ProjectsView(ProjectService projects, FatOverLeanService fatOverLean,
                        SubstituteService substitutes, ProgressCheckService progress, PlanPdf pdf) {
        this.projects = projects;
        this.fatOverLean = fatOverLean;
        this.substitutes = substitutes;
        this.progress = progress;
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
    public int shortcut() {
        return 1;
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

        // Ce qui est fait, par opposition a ce qui est decide. C'est cette case qui fait
        // courir le sechage, et donc qui alimente l'etabli.
        renderer.tracking(new PlanRenderer.Coats() {
            @Override
            public Instant appliedAt(int zoneIndex, int layerIndex) {
                return current == null ? null : storedLayer(zoneIndex, layerIndex).getAppliedAt();
            }

            @Override
            public void toggleApplied(int zoneIndex, int layerIndex) {
                ProjectsView.this.toggleApplied(zoneIndex, layerIndex);
            }
        });

        exportPdf.setDisable(true);
        exportPdf.setOnAction(event -> exportToPdf());

        detail.setPadding(new Insets(2));

        // Tout descend dans le meme defilement. Auparavant les tubes et les photos
        // occupaient une hauteur fixe au-dessus du plan, qui se retrouvait ecrase dans
        // une fenetre courte -- alors que c'est lui le contenu principal.
        VBox everything = new VBox(14, summary, new HBox(8, exportPdf), crackingCard(),
                paintsCard(), photosCard(), workshop, detail);
        everything.setPadding(new Insets(2));

        ScrollPane scroll = new ScrollPane(everything);
        scroll.setFitToWidth(true);
        VBox.setVgrow(scroll, Priority.ALWAYS);

        Card card = new Card("Plan du projet", scroll);
        VBox.setVgrow(card, Priority.ALWAYS);
        return card;
    }

    // --- Gras sur maigre ----------------------------------------------------

    /**
     * L'avertissement qui ne se rattrape pas.
     *
     * <p>Une couche maigre sur une couche grasse craquelle, et cela se voit des mois plus
     * tard sur une piece finie. La carte n'apparait que lorsqu'il y a quelque chose a
     * dire : un plan sain n'affiche rien du tout.</p>
     */
    private Node crackingCard() {
        cracking.setVisible(false);
        cracking.setManaged(false);
        return cracking;
    }

    private void refreshCracking() {
        cracking.getChildren().clear();

        List<Risk> risks = current == null ? List.of() : fatOverLean.inspect(current);
        cracking.setVisible(!risks.isEmpty());
        cracking.setManaged(!risks.isEmpty());
        if (risks.isEmpty()) {
            return;
        }

        VBox lines = new VBox(8);
        for (Risk risk : risks) {
            Label where = new Label("%s : %s sous %s".formatted(risk.where(), risk.under(), risk.over()));
            where.getStyleClass().add("milestone-title");

            Label why = new Label(risk.explanation());
            why.getStyleClass().add("hint");
            why.setWrapText(true);

            lines.getChildren().add(new VBox(2, where, why));
        }
        cracking.getChildren().add(new Card("Gras sur maigre",
                "Ces empilements peuvent craqueler en vieillissant. C'est le seul defaut de "
                + "cet atelier qui ne se rattrape pas une fois la piece finie.", lines));
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
        shelf.setVisible(false);
        shelf.setManaged(false);
        return new Card("Tubes du projet", new VBox(10, paintStrip, paletteState, syncPalette, shelf));
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

        refreshShelf();

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

    /**
     * Ce que le projet reclame et qui n'est pas sur l'etagere.
     *
     * <p>Annoncer le manque ne sert a rien : le peintre le sait. Ce qu'il veut savoir,
     * c'est avec quoi il s'en sort ce soir -- et si le remplacant est assez proche, le
     * tube manquant n'a meme pas besoin d'etre achete.</p>
     */
    private void refreshShelf() {
        shelf.getChildren().clear();

        List<Missing> missing = current == null ? List.of() : substitutes.missingFrom(current);
        shelf.setVisible(!missing.isEmpty());
        shelf.setManaged(!missing.isEmpty());
        if (missing.isEmpty()) {
            return;
        }

        VBox lines = new VBox(8);
        for (Missing gap : missing) {
            ColorSwatch wanted = new ColorSwatch(26, 20);
            wanted.setColor(gap.paint().color());

            HBox swatches = new HBox(4, wanted);
            gap.nearest().ifPresent(nearest -> {
                ColorSwatch replacement = new ColorSwatch(26, 20);
                replacement.setColor(nearest.color());
                swatches.getChildren().add(replacement);
            });
            swatches.setAlignment(javafx.geometry.Pos.CENTER_LEFT);

            Label name = new Label(gap.paint().displayName());
            name.getStyleClass().add("milestone-title");

            Label verdict = new Label(gap.isComfortable()
                    ? gap.verdict() + "  (ecart %.1f)".formatted(gap.deltaE())
                    : gap.verdict());
            verdict.getStyleClass().add("hint");
            verdict.setWrapText(true);

            VBox texts = new VBox(2, name, verdict);
            HBox.setHgrow(texts, Priority.ALWAYS);
            lines.getChildren().add(new HBox(10, swatches, texts));
        }

        long comfortable = missing.stream().filter(Missing::isComfortable).count();
        shelf.getChildren().add(new Card("Pas sur l'etagere",
                comfortable == missing.size()
                        ? "Tout ce qui manque a un equivalent chez vous : la piece est faisable ce soir."
                        : "%d tube(s) manquant(s), dont %d remplacable(s) sans que cela se voie."
                                .formatted(missing.size(), comfortable),
                lines));
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

        Button compare = new Button("Comparer une photo au plan");
        compare.setOnAction(event -> compareWithPlan());

        comparison.setVisible(false);
        comparison.setManaged(false);

        Label how = new Label(
                "La comparaison lit votre fichier d'origine, pas la photo rangee ici : celle-ci est "
                + "reduite et reencodee, ses couleurs ont bouge. Meme ainsi, une photo est prise sous "
                + "une lumiere quelconque -- l'ecart se lit comme une tendance, pas comme un verdict.");
        how.getStyleClass().add("hint");
        how.setWrapText(true);

        return new Card("Photos", new VBox(10,
                new HBox(8, addPiece, addReference, addProgress, compare),
                photoStrip, how, comparison));
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

    // --- Ce qui est sur la piece, compare a ce qui etait vise ---------------

    /**
     * Releve les teintes d'une photo d'avancement et les confronte au plan.
     *
     * <p>Le projet dit ce qui etait vise, la pipette sait relever ce qui est la : il ne
     * manquait que de les mettre face a face. Le peintre voit alors de combien il a
     * devie, et dans quel sens.</p>
     */
    private void compareWithPlan() {
        if (current == null) {
            return;
        }
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Photo d'avancement a comparer");
        chooser.getExtensionFilters().add(
                new FileChooser.ExtensionFilter("Images", "*.png", "*.jpg", "*.jpeg", "*.bmp"));
        File file = chooser.showOpenDialog(detail.getScene().getWindow());
        if (file == null) {
            return;
        }

        Project piece = current;
        Task<List<Observed>> task = new Task<>() {
            @Override
            protected List<Observed> call() throws Exception {
                return progress.compare(piece, Files.readAllBytes(file.toPath()), 8);
            }
        };
        task.setOnSucceeded(event -> showComparison(task.getValue(), file.getName()));
        task.setOnFailed(event -> {
            comparison.getChildren().setAll(new Label("Image illisible : "
                    + task.getException().getMessage()));
            comparison.setVisible(true);
            comparison.setManaged(true);
        });
        Thread.ofPlatform().daemon().name("progress-check").start(task);
    }

    private void showComparison(List<Observed> observed, String fileName) {
        VBox lines = new VBox(8);
        for (Observed seen : observed) {
            ColorSwatch measured = new ColorSwatch(30, 24);
            measured.setColor(seen.measured());

            HBox swatches = new HBox(4, measured);
            if (seen.matchesPlan()) {
                ColorSwatch aimed = new ColorSwatch(30, 24);
                aimed.setColor(seen.target());
                swatches.getChildren().add(aimed);
            }
            swatches.setAlignment(javafx.geometry.Pos.CENTER_LEFT);

            Label head = new Label("%s  -  %.0f %% de l'image".formatted(
                    seen.measured().toHex(), seen.share() * 100));
            head.getStyleClass().add("milestone-title");

            Label verdict = new Label(seen.matchesPlan()
                    ? seen.verdict() + "  (ecart %.1f)".formatted(seen.deltaE())
                    : seen.verdict());
            verdict.getStyleClass().add("hint");
            verdict.setWrapText(true);

            VBox texts = new VBox(2, head, verdict);
            HBox.setHgrow(texts, Priority.ALWAYS);
            lines.getChildren().add(new HBox(10, swatches, texts));
        }

        comparison.getChildren().setAll(new Card("Releve sur " + fileName,
                "A gauche ce qui est sur la piece, a droite ce que le plan visait.", lines));
        comparison.setVisible(true);
        comparison.setManaged(true);
    }

    // --- Modification du plan ----------------------------------------------

    private void editLayer(int zoneIndex, int layerIndex) {
        if (current == null || currentPlan == null) {
            return;
        }
        PaintingPlan.Layer layer = plannedLayer(zoneIndex, layerIndex);
        LayerEditor.show(layer).ifPresent(result -> {
            current = projects.updateLayer(current, zoneIndex, layerIndex,
                    result.target(), result.technique(), result.note());
            show(current);
        });
    }

    /**
     * Consigne qu'une couche vient d'etre peinte, ou revient sur une fausse manoeuvre.
     *
     * <p>La vitesse de sechage n'est pas demandee : elle se lit dans le melange calcule
     * pour cette couche, donc dans les tubes du projet. Elle est figee avec la pose, au
     * meme titre que les conditions -- remanier la palette ensuite ne doit pas reecrire
     * ce qui a deja seche.</p>
     */
    private void toggleApplied(int zoneIndex, int layerIndex) {
        if (current == null || currentPlan == null) {
            return;
        }
        if (storedLayer(zoneIndex, layerIndex).isApplied()) {
            current = projects.clearApplied(current, zoneIndex, layerIndex);
        } else {
            current = projects.markApplied(current, zoneIndex, layerIndex, workshop.current(),
                    PlanDryingService.dryingClassOf(plannedLayer(zoneIndex, layerIndex)));
        }
        show(current);
    }

    private ProjectLayer storedLayer(int zoneIndex, int layerIndex) {
        return current.getZones().get(zoneIndex).getLayers().get(layerIndex);
    }

    /**
     * La couche calculee correspondant a une ligne affichee.
     *
     * <p>Les variations locales sont rendues a la suite de l'echelle et portent donc un
     * rang qui continue le sien : c'est la meme convention que l'enregistrement, ou elles
     * sont ajoutees apres elle.</p>
     */
    private PaintingPlan.Layer plannedLayer(int zoneIndex, int layerIndex) {
        PaintingPlan.Zone zone = currentPlan.zones().get(zoneIndex);
        List<PaintingPlan.Layer> ladder = zone.layers();
        return layerIndex < ladder.size()
                ? ladder.get(layerIndex)
                : zone.accents().get(layerIndex - ladder.size());
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

    /**
     * Ce qu'une ouverture de projet va chercher en base : le projet avec ses photos, et
     * le plan dont les dosages viennent d'etre recalcules.
     */
    private record Loaded(Project project, PaintingPlan plan) {
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
        photoStrip.getChildren().clear();
        cracking.setVisible(false);
        cracking.setManaged(false);
        comparison.setVisible(false);
        comparison.setManaged(false);
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

        // Le recalcul des dosages parcourt la palette pour chaque couche, et les photos
        // sont des images entieres : les deux se font hors du fil d'affichage.
        Task<Loaded> task = new Task<>() {
            @Override
            protected Loaded call() {
                return new Loaded(projects.withPhotos(project), projects.plan(project));
            }
        };
        task.setOnSucceeded(event -> {
            Loaded loaded = task.getValue();
            current = loaded.project();
            currentPlan = loaded.plan();
            refreshPhotos();
            refreshCracking();
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
