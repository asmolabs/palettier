package be.asmolabs.palettier.domain.image

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.average

/**
 * Releve une teinte autour d'un point de l'image.
 *
 * <p>Un pixel seul ne vaut rien sur une photo : le bruit du capteur et la compression
 * JPEG le deplacent d'une fois sur l'autre, et deux clics voisins sur le meme aplat
 * rendraient deux couleurs differentes. On moyenne donc un petit carre, comme le faisait
 * la version JavaFX.</p>
 *
 * <p>La moyenne se fait en lumiere lineaire -- c'est le role de [average] -- parce que
 * moyenner du sRGB assombrit. Un carre a cheval sur deux aplats donne leur melange, et
 * c'est le comportement voulu : c'est au peintre de viser le milieu d'une zone.</p>
 *
 * @param x abscisse dans les coordonnees de l'image d'origine
 * @param y ordonnee dans les coordonnees de l'image d'origine
 * @param radius rayon du carre a moyenner, en pixels ; zero pour un pixel seul
 * @return la teinte relevee, ou null si le point tombe hors de l'image
 */
fun PixelMap.sampleAround(x: Int, y: Int, radius: Int): Rgb? {
    if (isEmpty || x < 0 || y < 0 || x >= width || y >= height) return null

    val samples = ArrayList<Rgb>()
    // Le carre est ramene dans l'image plutot que rejete : un point pres du bord se releve
    // comme les autres, sur ce qui existe.
    for (row in (y - radius).coerceAtLeast(0)..(y + radius).coerceAtMost(height - 1)) {
        for (column in (x - radius).coerceAtLeast(0)..(x + radius).coerceAtMost(width - 1)) {
            samples += argb[row * width + column].toRgb()
        }
    }
    return average(samples)
}

/** Un pixel 0xAARRGGBB tel que la plateforme le rend, ramene au domaine. L'alpha est ignore. */
private fun Int.toRgb(): Rgb = Rgb(
    ((this shr 16) and 0xFF) / 255.0,
    ((this shr 8) and 0xFF) / 255.0,
    (this and 0xFF) / 255.0,
)
