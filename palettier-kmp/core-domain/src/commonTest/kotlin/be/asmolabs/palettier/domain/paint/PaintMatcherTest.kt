package be.asmolabs.palettier.domain.paint

import be.asmolabs.palettier.domain.color.Rgb
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class PaintMatcherTest {

    private val matcher = PaintMatcher()

    private fun paint(brand: String, name: String, hex: String = "#4A3427") =
        Paint(id = name.hashCode().toLong(), brand = brand, name = name, hexColor = hex)

    private val catalogue = listOf(
        paint("Winsor & Newton", "Burnt Umber", "#4A3427"),
        paint("Winsor & Newton", "Winsor Lemon", "#F5E04B"),
        paint("Winsor & Newton", "Titanium White", "#F4F2EC"),
        paint("Gamblin", "Burnt Umber", "#4C3628"),
        paint("Schmincke Norma", "Terre d'ombre brulee", "#4B3529"),
    )

    @Test
    fun `un libelle complet retrouve le tube, bruit compris`() {
        val match = matcher.match("W&N Burnt Umber 37ml", catalogue)

        assertEquals("Burnt Umber", match?.paint?.name)
        assertTrue(match!!.isReliable, "confiance ${match.confidence}")
    }

    @Test
    fun `une marque seule ne designe aucun tube`() {
        // Sans cette regle, "Winsor & Newton" choisirait "Winsor Lemon" avec assurance,
        // puisque le mot se retrouve dans le nom.
        assertNull(matcher.match("Winsor & Newton", catalogue))
    }

    @Test
    fun `les accents ne font pas echouer la reconnaissance`() {
        val match = matcher.match("Terre d'ombre brûlée", catalogue)

        assertEquals("Terre d'ombre brulee", match?.paint?.name)
    }

    @Test
    fun `un libelle qui ne ressemble a rien n'associe rien`() {
        assertNull(matcher.match("pinceau martre numero 2", catalogue))
        assertNull(matcher.match("", catalogue))
        assertNull(matcher.match(null, catalogue))
    }

    @Test
    fun `les huiles se classent par ecart a une couleur visee`() {
        val closest = findClosest(Rgb.ofHex("#4A3427"), catalogue, 2)

        assertEquals(2, closest.size)
        assertTrue(closest[0].deltaE <= closest[1].deltaE)
        assertEquals("Burnt Umber", closest.first().paint.name)
    }
}
