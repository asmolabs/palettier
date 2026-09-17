package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.service.DryingTimeService;
import be.asmolabs.palettier.core.service.PlanDryingService;
import be.asmolabs.palettier.core.service.ProjectService;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.Formats;
import be.asmolabs.palettier.ui.component.WorkshopForm;
import java.time.Duration;
import java.util.List;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.control.ScrollPane;
import javafx.scene.control.Slider;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToggleGroup;
import javafx.scene.control.SplitPane;
import javafx.scene.layout.ColumnConstraints;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import org.springframework.stereotype.Component;

/**
 * Planificateur de sechage : la question la plus frequente a l'huile n'est pas
 * "quelle couleur ?" mais "quand puis-je reprendre la piece ?".
 */
@Component
public class DryingView implements AppView {

    private final DryingTimeService dryingTimeService;
    private final PlanDryingService planDrying;
    private final ProjectService projects;

    private final ToggleButton singleLayerMode = new ToggleButton("Une couche");
    private final ToggleButton projectMode = new ToggleButton("Un projet");
    private final ComboBox<Project> projectChoice = new ComboBox<>();
    private final VBox projectSchedule = new VBox(12);
    private final Label projectSummary = new Label();
    private final StackPane resultArea = new StackPane();

    private final ComboBox<Technique> technique = new ComboBox<>();
    private final ComboBox<DryingClass> dryingClass = new ComboBox<>();
    private final ComboBox<Medium> medium = new ComboBox<>();
    private final Slider ratio = new Slider(0, 100, 0);
    private final Label ratioLabel = new Label();
    private final ComboBox<LayerThickness> thickness = new ComboBox<>();
    private final WorkshopForm workshop = new WorkshopForm();
    private final ListView<String> advice = new ListView<>();
    private final FlowPane milestones = new FlowPane();

    /** Evite de rejouer le calcul pendant que la selection d'une technique remplit le formulaire. */
    private boolean updating;

    /** Le panneau du mode "une couche", conserve pour y revenir. */
    private Node singleLayerResults;

    public DryingView(DryingTimeService dryingTimeService, PlanDryingService planDrying,
                      ProjectService projects) {
        this.dryingTimeService = dryingTimeService;
        this.planDrying = planDrying;
        this.projects = projects;
    }

    @Override
    public String title() {
        return "Sechage";
    }

    @Override
    public String subtitle() {
        return "Quand pourrez-vous reprendre la piece : temps ouvert, recouvrable, sec a coeur, vernissable.";
    }

    @Override
    public int order() {
        // Mener la seance : quand reprendre la piece.
        return 70;
    }

    @Override
    public int shortcut() {
        return 7;
    }

    @Override
    public Node create() {
        SplitPane split = new SplitPane(form(), results());
        split.setDividerPositions(0.42);
        recompute();
        return split;
    }

    /**
     * Deux facons de poser la question : pour une couche que l'on s'apprete a peindre, ou
     * pour une piece entiere.
     *
     * <p>La seconde est la plus utile, et la plus facile a renseigner : un projet porte
     * deja ses melanges, donc ses pigments, donc ses vitesses de sechage. Rien a saisir.</p>
     */
    private Node modeSelector() {
        ToggleGroup group = new ToggleGroup();
        singleLayerMode.setToggleGroup(group);
        projectMode.setToggleGroup(group);
        singleLayerMode.setSelected(true);
        singleLayerMode.setMaxWidth(Double.MAX_VALUE);
        projectMode.setMaxWidth(Double.MAX_VALUE);
        HBox.setHgrow(singleLayerMode, Priority.ALWAYS);
        HBox.setHgrow(projectMode, Priority.ALWAYS);

        // Un groupe a bascule peut se retrouver sans selection : on l'interdit.
        group.selectedToggleProperty().addListener((obs, old, selected) -> {
            if (selected == null) {
                group.selectToggle(old);
            } else {
                applyMode();
            }
        });

        projectChoice.setMaxWidth(Double.MAX_VALUE);
        projectChoice.setConverter(new javafx.util.StringConverter<>() {
            @Override
            public String toString(Project project) {
                return project == null ? "" : project.getName();
            }

            @Override
            public Project fromString(String value) {
                return null;
            }
        });
        projectChoice.setOnShowing(event -> projectChoice.getItems().setAll(projects.findAll()));
        projectChoice.getItems().setAll(projects.findAll());
        projectChoice.valueProperty().addListener((obs, old, project) -> recomputeProject());
        if (!projectChoice.getItems().isEmpty()) {
            projectChoice.setValue(projectChoice.getItems().getFirst());
        }

        return new VBox(10, new HBox(6, singleLayerMode, projectMode), projectChoice);
    }

