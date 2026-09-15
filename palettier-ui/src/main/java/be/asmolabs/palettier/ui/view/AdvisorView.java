package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.ai.ModelCatalog;
import be.asmolabs.palettier.ai.PaintingPlanService;
import be.asmolabs.palettier.ai.PhotoInput;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.image.Photos;
import be.asmolabs.palettier.core.service.PaletteService;
import be.asmolabs.palettier.core.service.ProjectService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.Dialogs;
import be.asmolabs.palettier.ui.component.PlanRenderer;
import be.asmolabs.palettier.ui.export.PlanPdf;
import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Tooltip;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.SplitPane;
import javafx.scene.control.TextArea;
import javafx.scene.control.TextInputDialog;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.FileChooser;
import javafx.util.StringConverter;
import org.springframework.stereotype.Component;

/**
 * Plan de peinture par zones, etabli par l'assistant.
 *
 * <p>L'ecran montre volontairement, pour chaque couche, la couleur visee <em>et</em>
 * celle que la palette permet reellement d'obtenir, cote a cote. C'est la seule facon de
 * voir ou le conseil tient et ou la palette est prise en defaut.</p>
 */
@Component
public class AdvisorView implements AppView {

    private final PaintingPlanService planner;
    private final ModelCatalog models;
    private final PaletteService palettes;
    private final PlanPdf pdf;
    private final ProjectService projects;

    private final TextArea subject = new TextArea();
    private final ComboBox<Palette> paletteChoice = new ComboBox<>();
    private final ComboBox<Integer> maxPaints = new ComboBox<>();
    private final ComboBox<String> modelChoice = new ComboBox<>();
    private final Button generate = new Button("Proposer un plan");
    private final Button exportPdf = new Button("Exporter en PDF");
    private final Button saveProject = new Button("Enregistrer le projet");
    private final PlanRenderer renderer = new PlanRenderer();
    private final Label status = new Label();
    private final VBox results = new VBox(14);

    /** Dernier plan produit, celui que l'export met sur papier. */
    private PaintingPlan lastPlan;

    /** Photo de la piece, et references souhaitees. */
    private Photo figurine;
    private final List<Photo> references = new ArrayList<>();
    private final HBox figurineSlot = new HBox(10);
    private final FlowPane referenceSlots = new FlowPane(10, 10);

    /** Une image retenue : ses octets pour l'envoi, sa vignette pour l'ecran. */
    private record Photo(byte[] data, String name, Image thumbnail) {
    }

    public AdvisorView(PaintingPlanService planner, ModelCatalog models, PaletteService palettes,
                       ProjectService projects, PlanPdf pdf) {
        this.planner = planner;
        this.models = models;
        this.palettes = palettes;
        this.projects = projects;
        this.pdf = pdf;
    }

    @Override
    public String title() {
        return "Assistant";
    }

    @Override
    public String subtitle() {
        return "Decrivez un sujet : l'assistant le decoupe en zones et propose base, ombre et lumiere pour chacune.";
    }

    @Override
    public int order() {
        // Trouver une couleur : la faire proposer.
        return 40;
    }

    @Override
    public Node create() {
        SplitPane split = new SplitPane(settingsPanel(), resultsPanel());
        split.setDividerPositions(0.36);
        return split;
    }

    // --- Parametres --------------------------------------------------------

