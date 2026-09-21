package be.asmolabs.palettier.domain.image

import be.asmolabs.palettier.domain.color.Rgb
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SamplingTest {

    /** Une image de deux aplats, le premier a gauche sur [split] colonnes. */
    private fun twoTone(width: Int, height: Int, split: Int, left: Int, right: Int): PixelMap {
        val argb = IntArray(width * height) { index ->
            if (index % width < split) left else right
        }
        return PixelMap(width, height, argb)
    }

    private val ochre = 0xFFC98F72.toInt()
    private val umber = 0xFF4A3427.toInt()

    @Test
    fun `un aplat rend sa couleur, quel que soit le rayon`() {
        val image = twoTone(20, 20, 20, ochre, ochre)

        assertEquals("#C98F72", image.sampleAround(10, 10, 0)?.toHex())
        assertEquals("#C98F72", image.sampleAround(10, 10, 5)?.toHex())
    }

    @Test
    fun `un point hors de l'image ne rend rien`() {
        val image = twoTone(20, 20, 20, ochre, ochre)

        assertNull(image.sampleAround(-1, 10, 1))
        assertNull(image.sampleAround(20, 10, 1))
        assertNull(image.sampleAround(10, 20, 1))
    }

    @Test
    fun `au bord, le carre est ramene dans l'image plutot que rejete`() {
        val image = twoTone(20, 20, 20, ochre, ochre)

        // Le coin : les trois quarts du carre tombent dehors, le releve vaut quand meme.
        assertEquals("#C98F72", image.sampleAround(0, 0, 4)?.toHex())
    }

    @Test
    fun `a cheval sur deux aplats, le releve donne leur melange`() {
        val image = twoTone(20, 20, 10, ochre, umber)

        // Le carre couvre les colonnes 7 a 13 : trois d'ocre, quatre de terre d'ombre.
        val mixed = image.sampleAround(10, 10, 3)?.toHex()

        // Ni l'un ni l'autre, et plus proche de la terre d'ombre qui l'emporte en surface.
        assertEquals("#926852", mixed)
        val sample = image.sampleAround(10, 10, 3)!!
        assertTrue(sample.r > Rgb.ofHex("#4A3427").r && sample.r < Rgb.ofHex("#C98F72").r)
    }

    @Test
    fun `la moyenne se fait en lumiere lineaire, pas en sRGB`() {
        // Noir et blanc a parts egales. En sRGB la moyenne donnerait #808080 ; en lumiere
        // lineaire elle donne le gris a 50 % de luminance, nettement plus clair.
        val image = twoTone(2, 1, 1, 0xFF000000.toInt(), 0xFFFFFFFF.toInt())

        assertEquals("#BCBCBC", image.sampleAround(0, 0, 1)?.toHex())
    }

    @Test
    fun `une image vide ne rend rien`() {
        assertNull(PixelMap(0, 0, IntArray(0)).sampleAround(0, 0, 1))
    }
}
