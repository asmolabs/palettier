package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.service.MixModels.MixResult;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

class ColorMixServiceTest {

    private final ColorMixService service = new ColorMixService();

    private static final OilPaint TITANIUM_WHITE = new OilPaint("W&N", "Titanium White", "644", "#F7F5F0",
            Opacity.OPAQUE, DryingClass.SLOW, 1.0, Set.of("PW6"));
    private static final OilPaint BURNT_UMBER = new OilPaint("W&N", "Burnt Umber", "076", "#4A3427",
            Opacity.SEMI_OPAQUE, DryingClass.FAST, 0.8, Set.of("PBr7"));
    private static final OilPaint PRUSSIAN_BLUE = new OilPaint("W&N", "Prussian Blue", "538", "#16323C",
            Opacity.TRANSPARENT, DryingClass.FAST, 0.98, Set.of("PB27"));
    private static final OilPaint CADMIUM_YELLOW = new OilPaint("W&N", "Cadmium Yellow", "118", "#F6C500",
            Opacity.OPAQUE, DryingClass.VERY_SLOW, 0.8, Set.of("PY35"));

    @Test
    @DisplayName("les parts en volume et en pigment se repartissent chacune sur cent pour cent")
    void sharesAddUpToOne() {
        MixResult result = service.mix(List.of(
                PaintPart.of(TITANIUM_WHITE, 3),
                PaintPart.of(BURNT_UMBER, 1)));

        assertThat(result.components()).hasSize(2);
        assertThat(result.components().stream().mapToDouble(c -> c.volumeShare()).sum()).isCloseTo(1.0, org.assertj.core.data.Offset.offset(1e-9));
        assertThat(result.components().stream().mapToDouble(c -> c.pigmentShare()).sum()).isCloseTo(1.0, org.assertj.core.data.Offset.offset(1e-9));
    }

    @Test
    @DisplayName("le pigment le plus lent impose le sechage du melange")
    void slowestPigmentDrivesDrying() {
        MixResult result = service.mix(List.of(
                PaintPart.of(BURNT_UMBER, 5),
                PaintPart.of(CADMIUM_YELLOW, 1)));

        assertThat(result.dryingClass()).isEqualTo(DryingClass.VERY_SLOW);
        assertThat(result.pigments()).containsExactlyInAnyOrder("PBr7", "PY35");
    }

    @Test
    @DisplayName("un pigment tres colorant qui pese peu en volume est signale")
    void strongTinterIsFlagged() {
        MixResult result = service.mix(List.of(
                PaintPart.of(TITANIUM_WHITE, 20),
                PaintPart.of(PRUSSIAN_BLUE, 1)));

        assertThat(result.warnings()).isNotEmpty();
    }

    @Test
    @DisplayName("un melange de quatre huiles avertit du risque de gris")
    void muddyMixIsFlagged() {
        MixResult result = service.mix(List.of(
                PaintPart.of(TITANIUM_WHITE, 1),
                PaintPart.of(BURNT_UMBER, 1),
                PaintPart.of(PRUSSIAN_BLUE, 1),
                PaintPart.of(CADMIUM_YELLOW, 1)));

        assertThat(result.warnings()).anySatisfy(w -> assertThat(w).contains("gris"));
    }

    @Test
    @DisplayName("la recherche de recette retrouve une couleur issue du catalogue")
    void suggestionsFindAnExactCatalogColour() {
        List<MixSuggestion> suggestions = service.suggestMixes(
                BURNT_UMBER.color(),
                List.of(TITANIUM_WHITE, BURNT_UMBER, PRUSSIAN_BLUE, CADMIUM_YELLOW),
                5);

        assertThat(suggestions).isNotEmpty();
        assertThat(suggestions.getFirst().deltaE()).isLessThan(1.0);
        assertThat(suggestions).isSortedAccordingTo((a, b) -> Double.compare(a.deltaE(), b.deltaE()));
    }

    @Test
    @DisplayName("la recherche de recette approche une teinte absente du catalogue")
    void suggestionsApproximateAnUnlistedColour() {
        Rgb target = Rgb.ofHex("#9A8F80");

        MixSuggestion best = service.suggestMixes(
                target, List.of(TITANIUM_WHITE, BURNT_UMBER, PRUSSIAN_BLUE, CADMIUM_YELLOW), 1).getFirst();

        assertThat(best.deltaE()).isLessThan(Colors.deltaE2000(target, BURNT_UMBER.color()));
        assertThat(best.describe()).contains("part");
    }
}
