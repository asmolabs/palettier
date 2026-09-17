package be.asmolabs.palettier.domain.mix

import be.asmolabs.palettier.domain.color.Colorant
import be.asmolabs.palettier.domain.color.Lab
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.color.toLab
import be.asmolabs.palettier.domain.paint.Paint
import kotlin.math.PI
import kotlin.math.atan2
import kotlin.math.hypot

/**
 * Recherche du melange approchant une couleur cible.
 *
 * <p><b>1. Ne proposer que des dosages realisables.</b> Un optimum continu du type
 * "0,371 part contre 0,629" n'a aucun sens au bout d'un pinceau. La recherche n'explore
 * donc que des rapports d'entiers simples, et l'ecart annonce est celui du dosage
 * effectivement propose, pas celui d'un optimum theorique inatteignable.</p>
 *
 * <p><b>2. Deux passes, grossiere puis fine.</b> Toutes les paires sont evaluees a cinq
 * dosages seulement ; les meilleures sont reprises sur la grille complete. On paie le
 * prix fort uniquement la ou il peut changer le classement.</p>
 *
 * <p><b>3. Les melanges nombreux sur un vivier restreint.</b> Un bon melange se compose
 * presque toujours de tubes deja proches de la cible, plus un tube structurel a fort
 * pouvoir colorant qui sert a monter ou descendre la valeur.</p>
 */
internal class MixSearch(target: Rgb, paints: List<Paint>) {

    private val target: Lab = target.toLab()
    private val candidates: List<Candidate> = paints.map { paint ->
        val colorant = paint.colorant()
        Candidate(paint, colorant.absorption, colorant.scattering, paint.tintingStrength, paint.color.toLab())
    }

    /**
     * Un tube, constantes de melange precalculees. Absorption et diffusion sont gardees
     * separement : c'est ce qui permet au blanc, tres diffusant, de peser autrement que
     * par sa seule clarte.
     */
    private class Candidate(
        val paint: Paint,
        val absorption: DoubleArray,
        val scattering: DoubleArray,
        val tinting: Double,
        val lab: Lab,
    ) {
        /** Saturation percue : ce qui distingue une couleur franche d'un gris. */
        val chroma: Double get() = hypot(lab.a, lab.b)

        /** Angle de teinte, en degres. */
        val hue: Double
            get() {
                val degrees = atan2(lab.b, lab.a) * 180.0 / PI
                return if (degrees < 0) degrees + 360 else degrees
            }
    }

    /** Un melange evalue : les tubes, leurs parts entieres, la couleur obtenue, l'ecart. */
    private class Evaluation(
        val paints: List<Candidate>,
        val parts: IntArray,
        val color: Rgb,
        val deltaE: Double,
    )

    fun search(maxPaints: Int, maxResults: Int): List<MixSuggestion> {
        val found = ArrayList<Evaluation>(singles())
        if (maxPaints >= 2) found += pairs()
        for (count in 3..minOf(maxPaints, MAX_PAINTS)) {
            found += combinations(count)
        }
        return best(found, maxResults)
    }

    // --- Un seul tube ------------------------------------------------------

    private fun singles(): List<Evaluation> =
        candidates.map { evaluate(listOf(it), intArrayOf(1)) }

    // --- Deux tubes --------------------------------------------------------

    /**
     * Passe grossiere sur toutes les paires, puis grille complete des dosages realisables
     * sur les meilleures.
     *
     * <p>Sequentiel, la ou le Java parallelisait : commonMain n'a pas de flux paralleles.
     * La parallelisation appartiendra a l'appelant, qui sait sur quel repartiteur il
     * travaille.</p>
     */
    private fun pairs(): List<Evaluation> {
        val coarse = ArrayList<Evaluation>()
        for (i in candidates.indices) {
            for (j in i + 1 until candidates.size) {
                coarse += coarseBest(candidates[i], candidates[j])
            }
        }
        return coarse.sortedBy { it.deltaE }
            .take(PAIRS_KEPT)
            .map { refine(it.paints, PAIR_RATIOS) }
    }

    /** Meilleur des cinq dosages de reperage : sert uniquement a classer la paire. */
    private fun coarseBest(first: Candidate, second: Candidate): Evaluation {
        val pair = listOf(first, second)
        var best: Evaluation? = null
        for (weight in COARSE_WEIGHTS) {
            val color = blend(pair, doubleArrayOf(weight, 1 - weight))
            val deltaE = deltaE2000(target, color.toLab())
            if (best == null || deltaE < best.deltaE) {
                best = Evaluation(pair, intArrayOf(1, 1), color, deltaE)
            }
        }
        return best!!
    }

    // --- Trois tubes et plus -----------------------------------------------

