package be.asmolabs.palettier.ui;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.geom.RoundRectangle2D;
import java.awt.image.BufferedImage;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import javax.imageio.ImageIO;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Fabrique l'icone de l'application.
 *
 * <p>Trois taches de peinture qui se recouvrent, sur le fond sombre de l'atelier. Les
 * zones de recouvrement ne sont pas une transparence : ce sont les melanges reels,
 * calcules par {@link Colors#mix} comme n'importe quel melange de l'application. Bleu et
 * jaune y donnent du vert, pas du gris — l'icone dit donc exactement ce que fait le
 * logiciel.</p>
 *
 * <p>Desactive par defaut : on ne regenere l'icone que lorsqu'on la change. Retirer
 * l'annotation {@code @Disabled} et lancer ce seul test suffit a reecrire les fichiers.</p>
 */
@Disabled("Generation d'image : a lancer a la main quand l'icone change")
class AppIconGenerator {

    private static final Path OUTPUT = Path.of("src/main/resources/be/asmolabs/palettier/ui/icon");
    private static final int[] SIZES = {16, 32, 64, 128, 256, 512, 1024};

    /** Quatre echantillons par pixel et par axe : de quoi lisser les bords sans filtre. */
    private static final int SUPERSAMPLING = 4;

    /** Fond de l'atelier, repris de la feuille de style. */
    private static final Color BACKGROUND = new Color(0x1D1917);
    private static final Color BORDER = new Color(255, 255, 255, 46);

    /**
     * Trois huiles du catalogue, choisies pour que leurs melanges soient lisibles :
     * ocre et bleu donnent un vert franc, ocre et rouge un orange, les trois un brun
     * rompu. Ce sont les couleurs reelles de ces tubes.
     */
    private record Blob(Rgb color, double x, double y) {
    }

    private static final List<Blob> BLOBS = List.of(
            new Blob(Rgb.ofHex("#E0A030"), 0.50, 0.315),   // ocre jaune
            new Blob(Rgb.ofHex("#C8321E"), 0.323, 0.623),  // rouge de cadmium clair
            new Blob(Rgb.ofHex("#2257A8"), 0.677, 0.623)); // bleu de cobalt

    /**
     * Rayon juste inferieur au rayon du triangle des centres : les taches se recouvrent
     * deux a deux, mais jamais toutes les trois. Un recouvrement triple donnerait au
     * centre le gris boueux que trois huiles melangees produisent reellement -- exact,
     * mais illisible a seize pixels.
     */
    private static final double BLOB_RADIUS = 0.202;

    @Test
    @DisplayName("ecrit l'icone dans toutes les tailles utiles")
    void generate() throws IOException {
        Files.createDirectories(OUTPUT);
        for (int size : SIZES) {
            BufferedImage icon = render(size);
            Path file = OUTPUT.resolve("palettier-%d.png".formatted(size));
            ImageIO.write(icon, "png", file.toFile());
            System.out.println("ecrit " + file.toAbsolutePath());
        }
    }

    private static BufferedImage render(int size) {
        int large = size * SUPERSAMPLING;
        BufferedImage canvas = new BufferedImage(large, large, BufferedImage.TYPE_INT_ARGB);
        Graphics2D graphics = canvas.createGraphics();
        graphics.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

        // Le fond arrondi, dans la forme des icones systeme.
        graphics.setColor(BACKGROUND);
        double radius = large * 0.22;
        graphics.fill(new RoundRectangle2D.Double(0, 0, large, large, radius, radius));

        paintBlobs(canvas, large);

        graphics.setColor(BORDER);
        graphics.setStroke(new java.awt.BasicStroke(Math.max(1f, large / 256f)));
        graphics.draw(new RoundRectangle2D.Double(0.5, 0.5, large - 1.0, large - 1.0, radius, radius));
        graphics.dispose();

        return downscale(canvas, size);
    }

    /**
     * Chaque pixel prend la couleur du melange des taches qui le couvrent. On ne dessine
     * donc pas trois disques par-dessus un fond : on calcule, point par point, ce que
     * donnerait la superposition de ces peintures.
     */
    private static void paintBlobs(BufferedImage canvas, int large) {
        double radius = BLOB_RADIUS * large;
        for (int y = 0; y < large; y++) {
            for (int x = 0; x < large; x++) {
                List<Rgb> covering = new ArrayList<>(3);
                for (Blob blob : BLOBS) {
                    double dx = x - blob.x() * large;
                    double dy = y - blob.y() * large;
                    if (dx * dx + dy * dy <= radius * radius) {
                        covering.add(blob.color());
                    }
                }
                if (covering.isEmpty()) {
                    continue;
                }
                Rgb mixed = covering.size() == 1
                        ? covering.getFirst()
                        : Colors.mix(covering, covering.stream().map(c -> 1.0).toList());
                canvas.setRGB(x, y, 0xFF000000 | toPacked(mixed));
            }
        }
    }

    private static int toPacked(Rgb color) {
        int r = (int) Math.round(color.r() * 255);
        int g = (int) Math.round(color.g() * 255);
        int b = (int) Math.round(color.b() * 255);
        return (r << 16) | (g << 8) | b;
    }

    private static BufferedImage downscale(BufferedImage source, int size) {
        BufferedImage target = new BufferedImage(size, size, BufferedImage.TYPE_INT_ARGB);
        Graphics2D graphics = target.createGraphics();
        graphics.setRenderingHint(RenderingHints.KEY_INTERPOLATION,
                RenderingHints.VALUE_INTERPOLATION_BILINEAR);
        graphics.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);
        graphics.drawImage(source, 0, 0, size, size, null);
        graphics.dispose();
        return target;
    }
}
