package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.catalog.CatalogLoader;
import be.asmolabs.palettier.core.catalog.PigmentIndex;
import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/** Qualite et cout de la recherche de melange. */
class MixSearchTest {

    private final ColorMixService service = new ColorMixService();
    private final List<OilPaint> catalog = new CatalogLoader(new PigmentIndex()).load();

    /** Un echantillon de teintes reelles : carnations, terrains, blindages, tissus. */
    private static final List<String> TARGETS = List.of(
            "#C98F72", "#8A4A3C", "#E0B49A", "#6B5A42", "#4A5560",
            "#9A8F80", "#3E4A3A", "#B29B62", "#7A3A22", "#2C3E50");

    @Test
    @DisplayName("une couleur du catalogue est retrouvee a l'identique par un seul tube")
    void anExactCatalogColourIsFoundAlone() {
        OilPaint reference = catalog.stream()
                .filter(paint -> paint.getName().equals("Burnt Umber"))
                .findFirst()
                .orElseThrow();

        MixSuggestion best = service.suggestMixes(reference.color(), catalog, 1).getFirst();

        assertThat(best.deltaE()).isLessThan(0.5);
        assertThat(best.parts()).hasSize(1);
    }

    @Test
    @DisplayName("chaque proposition annonce l'ecart du dosage propose, pas celui d'un optimum theorique")
    void announcedDeltaMatchesTheProposedDosage() {
        for (String hex : TARGETS) {
            Rgb target = Rgb.ofHex(hex);
            for (MixSuggestion suggestion : service.suggestMixes(target, catalog, 5)) {
                Rgb actual = service.mix(suggestion.parts()).color();

                assertThat(Colors.deltaE2000(target, actual))
                        .as("%s : ecart annonce pour %s", hex, suggestion.describe())
                        .isCloseTo(suggestion.deltaE(), org.assertj.core.data.Offset.offset(0.01));
            }
        }
    }

    @Test
    @DisplayName("les dosages proposes sont des rapports d'entiers simples")
    void dosagesArePractical() {
        for (String hex : TARGETS) {
            for (MixSuggestion suggestion : service.suggestMixes(Rgb.ofHex(hex), catalog, 5)) {
                assertThat(suggestion.parts()).allSatisfy(part -> {
                    assertThat(part.parts()).isEqualTo(Math.rint(part.parts()));
                    assertThat(part.parts()).isBetween(1.0, 30.0);
                });
            }
        }
    }

    @Test
    @DisplayName("aucun melange ne depasse trois tubes")
    void neverMoreThanThreePaints() {
        for (String hex : TARGETS) {
            assertThat(service.suggestMixes(Rgb.ofHex(hex), catalog, 8))
                    .allSatisfy(suggestion -> assertThat(suggestion.parts()).hasSizeLessThanOrEqualTo(3));
        }
    }

    @Test
    @DisplayName("la liste propose des combinaisons de tubes differentes, pas des variantes de dosage")
    void resultsAreDistinctCombinations() {
        List<MixSuggestion> suggestions = service.suggestMixes(Rgb.ofHex("#C98F72"), catalog, 8);

        List<String> combinations = suggestions.stream()
                .map(suggestion -> suggestion.parts().stream()
                        .map(part -> part.paint().displayName())
                        .sorted()
                        .toList()
                        .toString())
                .toList();

        assertThat(combinations).doesNotHaveDuplicates();
    }

    @Test
    @DisplayName("autoriser un troisieme tube ne degrade jamais le resultat")
    void threePaintsNeverHurt() {
        for (String hex : TARGETS) {
            Rgb target = Rgb.ofHex(hex);
            double withTwo = service.suggestMixes(target, catalog, 1, 2).getFirst().deltaE();
            double withThree = service.suggestMixes(target, catalog, 1, 3).getFirst().deltaE();

            assertThat(withThree).as("cible %s", hex).isLessThanOrEqualTo(withTwo + 1e-9);
        }
    }

    @Test
    @DisplayName("les propositions sont classees par palier perceptuel, pas au centieme d'ecart")
    void resultsAreSortedByPerceptualTier() {
        List<MixSuggestion> suggestions = service.suggestMixes(Rgb.ofHex("#6B5A42"), catalog, 8);

        // A l'interieur d'un palier de 0,5 l'oeil ne fait pas la difference : l'ordre y est
        // decide par la simplicite du melange, pas par la troisieme decimale de l'ecart.
        List<Integer> tiers = suggestions.stream().map(s -> (int) (s.deltaE() / 0.5)).toList();

        assertThat(tiers).isSorted();
    }

    @Test
    @DisplayName("a ecart imperceptible, le melange le plus simple passe devant")
    void simplerMixesWinTies() {
        // Cette teinte est celle d'un tube du catalogue : un seul tube doit gagner, meme si
        // des melanges a trois tubes atteignent un ecart legerement inferieur.
        OilPaint reference = catalog.stream()
                .filter(paint -> paint.getName().equals("Yellow Ochre"))
                .findFirst()
                .orElseThrow();

        MixSuggestion best = service.suggestMixes(reference.color(), catalog, 5).getFirst();

        assertThat(best.parts()).hasSize(1);
        assertThat(best.deltaE()).isLessThan(0.5);
    }

    @Test
    @DisplayName("cout de la recherche sur le catalogue complet")
    void searchCostIsAcceptable() {
        long start = System.nanoTime();
        double worst = 0;
        for (String hex : TARGETS) {
            worst = Math.max(worst, service.suggestMixes(Rgb.ofHex(hex), catalog, 8).getFirst().deltaE());
        }
        long millis = (System.nanoTime() - start) / 1_000_000 / TARGETS.size();

        System.out.printf("recherche : %d tubes, %d ms par cible, pire ecart %.2f%n",
                catalog.size(), millis, worst);
        assertThat(millis).isLessThan(4_000);
    }
}
