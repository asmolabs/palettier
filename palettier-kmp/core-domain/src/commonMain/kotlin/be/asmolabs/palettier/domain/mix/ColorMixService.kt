package be.asmolabs.palettier.domain.mix

import be.asmolabs.palettier.domain.color.Colorant
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import kotlin.math.roundToLong

/** Simule le melange de plusieurs huiles et propose des recettes pour atteindre une teinte. */
class ColorMixService {

    fun mix(parts: List<PaintPart>): MixResult {
        require(parts.isNotEmpty()) { "Un melange demande au moins une huile" }

        val totalParts = parts.sumOf { it.parts }
        val totalPigment = parts.sumOf { it.parts * it.paint.tintingStrength }

        // Chaque tube apporte ses propres constantes d'absorption et de diffusion : celles
        // d'un vrai Kubelka-Munk a deux constantes quand sa teinte diluee est connue,
        // celles du modele a constante unique sinon.
        val mixed = Colorant.mix(
            parts.map { it.paint.colorant() },
            parts.map { it.parts * it.paint.tintingStrength },
        )

        val components = parts.map {
            MixComponent(
                it.paint, it.parts,
                it.parts / totalParts,
                it.parts * it.paint.tintingStrength / totalPigment,
            )
        }

        val drying = parts.fold(DryingClass.FAST) { slowest, part -> slowest.slowest(part.paint.dryingClass) }
        val pigments = parts.flatMapTo(LinkedHashSet()) { it.paint.pigments }

        return MixResult(mixed, components, drying, pigments, warningsFor(components, drying))
    }

    private fun warningsFor(components: List<MixComponent>, drying: DryingClass): List<String> = buildList {
        if (components.size >= MUDDY_MIX_THRESHOLD) {
            add(
                "${components.size} huiles dans le melange : le resultat va tendre vers le gris. " +
                    "Deux ou trois suffisent presque toujours."
            )
        }

        components.filter { it.pigmentShare > 0.6 && it.volumeShare < 0.25 }.forEach {
            add(
                "${it.paint.name} occupe ${(it.volumeShare * 100).roundToLong()} % du volume mais " +
                    "${(it.pigmentShare * 100).roundToLong()} % de la couleur finale : dosez-la a la pointe du pinceau."
            )
        }

        val mixesOpacities = components.any { it.paint.opacity == Opacity.TRANSPARENT } &&
            components.any { it.paint.opacity == Opacity.OPAQUE }
        if (mixesOpacities) {
            add("Melange d'une transparente et d'une opaque : le glacis perdra sa profondeur.")
        }

        if (drying >= DryingClass.SLOW) {
            add(
                "Sechage ${drying.label.lowercase()} impose par le pigment le plus lent : " +
                    "prevoyez la couche suivante en consequence."
            )
        }

        val pigments = components.flatMapTo(LinkedHashSet()) { it.paint.pigments }
        if (pigments.size > 4) {
            add(
                "${pigments.size} pigments differents en presence : au-dela de quatre, " +
                    "la teinte devient difficile a reproduire."
            )
        }
    }

    /**
     * Cherche comment obtenir une couleur cible avec les huiles fournies.
     *
     * <p>Les dosages proposes sont des rapports d'entiers simples : l'ecart annonce est
     * celui du melange qu'on peut reellement faire, pas celui d'un optimum theorique.</p>
     */
    suspend fun suggestMixes(
        target: Rgb,
        candidates: List<Paint>,
        maxResults: Int,
        maxPaints: Int = recommendedMaxPaints(candidates.size),
    ): List<MixSuggestion> {
        if (candidates.isEmpty() || maxResults <= 0) return emptyList()
        return MixSearch(target, candidates).search(maxPaints.coerceIn(1, MAX_PAINTS), maxResults)
    }

    companion object {
        /** Au-dela de ce nombre de tubes, le melange tourne au gris quoi qu'on fasse. */
        private const val MUDDY_MIX_THRESHOLD = 4

        /** Plafond absolu, voir MixSearch.MAX_PAINTS. */
        const val MAX_PAINTS = MixSearch.MAX_PAINTS

        /** Au-dela, une palette n'est plus une palette : c'est un catalogue. */
        private const val SHORT_PALETTE = 8

        /** Seuil au-dela duquel la recherche redevient econome. */
        private const val LARGE_SELECTION = 24

        /**
         * Combien de tubes il est raisonnable d'autoriser, au vu de ce qui est disponible.
         *
         * <p>Sur une palette courte, les tubes supplementaires sont la seule facon
         * d'atteindre certaines teintes. Sur le catalogue entier, deux ou trois tubes
         * atteignent deja la cible a l'oeil pres -- en autoriser davantage coute beaucoup
         * et n'apporte rien de mesurable.</p>
         */
        fun recommendedMaxPaints(candidateCount: Int): Int = when {
            candidateCount <= SHORT_PALETTE -> MAX_PAINTS
            candidateCount <= LARGE_SELECTION -> 4
            else -> 3
        }
    }
}
