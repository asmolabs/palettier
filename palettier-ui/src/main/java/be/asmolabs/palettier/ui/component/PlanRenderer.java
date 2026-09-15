package be.asmolabs.palettier.ui.component;

import be.asmolabs.palettier.core.plan.PaintingPlan;
import java.util.Locale;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;

/**
 * Affichage d'un plan de peinture.
 *
 * <p>Partage par l'assistant, qui vient de le produire, et par les projets, qui le
 * relisent : un plan enregistre doit se presenter exactement comme au moment ou il a ete
 * propose, sans quoi on doute d'avoir la meme chose sous les yeux.</p>
 */
public class PlanRenderer {

    /**
     * Ce qu'on peut modifier sur un plan enregistre.
     *
     * <p>Les rangs de zone et de couche servent de reference plutot que les objets eux-memes :
     * un plan est un enregistrement sans identite, alors qu'un projet a des lignes en base.
     * Le rang est ce qui relie les deux de facon sure.</p>
     */
    public interface Edits {

        void editZone(int zoneIndex);

        void editLayer(int zoneIndex, int layerIndex);
    }

    private Edits edits;

    /** Active les commandes de modification. Sans appel, le plan reste en lecture seule. */
    public void editable(Edits edits) {
        this.edits = edits;
    }

    /** Remplit le conteneur avec les cartes du plan. Le contenu precedent est remplace. */
    public void render(VBox target, PaintingPlan plan) {
        target.getChildren().clear();

        if (plan.approach() != null && !plan.approach().isBlank()) {
            Label approach = new Label(plan.approach());
            approach.setWrapText(true);
            target.getChildren().add(new Card("Approche generale", approach));
        }
        for (int i = 0; i < plan.zones().size(); i++) {
            target.getChildren().add(zoneCard(plan.zones().get(i), i));
        }

        if (plan.zones().isEmpty()) {
            Label empty = new Label("Ce plan ne contient aucune zone.");
            empty.getStyleClass().add("hint");
            target.getChildren().add(empty);
        }
    }

    private Node zoneCard(PaintingPlan.Zone zone, int zoneIndex) {
        VBox layers = new VBox(10);
        for (int i = 0; i < zone.layers().size(); i++) {
            layers.getChildren().add(layerRow(zone.layers().get(i), zoneIndex, i));
        }

        String title = zone.material() == null || zone.material().isBlank()
                ? zone.name()
                : zone.name() + " - " + zone.material();

        Node action = null;
        if (edits != null) {
            Button rename = new Button("Modifier la zone");
            rename.getStyleClass().add("link-button");
            rename.setOnAction(event -> edits.editZone(zoneIndex));
            action = rename;
        }
        return new Card(title, zone.note(), layers, action);
    }

    /** Couleur visee et couleur atteinte cote a cote : l'ecart se lit, il ne se raconte pas. */
    private Node layerRow(PaintingPlan.Layer layer, int zoneIndex, int layerIndex) {
        ColorSwatch wanted = new ColorSwatch(38, 44);
        wanted.setColor(layer.target());

        ColorSwatch achieved = new ColorSwatch(38, 44);
        achieved.setColor(layer.achieved());

        Label role = new Label(layer.role().toUpperCase(Locale.FRENCH));
        role.getStyleClass().add("milestone-title");
        role.setMinWidth(78);

        Label recipe = new Label(layer.recipe() == null ? "Palette vide" : layer.recipe().describe());
        recipe.setWrapText(true);

        Label detail = new Label("%s  -  %s  -  ecart %.1f, %s".formatted(
                layer.target().toHex(),
                layer.technique() == null || layer.technique().isBlank()
                        ? "technique non precisee" : layer.technique(),
                layer.deltaE(), layer.reachability()));
        detail.getStyleClass().add("hint");
        detail.setWrapText(true);

        VBox texts = new VBox(3, recipe, detail);
        if (layer.note() != null && !layer.note().isBlank()) {
            Label note = new Label(layer.note());
            note.getStyleClass().add("hint");
            note.setWrapText(true);
            texts.getChildren().add(note);
        }
        HBox.setHgrow(texts, Priority.ALWAYS);

        Region gap = new Region();
        gap.setMinWidth(4);

        HBox row = new HBox(10, role, wanted, achieved, gap, texts);
        row.setAlignment(Pos.TOP_LEFT);
        row.setPadding(new Insets(1));

        if (edits != null) {
            Button edit = new Button("Modifier");
            edit.getStyleClass().add("link-button");
            edit.setOnAction(event -> edits.editLayer(zoneIndex, layerIndex));
            row.getChildren().add(edit);
        }
        return row;
    }
}
