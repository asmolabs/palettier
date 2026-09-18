package be.asmolabs.palettier.image

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import be.asmolabs.palettier.domain.image.PixelMap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import kotlin.math.max
import kotlin.math.roundToInt

/**
 * Le decodage Android : BitmapFactory.
 *
 * <p>Les bitmaps sont recycles des qu'on a fini : sur un telephone, une photo pleine
 * resolution se compte en dizaines de megaoctets, et le ramasse-miettes arrive trop tard
 * pour eviter l'etouffement.</p>
 */
private class AndroidImageDecoder : ImageDecoder {

    override suspend fun decode(bytes: ByteArray): PixelMap = withContext(Dispatchers.IO) {
        val bitmap = read(bytes)
        try {
            val argb = IntArray(bitmap.width * bitmap.height)
            bitmap.getPixels(argb, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
            PixelMap(bitmap.width, bitmap.height, argb)
        } finally {
            bitmap.recycle()
        }
    }

    override suspend fun scaleTo(bytes: ByteArray, maxEdge: Int): ByteArray = withContext(Dispatchers.IO) {
        val source = read(bytes)
        try {
            val scale = minOf(1.0, maxEdge.toDouble() / max(source.width, source.height))
            if (scale >= 1.0) return@withContext bytes

            val width = max(1, (source.width * scale).roundToInt())
            val height = max(1, (source.height * scale).roundToInt())
            val reduced = Bitmap.createScaledBitmap(source, width, height, true)
            try {
                ByteArrayOutputStream()
                    .also { reduced.compress(Bitmap.CompressFormat.JPEG, 85, it) }
                    .toByteArray()
            } finally {
                if (reduced !== source) reduced.recycle()
            }
        } finally {
            source.recycle()
        }
    }

    private fun read(bytes: ByteArray): Bitmap =
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
            ?: throw IllegalArgumentException("Format d'image non reconnu")
}

actual fun imageDecoder(): ImageDecoder = AndroidImageDecoder()
