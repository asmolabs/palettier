package be.asmolabs.palettier.image

import be.asmolabs.palettier.domain.image.PixelMap

/**
 * Le decodage d'image, seul endroit ou la plateforme perce.
 *
 * <p>Lire un JPEG n'a rien de commun entre une machine de bureau et un telephone :
 * ImageIO d'un cote, BitmapFactory de l'autre. Ce qu'on en fait ensuite -- compter les
 * teintes, relever une couleur -- est identique, et vit dans le domaine.</p>
 *
 * <p>La frontiere est volontairement etroite : un PixelMap, et rien d'autre. Tout ce qui
 * passerait de plus serait a ecrire deux fois.</p>
 */
interface ImageDecoder {

    /**
     * @param bytes le fichier tel qu'il est, non reduit
     * @throws IllegalArgumentException si le format n'est pas reconnu
     */
    suspend fun decode(bytes: ByteArray): PixelMap

    /**
     * Reduit une image a son cote maximal, en gardant ses proportions.
     *
     * <p>Une photo de telephone pleine resolution n'a pas sa place dans une base locale,
     * et la teinte dominante ne demande pas de resolution.</p>
     */
    suspend fun scaleTo(bytes: ByteArray, maxEdge: Int): ByteArray
}

/** L'implementation de la plateforme courante. */
expect fun imageDecoder(): ImageDecoder