    private void applyMode() {
        boolean project = projectMode.isSelected();
        projectChoice.setDisable(!project);
        if (project) {
            recomputeProject();
        } else {
            recompute();
        }
    }

    /** Planning d'un projet entier : une colonne par zone, du sombre au clair. */
    private void recomputeProject() {
        Project project = projectChoice.getValue();
        projectSchedule.getChildren().clear();

        if (project == null) {
            projectSummary.setText("Aucun projet enregistre.");
            showProjectResults();
            return;
        }

        var schedule = planDrying.schedule(projects.plan(project), workshop.current());
        projectSummary.setText("%s  -  %s en menant les zones de front, %s en les enchainant".formatted(
                project.getName(),
                Formats.duration(schedule.parallel()),
                Formats.duration(schedule.sequential())));

        schedule.zones().forEach(zone -> projectSchedule.getChildren().add(zoneCard(zone)));
        schedule.advice().forEach(message -> {
            Label label = new Label(message);
            label.setWrapText(true);
            label.getStyleClass().add("hint");
            projectSchedule.getChildren().add(label);
        });
        showProjectResults();
    }

    private Node zoneCard(PlanDryingService.ZoneSchedule zone) {
        VBox rows = new VBox(8);
        for (var layer : zone.layers()) {
            Label role = new Label(layer.role().toUpperCase(java.util.Locale.FRENCH));
            role.getStyleClass().add("milestone-title");
            role.setWrapText(true);
            role.setMinWidth(132);
            role.setPrefWidth(132);
            role.setMaxWidth(132);

            Label what = new Label(layer.mix().isBlank() ? layer.technique().label() : layer.mix());
            what.setWrapText(true);

            Label when = new Label("%s  -  %s  -  puis %s avant la couche suivante".formatted(
                    layer.technique().label(),
                    layer.dryingClass().label().toLowerCase(),
                    layer.waitAfter().isZero() ? "rien" : Formats.duration(layer.waitAfter())));
            when.getStyleClass().add("hint");
            when.setWrapText(true);

            VBox texts = new VBox(2, what, when);
            HBox.setHgrow(texts, Priority.ALWAYS);
            rows.getChildren().add(new HBox(10, role, texts));
        }

        String title = zone.material() == null || zone.material().isBlank()
                ? zone.zone() : zone.zone() + " - " + zone.material();
        return new Card("%s  (%s)".formatted(title, Formats.duration(zone.span())), rows);
    }

    private void showProjectResults() {
        ScrollPane scroll = new ScrollPane(new VBox(12, projectSummary, projectSchedule));
        scroll.setFitToWidth(true);
        resultArea.getChildren().setAll(new Card("Planning du projet", scroll));
    }

    private Node form() {
        technique.getItems().setAll(Technique.values());
        technique.setValue(Technique.OIL_WASH);
        technique.setMaxWidth(Double.MAX_VALUE);
        technique.valueProperty().addListener((obs, old, value) -> applyTechnique(value));

        dryingClass.getItems().setAll(DryingClass.values());
        dryingClass.setValue(DryingClass.MEDIUM);
        dryingClass.setMaxWidth(Double.MAX_VALUE);
        dryingClass.valueProperty().addListener((obs, old, value) -> recompute());

        medium.getItems().setAll(Medium.values());
        medium.setValue(Medium.ODORLESS_THINNER);
        medium.setMaxWidth(Double.MAX_VALUE);
        medium.valueProperty().addListener((obs, old, value) -> recompute());

        ratio.setShowTickMarks(true);
        ratio.setMajorTickUnit(25);
        ratio.valueProperty().addListener((obs, old, value) -> recompute());
        ratioLabel.setMinWidth(50);

        thickness.getItems().setAll(LayerThickness.values());
        thickness.setValue(LayerThickness.GLAZE);
        thickness.setMaxWidth(Double.MAX_VALUE);
        thickness.valueProperty().addListener((obs, old, value) -> recompute());

        workshop.onChange(() -> applyMode());

        GridPane grid = new GridPane();
        grid.setHgap(12);
        grid.setVgap(10);
        grid.setPadding(new Insets(12));
        ColumnConstraints labels = new ColumnConstraints();
        labels.setMinWidth(150);
        ColumnConstraints fields = new ColumnConstraints();
        fields.setHgrow(Priority.ALWAYS);
        grid.getColumnConstraints().addAll(labels, fields);

        HBox ratioBox = new HBox(8, ratio, ratioLabel);
        ratioBox.setAlignment(Pos.CENTER_LEFT);
        HBox.setHgrow(ratio, Priority.ALWAYS);

        grid.addRow(0, new Label("Technique"), technique);
        grid.addRow(1, new Label("Sechage du pigment"), dryingClass);
        grid.addRow(2, new Label("Medium"), medium);
        grid.addRow(3, new Label("Proportion de medium"), ratioBox);
        grid.addRow(4, new Label("Epaisseur de couche"), thickness);

        advice.setPlaceholder(new Label("Rien a signaler."));
        advice.setCellFactory(view -> new WrappingCell());
        VBox.setVgrow(advice, Priority.ALWAYS);

        Card advicePane = new Card("A surveiller", advice);
        VBox.setVgrow(advicePane, Priority.ALWAYS);

        VBox content = new VBox(14, new Card("Que planifier ?", modeSelector()),
                new Card("La couche", grid), workshop, advicePane);
        applyTechnique(technique.getValue());
        return content;
    }

