package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.catalog.CatalogLoader;
import be.asmolabs.palettier.core.catalog.PigmentIndex;
import be.asmolabs.palettier.core.domain.OilPaint;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/** Association d'un libelle lu sur un tube avec une fiche du catalogue. */
class PaintMatcherTest {

    private final PaintMatcher matcher = new PaintMatcher();
    private final List<OilPaint> catalogue = new CatalogLoader(new PigmentIndex()).load();

    @Test
    @DisplayName("un libelle exact retrouve son tube")
    void anExactLabelFindsItsPaint() {
        var match = matcher.match("Winsor & Newton Burnt Umber", catalogue).orElseThrow();

        assertThat(match.paint().getName()).isEqualTo("Burnt Umber");
        assertThat(match.paint().getBrand()).isEqualTo("Winsor & Newton");
        assertThat(match.isReliable()).isTrue();
    }

    @Test
    @DisplayName("les accents, la ponctuation et le format du tube ne genent pas")
    void punctuationAndFormatAreIgnored() {
        assertThat(matcher.match("W&N  BURNT   UMBER  37ml", catalogue))
                .hasValueSatisfying(m -> assertThat(m.paint().getName()).isEqualTo("Burnt Umber"));
        assertThat(matcher.match("Terre d'ombre - Raw Umber (oil colour)", catalogue))
                .hasValueSatisfying(m -> assertThat(m.paint().getName()).contains("Raw Umber"));
    }

    @Test
    @DisplayName("un nom sans marque suffit quand il est distinctif")
    void adistinctiveNameIsEnough() {
        assertThat(matcher.match("Quinacridone Magenta", catalogue))
                .hasValueSatisfying(m -> assertThat(m.paint().getName()).isEqualTo("Quinacridone Magenta"));
    }

    @Test
    @DisplayName("une marque seule ne designe aucun tube en particulier")
    void abrandAloneMatchesNothing() {
        assertThat(matcher.match("Winsor & Newton", catalogue)).isEmpty();
        assertThat(matcher.match("Gamblin", catalogue)).isEmpty();
    }

    @Test
    @DisplayName("un libelle qui ne correspond a rien ne renvoie rien")
    void nonsenseMatchesNothing() {
        assertThat(matcher.match("boite de vis cruciformes", catalogue)).isEmpty();
        assertThat(matcher.match("", catalogue)).isEmpty();
        assertThat(matcher.match(null, catalogue)).isEmpty();
    }

    @Test
    @DisplayName("la marque departage deux tubes de meme nom")
    void thebrandDecidesBetweenIdenticalNames() {
        var gamblin = matcher.match("Gamblin Yellow Ochre", catalogue).orElseThrow();
        var winsor = matcher.match("Winsor & Newton Yellow Ochre", catalogue).orElseThrow();

        assertThat(gamblin.paint().getBrand()).isEqualTo("Gamblin");
        assertThat(winsor.paint().getBrand()).isEqualTo("Winsor & Newton");
    }
}
