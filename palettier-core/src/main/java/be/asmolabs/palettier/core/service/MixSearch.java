package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colorant;
import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Lab;
import be.asmolabs.palettier.core.color.Lab;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.List;
import java.util.Map;
import java.util.stream.IntStream;

/**
 * Recherche du melange approchant une couleur cible.
 *
 * <p>Trois idees portent l'algorithme.</p>
 *
 * <p><b>1. Ne proposer que des dosages realisables.</b> Un optimum continu du type
 * "0,371 part contre 0,629" n'a aucun sens au bout d'un pinceau. La recherche
 * n'explore donc que des rapports d'entiers simples, et l'ecart annonce est celui du
 * dosage effectivement propose, pas celui d'un optimum theorique inatteignable.</p>
 *
 * <p><b>2. Deux passes, grossiere puis fine.</b> Toutes les paires sont evaluees a cinq
 * dosages seulement ; les meilleures sont ensuite reprises sur la grille complete des
 * dosages realisables. On paie le prix fort uniquement la ou il peut changer le
 * classement.</p>
 *
 * <p><b>3. Les melanges a trois tubes sur un vivier restreint.</b> Les explorer tous
 * serait hors de portee ; mais un bon melange se compose presque toujours de tubes
 * deja proches de la cible, plus un tube "structurel" a fort pouvoir colorant (un
 * blanc, un noir) qui sert a monter ou descendre la valeur. Le vivier est constitue de
 * ces deux familles.</p>
 */
final class MixSearch {

    /** Nombre de paires retenues a l'issue de la passe grossiere. */
    private static final int PAIRS_KEPT = 60;

    /** Dosages de la passe grossiere, exprimes en part de pigment du premier tube. */
    private static final double[] COARSE_WEIGHTS = {0.15, 0.35, 0.5, 0.65, 0.85};

    /**
     * Nombre maximal de tubes par melange.
     *
     * <p>Cinq, parce que certaines teintes ne s'obtiennent pas autrement : les trois
     * primaires pour la couleur, un blanc pour la valeur, une terre pour rompre. Au-dela,
     * un melange cesse d'etre reproductible d'une seance a l'autre.</p>
     */
    static final int MAX_PAINTS = 5;

    /** Nombre de tubes structurels ajoutes au vivier (blancs, noirs, forts colorants). */
    private static final int STRUCTURAL_PAINTS = 4;

    /** Secteurs de teinte couverts par le vivier, en degres. */
    private static final int HUE_SECTORS = 6;

    /**
     * En deca de cet ecart, l'oeil ne distingue plus deux teintes. Deux propositions
     * separees par moins que cela sont donc tenues pour equivalentes, et c'est la
     * simplicite qui les departage.
     */
    private static final double PERCEPTUAL_TIE = 0.5;

    /**
     * En deca de cet ecart, deux propositions donnent la meme couleur et l'une des deux
     * n'apprend rien. Le seuil est volontairement bien plus serre que le seuil de
     * perception : deux recettes qui atteignent toutes les deux la cible sont des
     * alternatives utiles, pas des doublons. On ne veut ecarter que les vrais jumeaux.
     */
    private static final double DUPLICATE_RESULT = 0.15;

    private static final List<int[]> PAIR_RATIOS = practicalPairRatios();

    /** Grilles de dosage par nombre de tubes, calculees une fois. */
    private static final Map<Integer, List<int[]>> RATIOS = ratioGrids();

    private final Lab target;
    private final List<Candidate> candidates;

    /**
     * Un tube, avec ses constantes de melange precalculees. Absorption et diffusion sont
     * gardees separement : c'est ce qui permet au blanc, tres diffusant, de peser sur le
     * resultat autrement que par sa seule clarte.
     */
    private record Candidate(OilPaint paint, double[] absorption, double[] scattering,
                             double tinting, Lab lab) {

        /** Saturation percue : ce qui distingue une couleur franche d'un gris. */
        double chroma() {
            return Math.hypot(lab.a(), lab.b());
        }

        /** Angle de teinte, en degres. */
        double hue() {
            double degrees = Math.toDegrees(Math.atan2(lab.b(), lab.a()));
            return degrees < 0 ? degrees + 360 : degrees;
        }
    }

    /** Un melange evalue : les tubes, leurs parts entieres, la couleur obtenue, l'ecart. */
    private record Evaluation(List<Candidate> paints, int[] parts, Rgb color, double deltaE) {
    }

