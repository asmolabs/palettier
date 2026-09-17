package be.asmolabs.palettier.domain.color

import kotlin.math.roundToInt

/**
 * Couleur sRGB, chaque composante exprimee dans l'intervalle [0, 1].
 *
 * <p>Les composantes sont tronquees a l'intervalle plutot que refusees, exactement comme
 * le constructeur compact cote Java. Ce n'est pas de la complaisance : les conversions
 * lumiere lineaire / sRGB rendent des valeurs qui debordent d'un milliardieme, et lever
 * une exception la-dessus casserait des calculs parfaitement justes.</p>
 */
class Rgb private constructor(val r: Double, val g: Double, val b: Double) {

    fun toHex(): String = "#" + listOf(r, g, b).joinToString("") { channel ->
        (channel * 255.0).roundToInt().toString(16).uppercase().padStart(2, '0')
    }

    /** Luminance relative (WCAG), utile pour choisir une couleur de texte lisible sur la pastille. */
    fun relativeLuminance(): Double =
        0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b)

    override fun equals(other: Any?) = other is Rgb && r == other.r && g == other.g && b == other.b
    override fun hashCode() = (r.hashCode() * 31 + g.hashCode()) * 31 + b.hashCode()
    override fun toString() = toHex()

    operator fun component1() = r
    operator fun component2() = g
    operator fun component3() = b

    companion object {
        /** Seule voie de construction : elle tronque, comme le constructeur compact Java. */
        operator fun invoke(r: Double, g: Double, b: Double) =
            Rgb(r.coerceIn(0.0, 1.0), g.coerceIn(0.0, 1.0), b.coerceIn(0.0, 1.0))

        fun ofHex(hex: String): Rgb {
            val raw = hex.removePrefix("#")
            val value = when (raw.length) {
                3 -> raw.flatMap { listOf(it, it) }.joinToString("")
                6 -> raw
                else -> throw IllegalArgumentException("Code hexadecimal invalide : $hex")
            }
            val packed = value.toIntOrNull(16)
                ?: throw IllegalArgumentException("Code hexadecimal invalide : $hex")
            return Rgb(
                ((packed shr 16) and 0xFF) / 255.0,
                ((packed shr 8) and 0xFF) / 255.0,
                (packed and 0xFF) / 255.0,
            )
        }
    }
}

/** Couleur dans l'espace CIE L*a*b* (illuminant D65, observateur 2 degres). */
data class Lab(val l: Double, val a: Double, val b: Double)
