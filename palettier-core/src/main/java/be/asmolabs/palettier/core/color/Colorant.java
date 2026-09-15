package be.asmolabs.palettier.core.color;

import java.util.Arrays;
import java.util.List;

/**
 * Un colorant decrit par ses constantes de Kubelka-Munk : une absorption {@code K} et
 * une diffusion {@code S}, par canal.
 *
 * <p><b>Pourquoi deux constantes.</b> Le modele a constante unique suppose que tous les
 * pigments diffusent la lumiere de la meme facon, et ne retient donc que le rapport
 * {@code K/S}. Il ne peut alors pas representer le cas, pourtant banal, d'un pigment
 * dont le ton de masse et la teinte diluee divergent : le noir d'ivoire sort du tube
 * franchement chaud et donne pourtant des gris froids une fois coupe de blanc. Avec
 * deux constantes, les deux observations coexistent.</p>
 *
 * <p><b>Le mecanisme.</b> Au ton de masse, la couleur suit {@code K/S}. En teinte
 * diluee, la diffusion du blanc domine tellement que la couleur suit {@code K} seul.
 * Si la diffusion du pigment n'est pas la meme sur tous les canaux, ces deux couleurs
 * different : c'est exactement la divergence recherchee.</p>
 *
 * <p><b>Ce que ce n'est pas.</b> Les constantes sont ici ajustees sur deux couleurs
 * observees, pas mesurees au spectrophotometre sur trente longueurs d'onde. Le modele
 * reproduit fidelement ces deux points et interpole entre eux ; il ne pretend pas
 * decrire la physique du pigment.</p>
 */
public final class Colorant {

    /**
     * Le blanc de reference des teintes diluees : un blanc de titane. C'est lui qui fixe
     * l'echelle des constantes, arbitraire par nature.
     */
    public static final Rgb REFERENCE_WHITE = Rgb.ofHex("#F7F5F0");

    /** Diffusion du blanc de reference, qui sert d'unite. */
    private static final double REFERENCE_SCATTERING = 1.0;

    /** Proportion de colorant dans la teinte diluee de reference : une part pour neuf de blanc. */
    public static final double REFERENCE_TINT_CONCENTRATION = 0.1;

    /** En deca, la diffusion devient numeriquement instable. */
    private static final double MIN_SCATTERING = 1e-4;

    private final double[] absorption;
    private final double[] scattering;

    private Colorant(double[] absorption, double[] scattering) {
        this.absorption = absorption;
        this.scattering = scattering;
    }

    /**
     * Colorant decrit par son seul ton de masse.
     *
     * <p>La diffusion est posee a l'unite sur les trois canaux : le modele se ramene
     * alors exactement au modele a constante unique. C'est le repli quand la teinte
     * diluee d'une huile n'est pas renseignee, et il ne change rien aux resultats
     * obtenus jusque-la.</p>
     */
    public static Colorant ofMasstone(Rgb masstone) {
        double[] k = new double[3];
        double[] s = new double[3];
        double[] ratio = absorptionRatio(masstone);
        for (int channel = 0; channel < 3; channel++) {
            s[channel] = REFERENCE_SCATTERING;
            k[channel] = ratio[channel] * REFERENCE_SCATTERING;
        }
        return new Colorant(k, s);
    }

