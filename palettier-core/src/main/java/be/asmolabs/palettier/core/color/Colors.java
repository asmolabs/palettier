package be.asmolabs.palettier.core.color;

import java.util.Collection;
import java.util.List;

/**
 * Conversions colorimetriques et melange soustractif.
 *
 * <p>Le melange n'est volontairement pas une moyenne RGB : deux peintures qui se melangent
 * se comportent comme des milieux diffusants. On passe donc par le modele de
 * Kubelka-Munk a constante unique, ou le rapport absorption/diffusion {@code K/S} est
 * additif. C'est ce qui fait que bleu + jaune donne du vert, et non du gris.</p>
 */
public final class Colors {

    private Colors() {
    }

    // --- sRGB <-> lineaire -------------------------------------------------

    public static double srgbToLinear(double c) {
        return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
    }

    public static double linearToSrgb(double c) {
        double v = Math.clamp(c, 0.0, 1.0);
        return v <= 0.0031308 ? v * 12.92 : 1.055 * Math.pow(v, 1 / 2.4) - 0.055;
    }

    // --- sRGB -> Lab -------------------------------------------------------

    private static final double XN = 0.95047;
    private static final double YN = 1.00000;
    private static final double ZN = 1.08883;

    public static Lab toLab(Rgb rgb) {
        double r = srgbToLinear(rgb.r());
        double g = srgbToLinear(rgb.g());
        double b = srgbToLinear(rgb.b());

        double x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / XN;
        double y = (0.2126729 * r + 0.7151522 * g + 0.0721750 * b) / YN;
        double z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / ZN;

        double fx = pivot(x);
        double fy = pivot(y);
        double fz = pivot(z);

        return new Lab(116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz));
    }

    private static double pivot(double t) {
        return t > 216.0 / 24389.0 ? Math.cbrt(t) : (24389.0 / 27.0 * t + 16) / 116.0;
    }

    // --- Ecart percu -------------------------------------------------------

    /**
     * Ecart de couleur CIEDE2000. Reperes d'interpretation : moins de 1 = indiscernable,
     * moins de 2 = ecart visible seulement en comparaison cote a cote, plus de 5 = deux
     * couleurs clairement differentes.
     */
    public static double deltaE2000(Rgb first, Rgb second) {
        return deltaE2000(toLab(first), toLab(second));
    }

    public static double deltaE2000(Lab lab1, Lab lab2) {
        double kL = 1, kC = 1, kH = 1;

        double c1 = Math.hypot(lab1.a(), lab1.b());
        double c2 = Math.hypot(lab2.a(), lab2.b());
        double cBar = (c1 + c2) / 2.0;

        double cBar7 = Math.pow(cBar, 7);
        double g = 0.5 * (1 - Math.sqrt(cBar7 / (cBar7 + Math.pow(25, 7))));

        double a1p = (1 + g) * lab1.a();
        double a2p = (1 + g) * lab2.a();
        double c1p = Math.hypot(a1p, lab1.b());
        double c2p = Math.hypot(a2p, lab2.b());

        double h1p = hueAngle(lab1.b(), a1p);
        double h2p = hueAngle(lab2.b(), a2p);

        double dLp = lab2.l() - lab1.l();
        double dCp = c2p - c1p;

        double dhp;
        if (c1p * c2p == 0) {
            dhp = 0;
        } else if (Math.abs(h2p - h1p) <= 180) {
            dhp = h2p - h1p;
        } else {
            dhp = h2p - h1p > 180 ? h2p - h1p - 360 : h2p - h1p + 360;
        }
        double dHp = 2 * Math.sqrt(c1p * c2p) * Math.sin(Math.toRadians(dhp) / 2);

        double lBarP = (lab1.l() + lab2.l()) / 2;
        double cBarP = (c1p + c2p) / 2;

        double hBarP;
        if (c1p * c2p == 0) {
            hBarP = h1p + h2p;
        } else if (Math.abs(h1p - h2p) <= 180) {
            hBarP = (h1p + h2p) / 2;
        } else {
            hBarP = h1p + h2p < 360 ? (h1p + h2p + 360) / 2 : (h1p + h2p - 360) / 2;
        }

        double t = 1
                - 0.17 * Math.cos(Math.toRadians(hBarP - 30))
                + 0.24 * Math.cos(Math.toRadians(2 * hBarP))
                + 0.32 * Math.cos(Math.toRadians(3 * hBarP + 6))
                - 0.20 * Math.cos(Math.toRadians(4 * hBarP - 63));

        double dTheta = 30 * Math.exp(-Math.pow((hBarP - 275) / 25, 2));
        double cBarP7 = Math.pow(cBarP, 7);
        double rC = 2 * Math.sqrt(cBarP7 / (cBarP7 + Math.pow(25, 7)));
        double rT = -rC * Math.sin(2 * Math.toRadians(dTheta));

        double lBarP50 = Math.pow(lBarP - 50, 2);
        double sL = 1 + (0.015 * lBarP50) / Math.sqrt(20 + lBarP50);
        double sC = 1 + 0.045 * cBarP;
        double sH = 1 + 0.015 * cBarP * t;

        double termL = dLp / (kL * sL);
        double termC = dCp / (kC * sC);
        double termH = dHp / (kH * sH);

        return Math.sqrt(termL * termL + termC * termC + termH * termH + rT * termC * termH);
    }

    private static double hueAngle(double b, double ap) {
        if (b == 0 && ap == 0) {
            return 0;
        }
        double deg = Math.toDegrees(Math.atan2(b, ap));
        return deg >= 0 ? deg : deg + 360;
    }

    // --- Echantillonnage d'image -------------------------------------------

    /**
     * Moyenne d'un ensemble de pixels.
     *
     * <p>La moyenne est calculee en lumiere lineaire, pas sur les valeurs sRGB : le sRGB
     * est encode en gamma, et en moyenner les valeurs directement assombrit le resultat.
     * Sur un degrade de carnation, l'ecart est nettement visible.</p>
     *
     * @throws IllegalArgumentException si l'echantillon est vide
     */
    public static Rgb average(Collection<Rgb> samples) {
        if (samples.isEmpty()) {
            throw new IllegalArgumentException("Echantillon vide");
        }
        double r = 0;
        double g = 0;
        double b = 0;
        for (Rgb sample : samples) {
            r += srgbToLinear(sample.r());
            g += srgbToLinear(sample.g());
            b += srgbToLinear(sample.b());
        }
        int count = samples.size();
        return new Rgb(linearToSrgb(r / count), linearToSrgb(g / count), linearToSrgb(b / count));
    }

    /**
     * Corrige la dominante colorée d'une photo.
     *
     * <p>Une photo prise sous lampe de bureau tire au jaune, sous LED froide au bleu :
     * la couleur relevee n'est alors pas celle de la peinture. En designant un point de
     * l'image qui devrait etre gris neutre, on obtient le gain a appliquer a chaque
     * canal pour annuler cette dominante. C'est la balance des blancs du photographe,
     * reduite a son strict necessaire.</p>
     *
     * @param sample    couleur relevee sur la photo
     * @param greyPoint couleur d'un point cense etre neutre
     * @return la couleur echantillonnee debarrassee de la dominante
     */
    public static Rgb neutralise(Rgb sample, Rgb greyPoint) {
        double gr = Math.max(srgbToLinear(greyPoint.r()), 1e-4);
        double gg = Math.max(srgbToLinear(greyPoint.g()), 1e-4);
        double gb = Math.max(srgbToLinear(greyPoint.b()), 1e-4);

        // On vise la luminosite moyenne du point de reference : la correction change la
        // teinte, pas l'exposition.
        double target = (gr + gg + gb) / 3.0;

        return new Rgb(
                linearToSrgb(srgbToLinear(sample.r()) * target / gr),
                linearToSrgb(srgbToLinear(sample.g()) * target / gg),
                linearToSrgb(srgbToLinear(sample.b()) * target / gb));
    }

    // --- Melange soustractif (Kubelka-Munk, constante unique) ---------------

    /**
     * Melange des couleurs ponderees. Les poids representent la quantite de matiere
     * effectivement deposee (dose x pouvoir colorant) ; ils sont normalises en interne.
     *
     * @throws IllegalArgumentException si les listes n'ont pas la meme taille, sont vides,
     *                                  ou si la somme des poids est nulle
     */
    public static Rgb mix(List<Rgb> colors, List<Double> weights) {
        if (colors.isEmpty() || colors.size() != weights.size()) {
            throw new IllegalArgumentException("Il faut autant de poids que de couleurs, et au moins une couleur");
        }
        double total = weights.stream().mapToDouble(Double::doubleValue).sum();
        if (total <= 0) {
            throw new IllegalArgumentException("La somme des poids doit etre strictement positive");
        }

        double[] ks = new double[3];
        for (int i = 0; i < colors.size(); i++) {
            double w = weights.get(i) / total;
            Rgb c = colors.get(i);
            ks[0] += w * toKS(srgbToLinear(c.r()));
            ks[1] += w * toKS(srgbToLinear(c.g()));
            ks[2] += w * toKS(srgbToLinear(c.b()));
        }

        return new Rgb(linearToSrgb(fromKS(ks[0])),
                linearToSrgb(fromKS(ks[1])),
                linearToSrgb(fromKS(ks[2])));
    }

    /**
     * Coordonnees Kubelka-Munk d'une couleur, un canal par composante.
     *
     * <p>Interet pour la recherche de melange : dans cet espace, melanger est une
     * combinaison lineaire. L'ensemble des couleurs atteignables avec deux tubes est
     * donc un segment de droite, avec trois tubes un triangle. Chercher un melange
     * revient a chercher un point dans une figure simple, au lieu d'explorer a
     * l'aveugle.</p>
     */
    public static double[] toKs(Rgb color) {
        return new double[]{
                toKS(srgbToLinear(color.r())),
                toKS(srgbToLinear(color.g())),
                toKS(srgbToLinear(color.b()))};
    }

    /** Couleur correspondant a des coordonnees Kubelka-Munk. */
    public static Rgb fromKs(double[] ks) {
        return new Rgb(linearToSrgb(fromKS(ks[0])),
                linearToSrgb(fromKS(ks[1])),
                linearToSrgb(fromKS(ks[2])));
    }

    /** Rapport absorption / diffusion pour une reflectance donnee. */
    private static double toKS(double reflectance) {
        double r = Math.clamp(reflectance, 0.002, 0.998);
        return (1 - r) * (1 - r) / (2 * r);
    }

    /** Reflectance correspondant a un rapport K/S. */
    private static double fromKS(double ks) {
        return 1 + ks - Math.sqrt(ks * ks + 2 * ks);
    }
}
