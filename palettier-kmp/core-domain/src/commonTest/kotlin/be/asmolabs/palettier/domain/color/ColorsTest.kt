package be.asmolabs.palettier.domain.color

import kotlin.math.abs
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * Traduction fidele de ColorsTest cote Java. Les valeurs attendues sont identiques :
 * c'est le seul moyen de savoir que le portage n'a pas deplace une virgule.
 */
class ColorsTest {

    @Test
    fun `un aller-retour hexadecimal conserve la couleur`() {
        assertEquals("#4A3427", Rgb.ofHex("#4A3427").toHex())
        assertEquals("#4A3427", Rgb.ofHex("4a3427").toHex())
        assertEquals("#FFFFFF", Rgb.ofHex("#FFF").toHex())
    }

    @Test
    fun `une couleur ne presente aucun ecart avec elle-meme`() {
        val color = Rgb.ofHex("#7A3A22")
        assertEquals(0.0, deltaE2000(color, color))
    }

    @Test
    fun `le blanc et le noir sont a l'oppose l'un de l'autre`() {
        assertTrue(deltaE2000(Rgb.ofHex("#000000"), Rgb.ofHex("#FFFFFF")) > 90)
    }

    @Test
    fun `bleu et jaune donnent du vert, pas du gris`() {
        val blue = Rgb.ofHex("#1F2E7A")
        val yellow = Rgb.ofHex("#F6C500")

        val mixed = mix(listOf(blue, yellow), listOf(1.0, 1.0))

        assertTrue(mixed.g > mixed.r, "la composante verte doit dominer dans ${mixed.toHex()}")
        assertTrue(mixed.g > mixed.b, "la composante verte doit dominer dans ${mixed.toHex()}")
    }

    @Test
    fun `melanger une couleur avec elle-meme ne la change pas`() {
        val umber = Rgb.ofHex("#4A3427")
        val mixed = mix(listOf(umber, umber), listOf(3.0, 1.0))
        assertTrue(deltaE2000(umber, mixed) < 1.0)
    }

    @Test
    fun `un poids ecrasant impose sa couleur au melange`() {
        val white = Rgb.ofHex("#F7F5F0")
        val black = Rgb.ofHex("#1C1B1A")

        val mostlyWhite = mix(listOf(white, black), listOf(99.0, 1.0))
        val halfAndHalf = mix(listOf(white, black), listOf(1.0, 1.0))

        assertTrue(mostlyWhite.relativeLuminance() > halfAndHalf.relativeLuminance())
        assertTrue(halfAndHalf.relativeLuminance() < white.relativeLuminance())
    }

    @Test
    fun `un melange sans matiere est refuse`() {
        assertFailsWith<IllegalArgumentException> { mix(emptyList(), emptyList()) }
        assertFailsWith<IllegalArgumentException> { mix(listOf(Rgb.ofHex("#000000")), listOf(0.0)) }
    }

    @Test
    fun `la moyenne d'un echantillon se fait en lumiere lineaire, pas sur le sRGB`() {
        val mid = average(listOf(Rgb.ofHex("#000000"), Rgb.ofHex("#FFFFFF")))

        // La moyenne naive donnerait #808080. En lumiere lineaire, le gris a
        // mi-luminance remonte vers #BC.
        assertEquals("#BCBCBC", mid.toHex())
    }

    @Test
    fun `moyenner une couleur avec elle-meme la laisse inchangee`() {
        val flesh = Rgb.ofHex("#C98F72")
        assertEquals(flesh.toHex(), average(listOf(flesh, flesh, flesh)).toHex())
    }

    @Test
    fun `un echantillon vide est refuse`() {
        assertFailsWith<IllegalArgumentException> { average(emptyList()) }
    }

    @Test
    fun `la correction de dominante rend neutre le point choisi comme gris`() {
        val warmGrey = Rgb.ofHex("#A08860")
        val corrected = neutralise(warmGrey, warmGrey)

        assertTrue(abs(corrected.r - corrected.g) < 0.01)
        assertTrue(abs(corrected.g - corrected.b) < 0.01)
    }

    @Test
    fun `la correction retire la dominante jaune d'une photo sous lampe chaude`() {
        val photographedFlesh = Rgb.ofHex("#D9A070")
        val photographedWhite = Rgb.ofHex("#E8D2A8")

        val corrected = neutralise(photographedFlesh, photographedWhite)

        assertTrue((corrected.r - corrected.b) < (photographedFlesh.r - photographedFlesh.b))
    }

    @Test
    fun `un point de reference deja neutre ne change rien`() {
        val flesh = Rgb.ofHex("#C98F72")
        val corrected = neutralise(flesh, Rgb.ofHex("#808080"))
        assertTrue(deltaE2000(flesh, corrected) < 0.5)
    }
}

/**
 * Le constructeur tronque au lieu de refuser, comme cote Java. Les conversions
 * lumiere lineaire / sRGB rendent regulierement des valeurs qui debordent d'un
 * milliardieme : lever une exception la-dessus casserait des calculs justes.
 */
class RgbClampTest {

    @Test
    fun `les composantes hors bornes sont tronquees, pas refusees`() {
        assertEquals("#FFFFFF", Rgb(1.0000000000000002, 1.5, 42.0).toHex())
        assertEquals("#000000", Rgb(-1e-15, -0.5, -3.0).toHex())
    }

    @Test
    fun `deux couleurs de memes composantes sont egales`() {
        assertEquals(Rgb.ofHex("#C98F72"), Rgb.ofHex("#c98f72"))
        assertEquals(Rgb.ofHex("#C98F72").hashCode(), Rgb.ofHex("#c98f72").hashCode())
    }

    @Test
    fun `la destructuration reste possible`() {
        val (r, g, b) = Rgb.ofHex("#FF8000")
        assertEquals(1.0, r)
        assertTrue(g > 0.49 && g < 0.51)
        assertEquals(0.0, b)
    }
}
