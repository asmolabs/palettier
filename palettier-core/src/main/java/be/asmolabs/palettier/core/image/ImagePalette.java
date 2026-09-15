package be.asmolabs.palettier.core.image;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Lab;
import be.asmolabs.palettier.core.color.Rgb;
import java.awt.image.BufferedImage;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import javax.imageio.ImageIO;

/**
 * Les teintes reellement presentes sur une photo.
 *
 * <p>Sert a fonder un plan sur ce qu'on voit plutot que sur ce qu'on imagine. Un modele
 * de langage a qui l'on montre une photo de carnation propose volontiers des couleurs
 * de manuel ; lui donner les teintes mesurees l'oblige a travailler sur la piece qu'il
 * a sous les yeux.</p>
 *
 * <p>Le regroupement se fait par distance perceptuelle en L*a*b*, parce qu'un ecart de
 * dix unites y signifie la meme chose dans les ombres et dans les clairs, ce qui n'est
 * pas le cas en RVB. Les representants, eux, restent des moyennes en lumiere lineaire :
 * moyenner du sRGB assombrit.</p>
 */
public final class ImagePalette {

    /** Cote maximal avant analyse : la teinte dominante ne demande pas de resolution. */
    private static final int ANALYSIS_EDGE = 160;

    /** Nombre d'iterations : au-dela, les groupes ne bougent plus. */
    private static final int ITERATIONS = 12;

    private ImagePalette() {
    }

    /**
     * Une teinte dominante.
     *
     * @param share part de l'image occupee, de 0 a 1
     */
    public record DominantColour(Rgb color, double share) {
    }

    /**
     * Extrait les teintes dominantes d'une image.
     *
     * @param count nombre de teintes souhaitees
     * @return les teintes, de la plus presente a la moins presente
     */
    public static List<DominantColour> dominant(byte[] image, int count) {
        double[][] pixels = linearPixels(image);
        if (pixels.length == 0 || count < 1) {
            return List.of();
        }

        int groups = Math.min(count, pixels.length);
        double[][] centres = seed(pixels, groups);
        int[] assignment = new int[pixels.length];

        for (int pass = 0; pass < ITERATIONS; pass++) {
            Lab[] centreLab = labOf(centres);
            boolean moved = false;

            for (int i = 0; i < pixels.length; i++) {
                int best = nearest(pixels[i], centreLab);
                if (assignment[i] != best) {
                    assignment[i] = best;
                    moved = true;
                }
            }
            recentre(pixels, assignment, centres);
            if (!moved) {
                break;
            }
        }

        return summarise(pixels, assignment, centres);
    }

    // --- Lecture -----------------------------------------------------------

    /** Pixels en lumiere lineaire : c'est la qu'une moyenne a un sens. */
    private static double[][] linearPixels(byte[] image) {
        try {
            BufferedImage source = ImageIO.read(new ByteArrayInputStream(image));
            if (source == null) {
                throw new IllegalArgumentException("Format d'image non reconnu");
            }
            int step = Math.max(1, Math.max(source.getWidth(), source.getHeight()) / ANALYSIS_EDGE);

            List<double[]> pixels = new ArrayList<>();
            for (int y = 0; y < source.getHeight(); y += step) {
                for (int x = 0; x < source.getWidth(); x += step) {
                    int packed = source.getRGB(x, y);
                    pixels.add(new double[]{
                            Colors.srgbToLinear(((packed >> 16) & 0xFF) / 255.0),
                            Colors.srgbToLinear(((packed >> 8) & 0xFF) / 255.0),
                            Colors.srgbToLinear((packed & 0xFF) / 255.0)});
                }
            }
            return pixels.toArray(double[][]::new);
        } catch (IOException e) {
            throw new UncheckedIOException("Lecture de l'image impossible", e);
        }
    }

    // --- Regroupement ------------------------------------------------------

    /**
     * Points de depart etales : on prend le premier pixel, puis a chaque fois le plus
     * eloigne de ceux deja retenus. Deterministe, contrairement a un tirage au sort, ce
     * qui evite qu'une meme photo donne deux plans differents.
     */
    private static double[][] seed(double[][] pixels, int groups) {
        double[][] centres = new double[groups][];
        centres[0] = pixels[0].clone();

        for (int taken = 1; taken < groups; taken++) {
            Lab[] chosen = labOf(java.util.Arrays.copyOf(centres, taken));
            int farthest = 0;
            double worst = -1;
            for (int i = 0; i < pixels.length; i++) {
                Lab lab = toLab(pixels[i]);
                double nearest = Double.MAX_VALUE;
                for (Lab centre : chosen) {
                    nearest = Math.min(nearest, Colors.deltaE2000(lab, centre));
                }
                if (nearest > worst) {
                    worst = nearest;
                    farthest = i;
                }
            }
            centres[taken] = pixels[farthest].clone();
        }
        return centres;
    }

    private static int nearest(double[] pixel, Lab[] centres) {
        Lab lab = toLab(pixel);
        int best = 0;
        double shortest = Double.MAX_VALUE;
        for (int i = 0; i < centres.length; i++) {
            double distance = Colors.deltaE2000(lab, centres[i]);
            if (distance < shortest) {
                shortest = distance;
                best = i;
            }
        }
        return best;
    }

    private static void recentre(double[][] pixels, int[] assignment, double[][] centres) {
        double[][] sums = new double[centres.length][3];
        int[] counts = new int[centres.length];

        for (int i = 0; i < pixels.length; i++) {
            int group = assignment[i];
            counts[group]++;
            for (int channel = 0; channel < 3; channel++) {
                sums[group][channel] += pixels[i][channel];
            }
        }
        for (int group = 0; group < centres.length; group++) {
            if (counts[group] > 0) {
                for (int channel = 0; channel < 3; channel++) {
                    centres[group][channel] = sums[group][channel] / counts[group];
                }
            }
        }
    }

    private static List<DominantColour> summarise(double[][] pixels, int[] assignment, double[][] centres) {
        int[] counts = new int[centres.length];
        for (int group : assignment) {
            counts[group]++;
        }

        List<DominantColour> found = new ArrayList<>();
        for (int group = 0; group < centres.length; group++) {
            if (counts[group] > 0) {
                found.add(new DominantColour(toRgb(centres[group]),
                        (double) counts[group] / pixels.length));
            }
        }
        found.sort(Comparator.comparingDouble(DominantColour::share).reversed());
        return List.copyOf(found);
    }

    // --- Conversions -------------------------------------------------------

    private static Lab[] labOf(double[][] linear) {
        Lab[] labs = new Lab[linear.length];
        for (int i = 0; i < linear.length; i++) {
            labs[i] = toLab(linear[i]);
        }
        return labs;
    }

    private static Lab toLab(double[] linear) {
        return Colors.toLab(toRgb(linear));
    }

    private static Rgb toRgb(double[] linear) {
        return new Rgb(Colors.linearToSrgb(linear[0]),
                Colors.linearToSrgb(linear[1]),
                Colors.linearToSrgb(linear[2]));
    }
}
