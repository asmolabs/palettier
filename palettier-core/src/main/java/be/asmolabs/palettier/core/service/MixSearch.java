package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colorant;
import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Lab;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
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

    /** Taille du vivier pour les melanges a trois tubes. */
    private static final int TRIPLE_POOL = 18;

    /** Nombre de tubes structurels ajoutes au vivier (blancs, noirs, forts colorants). */
    private static final int STRUCTURAL_PAINTS = 4;

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
    private static final List<int[]> TRIPLE_RATIOS = practicalTripleRatios();

    private final Lab target;
    private final List<Candidate> candidates;

    /**
     * Un tube, avec ses constantes de melange precalculees. Absorption et diffusion sont
     * gardees separement : c'est ce qui permet au blanc, tres diffusant, de peser sur le
     * resultat autrement que par sa seule clarte.
     */
    private record Candidate(OilPaint paint, double[] absorption, double[] scattering, double tinting) {
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
                            paint.getTintingStrength());
                })
                .toList();
    }

    List<MixSuggestion> search(int maxPaints, int maxResults) {
        List<Evaluation> found = new ArrayList<>(singles());
        if (maxPaints >= 2) {
            found.addAll(pairs());
        }
        if (maxPaints >= 3) {
            found.addAll(triples());
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
     * Vivier : les tubes les plus proches de la cible pris un a un, plus quelques tubes
     * a fort pouvoir colorant. Ces derniers ne ressemblent pas a la cible mais sont ce
     * qui permet d'en ajuster la valeur, et ils manqueraient a une selection fondee sur
     * la seule proximite.
     */
    private List<Evaluation> triples() {
        List<Candidate> pool = new ArrayList<>(candidates.stream()
                .sorted(Comparator.comparingDouble(c -> evaluate(List.of(c), new int[]{1}).deltaE()))
                .limit(TRIPLE_POOL)
                .toList());

        candidates.stream()
                .sorted(Comparator.comparingDouble(Candidate::tinting).reversed())
                .filter(candidate -> !pool.contains(candidate))
                .limit(STRUCTURAL_PAINTS)
                .forEach(pool::add);

        int size = pool.size();
        return IntStream.range(0, size).parallel()
                .boxed()
                .flatMap(i -> IntStream.range(i + 1, size).boxed()
                        .flatMap(j -> IntStream.range(j + 1, size)
                                .mapToObj(k -> refine(List.of(pool.get(i), pool.get(j), pool.get(k)),
                                        TRIPLE_RATIOS))))
                .toList();
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

    /** A trois tubes, personne ne dose au-dela de quelques parts : on s'arrete a cinq. */
    private static List<int[]> practicalTripleRatios() {
        List<int[]> ratios = new ArrayList<>();
        for (int a = 1; a <= 5; a++) {
            for (int b = 1; b <= 5; b++) {
                for (int c = 1; c <= 5; c++) {
                    if (gcd(gcd(a, b), c) == 1) {
                        ratios.add(new int[]{a, b, c});
                    }
                }
            }
        }
        return List.copyOf(ratios);
    }

    private static int gcd(int a, int b) {
        return b == 0 ? a : gcd(b, a % b);
    }
}