    private Node settingsPanel() {
        subject.setPromptText("Buste de grognard napoleonien, manteau bleu, bonnet a poil, "
                + "visage burine d'un homme d'une quarantaine d'annees...");
        subject.setWrapText(true);
        subject.setPrefRowCount(5);

        paletteChoice.getItems().setAll(palettes.findAll());
        paletteChoice.setConverter(new StringConverter<>() {
            @Override
            public String toString(Palette palette) {
                return palette == null ? "" : "%s (%d tubes)".formatted(palette.getName(), palette.getPaints().size());
            }

            @Override
            public Palette fromString(String value) {
                return null;
            }
        });
        paletteChoice.setMaxWidth(Double.MAX_VALUE);
        paletteChoice.setOnShowing(event -> paletteChoice.getItems().setAll(palettes.findAll()));
        if (!paletteChoice.getItems().isEmpty()) {
            paletteChoice.setValue(paletteChoice.getItems().getFirst());
        }

        maxPaints.getItems().setAll(1, 2, 3);
        maxPaints.setValue(3);
        maxPaints.setConverter(new StringConverter<>() {
            @Override
            public String toString(Integer count) {
                return count == null ? "" : count == 1 ? "un seul tube" : count + " tubes au plus";
            }

            @Override
            public Integer fromString(String value) {
                return null;
            }
        });

        // Editable : Ollama sait dire ce qui est installe, les autres moteurs non. Dans
        // les deux cas on doit pouvoir taper un nom.
        modelChoice.setEditable(true);
        modelChoice.setMaxWidth(Double.MAX_VALUE);
        modelChoice.setPromptText("celui de la configuration");
        modelChoice.setOnShowing(event -> refreshModels());
        refreshModels();

        generate.setDefaultButton(true);
        generate.setOnAction(event -> generate());
        generate.setDisable(!planner.isAvailable());

        status.getStyleClass().add("hint");
        status.setWrapText(true);
        status.setText(planner.isAvailable()
                ? "Un moteur est configure. Le sujet et la composition de votre palette lui seront envoyes."
                : "Aucun moteur configure : l'assistant est inactif et rien n'est envoye nulle part.");

        Label modelHint = new Label("La qualite du decoupage en zones depend fortement du modele : "
                + "un petit modele se contente de grandes zones vagues.");
        modelHint.setWrapText(true);
        modelHint.getStyleClass().add("hint");

        VBox form = new VBox(10,
                new Label("Sujet"), subject,
                new Label("Palette"), paletteChoice,
                new Label("Melanges"), maxPaints,
                new Label("Modele"), modelChoice, modelHint,
                generate, status);

        Label division = new Label(
                "Repartition des roles : l'assistant choisit les couleurs a viser, ce qui releve du "
                + "jugement ; l'application calcule ensuite le melange qui y conduit avec vos tubes, "
                + "et mesure l'ecart obtenu. Aucun dosage, aucune duree de sechage ne vient du modele.");
        division.setWrapText(true);
        division.getStyleClass().add("hint");

        VBox content = new VBox(14, new Card("Que voulez-vous peindre ?", form),
                photosCard(),
                new Card("Ce que fait l'assistant, et ce qu'il ne fait pas", division),
                providerCard());
        ScrollPane scroll = new ScrollPane(content);
        scroll.setFitToWidth(true);
        return scroll;
    }

    // --- Photos ------------------------------------------------------------

    /**
     * Depot des images. Une photo de la piece suffit a l'assistant pour decouper le sujet
     * en zones ; les references servent a lui dire vers quoi tendre.
     */
    private Node photosCard() {
        Button chooseFigurine = new Button("Photo de la piece...");
        chooseFigurine.setOnAction(event -> choose(false));

        Button addReference = new Button("Ajouter une reference...");
        addReference.setOnAction(event -> choose(true));

        figurineSlot.setAlignment(Pos.CENTER_LEFT);
        referenceSlots.setPrefWrapLength(320);

        Label hint = new Label(
                "Les images sont reduites a 1024 pixels avant l'envoi : au-dela, on sature le "
                + "contexte du modele et l'attente s'allonge sans rien gagner. Elles servent a "
                + "reconnaitre les zones et juger l'avancement, jamais a relever une couleur : "
                + "pour cela, la pipette mesure sur votre photo, en local.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        VBox content = new VBox(10,
                new HBox(8, chooseFigurine, addReference),
                new Label("Piece a peindre"), figurineSlot,
                new Label("References (facultatif)"), referenceSlots,
                hint);

        refreshPhotos();
        return new Card("Photos", content);
    }

    private void choose(boolean asReference) {
        FileChooser chooser = new FileChooser();
        chooser.setTitle(asReference ? "Choisir une reference" : "Choisir la photo de la piece");
        chooser.getExtensionFilters().add(
                new FileChooser.ExtensionFilter("Images", "*.png", "*.jpg", "*.jpeg", "*.gif", "*.bmp"));
        File file = chooser.showOpenDialog(results.getScene().getWindow());
        if (file == null) {
            return;
        }
        try {
            byte[] data = Files.readAllBytes(file.toPath());
            Photo photo = new Photo(data, file.getName(),
                    new Image(new ByteArrayInputStream(data), 96, 96, true, true));
            if (asReference) {
                references.add(photo);
            } else {
                figurine = photo;
            }
            refreshPhotos();
        } catch (IOException e) {
            showPlaceholder("Lecture impossible de " + file.getName() + " : " + e.getMessage());
        }
    }

