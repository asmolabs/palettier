package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.MixSuggestion

/**
 * Plan de peinture par zones.
 *
 * <p>Repartition des roles : les couleurs a viser relevent du jugement -- celui de
 * l'assistant, ou le votre ; le melange qui y conduit est calcule. L'ecart affiche est
 * donc une mesure, pas une affirmation.</p>
 */
data class PaintingPlan(
    val subject: String,
    val paletteName: String,
    val approach: String,
    val zones: List<Zone>,
) {
    /**
     * Une partie du sujet.
     *
     * @param shadows    de la plus legere a la plus profonde
     * @param highlights du premier eclairci au point lumineux
     * @param accents    variations locales, chacune nommee par l'endroit ou elle se pose
     */
    data class Zone(
        val name: String,
        val material: String,
        val note: String,
        val base: Layer?,
        val shadows: List<Layer> = emptyList(),
        val highlights: List<Layer> = emptyList(),
        val accents: List<Layer> = emptyList(),
    ) {
        /** Toutes les couches, de la plus sombre a la plus claire : l'ordre de lecture d'un degrade. */
        fun layers(): List<Layer> = buildList {
            addAll(shadows.reversed())
            base?.let { add(it) }
            addAll(highlights)
        }
    }

    /**
     * Une couche a poser.
     *
     * @param recipe   melange calcule pour l'atteindre, ou null si la palette est vide
     * @param achieved couleur reellement obtenue par ce melange
     * @param deltaE   ecart mesure entre la couleur visee et celle obtenue
     */
    data class Layer(
        val role: String,
        val target: Rgb,
        val technique: String,
        val note: String,
        val recipe: MixSuggestion?,
        val achieved: Rgb,
        val deltaE: Double,
    ) {
        /** Ce que la palette permet reellement d'atteindre pour cette couche. */
        fun reachability(): String = when {
            recipe == null -> "palette vide"
            deltaE < 2 -> "atteignable tel quel"
            deltaE < 5 -> "tres proche, ecart invisible sur la piece"
            deltaE < 12 -> "approche, a rattraper au glacis"
            else -> "hors de portee de cette palette"
        }
    }
}
