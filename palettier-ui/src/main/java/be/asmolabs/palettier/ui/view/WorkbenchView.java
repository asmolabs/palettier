package be.asmolabs.palettier.ui.view;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.service.WorkbenchService;
import be.asmolabs.palettier.core.service.WorkbenchService.Bench;
import be.asmolabs.palettier.core.service.WorkbenchService.CoatState;
import be.asmolabs.palettier.core.service.WorkbenchService.PieceState;
import be.asmolabs.palettier.core.service.WorkbenchService.Stage;
import be.asmolabs.palettier.core.service.WorkbenchService.ZoneState;
import be.asmolabs.palettier.ui.AppView;
import be.asmolabs.palettier.ui.component.Card;
import be.asmolabs.palettier.ui.component.ColorSwatch;
import be.asmolabs.palettier.ui.component.Formats;
import be.asmolabs.palettier.ui.component.Pill;
import java.time.Duration;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.Locale;
import javafx.concurrent.Task;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import org.springframework.stereotype.Component;

/**
 * Ce que l'on peut reprendre aujourd'hui.
 *
 * <p>Les autres ecrans repondent a des questions ponctuelles -- quelle huile approche
 * cette teinte, que donne ce melange. Celui-ci repond a la question permanente, celle
 * qu'on se pose en entrant dans l'atelier : sur quoi puis-je travailler maintenant.</p>
 *
 * <p>Il ne calcule rien de plus que l'ecran de sechage ; il sait simplement quand chaque
 * couche a ete posee, ce qui transforme une duree en une date. "Comptez trois jours"
 * devient "recouvrable depuis hier soir", et cela suffit a changer l'usage.</p>
 *
 * <p>Une piece apparait comme disponible des qu'une seule de ses zones l'est. Attendre
 * que tout soit sec n'aurait aucun sens : pendant que le visage prend, la cape avance.</p>
 */
@Component
public class WorkbenchView implements AppView {

    private static final DateTimeFormatter POSED =
            DateTimeFormatter.ofPattern("d MMMM 'a' HH'h'mm", Locale.FRENCH).withZone(ZoneId.systemDefault());

    private final WorkbenchService workbench;

    private final Label summary = new Label();
    private final VBox pieces = new VBox(14);
    private final Button refresh = new Button("Rafraichir");

    public WorkbenchView(WorkbenchService workbench) {
        this.workbench = workbench;
    }

    @Override
    public String title() {
        return "Aujourd'hui";
    }

    @Override
    public String subtitle() {
        return "Ou en sont vos pieces, et lesquelles peuvent etre reprises maintenant.";
    }

    @Override
    public int order() {
        // Avant tout le reste : c'est la question du matin.
        return 5;
    }

    @Override
    public Node create() {
        summary.getStyleClass().add("result-summary");
        summary.setWrapText(true);

        refresh.setOnAction(event -> reload());

        VBox everything = new VBox(14, summary, new HBox(8, refresh), pieces);
        everything.setPadding(new Insets(2));

        ScrollPane scroll = new ScrollPane(everything);
        scroll.setFitToWidth(true);
        VBox.setVgrow(scroll, Priority.ALWAYS);

        Card card = new Card("L'etabli", scroll);
        VBox.setVgrow(card, Priority.ALWAYS);

        // Le temps passe pendant que l'ecran est ferme : revenir dessus doit donner
        // l'etat du moment, pas celui de la derniere ouverture. C'est aussi ce qui
        // declenche la premiere lecture -- la fenetre attache la section des qu'elle la
        // construit, et charger ici en plus la ferait deux fois.
        card.sceneProperty().addListener((obs, old, scene) -> {
            if (scene != null) {
                reload();
            }
        });

        return card;
    }

    /**
     * Relit l'etabli hors du fil d'affichage.
     *
     * <p>La lecture traverse tous les projets et, associations obligent, tout ce qui y
     * pend. Sur le fil d'affichage, la fenetre se figerait le temps du chargement.</p>
     */
    private void reload() {
        refresh.setDisable(true);
        summary.setText("Lecture de l'etabli...");

        Task<Bench> task = new Task<>() {
            @Override
            protected Bench call() {
                return workbench.bench();
            }
        };
        task.setOnSucceeded(event -> {
            refresh.setDisable(false);
            render(task.getValue());
        });
        task.setOnFailed(event -> {
            refresh.setDisable(false);
            pieces.getChildren().clear();
            summary.setText("Lecture impossible : " + task.getException().getMessage());
        });
        Thread.ofPlatform().daemon().name("workbench").start(task);
    }

