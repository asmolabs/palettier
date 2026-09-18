package be.asmolabs.palettier.image

import be.asmolabs.palettier.domain.image.ImagePalette
import kotlinx.coroutines.test.runTest
import java.awt.Color
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.math.abs
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * Le decodage du bureau, et ce que le domaine en tire.
 *
 * <p>Les deux ne peuvent se verifier ensemble que sur une cible ou l'on sait produire une
 * image : ici JVM. L'implementation Android compile et suit le meme contrat, mais aucun
 * essai ne la couvre -- c'est la limite assumee de cette etape.</p>
 */
class ImageDecoderTest {

    private val decoder = imageDecoder()

    /** Une image de deux aplats, dont on connait exactement les proportions. */
    private fun twoTone(left: String, right: String, leftShare: Double = 0.5): ByteArray {
        val image = BufferedImage(200, 100, BufferedImage.TYPE_INT_RGB)
        val g = image.createGraphics()
        val split = (200 * leftShare).toInt()
        g.color = Color.decode(left); g.fillRect(0, 0, split, 100)
        g.color = Color.decode(right); g.fillRect(split, 0, 200 - split, 100)
        g.dispose()
        return ByteArrayOutputStream().also { ImageIO.write(image, "png", it) }.toByteArray()
    }

    @Test
    fun `une image se decode en pixels, dimensions comprises`() = runTest {
        val pixels = decoder.decode(twoTone("#C98F72", "#8A5F4A"))

        assertEquals(200, pixels.width)
        assertEquals(100, pixels.height)
        assertEquals(200 * 100, pixels.argb.size)
        assertTrue(!pixels.isEmpty)
    }

    @Test
    fun `les deux aplats ressortent avec leurs proportions`() = runTest {
        // Trois quarts a gauche, un quart a droite.
        val dominant = ImagePalette.dominant(decoder.decode(twoTone("#C98F72", "#8A5F4A", 0.75)), 2)

        assertEquals(2, dominant.size)
        assertEquals("#C98F72", dominant.first().color.toHex())
        assertTrue(abs(dominant.first().share - 0.75) < 0.05, "part ${dominant.first().share}")
        assertEquals("#8A5F4A", dominant[1].color.toHex())
    }

    @Test
    fun `le releve est deterministe, la meme photo donne le meme plan`() = runTest {
        val image = twoTone("#C98F72", "#8A5F4A")
        val once = ImagePalette.dominant(decoder.decode(image), 3)
        val twice = ImagePalette.dominant(decoder.decode(image), 3)

        assertEquals(once.map { it.color.toHex() }, twice.map { it.color.toHex() })
    }

    @Test
    fun `une image trop grande est reduite, les petites sont laissees telles quelles`() = runTest {
        val big = BufferedImage(2400, 1800, BufferedImage.TYPE_INT_RGB).let { image ->
            image.createGraphics().apply { color = Color.ORANGE; fillRect(0, 0, 2400, 1800); dispose() }
            ByteArrayOutputStream().also { ImageIO.write(image, "png", it) }.toByteArray()
        }

        val reduced = decoder.decode(decoder.scaleTo(big, 1024))
        assertEquals(1024, reduced.width)
        assertEquals(768, reduced.height)

        // Une image deja petite ne gagne rien a etre reencodee : on rend l'original.
        val small = twoTone("#C98F72", "#8A5F4A")
        assertTrue(decoder.scaleTo(small, 1024) === small)
    }

    @Test
    fun `un fichier qui n'est pas une image est refuse clairement`() = runTest {
        assertFailsWith<IllegalArgumentException> { decoder.decode("pas une image".encodeToByteArray()) }
    }
}
