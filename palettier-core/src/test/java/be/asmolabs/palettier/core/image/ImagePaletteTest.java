package be.asmolabs.palettier.core.image;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.image.BufferedImage;
import java.io.ByteArrayOutputStream;
import java.util.List;
import javax.imageio.ImageIO;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

class ImagePaletteTest {

    /** Une image de bandes de proportions connues. */
    private static byte[] stripes(List<Color> colours) throws Exception {
        BufferedImage image = new BufferedImage(300, 300, BufferedImage.TYPE_INT_RGB);
        Graphics2D g = image.createGraphics();
        int height = 300 / colours.size();
        for (int i = 0; i < colours.size(); i++) {
            g.setColor(colours.get(i));
            g.fillRect(0, i * height, 300, height);
        }
        g.dispose();
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        ImageIO.write(image, "png", out);
        return out.toByteArray();
    }

    @Test
    @DisplayName("les teintes retrouvees sont celles qui sont reellement sur l'image")
    void findsTheColoursActuallyPresent() throws Exception {
        byte[] image = stripes(List.of(
                new Color(0xC9, 0x8F, 0x72),
                new Color(0x5A, 0x3B, 0x2E),
                new Color(0x2C, 0x3E, 0x50)));

        List<ImagePalette.DominantColour> found = ImagePalette.dominant(image, 3);

        assertThat(found).hasSize(3);
        for (String expected : List.of("#C98F72", "#5A3B2E", "#2C3E50")) {
            assertThat(found).anySatisfy(dominant ->
                    assertThat(Colors.deltaE2000(dominant.color(), Rgb.ofHex(expected)))
                            .as("teinte %s retrouvee", expected)
                            .isLessThan(3.0));
        }
    }

    @Test
    @DisplayName("les parts refletent la surface occupee")
    void sharesReflectTheArea() throws Exception {
        byte[] image = stripes(List.of(
                new Color(0xFF, 0x00, 0x00), new Color(0xFF, 0x00, 0x00),
                new Color(0xFF, 0x00, 0x00), new Color(0x00, 0x00, 0xFF)));

        List<ImagePalette.DominantColour> found = ImagePalette.dominant(image, 2);

        assertThat(found.getFirst().share()).isBetween(0.6, 0.85);
        assertThat(found.stream().mapToDouble(ImagePalette.DominantColour::share).sum())
                .isCloseTo(1.0, org.assertj.core.data.Offset.offset(0.01));
    }

    @Test
    @DisplayName("la meme photo donne toujours le meme resultat")
    void theSameImageAlwaysGivesTheSameResult() throws Exception {
        byte[] image = stripes(List.of(
                new Color(0xC9, 0x8F, 0x72), new Color(0x8A, 0x5F, 0x4A),
                new Color(0xE0, 0xB4, 0x9A), new Color(0x5A, 0x3B, 0x2E)));

        // Un plan ne doit pas changer parce qu'on relance la recherche.
        assertThat(ImagePalette.dominant(image, 4)).isEqualTo(ImagePalette.dominant(image, 4));
    }

    @Test
    @DisplayName("les teintes sont classees de la plus presente a la moins presente")
    void sortedByPresence() throws Exception {
        byte[] image = stripes(List.of(
                new Color(0xC9, 0x8F, 0x72), new Color(0xC9, 0x8F, 0x72),
                new Color(0xC9, 0x8F, 0x72), new Color(0x2C, 0x3E, 0x50)));

        assertThat(ImagePalette.dominant(image, 2))
                .isSortedAccordingTo((a, b) -> Double.compare(b.share(), a.share()));
    }
}