    private void render(Bench bench) {
        pieces.getChildren().clear();

        if (bench.pieces().isEmpty()) {
            summary.setText("Aucun projet enregistre.");
            pieces.getChildren().add(hint(
                    "Les plans proposes par l'assistant peuvent etre enregistres comme projets. "
                    + "Cochez ensuite chaque couche posee : cet ecran saura alors quand la piece "
                    + "redevient reprenable."));
            return;
        }

        summary.setText(headline(bench));
        bench.pieces().forEach(piece -> pieces.getChildren().add(pieceCard(piece)));
    }

    private static String headline(Bench bench) {
        int ready = bench.ready().size();
        if (ready > 0) {
            return ready == 1
                    ? "Une piece peut etre reprise maintenant."
                    : "%d pieces peuvent etre reprises maintenant.".formatted(ready);
        }
        return bench.nextAvailability()
                .map(wait -> "Rien a reprendre pour l'instant. La premiere zone se libere dans %s, %s."
                        .formatted(Formats.duration(wait), Formats.clockAfter(wait)))
                .orElse("Toutes les pieces sont terminees.");
    }

    // --- Une piece ---------------------------------------------------------

    private Node pieceCard(PieceState piece) {
        VBox zones = new VBox(10);
        piece.zones().forEach(zone -> zones.getChildren().add(zoneRow(zone)));

        if (piece.zones().isEmpty()) {
            zones.getChildren().add(hint("Ce projet ne contient aucune zone."));
        }
        return new Card(piece.project().getName(), pieceHint(piece), zones);
    }

    private static String pieceHint(PieceState piece) {
        String progress = "%d couches posees sur %d".formatted(piece.appliedCoats(), piece.totalCoats());

        if (piece.done()) {
            return progress + "  -  piece terminee.";
        }
        int ready = piece.readyZones().size();
        if (ready > 0) {
            return "%s  -  %s disponible%s : %s.".formatted(progress,
                    ready == 1 ? "une zone" : ready + " zones", ready == 1 ? "" : "s",
                    piece.readyZones().stream().map(ZoneState::name).reduce((a, b) -> a + ", " + b).orElse(""));
        }
        return piece.nextAvailability()
                .map(wait -> "%s  -  rien avant %s (dans %s)."
                        .formatted(progress, Formats.clockAfter(wait), Formats.duration(wait)))
                .orElse(progress + ".");
    }

    // --- Une zone ----------------------------------------------------------

    private Node zoneRow(ZoneState zone) {
        ColorSwatch swatch = new ColorSwatch(34, 34);
        if (zone.last() != null) {
            swatch.setColor(Rgb.ofHex(zone.last().targetHex()));
        } else {
            // Une zone jamais commencee n'a pas de derniere couleur : on montre un vide
            // plutot qu'une teinte arbitraire.
            swatch.setColor(new Rgb(0.13, 0.11, 0.10));
        }

        Label name = new Label(zone.material() == null || zone.material().isBlank()
                ? zone.name()
                : zone.name() + " - " + zone.material());
        name.getStyleClass().add("milestone-title");

        Label state = hint(zoneState(zone));
        VBox texts = new VBox(3, name, state);
        HBox.setHgrow(texts, Priority.ALWAYS);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        HBox row = new HBox(12, swatch, texts, spacer, pill(zone));
        row.setAlignment(Pos.CENTER_LEFT);
        return row;
    }

    private static String zoneState(ZoneState zone) {
        if (zone.done()) {
            return "Les %d couches prevues sont posees.".formatted(zone.total());
        }
        if (zone.notStarted()) {
            return "Aucune couche posee, %d prevues. Rien n'empeche de commencer.".formatted(zone.total());
        }

        CoatState last = zone.last();
        String posed = "%s posee le %s.".formatted(last.role(), POSED.format(last.appliedAt()));
        String next = last.nextRole() == null ? "" : " Au tour de : %s.".formatted(last.nextRole());

        if (zone.readyNow()) {
            return posed + "  " + last.stage().meaning() + next;
        }
        Duration wait = zone.remaining();
        return posed + "  " + last.stage().meaning()
                + " Disponible dans %s, %s.".formatted(Formats.duration(wait), Formats.clockAfter(wait));
    }

    /** Vert quand on peut y aller, chaud quand il faut attendre, eteint quand c'est fini. */
    private static Node pill(ZoneState zone) {
        if (zone.done()) {
            return Pill.of("Terminee", "pill-ghost");
        }
        if (zone.notStarted()) {
            return Pill.of("A commencer", "pill-fast");
        }
        Stage stage = zone.last().stage();
        if (zone.readyNow()) {
            return Pill.of(stage.label(), "pill-fast");
        }
        return Pill.of(stage.label(), stage == Stage.OPEN ? "pill-medium" : "pill-slow");
    }

    private static Label hint(String text) {
        Label label = new Label(text);
        label.getStyleClass().add("hint");
        label.setWrapText(true);
        return label;
    }
}
