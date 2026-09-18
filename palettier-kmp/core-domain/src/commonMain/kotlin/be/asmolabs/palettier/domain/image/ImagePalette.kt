package be.asmolabs.palettier.domain.image

import be.asmolabs.palettier.domain.color.Lab
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.color.linearToSrgb
import be.asmolabs.palettier.domain.color.srgbToLinear
import be.asmolabs.palettier.domain.color.toLab
import be.asmolabs.palettier.domain.project.DominantColour

/**
 * Une image reduite a ce que le domaine en a besoin : des pixels.
 *
 * <p>Decoder un JPEG n'a rien de commun d'une plateforme a l'autre, mais compter des
 * teintes, si. La frontiere passe donc ici : la plateforme rend un PixelMap, le domaine
 * fait le reste -- et ne le fait qu'une fois.</p>
 *
 * @param argb un entier par pixel, au format 0xAARRGGBB, en lecture ligne par ligne
 */
data class PixelMap(val width: Int, val height: Int, val argb: IntArray) {
    init {
        require(width >= 0 && height >= 0) { "Dimensions negatives" }
        require(argb.size == width * height) { "${argb.size} pixels pour ${width}x$height" }
    }

    val isEmpty: Boolean get() = argb.isEmpty()

    // IntArray casse l'egalite structurelle d'une data class : deux images sont les memes
    // si elles ont les memes pixels, pas la meme reference.
    override fun equals(other: Any?) = other is PixelMap &&
        width == other.width && height == other.height && argb.contentEquals(other.argb)

    override fun hashCode() = (width * 31 + height) * 31 + argb.contentHashCode()
}

/**
 * Les teintes reellement presentes sur une photo.
 *
 * <p>Sert a fonder un plan sur ce qu'on voit plutot que sur ce qu'on imagine. Un modele
 * de langage a qui l'on montre une photo de carnation propose volontiers des couleurs de
 * manuel ; lui donner les teintes mesurees l'oblige a travailler sur la piece qu'il a
 * sous les yeux.</p>
 *
 * <p>Le regroupement se fait par distance perceptuelle en L*a*b*, parce qu'un ecart de
 * dix unites y signifie la meme chose dans les ombres et dans les clairs, ce qui n'est
 * pas le cas en RVB. Les representants, eux, restent des moyennes en lumiere lineaire :
 * moyenner du sRGB assombrit.</p>
 */
object ImagePalette {

    /** Cote maximal avant analyse : la teinte dominante ne demande pas de resolution. */
    private const val ANALYSIS_EDGE = 160

    /** Nombre d'iterations : au-dela, les groupes ne bougent plus. */
    private const val ITERATIONS = 12

    /**
     * Extrait les teintes dominantes d'une image.
     *
     * @param count nombre de teintes souhaitees
     * @return les teintes, de la plus presente a la moins presente
     */
    fun dominant(image: PixelMap, count: Int): List<DominantColour> {
        val pixels = linearPixels(image)
        if (pixels.isEmpty() || count < 1) return emptyList()

        val groups = minOf(count, pixels.size)
        val centres = seed(pixels, groups)
        val assignment = IntArray(pixels.size)

        repeat(ITERATIONS) {
            val centreLab = centres.map { it.toLabPoint() }
            var moved = false

            for (i in pixels.indices) {
                val best = nearest(pixels[i], centreLab)
                if (assignment[i] != best) {
                    assignment[i] = best
                    moved = true
                }
            }
            recentre(pixels, assignment, centres)
            if (!moved) return@repeat
        }

        return summarise(pixels, assignment, centres)
    }

    /** Pixels en lumiere lineaire : c'est la qu'une moyenne a un sens. */
    private fun linearPixels(image: PixelMap): List<DoubleArray> {
        if (image.isEmpty) return emptyList()
        val step = maxOf(1, maxOf(image.width, image.height) / ANALYSIS_EDGE)

        val pixels = ArrayList<DoubleArray>()
        var y = 0
        while (y < image.height) {
            var x = 0
            while (x < image.width) {
                val packed = image.argb[y * image.width + x]
                pixels += doubleArrayOf(
                    srgbToLinear(((packed shr 16) and 0xFF) / 255.0),
                    srgbToLinear(((packed shr 8) and 0xFF) / 255.0),
                    srgbToLinear((packed and 0xFF) / 255.0),
                )
                x += step
            }
            y += step
        }
        return pixels
    }

    /**
     * Points de depart etales : le premier pixel, puis a chaque fois le plus eloigne de
     * ceux deja retenus. Deterministe, contrairement a un tirage au sort -- ce qui evite
     * qu'une meme photo donne deux plans differents.
     */
    private fun seed(pixels: List<DoubleArray>, groups: Int): MutableList<DoubleArray> {
        val centres = ArrayList<DoubleArray>(groups)
        centres += pixels[0].copyOf()

        while (centres.size < groups) {
            val chosen = centres.map { it.toLabPoint() }
            var farthest = 0
            var worst = -1.0
            for (i in pixels.indices) {
                val lab = pixels[i].toLabPoint()
                val nearest = chosen.minOf { deltaE2000(lab, it) }
                if (nearest > worst) {
                    worst = nearest
                    farthest = i
                }
            }
            centres += pixels[farthest].copyOf()
        }
        return centres
    }

    private fun nearest(pixel: DoubleArray, centres: List<Lab>): Int {
        val lab = pixel.toLabPoint()
        var best = 0
        var shortest = Double.MAX_VALUE
        for (i in centres.indices) {
            val distance = deltaE2000(lab, centres[i])
            if (distance < shortest) {
                shortest = distance
                best = i
            }
        }
        return best
    }

    private fun recentre(pixels: List<DoubleArray>, assignment: IntArray, centres: MutableList<DoubleArray>) {
        val sums = Array(centres.size) { DoubleArray(3) }
        val counts = IntArray(centres.size)

        for (i in pixels.indices) {
            val group = assignment[i]
            counts[group]++
            for (channel in 0..2) sums[group][channel] += pixels[i][channel]
        }
        for (group in centres.indices) {
            if (counts[group] > 0) {
                for (channel in 0..2) centres[group][channel] = sums[group][channel] / counts[group]
            }
        }
    }

    private fun summarise(
        pixels: List<DoubleArray>,
        assignment: IntArray,
        centres: List<DoubleArray>,
    ): List<DominantColour> {
        val counts = IntArray(centres.size)
        for (group in assignment) counts[group]++

        return centres.indices
            .filter { counts[it] > 0 }
            .map { DominantColour(centres[it].toRgb(), counts[it].toDouble() / pixels.size) }
            .sortedByDescending { it.share }
    }

    private fun DoubleArray.toRgb() = Rgb(linearToSrgb(this[0]), linearToSrgb(this[1]), linearToSrgb(this[2]))
    private fun DoubleArray.toLabPoint(): Lab = toRgb().toLab()
}
