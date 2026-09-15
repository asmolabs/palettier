package be.asmolabs.palettier.core.color;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatIllegalArgumentException;

import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

class ColorsTest {

    @Test
    @DisplayName("un aller-retour hexadecimal conserve la couleur")
    void hexRoundTrip() {
        assertThat(Rgb.ofHex("#4A3427").toHex()).isEqualTo("#4A3427");
        assertThat(Rgb.ofHex("4a3427").toHex()).isEqualTo("#4A3427");
        assertThat(Rgb.ofHex("#FFF").toHex()).isEqualTo("#FFFFFF");
    }

    @Test
    @DisplayName("une couleur ne presente aucun ecart avec elle-meme")
    void deltaEIsZeroForIdenticalColors() {
        Rgb color = Rgb.ofHex("#7A3A22");
        assertThat(Colors.deltaE2000(color, color)).isZero();
    }

    @Test
    @DisplayName("le blanc et le noir sont a l'oppose l'un de l'autre")
    void deltaEIsLargeBetweenBlackAndWhite() {
        assertThat(Colors.deltaE2000(Rgb.ofHex("#000000"), Rgb.ofHex("#FFFFFF"))).isGreaterThan(90);
    }

    @Test
    @DisplayName("bleu et jaune donnent du vert, pas du gris")
    void subtractiveMixProducesGreen() {
        Rgb blue = Rgb.ofHex("#1F2E7A");
        Rgb yellow = Rgb.ofHex("#F6C500");

        Rgb mixed = Colors.mix(List.of(blue, yellow), List.of(1.0, 1.0));

        assertThat(mixed.g())
                .as("la composante verte doit dominer dans %s", mixed.toHex())
                .isGreaterThan(mixed.r())
                .isGreaterThan(mixed.b());
    }

    @Test
    @DisplayName("melanger une couleur avec elle-meme ne la change pas")
    void mixingIdenticalColorsIsStable() {
        Rgb umber = Rgb.ofHex("#4A3427");

        Rgb mixed = Colors.mix(List.of(umber, umber), List.of(3.0, 1.0));

        assertThat(Colors.deltaE2000(umber, mixed)).isLessThan(1.0);
    }

    @Test
    @DisplayName("un poids ecrasant impose sa couleur au melange")
    void dominantWeightDrivesTheMix() {
        Rgb white = Rgb.ofHex("#F7F5F0");
        Rgb black = Rgb.ofHex("#1C1B1A");

        Rgb mostlyWhite = Colors.mix(List.of(white, black), List.of(99.0, 1.0));
        Rgb halfAndHalf = Colors.mix(List.of(white, black), List.of(1.0, 1.0));

        assertThat(mostlyWhite.relativeLuminance()).isGreaterThan(halfAndHalf.relativeLuminance());
        assertThat(halfAndHalf.relativeLuminance()).isLessThan(white.relativeLuminance());
    }

    @Test
    @DisplayName("un melange sans matiere est refuse")
    void mixRejectsEmptyInput() {
        assertThatIllegalArgumentException()
                .isThrownBy(() -> Colors.mix(List.of(), List.of()));
        assertThatIllegalArgumentException()
                .isThrownBy(() -> Colors.mix(List.of(Rgb.ofHex("#000000")), List.of(0.0)));
    }

    @Test
    @DisplayName("la moyenne d'un echantillon se fait en lumiere lineaire, pas sur le sRGB")
    void averageIsComputedInLinearLight() {
        Rgb black = Rgb.ofHex("#000000");
        Rgb white = Rgb.ofHex("#FFFFFF");

        Rgb mid = Colors.average(List.of(black, white));

        // La moyenne naive donnerait #808080 (50 % de 255). En lumiere lineaire, le gris
        // a mi-luminance remonte vers #BC.
        assertThat(mid.toHex()).isEqualTo("#BCBCBC");
    }

    @Test
    @DisplayName("moyenner une couleur avec elle-meme la laisse inchangee")
    void averageOfIdenticalSamplesIsStable() {
        Rgb flesh = Rgb.ofHex("#C98F72");

        assertThat(Colors.average(List.of(flesh, flesh, flesh)).toHex()).isEqualTo(flesh.toHex());
    }

    @Test
    @DisplayName("un echantillon vide est refuse")
    void averageRejectsEmptySample() {
        assertThatIllegalArgumentException().isThrownBy(() -> Colors.average(List.of()));
    }

    @Test
    @DisplayName("la correction de dominante rend neutre le point choisi comme gris")
    void neutralisingTheGreyPointMakesItNeutral() {
        Rgb warmGrey = Rgb.ofHex("#A08860");

        Rgb corrected = Colors.neutralise(warmGrey, warmGrey);

        assertThat(corrected.r()).isCloseTo(corrected.g(), org.assertj.core.data.Offset.offset(0.01));
        assertThat(corrected.g()).isCloseTo(corrected.b(), org.assertj.core.data.Offset.offset(0.01));
    }

    @Test
    @DisplayName("la correction retire la dominante jaune d'une photo sous lampe chaude")
    void neutralisingRemovesAWarmCast() {
        // Une carnation photographiee sous lampe chaude, et le blanc du socle sur la meme photo.
        Rgb photographedFlesh = Rgb.ofHex("#D9A070");
        Rgb photographedWhite = Rgb.ofHex("#E8D2A8");

        Rgb corrected = Colors.neutralise(photographedFlesh, photographedWhite);

        double warmthBefore = photographedFlesh.r() - photographedFlesh.b();
        double warmthAfter = corrected.r() - corrected.b();
        assertThat(warmthAfter).isLessThan(warmthBefore);
    }

    @Test
    @DisplayName("un point de reference deja neutre ne change rien")
    void neutralisingWithANeutralReferenceIsAnIdentity() {
        Rgb flesh = Rgb.ofHex("#C98F72");

        Rgb corrected = Colors.neutralise(flesh, Rgb.ofHex("#808080"));

        assertThat(Colors.deltaE2000(flesh, corrected)).isLessThan(0.5);
    }
}
