package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

/** Declaration de ce que le peintre possede reellement. */
@SpringBootTest
@Transactional
class OwnershipTest {

    @Autowired
    private PaintCatalogService catalog;

    @Autowired
    private ColorMixService mixer;

    @Test
    @DisplayName("partir de zero vide l'inventaire sans toucher au catalogue")
    void startingFromNothingEmptiesTheInventory() {
        int total = catalog.findAll().size();

        catalog.declareNothingOwned();

        assertThat(catalog.countOwned()).isZero();
        assertThat(catalog.findAll()).hasSize(total);
        assertThat(catalog.findInStock()).isEmpty();
    }

    @Test
    @DisplayName("declarer quelques tubes les rend seuls disponibles aux recherches")
    void declaredPaintsAreTheOnesSearched() {
        catalog.declareNothingOwned();
        List<OilPaint> mine = List.of(
                catalog.search("Titanium White").getFirst(),
                catalog.search("Burnt Umber").getFirst(),
                catalog.search("Yellow Ochre").getFirst());
        catalog.setOwned(mine, true);

        assertThat(catalog.countOwned()).isEqualTo(3);

        var suggestions = mixer.suggestMixes(Rgb.ofHex("#8A6A4A"), catalog.findInStock(), 3);
        assertThat(suggestions).isNotEmpty();
        assertThat(suggestions).allSatisfy(suggestion ->
                assertThat(suggestion.parts()).allSatisfy(part ->
                        assertThat(mine).contains(part.paint())));
    }

    @Test
    @DisplayName("un inventaire court autorise des melanges plus riches qu'un catalogue entier")
    void ashortInventoryAllowsRicherMixes() {
        catalog.declareNothingOwned();
        catalog.setOwned(List.of(
                catalog.search("Titanium White").getFirst(),
                catalog.search("Cadmium Yellow Pale").getFirst(),
                catalog.search("Permanent Rose").getFirst(),
                catalog.search("Burnt Umber").getFirst()), true);

        // La regle d'adaptation s'applique donc a ce que l'on a, pas a ce qui existe.
        assertThat(ColorMixService.recommendedMaxPaints(catalog.findInStock().size())).isEqualTo(5);
        assertThat(ColorMixService.recommendedMaxPaints(catalog.findAll().size())).isEqualTo(3);
    }

    @Test
    @DisplayName("retirer un tube le fait disparaitre des recherches sans l'effacer")
    void removingAPaintKeepsItInTheCatalogue() {
        OilPaint paint = catalog.search("Burnt Umber").getFirst();
        catalog.setOwned(paint, false);

        assertThat(paint.isInStock()).isFalse();
        assertThat(catalog.findInStock()).doesNotContain(paint);
        assertThat(catalog.findById(paint.getId())).isPresent();
    }
}
