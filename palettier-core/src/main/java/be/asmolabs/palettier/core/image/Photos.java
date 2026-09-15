package be.asmolabs.palettier.core.image;

import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import javax.imageio.ImageIO;

/**
 * Preparation des photos avant envoi.
 *
 * <p>Une photo de telephone pese dix megaoctets et mesure quatre mille pixels de large.
 * Envoyee telle quelle a un modele, elle sature son contexte et allonge l'attente de
 * plusieurs minutes ; conservee telle quelle dans un projet, elle fait enfler la base
 * sans rien apporter. Aucun des deux usages n'a besoin de cette resolution pour
 * distinguer une cape d'un plastron.</p>
 */
public final class Photos {

    /** Cote le plus long apres reduction. Au-dela, on paie sans rien gagner. */
    public static final int MAX_EDGE = 1024;

    /** Type de contenu des images produites. */
    public static final String MIME_TYPE = "image/jpeg";

    private Photos() {
    }

    /**
     * Reduit une image et la reencode en JPEG.
     *
     * @throws IllegalArgumentException si le contenu n'est pas une image lisible
     */
    public static byte[] prepare(byte[] original) {
        try {
            BufferedImage source = ImageIO.read(new ByteArrayInputStream(original));
            if (source == null) {
                throw new IllegalArgumentException("Format d'image non reconnu");
            }

            int width = source.getWidth();
            int height = source.getHeight();
            double scale = Math.min(1.0, (double) MAX_EDGE / Math.max(width, height));
            int targetWidth = Math.max(1, (int) Math.round(width * scale));
            int targetHeight = Math.max(1, (int) Math.round(height * scale));

            // TYPE_INT_RGB : le JPEG ne porte pas de transparence, autant l'aplatir ici.
            BufferedImage reduced = new BufferedImage(targetWidth, targetHeight, BufferedImage.TYPE_INT_RGB);
            Graphics2D graphics = reduced.createGraphics();
            graphics.setRenderingHint(RenderingHints.KEY_INTERPOLATION,
                    RenderingHints.VALUE_INTERPOLATION_BILINEAR);
            graphics.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);
            graphics.drawImage(source, 0, 0, targetWidth, targetHeight, null);
            graphics.dispose();

            ByteArrayOutputStream out = new ByteArrayOutputStream();
            ImageIO.write(reduced, "jpg", out);
            return out.toByteArray();
        } catch (IOException e) {
            throw new UncheckedIOException("Lecture de l'image impossible", e);
        }
    }
}