    /**
     * Colorant ajuste sur deux observations : le ton de masse et la teinte diluee.
     *
     * <p>Resolution, canal par canal. Le ton de masse donne directement
     * {@code K/S = F(Rmasse)}, donc {@code K = F(Rmasse) x S}. La teinte diluee, elle,
     * melange le colorant au blanc de reference :</p>
     *
     * <pre>  F(Rteinte) = (c.K + (1-c).Kblanc) / (c.S + (1-c).Sblanc)</pre>
     *
     * <p>En y injectant la premiere relation, {@code S} se degage :</p>
     *
     * <pre>  S = (1-c).(F(Rteinte).Sblanc - Kblanc) / (c.(F(Rmasse) - F(Rteinte)))</pre>
     *
     * <p>Si la teinte fournie n'est pas plus claire que le ton de masse, le systeme n'a
     * pas de solution utilisable : on retombe alors sur le modele a constante unique
     * plutot que de produire des constantes negatives.</p>
     *
     * @param masstone      couleur sortie de tube
     * @param tint          couleur du melange avec le blanc de reference
     * @param concentration part de colorant dans ce melange, entre 0 et 1
     */
    public static Colorant ofMasstoneAndTint(Rgb masstone, Rgb tint, double concentration) {
        double c = Math.clamp(concentration, 1e-3, 1 - 1e-3);
        double[] masstoneRatio = absorptionRatio(masstone);
        double[] tintRatio = absorptionRatio(tint);
        double[] whiteRatio = absorptionRatio(REFERENCE_WHITE);

        double[] k = new double[3];
        double[] s = new double[3];

        for (int channel = 0; channel < 3; channel++) {
            double whiteAbsorption = whiteRatio[channel] * REFERENCE_SCATTERING;
            double denominator = c * (masstoneRatio[channel] - tintRatio[channel]);
            double numerator = (1 - c) * (tintRatio[channel] * REFERENCE_SCATTERING - whiteAbsorption);

            if (denominator <= 0 || numerator <= 0) {
                // Teinte incoherente avec le ton de masse : on ne devine pas, on se replie.
                s[channel] = REFERENCE_SCATTERING;
            } else {
                s[channel] = Math.max(numerator / denominator, MIN_SCATTERING);
            }
            k[channel] = masstoneRatio[channel] * s[channel];
        }
        return new Colorant(k, s);
    }

    /**
     * Melange de plusieurs colorants.
     *
     * <p>Absorption et diffusion se combinent lineairement, chacune de son cote. C'est
     * la seule difference avec le modele a constante unique, mais c'est elle qui permet
     * au resultat de dependre de la diffusion respective des composants, et donc a un
     * blanc tres diffusant d'imposer sa loi des qu'il est present.</p>
     *
     * @param colorants les colorants presents
     * @param weights   leur quantite de matiere respective, normalisee en interne
     */
    public static Rgb mix(List<Colorant> colorants, List<Double> weights) {
        if (colorants.isEmpty() || colorants.size() != weights.size()) {
            throw new IllegalArgumentException("Il faut autant de poids que de colorants, et au moins un colorant");
        }
        double total = weights.stream().mapToDouble(Double::doubleValue).sum();
        if (total <= 0) {
            throw new IllegalArgumentException("La somme des poids doit etre strictement positive");
        }

        double[] k = new double[3];
        double[] s = new double[3];
        for (int i = 0; i < colorants.size(); i++) {
            double share = weights.get(i) / total;
            Colorant colorant = colorants.get(i);
            for (int channel = 0; channel < 3; channel++) {
                k[channel] += share * colorant.absorption[channel];
                s[channel] += share * colorant.scattering[channel];
            }
        }
        return fromAbsorptionRatio(new double[]{
                k[0] / Math.max(s[0], MIN_SCATTERING),
                k[1] / Math.max(s[1], MIN_SCATTERING),
                k[2] / Math.max(s[2], MIN_SCATTERING)});
    }

    /** Couleur du colorant pur, telle qu'elle sort du tube. */
    public Rgb masstone() {
        return fromAbsorptionRatio(new double[]{
                absorption[0] / scattering[0],
                absorption[1] / scattering[1],
                absorption[2] / scattering[2]});
    }

    /** Couleur du colorant coupe de blanc de reference, a la concentration indiquee. */
    public Rgb tint(double concentration) {
        return mix(List.of(this, ofMasstone(REFERENCE_WHITE)),
                List.of(concentration, 1 - concentration));
    }

    /** Absorption par canal. Copie defensive : la recherche de melange precalcule ces tableaux. */
    public double[] absorption() {
        return Arrays.copyOf(absorption, 3);
    }

    /** Diffusion par canal. */
    public double[] scattering() {
        return Arrays.copyOf(scattering, 3);
    }

    /**
     * Couleur d'un melange dont l'absorption et la diffusion sont deja cumulees.
     * Raccourci pour la recherche de melange, qui fait ses propres sommes.
     */
    public static Rgb colorOf(double[] absorption, double[] scattering) {
        return fromAbsorptionRatio(new double[]{
                absorption[0] / Math.max(scattering[0], MIN_SCATTERING),
                absorption[1] / Math.max(scattering[1], MIN_SCATTERING),
                absorption[2] / Math.max(scattering[2], MIN_SCATTERING)});
    }

    // --- Passage couleur <-> rapport K/S -----------------------------------

    private static double[] absorptionRatio(Rgb color) {
        return Colors.toKs(color);
    }

    private static Rgb fromAbsorptionRatio(double[] ratio) {
        return Colors.fromKs(ratio);
    }
}
