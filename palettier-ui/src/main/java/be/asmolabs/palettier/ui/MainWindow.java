package be.asmolabs.palettier.ui;

import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.control.ToggleButton;
import javafx.scene.control.ToggleGroup;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyCodeCombination;
import javafx.scene.input.KeyCombination;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;

/**
 * Coquille de l'application : barre laterale de navigation a gauche, en-tete de page
 * et zone de contenu a droite.
 *
 * <p>Les sections viennent du contexte Spring : ajouter un onglet revient a ajouter un
 * bean {@link AppView}, sans toucher a cette classe. Le contenu d'une section n'est
 * construit qu'a sa premiere ouverture, ce qui garde le demarrage rapide.</p>
 */
@Component
public class MainWindow {

    private static final String TITLE = "Palettier";
    private static final double WIDTH = 1440;
    private static final double HEIGHT = 920;

    private final List<AppView> views;
    private final Map<AppView, Node> content = new HashMap<>();
    private final Map<AppView, ToggleButton> navItems = new HashMap<>();

    private final StackPane contentArea = new StackPane();
    private final Label pageTitle = new Label();
    private final Label pageSubtitle = new Label();

    public MainWindow(List<AppView> views) {
        this.views = views.stream().sorted(Comparator.comparingInt(AppView::order)).toList();
    }

    @EventListener
    public void onStageReady(StageReadyEvent event) {
        BorderPane main = new BorderPane(contentArea);
        main.setTop(pageHeader());
        main.getStyleClass().add("main-area");

        BorderPane root = new BorderPane(main);
        root.setLeft(sidebar());

        Scene scene = new Scene(root, WIDTH, HEIGHT);
        scene.getStylesheets().add(MainWindow.class.getResource("app.css").toExternalForm());
        installShortcuts(scene);

        Stage stage = event.stage();
        stage.getIcons().addAll(AppIcons.all());
        AppIcons.applyToTaskbar();
        stage.setTitle(TITLE);
        stage.setScene(scene);
        stage.setMinWidth(1180);
        stage.setMinHeight(760);
        stage.show();
    }

    // --- Barre laterale ----------------------------------------------------

    private Node sidebar() {
        VBox nav = new VBox(2);
        nav.getStyleClass().add("nav-list");

        ToggleGroup group = new ToggleGroup();
        for (AppView view : views) {
            if (view.separatorBefore() && !nav.getChildren().isEmpty()) {
                nav.getChildren().add(navSeparator());
            }
            nav.getChildren().add(navItem(view, group));
        }

        Region spacer = new Region();
        VBox.setVgrow(spacer, Priority.ALWAYS);

        VBox sidebar = new VBox(brand(), nav, spacer, footer());
        sidebar.getStyleClass().add("sidebar");
        sidebar.setPrefWidth(248);
        sidebar.setMinWidth(248);

        nav.getChildren().stream()
                .filter(ToggleButton.class::isInstance)
                .map(ToggleButton.class::cast)
                .findFirst()
                .ifPresent(first -> first.setSelected(true));
        return sidebar;
    }

    private ToggleButton navItem(AppView view, ToggleGroup group) {
        Region dot = new Region();
        dot.getStyleClass().add("nav-dot");

        Label label = new Label(view.title());
        label.getStyleClass().add("nav-label");

        // Le raccourci s'affiche, sinon personne ne le devine.
        Label hint = new Label(view.shortcut() < 0 ? "" : String.valueOf(view.shortcut()));
        hint.getStyleClass().add("nav-shortcut");

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        HBox layout = new HBox(10, dot, label, spacer, hint);
        layout.setAlignment(Pos.CENTER_LEFT);
        layout.setMinWidth(200);

        ToggleButton item = new ToggleButton();
        navItems.put(view, item);
        item.setGraphic(layout);
        item.getStyleClass().add("nav-item");
        item.setToggleGroup(group);
        item.setMaxWidth(Double.MAX_VALUE);
        item.setOnAction(e -> {
            // Un onglet deja actif ne doit pas pouvoir se deselectionner.
            item.setSelected(true);
            show(view);
        });
        item.selectedProperty().addListener((obs, old, selected) -> {
            if (selected) {
                show(view);
            }
        });
        return item;
    }

    /**
     * Raccourcis clavier vers les sections : commande et le rang de la section.
     *
     * <p>On peint d'une main, la souris est souvent occupee a tenir autre chose. Passer
     * du releve au melange doit pouvoir se faire sans lacher le pinceau.</p>
     */
    private void installShortcuts(Scene scene) {
        for (AppView view : views) {
            int digit = view.shortcut();
            if (digit < 0 || digit > 9) {
                continue;
            }
            KeyCombination shortcut = new KeyCodeCombination(
                    KeyCode.getKeyCode(String.valueOf(digit)), KeyCombination.SHORTCUT_DOWN);
            scene.getAccelerators().put(shortcut, () -> select(view));
        }
    }

    /** Selectionne l'entree de navigation correspondante, ce qui declenche l'affichage. */
    private void select(AppView view) {
        navItems.get(view).setSelected(true);
    }

    /**
     * Filet de separation entre deux groupes de sections.
     *
     * <p>La marge se pose sur le conteneur et non en feuille de style : JavaFX n'a pas
     * de propriete de marge, c'est le parent qui la porte.</p>
     */
    private Node navSeparator() {
        Region line = new Region();
        line.getStyleClass().add("nav-separator");
        VBox.setMargin(line, new Insets(10, 12, 10, 12));
        return line;
    }

    private Node brand() {
        Region mark = new Region();
        mark.getStyleClass().add("app-mark");
        // Une goutte d'huile sur la palette : ocre vers terre brulee.
        mark.setStyle("-fx-background-color: linear-gradient(to bottom right, #e2a05f, #7a3a22);");

        Label name = new Label(TITLE);
        name.getStyleClass().add("brand-name");

        Label kind = new Label("Atelier huile");
        kind.getStyleClass().add("brand-kind");

        HBox brand = new HBox(11, mark, new VBox(0, name, kind));
        brand.setAlignment(Pos.CENTER_LEFT);
        brand.getStyleClass().add("brand");
        return brand;
    }

    private Node footer() {
        Label note = new Label("Catalogue et recettes stockes sur le poste.\nAucune donnee ne part ailleurs.");
        note.getStyleClass().add("sidebar-footer");
        note.setWrapText(true);
        return note;
    }

    // --- Zone de contenu ---------------------------------------------------

    private Node pageHeader() {
        pageTitle.getStyleClass().add("page-title");
        pageSubtitle.getStyleClass().add("page-subtitle");
        pageSubtitle.setWrapText(true);

        VBox header = new VBox(3, pageTitle, pageSubtitle);
        header.getStyleClass().add("page-header");
        return header;
    }

    private void show(AppView view) {
        pageTitle.setText(view.title());
        pageSubtitle.setText(view.subtitle());
        contentArea.getChildren().setAll(content.computeIfAbsent(view, AppView::create));
    }
}
