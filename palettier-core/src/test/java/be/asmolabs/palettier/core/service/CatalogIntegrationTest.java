package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.repository.RecipeRepository;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import be.asmolabs.palettier.core.service.MixModels.PaintMatch;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;

@SpringBootTest
class CatalogIntegrationTest {

    @Autowired
    private PaintCatalogService catalog;

    @Autowired
    private RecipeRepository recipes;

    @Autowired
    private RecipeTimelineService timelineService;

    @Autowired
    private ColorMixService mixer;

    @Test
    @DisplayName("le catalogue de depart est charge au demarrage")
    void catalogIsSeeded() {
        List<OilPaint> paints = catalog.findAll();

        assertThat(paints).hasSizeGreaterThan(20);
        assertThat(paints).allSatisfy(paint -> {
            assertThat(paint.getHexColor()).matches("#[0-9A-F]{6}");
            assertThat(paint.getPigments()).isNotEmpty();
            assertThat(paint.getTintingStrength()).isBetween(0.05, 1.0);
        });
    }

    @Test
    @DisplayName("la recherche textuelle couvre la marque, le nom et la reference")
    void searchLooksAtEveryField() {
        assertThat(catalog.search("umber")).isNotEmpty();
        assertThat(catalog.search("Abteilung")).isNotEmpty();
        assertThat(catalog.search("644")).isNotEmpty();
        assertThat(catalog.search("xyzzy")).isEmpty();
    }

    @Test
    @DisplayName("la recherche par couleur trouve le tube exact en tete")
    void closestMatchFindsTheExactPaint() {
        OilPaint reference = catalog.search("Burnt Umber").getFirst();

        List<PaintMatch> matches = catalog.findClosest(reference.color(), false, 5);

        assertThat(matches).hasSize(5);
        assertThat(matches.getFirst().paint().getName()).isEqualTo(reference.getName());
        assertThat(matches.getFirst().deltaE()).isZero();
    }

    @Test
    @DisplayName("une recette d'exemple se deroule sur plusieurs jours d'attente")
    void seededRecipeSpansSeveralDays() {
        var recipe = recipes.findAllByOrderByNameAsc().getFirst();

        var timeline = timelineService.plan(recipe, DryingClass.MEDIUM, Workshop.standard());

        assertThat(timeline.entries()).hasSameSizeAs(recipe.getSteps());
        assertThat(timeline.entries().getFirst().startOffset()).isZero();
        assertThat(timeline.entries().getLast().waitAfter()).isZero();
        assertThat(timeline.totalActiveSpan()).isPositive();
        assertThat(timeline.untilVarnish()).isGreaterThan(timeline.totalActiveSpan());
    }

    @Test
    @DisplayName("une recette est proposee a partir des seuls tubes en stock")
    void suggestionsUseTheShelf() {
        var suggestions = mixer.suggestMixes(Rgb.ofHex("#6B5A42"), catalog.findInStock(), 3);

        assertThat(suggestions).hasSize(3);
        assertThat(suggestions.getFirst().deltaE()).isLessThan(10.0);
        assertThat(suggestions.getFirst().parts())
                .allSatisfy(part -> assertThat(part.paint().isInStock()).isTrue());
    }
}
