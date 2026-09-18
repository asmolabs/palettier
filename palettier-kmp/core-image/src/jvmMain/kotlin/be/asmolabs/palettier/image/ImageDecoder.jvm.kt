package be.asmolabs.palettier.image

import be.asmolabs.palettier.domain.image.PixelMap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.math.max
import kotlin.math.roundToInt

/** Le decodage du bureau : ImageIO, comme dans l'application JavaFX. */
private class JvmImageDecoder : ImageDecoder {

    override suspend fun decode(bytes: ByteArray): PixelMap = withContext(Dispatchers.IO) {
        val image = read(bytes)
        val argb = IntArray(image.width * image.height)
        image.getRGB(0, 0, image.width, image.height, argb, 0, image.width)
        PixelMap(image.width, image.height, argb)
    }

    override suspend fun scaleTo(bytes: ByteArray, maxEdge: Int): ByteArray = withContext(Dispatchers.IO) {
        val source = read(bytes)
        val scale = minOf(1.0, maxEdge.toDouble() / max(source.width, source.height))
        if (scale >= 1.0) return@withContext bytes

        val width = max(1, (source.width * scale).roundToInt())
        val height = max(1, (source.height * scale).roundToInt())

        // TYPE_INT_RGB et non ARGB : la sortie est un JPEG, qui n'a pas de canal alpha.
        val reduced = BufferedImage(width, height, BufferedImage.TYPE_INT_RGB)
        reduced.createGraphics().apply {
            setRenderingHint(
                java.awt.RenderingHints.KEY_INTERPOLATION,
                java.awt.RenderingHints.VALUE_INTERPOLATION_BILINEAR,
            )
            drawImage(source, 0, 0, width, height, null)
            dispose()
        }

        ByteArrayOutputStream().also { ImageIO.write(reduced, "jpg", it) }.toByteArray()
    }

    /**
     * ImageIO signale un fichier illisible de deux facons : en rendant null, ou en levant
     * une IIOException. Les deux disent la meme chose et doivent se rattraper au meme
     * endroit -- sinon l'une des deux traverse l'ecran.
     */
    private fun read(bytes: ByteArray): BufferedImage {
        val image = try {
            ImageIO.read(ByteArrayInputStream(bytes))
        } catch (e: Exception) {
            throw IllegalArgumentException("Format d'image non reconnu", e)
        }
        return image ?: throw IllegalArgumentException("Format d'image non reconnu")
    }
}

actual fun imageDecoder(): ImageDecoder = JvmImageDecoder()