    private fun combinations(count: Int): List<Evaluation> {
        val pool = poolFor(count)
        val ratios = RATIOS[count]
        if (pool.size < count || ratios == null) return emptyList()

        val picks = ArrayList<IntArray>()
        combine(pool.size, count, 0, 0, IntArray(count), picks)

        return picks.map { indexes -> refine(indexes.map { pool[it] }, ratios) }
    }

    /**
     * Le vivier dans lequel piocher.
     *
     * <p>Trois familles. Les tubes proches de la cible donnent la teinte generale. Les
     * tubes structurels -- le plus clair, le plus sombre, les plus colorants -- reglent
     * la valeur. Et les representants de chaque secteur de teinte garantissent qu'on
     * dispose des primaires : une selection fondee sur la seule proximite ne contiendrait
     * jamais le bleu necessaire a rompre un orange.</p>
     */
    private fun poolFor(count: Int): List<Candidate> {
        val nearest = when (count) {
            3 -> 18
            4 -> 12
            else -> 9
        }

        val pool = LinkedHashSet<Candidate>(
            candidates.sortedBy { evaluate(listOf(it), intArrayOf(1)).deltaE }.take(nearest)
        )

        // Les extremes de valeur : de quoi monter ou descendre sans changer la teinte.
        candidates.maxByOrNull { it.lab.l }?.let(pool::add)
        candidates.minByOrNull { it.lab.l }?.let(pool::add)

        candidates.sortedByDescending { it.tinting }.take(STRUCTURAL_PAINTS).forEach(pool::add)

        // Un representant par secteur de teinte, le plus franc de son secteur.
        for (sector in 0 until HUE_SECTORS) {
            val from = sector * 360.0 / HUE_SECTORS
            val to = (sector + 1) * 360.0 / HUE_SECTORS
            candidates.filter { it.hue >= from && it.hue < to }.maxByOrNull { it.chroma }?.let(pool::add)
        }
        return pool.toList()
    }

    // --- Evaluation --------------------------------------------------------

    /** Essaie tous les dosages realisables et garde le meilleur. */
    private fun refine(paints: List<Candidate>, ratios: List<IntArray>): Evaluation {
        var best: Evaluation? = null
        for (parts in ratios) {
            val evaluation = evaluate(paints, parts)
            if (best == null || evaluation.deltaE < best.deltaE) best = evaluation
        }
        return best!!
    }

    private fun evaluate(paints: List<Candidate>, parts: IntArray): Evaluation {
        // Le poids dans la couleur, c'est la dose multipliee par le pouvoir colorant.
        val weights = DoubleArray(paints.size) { parts[it] * paints[it].tinting }
        val color = blend(paints, weights)
        return Evaluation(paints, parts, color, deltaE2000(target, color.toLab()))
    }

