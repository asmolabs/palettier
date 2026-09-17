package be.asmolabs.palettier.domain.color

import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.atan2
import kotlin.math.cbrt
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.hypot
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.sqrt

/**
 * Conversions colorimetriques et melange soustractif.
 *
 * <p>Le melange n'est volontairement pas une moyenne RGB : deux peintures qui se
 * melangent se comportent comme des milieux diffusants. On passe donc par le modele de
 * Kubelka-Munk a constante unique, ou le rapport absorption/diffusion K/S est additif.
 * C'est ce qui fait que bleu + jaune donne du vert, et non du gris.</p>
 */

// --- sRGB <-> lineaire ------------------------------------------------------

fun srgbToLinear(c: Double): Double =
    if (c <= 0.04045) c / 12.92 else ((c + 0.055) / 1.055).pow(2.4)

fun linearToSrgb(c: Double): Double {
    val v = c.coerceIn(0.0, 1.0)
    return if (v <= 0.0031308) v * 12.92 else 1.055 * v.pow(1 / 2.4) - 0.055
}

// --- sRGB -> Lab ------------------------------------------------------------

private const val XN = 0.95047
private const val YN = 1.00000
private const val ZN = 1.08883

fun Rgb.toLab(): Lab {
    val lr = srgbToLinear(r)
    val lg = srgbToLinear(g)
    val lb = srgbToLinear(b)

    val x = (0.4124564 * lr + 0.3575761 * lg + 0.1804375 * lb) / XN
    val y = (0.2126729 * lr + 0.7151522 * lg + 0.0721750 * lb) / YN
    val z = (0.0193339 * lr + 0.1191920 * lg + 0.9503041 * lb) / ZN

    val fx = pivot(x)
    val fy = pivot(y)
    val fz = pivot(z)

    return Lab(116 * fy - 16, 500 * (fx - fy), 200 * (fy - fz))
}

private fun pivot(t: Double): Double =
    if (t > 216.0 / 24389.0) cbrt(t) else (24389.0 / 27.0 * t + 16) / 116.0

private fun Double.toRadians(): Double = this * PI / 180.0
private fun Double.toDegrees(): Double = this * 180.0 / PI

// --- Ecart percu ------------------------------------------------------------

/**
 * Ecart de couleur CIEDE2000. Reperes d'interpretation : moins de 1 = indiscernable,
 * moins de 2 = ecart visible seulement en comparaison cote a cote, plus de 5 = deux
 * couleurs clairement differentes.
 */
fun deltaE2000(first: Rgb, second: Rgb): Double = deltaE2000(first.toLab(), second.toLab())

fun deltaE2000(lab1: Lab, lab2: Lab): Double {
    val kL = 1.0
    val kC = 1.0
    val kH = 1.0

    val c1 = hypot(lab1.a, lab1.b)
    val c2 = hypot(lab2.a, lab2.b)
    val cBar = (c1 + c2) / 2.0

    val cBar7 = cBar.pow(7)
    val g = 0.5 * (1 - sqrt(cBar7 / (cBar7 + 25.0.pow(7))))

    val a1p = (1 + g) * lab1.a
    val a2p = (1 + g) * lab2.a
    val c1p = hypot(a1p, lab1.b)
    val c2p = hypot(a2p, lab2.b)

    val h1p = hueAngle(lab1.b, a1p)
    val h2p = hueAngle(lab2.b, a2p)

    val dLp = lab2.l - lab1.l
    val dCp = c2p - c1p

    val dhp = when {
        c1p * c2p == 0.0 -> 0.0
        abs(h2p - h1p) <= 180 -> h2p - h1p
        h2p - h1p > 180 -> h2p - h1p - 360
        else -> h2p - h1p + 360
    }
    val dHp = 2 * sqrt(c1p * c2p) * sin(dhp.toRadians() / 2)

    val lBarP = (lab1.l + lab2.l) / 2
    val cBarP = (c1p + c2p) / 2

    val hBarP = when {
        c1p * c2p == 0.0 -> h1p + h2p
        abs(h1p - h2p) <= 180 -> (h1p + h2p) / 2
        h1p + h2p < 360 -> (h1p + h2p + 360) / 2
        else -> (h1p + h2p - 360) / 2
    }

    val t = 1 -
        0.17 * cos((hBarP - 30).toRadians()) +
        0.24 * cos((2 * hBarP).toRadians()) +
        0.32 * cos((3 * hBarP + 6).toRadians()) -
        0.20 * cos((4 * hBarP - 63).toRadians())

    val dTheta = 30 * exp(-((hBarP - 275) / 25).pow(2))
    val cBarP7 = cBarP.pow(7)
    val rC = 2 * sqrt(cBarP7 / (cBarP7 + 25.0.pow(7)))
    val rT = -rC * sin(2 * dTheta.toRadians())

    val lBarP50 = (lBarP - 50).pow(2)
    val sL = 1 + (0.015 * lBarP50) / sqrt(20 + lBarP50)
    val sC = 1 + 0.045 * cBarP
    val sH = 1 + 0.015 * cBarP * t

    val termL = dLp / (kL * sL)
    val termC = dCp / (kC * sC)
    val termH = dHp / (kH * sH)

    return sqrt(termL * termL + termC * termC + termH * termH + rT * termC * termH)
}