    private Node results() {
        milestones.setHgap(14);
        milestones.setVgap(14);
        milestones.setPadding(new Insets(16));

        ScrollPane scroll = new ScrollPane(milestones);
        scroll.setFitToWidth(true);
        Card pane = new Card("Jalons de sechage", scroll);

        Label hint = new Label("""
                Ces durees sont des ordres de grandeur destines a planifier une seance. \
                Sur la piece, la verification reste tactile : un doigt sur le socle, jamais sur le visage.""");
        hint.setWrapText(true);
        hint.getStyleClass().add("hint");
        hint.setPadding(new Insets(10));

        VBox box = new VBox(10, pane, hint);
        VBox.setVgrow(pane, Priority.ALWAYS);

        resultArea.getChildren().setAll(box);
        projectSummary.getStyleClass().add("result-summary");
        projectSummary.setWrapText(true);
        singleLayerResults = box;
        return resultArea;
    }

    /** Pre-remplit le formulaire avec les reglages usuels de la technique choisie. */
    private void applyTechnique(Technique value) {
        if (value == null) {
            return;
        }
        updating = true;
        medium.setValue(value.defaultMedium());
        ratio.setValue(value.defaultRatio() * 100);
        thickness.setValue(value.typicalThickness());
        updating = false;
        recompute();
    }

    private void recompute() {
        if (updating) {
            return;
        }
        if (projectMode.isSelected()) {
            return;
        }
        if (singleLayerResults != null && !resultArea.getChildren().contains(singleLayerResults)) {
            resultArea.getChildren().setAll(singleLayerResults);
        }
        ratioLabel.setText("%.0f %%".formatted(ratio.getValue()));

        DryingContext context = new DryingContext(dryingClass.getValue(), medium.getValue(),
                ratio.getValue() / 100.0, thickness.getValue(), workshop.current());
        DryingEstimate estimate = dryingTimeService.estimate(context);

        milestones.getChildren().clear();
        addMilestone("Temps ouvert", estimate.openTime(),
                "La couche reste travaillable : fondus, dot fading, retrait du jus.");
        addMilestone("Sec au toucher", estimate.touchDry(),
                "La couche ne marque plus, mais elle se releve encore si on frotte.");
        addMilestone("Recouvrable", estimate.recoat(),
                "On peut poser la couche suivante sans arracher celle-ci.");
        addMilestone("Sec a coeur", estimate.throughDry(),
                "Manipulation, masquage et techniques qui demandent un support ferme.");
        addMilestone("Polymerisation complete", estimate.fullCure(),
                "Avant vernis final : un vernis pose trop tot emprisonne le solvant.");

        List<String> messages = estimate.advice();
        advice.getItems().setAll(messages);

        if (technique.getValue() != null) {
            advice.getItems().addAll(technique.getValue().tips());
        }
    }

    /** Une carte par jalon : le chiffre d'abord, l'explication ensuite. */
    private void addMilestone(String title, Duration duration, String explanation) {
        Label name = new Label(title.toUpperCase(java.util.Locale.FRENCH));
        name.getStyleClass().add("milestone-title");

        Label value = new Label(Formats.duration(duration));
        value.getStyleClass().add("milestone-value");

        Label when = new Label("soit " + Formats.clockAfter(duration));
        when.getStyleClass().add("milestone-when");
        when.setWrapText(true);

        Label detail = new Label(explanation);
        detail.getStyleClass().add("hint");
        detail.setWrapText(true);

        VBox card = new VBox(4, name, value, when, detail);
        card.getStyleClass().add("milestone-card");
        milestones.getChildren().add(card);
    }

    private static class WrappingCell extends javafx.scene.control.ListCell<String> {

        private final Label label = new Label();

        WrappingCell() {
            label.setWrapText(true);
        }

        @Override
        protected void updateItem(String text, boolean empty) {
            super.updateItem(text, empty);
            label.setText(empty ? "" : text);
            setGraphic(empty ? null : label);
        }
    }
}
