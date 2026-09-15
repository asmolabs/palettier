package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.ai.AiSettings;
import be.asmolabs.palettier.ai.ModelCatalog;
import be.asmolabs.palettier.ai.ModelCatalog.ModelInfo;
import be.asmolabs.palettier.ai.ModelCatalog.PullProgress;
import be.asmolabs.palettier.ai.PaintingPlanService;
import be.asmolabs.palettier.core.backup.BackupService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.Pill;
import java.io.File;
import java.util.List;
import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.ProgressBar;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.TextField;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.FileChooser;
import org.springframework.stereotype.Component;

/**
 * Parametres de l'assistant : quel moteur repond, quel modele il emploie, et comment en
 * installer un autre sans quitter l'application.
 *
 * <p>Le modele retenu ici vaut pour la session et prend effet immediatement : changer de
 * modele pour comparer deux plans est un geste d'essai, il ne doit pas demander un
 * redemarrage.</p>
 */
@Component
public class SettingsView implements AppView {

    private final ModelCatalog catalog;
    private final AiSettings settings;
    private final PaintingPlanService planner;
    private final BackupService backup;

    private final ObservableList<ModelInfo> models = FXCollections.observableArrayList();
    private final ListView<ModelInfo> modelList = new ListView<>(models);
    private final Label engineState = new Label();
    private final Label activeModel = new Label();
    private final TextField pullName = new TextField();
    private final Button pullButton = new Button("Telecharger");
    private final ProgressBar pullProgress = new ProgressBar(0);
    private final Label pullStatus = new Label();
    private final Label backupState = new Label();

    public SettingsView(ModelCatalog catalog, AiSettings settings, PaintingPlanService planner,
                        BackupService backup) {
        this.catalog = catalog;
        this.settings = settings;
        this.planner = planner;
        this.backup = backup;
    }

    @Override
    public String title() {
        return "Parametres";
    }

    @Override
    public String subtitle() {
        return "Moteur de l'assistant, modele employe, et installation de nouveaux modeles.";
    }

    @Override
    public int order() {
        // Reglages, a l'ecart du travail courant.
        return 100;
    }

    @Override
    public boolean separatorBefore() {
        return true;
    }

    @Override
    public Node create() {
        VBox content = new VBox(14, backupPanel(), enginePanel(), modelsPanel(), pullPanel());
        content.setPadding(new Insets(2));

        ScrollPane scroll = new ScrollPane(content);
        scroll.setFitToWidth(true);
        refreshEngine();
        refreshModels();
        return scroll;
    }

    // --- Sauvegarde --------------------------------------------------------

    /**
     * Emporter son travail ailleurs.
     *
     * <p>Une archive avec un JSON lisible et les photos a cote, en fichiers ordinaires :
     * on doit pouvoir l'ouvrir dans des annees, sans l'application, et y retrouver ses
     * recettes a l'oeil nu.</p>
     */
    private Node backupPanel() {
        Button export = new Button("Exporter tout dans un zip...");
        export.setOnAction(event -> exportBackup());

        backupState.getStyleClass().add("hint");
        backupState.setWrapText(true);
        backupState.setText("Palettes, projets, photos, recettes, inventaire et corrections du "
                + "catalogue. Le catalogue d'origine n'a pas besoin d'etre sauvegarde : il est "
                + "livre avec l'application.");

        return new Card("Sauvegarde", new VBox(10, export, backupState));
    }

    private void exportBackup() {
        FileChooser chooser = new FileChooser();
        chooser.setTitle("Enregistrer la sauvegarde");
        chooser.getExtensionFilters().add(new FileChooser.ExtensionFilter("Archive zip", "*.zip"));
        chooser.setInitialFileName("palettier-%s.zip".formatted(java.time.LocalDate.now()));
        File file = chooser.showSaveDialog(modelList.getScene().getWindow());
        if (file == null) {
            return;
        }

        backupState.setText("Ecriture en cours...");
        Task<BackupService.Summary> task = new Task<>() {
            @Override
            protected BackupService.Summary call() throws Exception {
                return backup.export(file.toPath());
            }
        };
        task.setOnSucceeded(event -> {
            var summary = task.getValue();
            backupState.setText("%s  -  %d palettes, %d projets, %d photos, %d recettes, %d tubes, %d ko"
                    .formatted(file.getName(), summary.palettes(), summary.projects(),
                            summary.photos(), summary.recipes(), summary.paints(),
                            summary.bytes() / 1024));
        });
        task.setOnFailed(event -> {
            Throwable error = task.getException();
            backupState.setText("Sauvegarde impossible : "
                    + (error == null ? "cause inconnue" : error.getMessage()));
        });
        // Lecture de toute la base et ecriture d'un fichier : hors du fil d'affichage.
        Thread.ofPlatform().daemon().name("backup").start(task);
    }

