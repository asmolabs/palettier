package be.asmolabs.palettier.domain.paint

import be.asmolabs.palettier.domain.color.Colorant
import be.asmolabs.palettier.domain.color.Rgb

/**
 * Un tube d'huile du catalogue ou de l'etagere du peintre.
 *
 * <p>Le pendant de l'entite JPA OilPaint, debarrasse de la persistance. Les champs qui
 * relevaient de la base -- l'identifiant, l'origine de la fiche -- restent presents :
 * ils portent du sens metier (une fiche saisie a la main ne se retire jamais du menage,
 * un tube renumerote garde son ancienne reference).</p>
 */
data class Paint(
    val id: Long? = null,
    val brand: String,
    val name: String,
    val code: String = "",
    /** Reference sous laquelle le fabricant vendait le meme tube avant une renumerotation. */
    val legacyCode: String = "",
    /** Codes normalises des pigments, par exemple PBr7 ou PW6. */
    val pigments: Set<String> = emptySet(),
    val hexColor: String,
    /**
     * Couleur de l'huile coupee de blanc, si elle a ete relevee. Sa presence fait
     * basculer les melanges vers le Kubelka-Munk a deux constantes.
     */
    val tintHex: String? = null,
    val opacity: Opacity = Opacity.SEMI_OPAQUE,
    val dryingClass: DryingClass = DryingClass.MEDIUM,
    /** Pouvoir colorant relatif, entre 0,05 et 1. */
    val tintingStrength: Double = 0.5,
    /** Vrai quand la teinte est deduite des pigments plutot que relevee. */
    val colorDerived: Boolean = false,
    /** Vrai quand les pigments viennent du fabricant, et non d'une reconstitution. */
    val pigmentsVerified: Boolean = false,
    /** Vrai pour un tube saisi par le peintre, et non livre avec l'application. */
    val userAdded: Boolean = false,
    /** Vrai si le tube est effectivement sur l'etagere du peintre. */
    val inStock: Boolean = true,
    val notes: String = "",
) {

    val color: Rgb get() = Rgb.ofHex(hexColor)

    /** Libelle court pour les listes deroulantes : "Marque - Nom". */
    val displayName: String get() = "$brand - $name"

    /**
     * Constantes de melange du tube. A deux constantes si sa teinte diluee est connue,
     * a constante unique sinon.
     */
    fun colorant(): Colorant =
        if (tintHex.isNullOrBlank()) {
            Colorant.ofMasstone(color)
        } else {
            Colorant.ofMasstoneAndTint(color, Rgb.ofHex(tintHex), Colorant.REFERENCE_TINT_CONCENTRATION)
        }
}
