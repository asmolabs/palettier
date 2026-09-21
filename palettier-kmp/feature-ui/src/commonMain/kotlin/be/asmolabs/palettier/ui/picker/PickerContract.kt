package be.asmolabs.palettier.ui.picker

import be.asmolabs.palettier.domain.paint.PaintMatchByColour
import be.asmolabs.palettier.domain.project.DominantColour

/**
 * La photo affichee : ses octets d'origine, et ce qu'il faut pour placer un clic.
 *
 * <p>Classe ordinaire et non data class, volontairement. L'etat est un StateFlow, qui
 * compare chaque emission a la precedente ; une egalite structurelle comparerait les
 * millions d'octets de la photo a chaque frappe au clavier. Ici deux photos sont la meme
 * si c'est le meme objet, ce qui est exactement ce que l'ecran a besoin de savoir.</p>
 */
class LoadedImage(val bytes: ByteArray, val width: Int, val height: Int)

/** Le point releve, dans les coordonnees de l'image d'origine. */
data class PickPoint(val x: Int, val y: Int)

data class PickerUiState(
    val loading: Boolean = false,
    /** La photo a montrer, telle qu'elle a ete ouverte. */
    val image: LoadedImage? = null,
    /** Teintes relevees sur la photo, de la plus presente a la moins presente. */
    val dominant: List<DominantColour> = emptyList(),
    val selectedHex: String? = null,
    /** Ou le dernier releve a ete pris, pour le marquer sur la photo. */
    val pickedAt: PickPoint? = null,
    /** Rayon du carre moyenne autour du clic, en pixels de l'image. */
    val sampleRadius: Int = DEFAULT_RADIUS,
    /** Vrai quand le prochain clic designe le gris neutre au lieu de la teinte visee. */
    val pickingGrey: Boolean = false,
    /** Teinte du point choisi comme gris neutre, quand le peintre en a designe un. */
    val greyPointHex: String? = null,
    /** La teinte retenue une fois la dominante de la photo annulee. */
    val correctedHex: String? = null,
    /** Les tubes du catalogue les plus proches de la teinte retenue. */
    val closest: List<PaintMatchByColour> = emptyList(),
    val onlyInStock: Boolean = true,
    val error: String? = null,
) {
    /** La teinte sur laquelle tout le reste travaille : corrigee si elle l'a ete. */
    val workingHex: String? get() = correctedHex ?: selectedHex

    companion object {
        /** Trois pixels de rayon : assez pour noyer le bruit, trop peu pour deborder. */
        const val DEFAULT_RADIUS = 3
    }
}

sealed interface PickerIntent {
    /** Une photo choisie par la plateforme : le domaine ne connait que ses octets. */
    data class Load(val bytes: ByteArray) : PickerIntent

    /**
     * Un clic sur la photo, dans les coordonnees de l'image d'origine.
     *
     * <p>L'ecran convertit lui-meme la position du clic, parce qu'il est le seul a savoir
     * a quel zoom il affiche. Le releve, lui, se fait toujours sur les pixels du fichier
     * ouvert : une image mise a l'echelle pour l'affichage a des couleurs interpolees, et
     * relever dessus reviendrait a mesurer le travail de la carte graphique.</p>
     */
    data class PickAt(val x: Int, val y: Int) : PickerIntent

    data class Select(val hex: String) : PickerIntent

    /** Designe une teinte censee etre grise, pour annuler la dominante de la photo. */
    data class UseAsGrey(val hex: String?) : PickerIntent

    /** Arme ou desarme le prochain clic pour designer le gris neutre. */
    data class PickGrey(val arming: Boolean) : PickerIntent

    data class SetRadius(val radius: Int) : PickerIntent

    data class OnlyInStock(val only: Boolean) : PickerIntent
}
