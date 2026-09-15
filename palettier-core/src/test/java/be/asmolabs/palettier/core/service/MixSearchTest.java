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
    @DisplayName("le nombre de tubes demande est respecte")
    void theRequestedLimitIsHonoured() {
        for (String hex : TARGETS) {
            for (int max = 1; max <= 5; max++) {
                int limit = max;
                assertThat(service.suggestMixes(Rgb.ofHex(hex), catalog, 8, max))
                        .allSatisfy(s -> assertThat(s.parts()).hasSizeLessThanOrEqualTo(limit));
            }
        }
    }

    @Test
    @DisplayName("une palette courte tire profit des tubes supplementaires")
    void ashortPaletteBenefitsFromMorePaints() {
        // Trois primaires, un blanc et une terre : le cas ou trois tubes ne suffisent pas.
        List<OilPaint> shortPalette = List.of(
                find("Cadmium Yellow Pale"), find("Permanent Rose"),
                find("Winsor Blue (Green Shade)"), find("Titanium White"), find("Burnt Umber"));

        double withThree = 0;
        double withFour = 0;
        for (String hex : TARGETS) {
            withThree += service.suggestMixes(Rgb.ofHex(hex), shortPalette, 1, 3).getFirst().deltaE();
            withFour += service.suggestMixes(Rgb.ofHex(hex), shortPalette, 1, 4).getFirst().deltaE();
        }

        assertThat(withFour)
                .as("un quatrieme tube doit rapprocher de la cible : %.2f puis %.2f", withThree, withFour)
                .isLessThan(withThree);
    }

    private OilPaint find(String name) {
        return catalog.stream()
                .filter(p -> p.getBrand().equals("Winsor & Newton") && p.getName().equals(name))
                .findFirst()
                .orElseThrow();
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
    @DisplayName("autoriser un tube de plus ne degrade jamais le resultat")
    void moreePaintsNeverHurt() {
        for (String hex : TARGETS) {
            Rgb target = Rgb.ofHex(hex);
            double previous = Double.MAX_VALUE;
            for (int max = 1; max <= 5; max++) {
                double current = service.suggestMixes(target, catalog, 1, max).getFirst().deltaE();
                assertThat(current).as("cible %s avec %d tubes", hex, max)
                        .isLessThanOrEqualTo(previous + 1e-9);
                previous = current;
            }
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

    @Test
    @DisplayName("le nombre de tubes s'adapte a ce qui est disponible")
    void theLimitAdaptsToTheSelection() {
        // Une palette courte merite tous les tubes : c'est la seule facon d'y arriver.
        assertThat(ColorMixService.recommendedMaxPaints(4)).isEqualTo(5);
        assertThat(ColorMixService.recommendedMaxPaints(8)).isEqualTo(5);

        // Une selection moyenne se contente de quatre.
        assertThat(ColorMixService.recommendedMaxPaints(9)).isEqualTo(4);
        assertThat(ColorMixService.recommendedMaxPaints(24)).isEqualTo(4);

        // Le catalogue entier n'a rien a gagner au-dela de trois.
        assertThat(ColorMixService.recommendedMaxPaints(25)).isEqualTo(3);
        assertThat(ColorMixService.recommendedMaxPaints(439)).isEqualTo(3);
    }

    @Test
    @DisplayName("sans consigne, une palette courte recoit des melanges plus riches que le catalogue")
    void aShortPaletteGetsRicherMixesByDefault() {
        List<OilPaint> shortPalette = List.of(
                find("Cadmium Yellow Pale"), find("Permanent Rose"),
                find("Winsor Blue (Green Shade)"), find("Titanium White"), find("Burnt Umber"));

        int onPalette = service.suggestMixes(Rgb.ofHex("#6B5A42"), shortPalette, 1)
                .getFirst().parts().size();
        int onCatalogue = service.suggestMixes(Rgb.ofHex("#6B5A42"), catalog, 1)
                .getFirst().parts().size();

        assertThat(onPalette).isGreaterThan(onCatalogue);
        assertThat(onCatalogue).isLessThanOrEqualTo(3);
    }
}
