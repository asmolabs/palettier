package be.asmolabs.palettier.core.color;

/**
 * Couleur sRGB, chaque composante exprimee dans l'intervalle [0, 1].
 */
public record Rgb(double r, double g, double b) {

    public Rgb {
        r = clamp01(r);
        g = clamp01(g);
        b = clamp01(b);
    }

    public static Rgb ofHex(String hex) {
        String value = hex.startsWith("#") ? hex.substring(1) : hex;
        if (value.length() == 3) {
            value = "" + value.charAt(0) + value.charAt(0)
                    + value.charAt(1) + value.charAt(1)
                    + value.charAt(2) + value.charAt(2);
        }
        if (value.length() != 6) {
            throw new IllegalArgumentException("Code hexadecimal invalide : " + hex);
        }
        int packed = Integer.parseInt(value, 16);
        return new Rgb(((packed >> 16) & 0xFF) / 255.0,
                ((packed >> 8) & 0xFF) / 255.0,
                (packed & 0xFF) / 255.0);
    }

    public String toHex() {
        return "#%02X%02X%02X".formatted(to255(r), to255(g), to255(b));
    }

    /** Luminance relative (WCAG), utile pour choisir une couleur de texte lisible sur la pastille. */
    public double relativeLuminance() {
        return 0.2126 * Colors.srgbToLinear(r)
                + 0.7152 * Colors.srgbToLinear(g)
                + 0.0722 * Colors.srgbToLinear(b);
    }

    private static int to255(double v) {
        return (int) Math.round(v * 255.0);
    }

    private static double clamp01(double v) {
        return Math.clamp(v, 0.0, 1.0);
    }
}