    MixSearch(Rgb target, List<OilPaint> paints) {
        this.target = Colors.toLab(target);
        this.candidates = paints.stream()
                .map(paint -> {
                    Colorant colorant = paint.colorant();
                    return new Candidate(paint, colorant.absorption(), colorant.scattering(),
                            paint.getTintingStrength(), Colors.toLab(paint.color()));
                })
                .toList();
    }

    List<MixSuggestion> search(int maxPaints, int maxResults) {
        List<Evaluation> found = new ArrayList<>(singles());
        if (maxPaints >= 2) {
            found.addAll(pairs());
        }
        for (int count = 3; count <= Math.min(maxPaints, MAX_PAINTS); count++) {
            found.addAll(combinations(count));
        }
        return best(found, maxResults);
    }

    // --- Un seul tube ------------------------------------------------------

    private List<Evaluation> singles() {
        return candidates.stream()
                .map(candidate -> evaluate(List.of(candidate), new int[]{1}))
                .toList();
    }

    // --- Deux tubes --------------------------------------------------------

    /**
     * Passe grossiere sur toutes les paires, puis grille complete des dosages
     * realisables sur les meilleures.
     */
    private List<Evaluation> pairs() {
        int size = candidates.size();

        List<Evaluation> coarse = IntStream.range(0, size).parallel()
                .boxed()
                .flatMap(i -> IntStream.range(i + 1, size)
                        .mapToObj(j -> coarseBest(candidates.get(i), candidates.get(j))))
                .sorted(Comparator.comparingDouble(Evaluation::deltaE))
                .limit(PAIRS_KEPT)
                .toList();

        return coarse.stream().parallel()
                .map(pair -> refine(pair.paints(), PAIR_RATIOS))
                .toList();
    }

    /** Meilleur des cinq dosages de reperage : sert uniquement a classer la paire. */
    private Evaluation coarseBest(Candidate first, Candidate second) {
        Evaluation best = null;
        List<Candidate> pair = List.of(first, second);
        for (double weight : COARSE_WEIGHTS) {
            Rgb color = blend(pair, new double[]{weight, 1 - weight});
            double deltaE = Colors.deltaE2000(target, Colors.toLab(color));
            if (best == null || deltaE < best.deltaE()) {
                best = new Evaluation(List.of(first, second), new int[]{1, 1}, color, deltaE);
            }
        }
        return best;
    }

    // --- Trois tubes -------------------------------------------------------

    /**
     * Melanges a {@code count} tubes, pris dans un vivier restreint.
     *
     * <p>Les explorer tous serait hors de portee : quatre cents tubes pris cinq a cinq
     * font des milliards de combinaisons. Le vivier retient donc ce qui sert vraiment,
     * et le nombre de tubes fait retrecir le vivier pour que le cout reste tenable.</p>
     */
    private List<Evaluation> combinations(int count) {
        List<Candidate> pool = poolFor(count);
        List<int[]> ratios = RATIOS.get(count);
        if (pool.size() < count || ratios == null) {
            return List.of();
        }

        List<int[]> picks = new ArrayList<>();
        combine(pool.size(), count, 0, 0, new int[count], picks);

        return picks.stream().parallel()
                .map(indexes -> {
                    List<Candidate> paints = new ArrayList<>(count);
                    for (int index : indexes) {
                        paints.add(pool.get(index));
                    }
                    return refine(paints, ratios);
                })
                .toList();
    }

    /** Toutes les facons de choisir {@code count} rangs distincts parmi {@code size}. */
    private static void combine(int size, int count, int filled, int start, int[] current, List<int[]> out) {
        if (filled == count) {
            out.add(current.clone());
            return;
        }
        for (int i = start; i < size; i++) {
            current[filled] = i;
            combine(size, count, filled + 1, i + 1, current, out);
        }
    }