    // --- Moteur ------------------------------------------------------------

    private Node enginePanel() {
        engineState.setWrapText(true);
        activeModel.getStyleClass().add("result-summary");

        Button test = new Button("Tester la connexion");
        test.setOnAction(event -> refreshEngine());

        Label how = new Label(
                "Le moteur se choisit au demarrage, par la variable PALETTIER_AI_PROVIDER : "
                + "google-genai pour Gemini, ollama pour un serveur local, openai pour tout "
                + "serveur compatible. Sans elle, l'assistant reste inactif et rien ne sort du poste.");
        how.setWrapText(true);
        how.getStyleClass().add("hint");

        return new Card("Moteur", new VBox(10, activeModel, engineState, test, how));
    }

    private void refreshEngine() {
        String provider = catalog.provider();
        boolean available = planner.isAvailable();

        activeModel.setText(switch (provider == null ? "none" : provider.toLowerCase()) {
            case "none", "" -> "Assistant inactif";
            case "ollama" -> "Ollama, en local";
            case "google-genai" -> "Gemini (Google AI Studio)";
            case "openai" -> "Serveur compatible OpenAI";
            default -> provider;
        });

        StringBuilder state = new StringBuilder();
        if (!available) {
            state.append("Aucun moteur configure : l'assistant ne repondra pas, et le reste de "
                    + "l'application fonctionne normalement.");
        } else if (catalog.isManageable()) {
            state.append(catalog.reachable()
                    ? "Serveur joignable sur " + catalog.baseUrl()
                    : "Serveur injoignable sur " + catalog.baseUrl() + ". Ollama est-il demarre ?");
        } else {
            state.append("Moteur distant : la liste des modeles depend de votre compte, "
                    + "le nom se saisit a la main dans l'Assistant.");
        }
        settings.model().ifPresent(model -> state.append("\nModele retenu pour cette session : ").append(model));
        engineState.setText(state.toString());
    }

    // --- Modeles installes -------------------------------------------------

