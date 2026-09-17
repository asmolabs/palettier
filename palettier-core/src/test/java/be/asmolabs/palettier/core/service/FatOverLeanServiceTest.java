package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.domain.Recipe;
import be.asmolabs.palettier.core.domain.RecipeStep;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.service.FatOverLeanService.Risk;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * La regle doit se taire sur ce qui se pratique tous les jours, et parler quand la piece
 * est reellement en jeu. Une alerte qui crie a chaque glacis serait desactivee le premier
 * soir, et ne servirait plus a rien le jour ou elle aurait raison.
 */
class FatOverLeanServiceTest {

    private final FatOverLeanService service = new FatOverLeanService();

    private static Project pieceWith(String... techniques) {
        Project project = new Project("Piece", "Buste", null);
        ProjectZone zone = new ProjectZone("Visage", "Peau", "");
        for (int i = 0; i < techniques.length; i++) {
            zone.addLayer(new ProjectLayer("Couche " + (i + 1), Rgb.ofHex("#C98F72"),
                    techniques[i], "", ProjectLayer.Kind.LADDER));
        }
        project.addZone(zone);
        return project;
    }

    @Test
    @DisplayName("un plan tout en glacis ne declenche rien")
    void glazesAllTheWayAreFine() {
        assertThat(service.inspect(pieceWith(
                Technique.GLAZE.label(), Technique.GLAZE.label(), Technique.GLAZE.label())))
                .isEmpty();
    }

    @Test
    @DisplayName("un glacis sur un aplat de base ne declenche rien : un voile ne tire pas")
    void aveilOverAFatBaseIsFine() {
        // Le glacis est pourtant bien plus maigre que l'aplat : c'est l'epaisseur qui
        // decide, et c'est la pratique courante sur figurine.
        assertThat(service.fatnessOf(Technique.GLAZE.label()))
                .isLessThan(service.fatnessOf(Technique.BASE_LAYER.label()));
        assertThat(service.isVeil(Technique.GLAZE.label())).isTrue();

        assertThat(service.inspect(pieceWith(Technique.BASE_LAYER.label(), Technique.GLAZE.label())))
                .isEmpty();
    }

    @Test
    @DisplayName("des coulures maigres sur un fondu gras, a epaisseur egale, sont signalees")
    void leanOverFatAtEqualThicknessIsReported() {
        var risks = service.inspect(pieceWith(
                Technique.BLENDING.label(), Technique.STREAKING_GRIME.label()));

        assertThat(risks).hasSize(1);
        Risk risk = risks.getFirst();
        assertThat(risk.where()).isEqualTo("Visage");
        assertThat(risk.under()).isEqualTo("Couche 1");
        assertThat(risk.over()).isEqualTo("Couche 2");
        assertThat(risk.drop()).isGreaterThan(0.4);
        assertThat(risk.explanation())
                .contains(Technique.STREAKING_GRIME.label())
                .contains(Technique.BLENDING.label())
                .contains("tirera");
    }

    @Test
    @DisplayName("l'ordre inverse, du maigre vers le gras, est la bonne facon de faire")
    void fatOverLeanIsTheRightWayRound() {
        assertThat(service.inspect(pieceWith(
                Technique.STREAKING_GRIME.label(), Technique.BLENDING.label())))
                .isEmpty();
    }

    @Test
    @DisplayName("une recette est verifiee sur ses valeurs saisies, pas sur des moyennes")
    void arecipeIsCheckedOnItsOwnValues() {
        // Deux etapes au meme medium et a la meme epaisseur : c'est la dilution saisie,
        // et elle seule, qui fait passer la seconde du gras au maigre.
        Recipe recipe = new Recipe("Cape rouge", "Tissu");
        recipe.addStep(new RecipeStep(Technique.BLENDING, "Rouge + noir",
                Medium.LINSEED_OIL, 0.30, LayerThickness.NORMAL));
        recipe.addStep(new RecipeStep(Technique.BLENDING, "Rouge pur",
                Medium.ODORLESS_THINNER, 0.80, LayerThickness.NORMAL));

        var risks = service.inspect(recipe);

        assertThat(risks).hasSize(1);
        assertThat(risks.getFirst().where()).isEqualTo("Cape rouge");
        assertThat(risks.getFirst().under()).isEqualTo("Etape 1");
        assertThat(risks.getFirst().over()).isEqualTo("Etape 2");
    }

    @Test
    @DisplayName("la meme recette dans le bon ordre ne declenche rien")
    void therightOrderIsSilent() {
        Recipe recipe = new Recipe("Cape rouge", "Tissu");
        recipe.addStep(new RecipeStep(Technique.BLENDING, "Rouge pur",
                Medium.ODORLESS_THINNER, 0.80, LayerThickness.NORMAL));
        recipe.addStep(new RecipeStep(Technique.BLENDING, "Rouge + noir",
                Medium.LINSEED_OIL, 0.30, LayerThickness.NORMAL));

        assertThat(service.inspect(recipe)).isEmpty();
    }

    @Test
    @DisplayName("une technique inconnue ne fait pas echouer la verification")
    void anUnknownTechniqueIsTolerated() {
        assertThat(service.inspect(pieceWith("Sfumato maison", Technique.GLAZE.label())))
                .isEmpty();
    }
}
