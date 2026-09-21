package be.asmolabs.palettier.domain.catalog

import be.asmolabs.palettier.domain.color.Colorant
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.linearToSrgb
import be.asmolabs.palettier.domain.color.mix
import be.asmolabs.palettier.domain.color.srgbToLinear
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Pigment
import kotlin.math.max

/**
 * Table des pigments, construite depuis le texte de {@code catalog/pigments.csv}.
 *
 * <p>Sert a deduire les proprietes d'une huile a partir de sa composition, plutot que de
 * les saisir tube par tube : le sechage d'un melange est celui de son pigment le plus
 * lent, le pouvoir colorant celui de son pigment le plus fort.</p>
 */
class PigmentIndex(pigmentsCsv: String = CatalogFiles.PIGMENTS) {

    private val byCode: Map<String, Pigment> = Csv.read(pigmentsCsv, "pigments.csv")
        .associate { row ->
            val pigment = Pigment(
                code = row["code"],
                name = row["name"],
                color = Rgb.ofHex(row["hex"]),
                opacity = Opacity.valueOf(row["opacity"]),
                dryingClass = DryingClass.valueOf(row["drying"]),
                tintingStrength = row["tinting"].toDouble(),
                tint = row.getOrDefault("tint", "").takeIf { it.isNotBlank() }?.let(Rgb::ofHex),
            )
            normalise(pigment.code) to pigment
        }

    fun find(code: String): Pigment? = byCode[normalise(code)]

    fun all(): Collection<Pigment> = byCode.values

    /** Codes cites par le catalogue mais absents de la table : liste de travail. */
    fun unknownAmong(codes: Collection<String>): List<String> =
        codes.filter { find(it) == null }.distinct().sorted()

    // --- Proprietes deduites d'une composition -----------------------------

    private fun resolve(codes: Set<String>): List<Pigment> =
        codes.map { find(it) ?: Pigment.unknown(it) }

    /** Le pigment le plus lent commande : une trace de cadmium suffit a tout ralentir. */
    fun dryingClassFor(codes: Set<String>): DryingClass =
        resolve(codes).fold(DryingClass.FAST) { slowest, pigment -> slowest.slowest(pigment.dryingClass) }

    /** Le pigment le plus colorant domine le melange. */
    fun tintingStrengthFor(codes: Set<String>): Double =
        resolve(codes).maxOfOrNull { it.tintingStrength } ?: 0.5

    /** A defaut d'indication du fabricant, le pigment le plus couvrant donne le ton. */
    fun opacityFor(codes: Set<String>): Opacity =
        resolve(codes).fold(Opacity.TRANSPARENT) { most, pigment ->
            if (most.ordinal >= pigment.opacity.ordinal) most else pigment.opacity
        }

    /**
     * Couleur approchee d'une huile dont la teinte n'est pas renseignee, obtenue en
     * melangeant ses pigments a parts egales. Correct pour un tube monopigmentaire,
     * approximatif des que le fabricant dose ses pigments, ce que personne ne publie.
     */
    fun colorFor(codes: Set<String>): Rgb {
        val pigments = resolve(codes)
        if (pigments.isEmpty()) return Rgb(0.5, 0.5, 0.5)
        return mix(pigments.map { it.color }, pigments.map { it.tintingStrength })
    }

    /**
     * Teinte diluee estimee d'une huile, deduite de ses pigments.
     *
     * <p>La divergence entre ton de masse et teinte diluee est une propriete du pigment ;
     * le ton de masse, lui, est propre au tube. On ne recopie donc pas la teinte du
     * pigment telle quelle : on mesure de combien elle s'ecarte de ce que le modele a
     * constante unique aurait predit, et on reporte cet ecart sur la teinte predite pour
     * le tube. Un pigment sans teinte relevee n'apporte aucun ecart, et le tube garde
     * alors le comportement d'avant.</p>
     *
     * @return la teinte estimee, ou null si aucun pigment du tube n'a de teinte relevee
     */
    fun tintFor(codes: Set<String>, masstone: Rgb): Rgb? {
        val pigments = resolve(codes)
        if (pigments.all { it.tint == null }) return null

        val gain = DoubleArray(3)
        var total = 0.0
        for (pigment in pigments) {
            val weight = pigment.tintingStrength
            val pigmentGain = divergenceOf(pigment)
            for (channel in 0..2) {
                gain[channel] += weight * pigmentGain[channel]
            }
            total += weight
        }

        val predicted = Colorant.ofMasstone(masstone).tint(Colorant.REFERENCE_TINT_CONCENTRATION)
        val predictedLinear = doubleArrayOf(
            srgbToLinear(predicted.r),
            srgbToLinear(predicted.g),
            srgbToLinear(predicted.b),
        )

        return Rgb(
            linearToSrgb(predictedLinear[0] * gain[0] / total),
            linearToSrgb(predictedLinear[1] * gain[1] / total),
            linearToSrgb(predictedLinear[2] * gain[2] / total),
        )
    }

    /**
     * De combien la teinte relevee d'un pigment s'ecarte de celle que le modele a
     * constante unique aurait predite, canal par canal. Vaut 1 partout quand le pigment
     * n'a pas de teinte relevee : aucun ecart, donc aucune correction.
     */
    private fun divergenceOf(pigment: Pigment): DoubleArray {
        val observed = pigment.tint ?: return doubleArrayOf(1.0, 1.0, 1.0)
        val predicted = Colorant.ofMasstone(pigment.color).tint(Colorant.REFERENCE_TINT_CONCENTRATION)
        return doubleArrayOf(
            ratio(observed.r, predicted.r),
            ratio(observed.g, predicted.g),
            ratio(observed.b, predicted.b),
        )
    }

    private fun ratio(observed: Double, predicted: Double): Double =
        srgbToLinear(observed) / max(srgbToLinear(predicted), 1e-5)

    private fun normalise(code: String): String = code.trim().uppercase().replace(" ", "")
}
