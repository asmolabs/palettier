package be.asmolabs.palettier.core.color;

import static org.assertj.core.api.Assertions.assertThat;

import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/** Le modele de Kubelka-Munk a deux constantes. */
class ColorantTest {

    private static final double TINT = Colorant.REFERENCE_TINT_CONCENTRATION;

    @Test
    @DisplayName("un colorant ajuste sur deux couleurs reproduit ces deux couleurs")
    void theFitReproducesBothObservations() {
        // Noir d'ivoire : ton de masse chaud, teinte diluee froide. C'est le cas que le
        // modele a constante unique ne savait pas representer.
        Rgb masstone = Rgb.ofHex("#221F1C");
        Rgb tint = Rgb.ofHex("#8E9094");

        Colorant black = Colorant.ofMasstoneAndTint(masstone, tint, TINT);

        assertThat(Colors.deltaE2000(masstone, black.masstone())).isLessThan(0.5);
        assertThat(Colors.deltaE2000(tint, black.tint(TINT))).isLessThan(0.5);
    }

    @Test
    @DisplayName("le ton de masse reste chaud alors que la teinte diluee devient froide")
    void masstoneAndTintCanDiverge() {
        Colorant black = Colorant.ofMasstoneAndTint(Rgb.ofHex("#221F1C"), Rgb.ofHex("#8E9094"), TINT);

        Rgb masstone = black.masstone();
        Rgb tint = black.tint(TINT);

        assertThat(masstone.r()).as("ton de masse chaud : %s", masstone.toHex()).isGreaterThan(masstone.b());
        assertThat(tint.b()).as("teinte diluee froide : %s", tint.toHex()).isGreaterThan(tint.r());
    }

    @Test
    @DisplayName("sans teinte diluee, le modele se ramene exactement au modele a constante unique")
    void withoutATintTheModelDegradesToTheSingleConstantOne() {
        List<Rgb> colors = List.of(Rgb.ofHex("#1F2E7A"), Rgb.ofHex("#F6C500"));
        List<Double> weights = List.of(1.0, 1.0);

        Rgb singleConstant = Colors.mix(colors, weights);
        Rgb twoConstant = Colorant.mix(colors.stream().map(Colorant::ofMasstone).toList(), weights);

        assertThat(twoConstant.toHex()).isEqualTo(singleConstant.toHex());
    }

    @Test
    @DisplayName("un colorant melange a lui-meme ne change pas")
    void mixingAColorantWithItselfIsStable() {
        Colorant umber = Colorant.ofMasstoneAndTint(Rgb.ofHex("#4A3427"), Rgb.ofHex("#C9B9AC"), TINT);

        Rgb mixed = Colorant.mix(List.of(umber, umber), List.of(3.0, 1.0));

        assertThat(Colors.deltaE2000(umber.masstone(), mixed)).isLessThan(0.5);
    }

    @Test
    @DisplayName("une teinte diluee incoherente ne produit pas de constantes aberrantes")
    void anInconsistentTintFallsBackInsteadOfBreaking() {
        // Teinte annoncee plus sombre que le ton de masse : impossible avec du blanc.
        Colorant colorant = Colorant.ofMasstoneAndTint(Rgb.ofHex("#C08A34"), Rgb.ofHex("#201810"), TINT);

        assertThat(colorant.masstone().toHex()).isEqualTo("#C08A34");
        assertThat(Colors.deltaE2000(colorant.tint(TINT), Colorant.REFERENCE_WHITE))
                .as("la teinte reste plus claire que le ton de masse")
                .isLessThan(Colors.deltaE2000(Rgb.ofHex("#C08A34"), Colorant.REFERENCE_WHITE));
    }

    @Test
    @DisplayName("un blanc tres diffusant impose sa loi des qu'il est present")
    void aHighlyScatteringWhiteDominates() {
        Colorant white = Colorant.ofMasstone(Colorant.REFERENCE_WHITE);
        Colorant black = Colorant.ofMasstoneAndTint(Rgb.ofHex("#221F1C"), Rgb.ofHex("#8E9094"), TINT);

        Rgb aLittleBlack = Colorant.mix(List.of(white, black), List.of(19.0, 1.0));
        Rgb moreBlack = Colorant.mix(List.of(white, black), List.of(4.0, 1.0));

        assertThat(aLittleBlack.relativeLuminance()).isGreaterThan(moreBlack.relativeLuminance());
        assertThat(moreBlack.relativeLuminance()).isLessThan(Colorant.REFERENCE_WHITE.relativeLuminance());
    }
}
