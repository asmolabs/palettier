package be.asmolabs.palettier.domain.paint

import be.asmolabs.palettier.domain.color.Colorant
import be.asmolabs.palettier.domain.color.Rgb

/**
 * Un pigment du Colour Index, avec les proprietes qui en decoulent.
 *
 * <p>C'est la piece centrale du catalogue : la vitesse de sechage et le pouvoir colorant
 * d'une huile ne sont pas des choix de marque, ce sont des proprietes du pigment. Une
 * terre d'ombre (PBr7) seche vite chez tout le monde, un cadmium (PY35) seche lentement
 * chez tout le monde. Les stocker une fois ici evite de les re-deviner pour chaque tube
 * de chaque gamme.</p>
 *
 * @param code            code Colour Index, par exemple PBr7 ou PW6
 * @param color           teinte en ton de masse, indicative
 * @param tintingStrength pouvoir colorant relatif, entre 0,05 et 1
 * @param tint            couleur du pigment coupe de blanc, ou null si elle n'a pas ete
 *                        relevee. Quand elle l'est, le melange passe au Kubelka-Munk a
 *                        deux constantes, seul capable de representer un pigment dont le
 *                        ton de masse et la teinte diluee divergent.
 */
data class Pigment(
    val code: String,
    val name: String,
    val color: Rgb,
    val opacity: Opacity,
    val dryingClass: DryingClass,
    val tintingStrength: Double,
    val tint: Rgb? = null,
) {

    /** Constantes de melange du pigment, a deux constantes si sa teinte diluee est connue. */
    fun colorant(): Colorant =
        if (tint == null) {
            Colorant.ofMasstone(color)
        } else {
            Colorant.ofMasstoneAndTint(color, tint, Colorant.REFERENCE_TINT_CONCENTRATION)
        }

    companion object {
        /** Pigment de repli, utilise quand un code n'est pas encore reference. */
        fun unknown(code: String) = Pigment(
            code = code,
            name = "Pigment non reference",
            color = Rgb(0.5, 0.5, 0.5),
            opacity = Opacity.SEMI_OPAQUE,
            dryingClass = DryingClass.MEDIUM,
            tintingStrength = 0.5,
            tint = null,
        )
    }
}