    /**
     * Le vivier dans lequel piocher.
     *
     * <p>Trois familles, et la troisieme est celle qui compte pour les melanges nombreux.
     * Les tubes proches de la cible donnent la teinte generale. Les tubes structurels --
     * le plus clair, le plus sombre, les plus colorants -- servent a regler la valeur.
     * Et les representants de chaque secteur de teinte garantissent qu'on dispose des
     * primaires : une selection fondee sur la seule proximite ne contiendrait jamais le
     * bleu necessaire a rompre un orange.</p>
     */
    private List<Candidate> poolFor(int count) {
        int nearest = switch (count) {
            case 3 -> 18;
            case 4 -> 12;
            default -> 9;
        };

        LinkedHashSet<Candidate> pool = new LinkedHashSet<>(candidates.stream()
                .sorted(Comparator.comparingDouble(c -> evaluate(List.of(c), new int[]{1}).deltaE()))
                .limit(nearest)
                .toList());

        // Les extremes de valeur : de quoi monter ou descendre sans changer la teinte.
        candidates.stream().max(Comparator.comparingDouble(c -> c.lab().l())).ifPresent(pool::add);
        candidates.stream().min(Comparator.comparingDouble(c -> c.lab().l())).ifPresent(pool::add);

        candidates.stream()
                .sorted(Comparator.comparingDouble(Candidate::tinting).reversed())
                .limit(STRUCTURAL_PAINTS)
                .forEach(pool::add);

        // Un representant par secteur de teinte, le plus franc de son secteur.
        for (int sector = 0; sector < HUE_SECTORS; sector++) {
            double from = sector * 360.0 / HUE_SECTORS;
            double to = (sector + 1) * 360.0 / HUE_SECTORS;
            candidates.stream()
                    .filter(c -> c.hue() >= from && c.hue() < to)
                    .max(Comparator.comparingDouble(Candidate::chroma))
                    .ifPresent(pool::add);
        }
        return List.copyOf(pool);
    }

    // --- Evaluation --------------------------------------------------------

    /** Essaie tous les dosages realisables et garde le meilleur. */
    private Evaluation refine(List<Candidate> paints, List<int[]> ratios) {
        Evaluation best = null;
        for (int[] parts : ratios) {
            Evaluation evaluation = evaluate(paints, parts);
            if (best == null || evaluation.deltaE() < best.deltaE()) {
                best = evaluation;
            }
        }
        return best;
    }

    private Evaluation evaluate(List<Candidate> paints, int[] parts) {
        double[] weights = new double[paints.size()];
        for (int i = 0; i < paints.size(); i++) {
            // Le poids dans la couleur, c'est la dose multipliee par le pouvoir colorant.
            weights[i] = parts[i] * paints.get(i).tinting();
        }
        Rgb color = blend(paints, weights);
        return new Evaluation(paints, parts, color, Colors.deltaE2000(target, Colors.toLab(color)));
    }

    /**
     * Melange de Kubelka-Munk a deux constantes : absorption et diffusion se cumulent
     * lineairement, chacune de son cote, et c'est leur rapport qui donne la couleur.
     */
    private static Rgb blend(List<Candidate> paints, double[] weights) {
        double total = 0;
        for (double weight : weights) {
            total += weight;
        }
        double[] absorption = new double[3];
        double[] scattering = new double[3];
        for (int i = 0; i < paints.size(); i++) {
            double share = weights[i] / total;
            Candidate candidate = paints.get(i);
            for (int channel = 0; channel < 3; channel++) {
                absorption[channel] += share * candidate.absorption()[channel];
                scattering[channel] += share * candidate.scattering()[channel];
            }
        }
        return Colorant.colorOf(absorption, scattering);
    }

    // --- Classement --------------------------------------------------------

    /**
     * Classe et epure les resultats.
     *
     * <p>Trois nettoyages, dans cet ordre.</p>
     *
     * <p>Une seule proposition par combinaison de tubes : sinon la liste se remplit de
     * dosages voisins des deux memes tubes, ce qui n'offre aucun choix reel.</p>
     *
     * <p>A ecart perceptuellement equivalent, le melange le plus simple gagne. Sans cette
     * regle, la recherche peut repondre "quatre parts de terre d'ombre brulee Winsor &
     * Newton, cinq de terre d'ombre brulee Schmincke et trois de terre d'ombre brulee
     * Norma Blue" la ou un seul tube suffisait : mathematiquement juste, inutilisable au
     * pinceau.</p>
     *
     * <p>Enfin, les propositions qui aboutissent a la meme teinte sont regroupees : le
     * catalogue contient la meme couleur chez plusieurs fabricants, et les voir alignees
     * n'apprend rien.</p>
     */
    private List<MixSuggestion> best(List<Evaluation> evaluations, int maxResults) {
        Map<String, Evaluation> bySet = new LinkedHashMap<>();
        for (Evaluation evaluation : evaluations) {
            String key = evaluation.paints().stream()
                    .map(candidate -> candidate.paint().displayName())
                    .sorted()
                    .reduce("", (a, b) -> a + "|" + b);
            Evaluation previous = bySet.get(key);
            if (previous == null || evaluation.deltaE() < previous.deltaE()) {
                bySet.put(key, evaluation);
            }
        }

        List<Evaluation> ranked = bySet.values().stream().sorted(byQualityThenSimplicity()).toList();

        List<Evaluation> distinct = new ArrayList<>();
        for (Evaluation evaluation : ranked) {
            boolean alreadyCovered = distinct.stream().anyMatch(kept ->
                    Colors.deltaE2000(kept.color(), evaluation.color()) < DUPLICATE_RESULT);
            if (!alreadyCovered) {
                distinct.add(evaluation);
            }
            if (distinct.size() == maxResults) {
                break;
            }
        }
        return distinct.stream().map(MixSearch::toSuggestion).toList();
    }