    private void refreshPhotos() {
        figurineSlot.getChildren().setAll(figurine == null
                ? List.of(emptySlot("aucune photo"))
                : List.of(thumbnail(figurine, false)));

        referenceSlots.getChildren().clear();
        if (references.isEmpty()) {
            referenceSlots.getChildren().add(emptySlot("aucune reference"));
        } else {
            references.forEach(photo -> referenceSlots.getChildren().add(thumbnail(photo, true)));
        }
    }

    private Node emptySlot(String message) {
        Label label = new Label(message);
        label.getStyleClass().add("hint");
        return label;
    }

    private Node thumbnail(Photo photo, boolean isReference) {
        ImageView view = new ImageView(photo.thumbnail());
        view.setFitWidth(88);
        view.setFitHeight(88);
        view.setPreserveRatio(true);

        Button remove = new Button("\u2715");
        remove.getStyleClass().add("icon-button");
        remove.setTooltip(new Tooltip("Retirer cette photo"));
        remove.setOnAction(event -> {
            if (isReference) {
                references.remove(photo);
            } else {
                figurine = null;
            }
            refreshPhotos();
        });

        Label name = new Label(photo.name());
        name.getStyleClass().add("hint");
        name.setMaxWidth(110);

        VBox box = new VBox(4, view, new HBox(4, name, remove));
        box.getStyleClass().add("palette-tile");
        box.setPadding(new Insets(8));
        return box;
    }

    /** Sans moteur configure, l'ecran doit dire comment en activer un, pas rester muet. */
    private Node providerCard() {
        Label how = new Label("""
                Trois moteurs au choix, par variables d'environnement :

                Gemini, palier gratuit de Google AI Studio
                  PALETTIER_AI_PROVIDER=google-genai et GEMINI_API_KEY=...
                  Attention : le palier gratuit autorise Google a reutiliser les contenus envoyes.

                Ollama, en local, rien ne quitte le poste
                  ollama pull qwen2.5:14b puis PALETTIER_AI_PROVIDER=ollama

                Tout serveur compatible OpenAI, local ou distant
                  PALETTIER_AI_PROVIDER=openai, OPENAI_BASE_URL=http://localhost:1234, OPENAI_API_KEY=...""");
        how.setWrapText(true);
        how.getStyleClass().add("hint");
        return new Card("Activer un moteur", how);
    }

    // --- Resultats ---------------------------------------------------------

    private Node resultsPanel() {
        results.setPadding(new Insets(2));
        ScrollPane scroll = new ScrollPane(results);
        scroll.setFitToWidth(true);
        VBox.setVgrow(scroll, Priority.ALWAYS);

        showPlaceholder("Decrivez un sujet, choisissez une palette, puis lancez la proposition.");

        exportPdf.setDisable(true);
        exportPdf.setOnAction(event -> exportToPdf());
        saveProject.setDisable(true);
        saveProject.setOnAction(event -> saveAsProject());

        Card card = new Card("Plan de peinture",
                new VBox(12, new HBox(8, saveProject, exportPdf), scroll));
        VBox.setVgrow(card, Priority.ALWAYS);
        return card;
    }

    private void showPlaceholder(String message) {
        Label label = new Label(message);
        label.setWrapText(true);
        label.getStyleClass().add("hint");
        results.getChildren().setAll(label);
    }

    /** Ollama publie ce qui est installe ; on relit a chaque ouverture, cela bouge. */
    private void refreshModels() {
        String current = modelChoice.getEditor().getText();
        modelChoice.getItems().setAll(models.available());
        modelChoice.getEditor().setText(current);
    }

