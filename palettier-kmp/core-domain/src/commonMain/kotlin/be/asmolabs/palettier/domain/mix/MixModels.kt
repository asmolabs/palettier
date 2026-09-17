package be.asmolabs.palettier.domain.mix

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Paint
import kotlin.math.floor

/**
 * Une huile et sa dose dans le melange, exprimee en parts (gouttes, pointes de pinceau,
 * peu importe l'unite tant qu'elle est la meme pour tout le melange).
 */
data class PaintPart(val paint: Paint, val parts: Double) {
    init {
        require(parts > 0) { "La dose doit etre strictement positive" }
    }

    companion object {
        fun of(paint: Paint, parts: Number) = PaintPart(paint, parts.toDouble())
    }
}

/**
 * Contribution d'une huile au melange.
 *
 * @param volumeShare  part en volume, entre 0 et 1
 * @param pigmentShare part dans la couleur finale une fois le pouvoir colorant pris en compte
 */
data class MixComponent(
    val paint: Paint,
    val parts: Double,
    val volumeShare: Double,
    val pigmentShare: Double,
)

/** Resultat complet d'un melange. */
data class MixResult(
    val color: Rgb,
    val components: List<MixComponent>,
    val dryingClass: DryingClass,
    val pigments: Set<String>,
    val warnings: List<String>,
) {
    val hex: String get() = color.toHex()
}

/** Une proposition de melange pour approcher une couleur cible. */
data class MixSuggestion(
    val parts: List<PaintPart>,
    val color: Rgb,
    val deltaE: Double,
) {
    /**
     * Vitesse de sechage du melange : celle de son tube le plus lent. Un cadmium glisse
     * dans un point lumineux impose plusieurs jours a lui seul, quelle que soit sa dose.
     */
    fun dryingClass(): DryingClass =
        parts.fold(DryingClass.FAST) { slowest, part -> slowest.slowest(part.paint.dryingClass) }

    /** Description lisible du type "3 parts de X + 1 part de Y". */
    fun describe(): String = parts.joinToString("  +  ") { part ->
        val amount = if (part.parts == floor(part.parts)) part.parts.toLong().toString() else part.parts.toString()
        "$amount part${if (part.parts > 1) "s" else ""} de ${part.paint.displayName}"
    }
}

/** Une huile du catalogue et son ecart a une couleur cible. */
data class PaintMatch(val paint: Paint, val deltaE: Double) {

    /** Traduction du delta E en langage de peintre. */
    fun verdict(): String = when (floor(deltaE).toInt()) {
        0, 1 -> "Identique a l'oeil"
        2, 3 -> "Tres proche"
        in 4..7 -> "Proche, ecart visible cote a cote"
        in 8..14 -> "Meme famille de teinte"
        else -> "Couleur differente"
    }
}