    companion object {
        /** Nombre de paires retenues a l'issue de la passe grossiere. */
        private const val PAIRS_KEPT = 60

        /** Dosages de la passe grossiere, en part de pigment du premier tube. */
        private val COARSE_WEIGHTS = doubleArrayOf(0.15, 0.35, 0.5, 0.65, 0.85)

        /**
         * Nombre maximal de tubes par melange. Cinq, parce que certaines teintes ne
         * s'obtiennent pas autrement : les trois primaires pour la couleur, un blanc pour
         * la valeur, une terre pour rompre. Au-dela, un melange cesse d'etre
         * reproductible d'une seance a l'autre.
         */
        const val MAX_PAINTS = 5

        /** Nombre de tubes structurels ajoutes au vivier. */
        private const val STRUCTURAL_PAINTS = 4

        /** Secteurs de teinte couverts par le vivier, en degres. */
        private const val HUE_SECTORS = 6

        /**
         * En deca de cet ecart, l'oeil ne distingue plus deux teintes. Deux propositions
         * separees par moins que cela sont tenues pour equivalentes, et c'est la
         * simplicite qui les departage.
         */
        private const val PERCEPTUAL_TIE = 0.5

        /**
         * En deca de cet ecart, deux propositions donnent la meme couleur. Le seuil est
         * bien plus serre que celui de perception : deux recettes qui atteignent toutes
         * les deux la cible sont des alternatives utiles, pas des doublons.
         */
        private const val DUPLICATE_RESULT = 0.15

        private val PAIR_RATIOS: List<IntArray> = practicalPairRatios()

        /** Grilles de dosage par nombre de tubes, calculees une fois. */
        private val RATIOS: Map<Int, List<IntArray>> = ratioGrids()

        /**
         * Melange de Kubelka-Munk a deux constantes : absorption et diffusion se cumulent
         * lineairement, chacune de son cote, et c'est leur rapport qui donne la couleur.
         */
        private fun blend(paints: List<Candidate>, weights: DoubleArray): Rgb {
            val total = weights.sum()
            val absorption = DoubleArray(3)
            val scattering = DoubleArray(3)
            for (i in paints.indices) {
                val share = weights[i] / total
                val candidate = paints[i]
                for (channel in 0..2) {
                    absorption[channel] += share * candidate.absorption[channel]
                    scattering[channel] += share * candidate.scattering[channel]
                }
            }
            return Colorant.colorOf(absorption, scattering)
        }

        /** Toutes les facons de choisir count rangs distincts parmi size. */
        private fun combine(size: Int, count: Int, filled: Int, start: Int, current: IntArray, out: MutableList<IntArray>) {
            if (filled == count) {
                out += current.copyOf()
                return
            }
            for (i in start until size) {
                current[filled] = i
                combine(size, count, filled + 1, i + 1, current, out)
            }
        }

        /**
         * Rapports a deux tubes qu'on sait reellement doser : entiers premiers entre eux
         * jusqu'a huit parts, plus quelques ajouts infimes. Ces derniers comptent : une
         * pointe de bleu de Prusse dans un blanc, c'est du 1:20, et c'est un geste courant.
         */
        private fun practicalPairRatios(): List<IntArray> = buildList {
            for (a in 1..8) {
                for (b in 1..8) {
                    if (gcd(a, b) == 1) add(intArrayOf(a, b))
                }
            }
            for (tiny in intArrayOf(10, 12, 16, 20, 30)) {
                add(intArrayOf(1, tiny))
                add(intArrayOf(tiny, 1))
            }
        }

        /**
         * Grilles de dosage, du triple au quintuple. Plus il y a de tubes, plus les parts
         * restent petites : personne ne dose "sept parts de l'un, trois de l'autre, cinq
         * du troisieme et deux du quatrieme".
         */
        private fun ratioGrids(): Map<Int, List<IntArray>> = buildMap {
            for (count in 3..MAX_PAINTS) {
                val max = when (count) {
                    3 -> 5
                    4 -> 4
                    else -> 3
                }
                val grid = ArrayList<IntArray>()
                fillRatios(IntArray(count), 0, max, grid)
                put(count, grid.toList())
            }
        }

        private fun fillRatios(current: IntArray, index: Int, max: Int, out: MutableList<IntArray>) {
            if (index == current.size) {
                var divisor = current[0]
                for (part in current) divisor = gcd(divisor, part)
                // Un dosage et son double decrivent le meme melange : on ne garde que le reduit.
                if (divisor == 1) out += current.copyOf()
                return
            }
            for (part in 1..max) {
                current[index] = part
                fillRatios(current, index + 1, max, out)
            }
        }

        private tailrec fun gcd(a: Int, b: Int): Int = if (b == 0) a else gcd(b, a % b)
    }

    // --- Classement --------------------------------------------------------

    /**
     * Classe et epure les resultats.
     *
     * <p>Une seule proposition par combinaison de tubes, sinon la liste se remplit de
     * dosages voisins des deux memes tubes. A ecart perceptuellement equivalent, le
     * melange le plus simple gagne -- sans cette regle, la recherche peut repondre
     * "quatre parts de terre d'ombre brulee Winsor & Newton, cinq de la meme chez
     * Schmincke" la ou un seul tube suffisait. Enfin, les propositions qui aboutissent a
     * la meme teinte sont regroupees : le catalogue contient la meme couleur chez
     * plusieurs fabricants.</p>
     */
    private fun best(evaluations: List<Evaluation>, maxResults: Int): List<MixSuggestion> {
        val bySet = LinkedHashMap<String, Evaluation>()
        for (evaluation in evaluations) {
            val key = evaluation.paints.map { it.paint.displayName }.sorted().joinToString("|", prefix = "|")
            val previous = bySet[key]
            if (previous == null || evaluation.deltaE < previous.deltaE) bySet[key] = evaluation
        }

        val ranked = bySet.values.sortedWith(byQualityThenSimplicity)

        val distinct = ArrayList<Evaluation>()
        for (evaluation in ranked) {
            val alreadyCovered = distinct.any { deltaE2000(it.color, evaluation.color) < DUPLICATE_RESULT }
            if (!alreadyCovered) distinct += evaluation
            if (distinct.size == maxResults) break
        }
        return distinct.map(::toSuggestion)
    }

    /**
     * Ecart d'abord, mais par paliers de la taille du seuil de perception : a l'interieur
     * d'un palier, le melange le moins complique passe devant.
     */
    private val byQualityThenSimplicity = compareBy<Evaluation>(
        { (it.deltaE / PERCEPTUAL_TIE).toInt() },
        { it.paints.size },
        // Un melange en 1:2 se dose plus vite qu'un melange en 7:8, a qualite egale.
        { it.parts.sum() },
        { it.deltaE },
    )

    private fun toSuggestion(evaluation: Evaluation): MixSuggestion = MixSuggestion(
        evaluation.paints.mapIndexed { i, candidate -> PaintPart.of(candidate.paint, evaluation.parts[i]) },
        evaluation.color,
        evaluation.deltaE,
    )
}