private fun hueAngle(b: Double, ap: Double): Double {
    if (b == 0.0 && ap == 0.0) return 0.0
    val deg = atan2(b, ap).toDegrees()
    return if (deg >= 0) deg else deg + 360
}

// --- Echantillonnage d'image ------------------------------------------------

/**
 * Moyenne d'un ensemble de pixels.
 *
 * <p>La moyenne est calculee en lumiere lineaire, pas sur les valeurs sRGB : le sRGB est
 * encode en gamma, et en moyenner les valeurs directement assombrit le resultat. Sur un
 * degrade de carnation, l'ecart est nettement visible.</p>
 */
fun average(samples: Collection<Rgb>): Rgb {
    require(samples.isNotEmpty()) { "Echantillon vide" }
    var r = 0.0
    var g = 0.0
    var b = 0.0
    for (sample in samples) {
        r += srgbToLinear(sample.r)
        g += srgbToLinear(sample.g)
        b += srgbToLinear(sample.b)
    }
    val count = samples.size
    return Rgb(linearToSrgb(r / count), linearToSrgb(g / count), linearToSrgb(b / count))
}

/**
 * Corrige la dominante coloree d'une photo.
 *
 * <p>Une photo prise sous lampe de bureau tire au jaune, sous LED froide au bleu : la
 * couleur relevee n'est alors pas celle de la peinture. En designant un point de l'image
 * cense etre gris neutre, on obtient le gain a appliquer a chaque canal pour annuler
 * cette dominante.</p>
 */
fun neutralise(sample: Rgb, greyPoint: Rgb): Rgb {
    val gr = maxOf(srgbToLinear(greyPoint.r), 1e-4)
    val gg = maxOf(srgbToLinear(greyPoint.g), 1e-4)
    val gb = maxOf(srgbToLinear(greyPoint.b), 1e-4)

    // On vise la luminosite moyenne du point de reference : la correction change la
    // teinte, pas l'exposition.
    val target = (gr + gg + gb) / 3.0

    return Rgb(
        linearToSrgb(srgbToLinear(sample.r) * target / gr),
        linearToSrgb(srgbToLinear(sample.g) * target / gg),
        linearToSrgb(srgbToLinear(sample.b) * target / gb),
    )
}

// --- Melange soustractif (Kubelka-Munk, constante unique) -------------------

/**
 * Melange des couleurs ponderees. Les poids representent la quantite de matiere
 * effectivement deposee (dose x pouvoir colorant) ; ils sont normalises en interne.
 */
fun mix(colors: List<Rgb>, weights: List<Double>): Rgb {
    require(colors.isNotEmpty() && colors.size == weights.size) {
        "Il faut autant de poids que de couleurs, et au moins une couleur"
    }
    val total = weights.sum()
    require(total > 0) { "La somme des poids doit etre strictement positive" }

    val ks = DoubleArray(3)
    for (i in colors.indices) {
        val w = weights[i] / total
        val c = colors[i]
        ks[0] += w * toKS(srgbToLinear(c.r))
        ks[1] += w * toKS(srgbToLinear(c.g))
        ks[2] += w * toKS(srgbToLinear(c.b))
    }
    return Rgb(linearToSrgb(fromKS(ks[0])), linearToSrgb(fromKS(ks[1])), linearToSrgb(fromKS(ks[2])))
}

/**
 * Coordonnees Kubelka-Munk d'une couleur, un canal par composante.
 *
 * <p>Interet pour la recherche de melange : dans cet espace, melanger est une
 * combinaison lineaire. L'ensemble des couleurs atteignables avec deux tubes est donc un
 * segment de droite, avec trois tubes un triangle.</p>
 */
fun toKs(color: Rgb): DoubleArray = doubleArrayOf(
    toKS(srgbToLinear(color.r)),
    toKS(srgbToLinear(color.g)),
    toKS(srgbToLinear(color.b)),
)

/** Couleur correspondant a des coordonnees Kubelka-Munk. */
fun fromKs(ks: DoubleArray): Rgb =
    Rgb(linearToSrgb(fromKS(ks[0])), linearToSrgb(fromKS(ks[1])), linearToSrgb(fromKS(ks[2])))

/** Rapport absorption / diffusion pour une reflectance donnee. */
private fun toKS(reflectance: Double): Double {
    val r = reflectance.coerceIn(0.002, 0.998)
    return (1 - r) * (1 - r) / (2 * r)
}

/** Reflectance correspondant a un rapport K/S. */
private fun fromKS(ks: Double): Double = 1 + ks - sqrt(ks * ks + 2 * ks)