    private void generate() {
        Palette palette = paletteChoice.getValue();
        String description = subject.getText() == null ? "" : subject.getText().trim();
        if (palette == null) {
            showPlaceholder("Choisissez une palette.");
            return;
        }
        if (description.isEmpty() && figurine == null) {
            showPlaceholder("Decrivez le sujet, ou deposez une photo de la piece.");
            return;
        }

        PhotoInput piece = figurine == null ? null
                : new PhotoInput(figurine.data(), "la piece a peindre, etat actuel");
        List<PhotoInput> refs = references.stream()
                .map(photo -> new PhotoInput(photo.data(), "reference souhaitee"))
                .toList();

        generate.setDisable(true);
        showPlaceholder("L'assistant reflechit... (quelques secondes a une minute selon le moteur)");
        exportPdf.setDisable(true);
        saveProject.setDisable(true);
        lastPlan = null;

        int limit = maxPaints.getValue() == null ? 3 : maxPaints.getValue();
        String model = modelChoice.getEditor().getText();
        Task<PaintingPlan> task = new Task<>() {
            @Override
            protected PaintingPlan call() {
                return planner.plan(description, palette, limit, piece, refs, model);
            }
        };
        task.setOnSucceeded(event -> {
            render(task.getValue());
            generate.setDisable(false);
        });
        task.setOnFailed(event -> {
            Throwable error = task.getException();
            showPlaceholder("L'assistant n'a pas pu repondre : "
                    + (error == null ? "cause inconnue" : error.getMessage()));
            generate.setDisable(false);
        });
        // Appel reseau ou inference locale : jamais sur le fil d'affichage.
        Thread.ofPlatform().daemon().name("painting-plan").start(task);
    }

    /** Ecrit le plan courant sur papier : c'est a l'etabli qu'il sert, pas devant l'ecran. */
    private void exportToPdf() {
        if (lastPlan == null) {
            return;
        }
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Enregistrer le plan");
        chooser.getExtensionFilters().add(new FileChooser.ExtensionFilter("Document PDF", "*.pdf"));
        chooser.setInitialFileName(suggestedFileName(lastPlan));
        File file = chooser.showSaveDialog(results.getScene().getWindow());
        if (file == null) {
            return;
        }
        try {
            pdf.write(lastPlan, file.toPath());
            status.setText("Plan enregistre : " + file.getName());
        } catch (IOException e) {
            status.setText("Enregistrement impossible : " + e.getMessage());
        }
    }

    private static String suggestedFileName(PaintingPlan plan) {
        String base = plan.subject() == null || plan.subject().isBlank() ? "plan" : plan.subject();
        String slug = base.toLowerCase(java.util.Locale.FRENCH)
                .replaceAll("[^a-z0-9]+", "-")
                .replaceAll("(^-|-$)", "");
        return (slug.isBlank() ? "plan" : slug.substring(0, Math.min(40, slug.length()))) + ".pdf";
    }

    private void render(PaintingPlan plan) {
        lastPlan = plan;
        boolean empty = plan.zones().isEmpty();
        exportPdf.setDisable(empty);
        saveProject.setDisable(empty);

        if (empty) {
            showPlaceholder("L'assistant n'a propose aucune zone. Precisez le sujet.");
            return;
        }
        renderer.render(results, plan);
    }

    /** Conserve le plan courant sous forme de projet, avec la palette qui a servi a l'etablir. */
    private void saveAsProject() {
        Palette palette = paletteChoice.getValue();
        if (lastPlan == null || palette == null) {
            return;
        }
        TextInputDialog dialog = new TextInputDialog(suggestedProjectName(lastPlan));
        dialog.setTitle("Enregistrer le projet");
        dialog.setContentText("Nom du projet");
        Dialogs.themed(dialog);
        dialog.showAndWait().map(String::trim).filter(name -> !name.isEmpty()).ifPresent(name -> {
            // Les photos qui ont servi a etablir le plan partent avec lui : sans elles, on
            // relit dans six mois un plan dont on ne sait plus a quoi il s'appliquait.
            List<ProjectPhoto> kept = new ArrayList<>();
            if (figurine != null) {
                kept.add(new ProjectPhoto(Photos.prepare(figurine.data()),
                        ProjectPhoto.Role.PIECE, figurine.name()));
            }
            references.forEach(photo -> kept.add(new ProjectPhoto(Photos.prepare(photo.data()),
                    ProjectPhoto.Role.REFERENCE, photo.name())));

            var project = projects.save(lastPlan, palette, name, kept);
            status.setText("Projet enregistre : " + project.getName()
                    + (kept.isEmpty() ? "" : " avec %d photo(s)".formatted(kept.size()))
                    + ". Retrouvez-le dans l'onglet Projets.");
        });
    }

    private static String suggestedProjectName(PaintingPlan plan) {
        String subject = plan.subject() == null ? "" : plan.subject().trim();
        if (subject.isBlank()) {
            return "Projet sans titre";
        }
        return subject.length() <= 50 ? subject : subject.substring(0, 50).trim();
    }

}
