package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Recipe;
import be.asmolabs.palettier.core.repository.RecipeRepository;
import be.asmolabs.palettier.core.service.RecipeTimelineService;
import be.asmolabs.palettier.core.service.RecipeTimelineService.Timeline;
import be.asmolabs.palettier.core.service.RecipeTimelineService.TimelineEntry;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.Formats;
import be.asmolabs.palettier.ui.component.WorkshopForm;
import java.time.Duration;
import javafx.beans.property.SimpleStringProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.geometry.Insets;
import javafx.scene.Node;
import javafx.scene.control.ComboBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.control.SplitPane;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import org.springframework.stereotype.Component;

/**
 * Recettes enregistrees et leur deroule dans le temps. L'interet a l'huile est de voir
 * d'un coup d'oeil combien de seances une recette represente reellement.
 */
@Component
public class RecipeView implements AppView {

    private final RecipeRepository repository;
    private final RecipeTimelineService timelineService;

    private final ObservableList<Recipe> recipes = FXCollections.observableArrayList();
    private final ObservableList<TimelineEntry> entries = FXCollections.observableArrayList();
    private final ListView<Recipe> recipeList = new ListView<>(recipes);
    private final ComboBox<DryingClass> dryingClass = new ComboBox<>();
    private final WorkshopForm workshop = new WorkshopForm();
    private final Label summary = new Label();
    private final Label notes = new Label();

    public RecipeView(RecipeRepository repository, RecipeTimelineService timelineService) {
        this.repository = repository;
        this.timelineService = timelineService;
    }

    @Override
    public String title() {
        return "Recettes";
    }

    @Override
    public String subtitle() {
        return "Le deroule reel d'une recette : a l'huile, l'essentiel du planning est fait d'attente.";
    }

    @Override
    public int order() {
        // Mener la seance : l'enchainement des couches.
        return 80;
    }

    @Override
    public int shortcut() {
        return 8;
    }

    @Override
    public Node create() {
        recipes.setAll(repository.findAllByOrderByNameAsc());
        recipeList.getSelectionModel().selectedItemProperty()
                .addListener((obs, old, recipe) -> refresh(recipe));

        dryingClass.getItems().setAll(DryingClass.values());
        dryingClass.setValue(DryingClass.MEDIUM);
        dryingClass.valueProperty().addListener((obs, old, value) -> refresh(selected()));
        workshop.onChange(() -> refresh(selected()));

        HBox dryingRow = new HBox(10, new Label("Sechage retenu"), dryingClass);
        dryingRow.setAlignment(javafx.geometry.Pos.CENTER_LEFT);
        dryingClass.setMaxWidth(Double.MAX_VALUE);
        HBox.setHgrow(dryingClass, Priority.ALWAYS);

        Card listCard = new Card("Recettes enregistrees", new VBox(12, recipeList, dryingRow));
        VBox.setVgrow(recipeList, Priority.ALWAYS);
        VBox.setVgrow(listCard, Priority.ALWAYS);

        VBox left = new VBox(14, listCard, workshop);

        SplitPane split = new SplitPane(left, timelinePanel());
        split.setDividerPositions(0.34);

        if (!recipes.isEmpty()) {
            recipeList.getSelectionModel().selectFirst();
        }
        return split;
    }

    private Node timelinePanel() {
        summary.getStyleClass().add("result-summary");
        summary.setWrapText(true);
        notes.setWrapText(true);
        notes.getStyleClass().add("hint");

        VBox header = new VBox(6, summary, notes);

        BorderPane content = new BorderPane(timelineTable());
        content.setTop(header);
        BorderPane.setMargin(header, new Insets(0, 0, 12, 0));

        return new Card("Deroule de la recette", content);
    }

    private TableView<TimelineEntry> timelineTable() {
        TableView<TimelineEntry> table = new TableView<>(entries);
        table.setPlaceholder(new Label("Selectionnez une recette."));

        table.getColumns().addAll(
                column("#", 40, e -> String.valueOf(e.position())),
                column("Technique", 180, e -> e.step().getTechnique().label()),
                column("Melange", 280, e -> e.step().getPaintMix()),
                column("Dilution", 110, e -> Formats.percent(e.step().getMediumRatio())),
                column("Debut", 110, e -> e.startOffset().isZero() ? "immediat" : "T + " + Formats.duration(e.startOffset())),
                column("Attente ensuite", 130, e -> e.waitAfter().isZero() ? "-" : Formats.duration(e.waitAfter())),
                column("Note", 280, e -> e.step().getNotes()));

        table.setColumnResizePolicy(TableView.CONSTRAINED_RESIZE_POLICY_FLEX_LAST_COLUMN);
        table.setRowFactory(view -> {
            var row = new javafx.scene.control.TableRow<TimelineEntry>();
            row.itemProperty().addListener((obs, old, entry) ->
                    row.setTooltip(entry == null ? null : new javafx.scene.control.Tooltip(tooltipFor(entry))));
            return row;
        });
        return table;
    }

    private static String tooltipFor(TimelineEntry entry) {
        return """
                %s
                Medium : %s
                Epaisseur : %s
                Temps ouvert : %s
                Recouvrable apres : %s""".formatted(
                entry.step().getTechnique().label(),
                entry.step().getMedium().label(),
                entry.step().getThickness().label(),
                Formats.duration(entry.drying().openTime()),
                Formats.duration(entry.drying().recoat()));
    }

    private static TableColumn<TimelineEntry, String> column(String title, double width,
                                                             java.util.function.Function<TimelineEntry, String> extractor) {
        TableColumn<TimelineEntry, String> column = new TableColumn<>(title);
        column.setCellValueFactory(cell -> new SimpleStringProperty(extractor.apply(cell.getValue())));
        column.setPrefWidth(width);
        return column;
    }

    private Recipe selected() {
        return recipeList.getSelectionModel().getSelectedItem();
    }

    private void refresh(Recipe recipe) {
        if (recipe == null) {
            entries.clear();
            summary.setText("");
            notes.setText("");
            return;
        }

        Timeline timeline = timelineService.plan(recipe, dryingClass.getValue(), workshop.current());
        entries.setAll(timeline.entries());

        Duration span = timeline.totalActiveSpan();
        summary.setText("%s  -  %d etapes, %s d'attente cumulee, vernis final envisageable apres %s"
                .formatted(recipe.getName(), timeline.entries().size(),
                        span.isZero() ? "aucune" : Formats.duration(span),
                        Formats.duration(timeline.untilVarnish())));
        notes.setText(recipe.getNotes());
    }
}