    /**
     * Ecart d'abord, mais par paliers de la taille du seuil de perception : a l'interieur
     * d'un palier, le melange le moins complique passe devant.
     */
    private static Comparator<Evaluation> byQualityThenSimplicity() {
        return Comparator
                .comparingInt((Evaluation e) -> (int) (e.deltaE() / PERCEPTUAL_TIE))
                .thenComparingInt(e -> e.paints().size())
                .thenComparingInt(MixSearch::totalParts)
                .thenComparingDouble(Evaluation::deltaE);
    }

    /** Un melange en 1:2 se dose plus vite qu'un melange en 7:8, a qualite egale. */
    private static int totalParts(Evaluation evaluation) {
        int total = 0;
        for (int parts : evaluation.parts()) {
            total += parts;
        }
        return total;
    }

    private static MixSuggestion toSuggestion(Evaluation evaluation) {
        List<PaintPart> parts = new ArrayList<>();
        for (int i = 0; i < evaluation.paints().size(); i++) {
            parts.add(PaintPart.of(evaluation.paints().get(i).paint(), evaluation.parts()[i]));
        }
        return new MixSuggestion(List.copyOf(parts), evaluation.color(), evaluation.deltaE());
    }

    // --- Dosages realisables -----------------------------------------------

    /**
     * Rapports a deux tubes qu'on sait reellement doser : entiers premiers entre eux
     * jusqu'a huit parts, plus quelques ajouts infimes. Ces derniers comptent : une
     * pointe de bleu de Prusse dans un blanc, c'est du 1:20, et c'est un geste courant.
     */
    private static List<int[]> practicalPairRatios() {
        List<int[]> ratios = new ArrayList<>();
        for (int a = 1; a <= 8; a++) {
            for (int b = 1; b <= 8; b++) {
                if (gcd(a, b) == 1) {
                    ratios.add(new int[]{a, b});
                }
            }
        }
        for (int tiny : new int[]{10, 12, 16, 20, 30}) {
            ratios.add(new int[]{1, tiny});
            ratios.add(new int[]{tiny, 1});
        }
        return List.copyOf(ratios);
    }

    /**
     * Grilles de dosage, du triple au quintuple.
     *
     * <p>Plus il y a de tubes, plus les parts restent petites : personne ne dose
     * "sept parts de l'un, trois de l'autre, cinq du troisieme et deux du quatrieme".
     * Cette limite tient autant du realisme que du cout de la recherche.</p>
     */
    private static Map<Integer, List<int[]>> ratioGrids() {
        Map<Integer, List<int[]>> grids = new LinkedHashMap<>();
        for (int count = 3; count <= MAX_PAINTS; count++) {
            int max = switch (count) {
                case 3 -> 5;
                case 4 -> 4;
                default -> 3;
            };
            List<int[]> grid = new ArrayList<>();
            fillRatios(new int[count], 0, max, grid);
            grids.put(count, List.copyOf(grid));
        }
        return Map.copyOf(grids);
    }

    private static void fillRatios(int[] current, int index, int max, List<int[]> out) {
        if (index == current.length) {
            int divisor = current[0];
            for (int part : current) {
                divisor = gcd(divisor, part);
            }
            // Un dosage et son double decrivent le meme melange : on ne garde que le reduit.
            if (divisor == 1) {
                out.add(current.clone());
            }
            return;
        }
        for (int part = 1; part <= max; part++) {
            current[index] = part;
            fillRatios(current, index + 1, max, out);
        }
    }

    private static int gcd(int a, int b) {
        return b == 0 ? a : gcd(b, a % b);
    }
}