    private Node modelsPanel() {
        modelList.setCellFactory(view -> new ModelCell());
        modelList.setPrefHeight(230);
        modelList.setPlaceholder(new Label("Aucun modele installe, ou serveur injoignable."));

        Button use = new Button("Utiliser ce modele");
        use.setOnAction(event -> {
            ModelInfo selected = modelList.getSelectionModel().getSelectedItem();
            if (selected != null) {
                settings.useModel(selected.name());
                refreshEngine();
                modelList.refresh();
            }
        });

        Button useDefault = new Button("Revenir a celui de la configuration");
        useDefault.setOnAction(event -> {
            settings.useModel(null);
            refreshEngine();
            modelList.refresh();
        });

        Button refresh = new Button("Rafraichir");
        refresh.setOnAction(event -> refreshModels());

        Label hint = new Label(
                "L'assistant a besoin d'un modele capable de lire les images pour exploiter vos "
                + "photos : c'est ce qu'indique l'etiquette Vision. Un modele qui en est depourvu "
                + "reste utilisable a partir d'une description ecrite.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        VBox content = new VBox(10, modelList, new HBox(8, use, useDefault, refresh), hint);
        return new Card("Modeles installes", content);
    }

    private void refreshModels() {
        modelList.setPlaceholder(new Label("Lecture de la liste..."));
        Task<List<ModelInfo>> task = new Task<>() {
            @Override
            protected List<ModelInfo> call() {
                return catalog.installed();
            }
        };
        task.setOnSucceeded(event -> {
            models.setAll(task.getValue());
            modelList.setPlaceholder(new Label("Aucun modele installe, ou serveur injoignable."));
        });
        // Une requete reseau, meme locale : jamais sur le fil d'affichage.
        Thread.ofPlatform().daemon().name("model-list").start(task);
    }

    // --- Telechargement ----------------------------------------------------

    private Node pullPanel() {
        pullName.setPromptText("nom du modele, par exemple qwen2.5vl:7b");
        HBox.setHgrow(pullName, Priority.ALWAYS);
        // Surtout pas un bouton par defaut : une touche Entree malencontreuse lancerait
        // le telechargement de plusieurs gigaoctets.
        pullButton.setOnAction(event -> pull());
        pullName.setOnAction(event -> pull());

        pullProgress.setMaxWidth(Double.MAX_VALUE);
        pullProgress.setVisible(false);
        pullProgress.setManaged(false);
        pullStatus.getStyleClass().add("hint");
        pullStatus.setWrapText(true);

        Label hint = new Label(
                "Le modele est telecharge par le serveur Ollama et devient utilisable des la fin, "
                + "sans redemarrer l'application. Comptez plusieurs gigaoctets : l'operation se "
                + "poursuit en arriere-plan et vous pouvez continuer a travailler.");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");

        VBox content = new VBox(10,
                new HBox(8, pullName, pullButton), pullProgress, pullStatus, hint);
        return new Card("Charger un nouveau modele", content);
    }

    private void pull() {
        String name = pullName.getText() == null ? "" : pullName.getText().trim();
        if (name.isEmpty()) {
            pullStatus.setText("Indiquez le nom d'un modele.");
            return;
        }
        if (!catalog.isManageable()) {
            pullStatus.setText("Seul un serveur Ollama local permet d'installer un modele depuis ici.");
            return;
        }

        pullButton.setDisable(true);
        pullProgress.setVisible(true);
        pullProgress.setManaged(true);
        pullProgress.setProgress(ProgressBar.INDETERMINATE_PROGRESS);
        pullStatus.setText("Demande envoyee...");

        Task<Boolean> task = new Task<>() {
            @Override
            protected Boolean call() {
                return catalog.pull(name, SettingsView.this::onPullProgress);
            }
        };
        task.setOnSucceeded(event -> finishPull(task.getValue(), name));
        task.setOnFailed(event -> finishPull(false, name));
        Thread.ofPlatform().daemon().name("model-pull").start(task);
    }

    /** Appele depuis le fil de telechargement : tout affichage repasse par le fil JavaFX. */
    private void onPullProgress(PullProgress progress) {
        Platform.runLater(() -> {
            double fraction = progress.fraction();
            pullProgress.setProgress(fraction < 0 ? ProgressBar.INDETERMINATE_PROGRESS : fraction);
            pullStatus.setText(fraction < 0
                    ? progress.status()
                    : "%s  -  %.0f %% (%.1f Go sur %.1f)".formatted(progress.status(), fraction * 100,
                            progress.completed() / 1e9, progress.total() / 1e9));
        });
    }

    private void finishPull(boolean success, String name) {
        pullButton.setDisable(false);
        pullProgress.setVisible(false);
        pullProgress.setManaged(false);
        if (success) {
            pullStatus.setText(name + " est installe et utilisable.");
            pullName.clear();
            refreshModels();
        } else {
            pullStatus.setText("Le telechargement de " + name + " n'a pas abouti. "
                    + "Verifiez le nom du modele et que le serveur est demarre.");
        }
    }

    // --- Cellule -----------------------------------------------------------

    private class ModelCell extends ListCell<ModelInfo> {

        @Override
        protected void updateItem(ModelInfo model, boolean empty) {
            super.updateItem(model, empty);
            if (empty || model == null) {
                setGraphic(null);
                return;
            }

            Label name = new Label(model.name());
            Label size = new Label(model.sizeLabel());
            size.getStyleClass().add("hint");

            Region gap = new Region();
            HBox.setHgrow(gap, Priority.ALWAYS);

            HBox row = new HBox(10, name, size, gap);
            row.setAlignment(Pos.CENTER_LEFT);

            if (model.supportsVision()) {
                row.getChildren().add(Pill.of("Vision", "pill-fast"));
            }
            if (settings.model().filter(model.name()::equals).isPresent()) {
                row.getChildren().add(Pill.of("En service", "pill-solid"));
            }
            setGraphic(row);
        }
    }
}
