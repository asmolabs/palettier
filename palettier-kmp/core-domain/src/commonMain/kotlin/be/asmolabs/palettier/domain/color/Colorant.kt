package be.asmolabs.palettier.domain.color

/**
 * Un colorant decrit par ses constantes de Kubelka-Munk : une absorption K et une
 * diffusion S, par canal.
 *
 * <p><b>Pourquoi deux constantes.</b> Le modele a constante unique suppose que tous les
 * pigments diffusent la lumiere de la meme facon, et ne retient donc que le rapport K/S.
 * Il ne peut alors pas representer le cas, pourtant banal, d'un pigment dont le ton de
 * masse et la teinte diluee divergent : le noir d'ivoire sort du tube franchement chaud
 * et donne pourtant des gris froids une fois coupe de blanc.</p>
 *
 * <p><b>Ce que ce n'est pas.</b> Les constantes sont ajustees sur deux couleurs
 * observees, pas mesurees au spectrophotometre. Le modele reproduit fidelement ces deux
 * points et interpole entre eux ; il ne pretend pas decrire la physique du pigment.</p>
 */
class Colorant private constructor(
    private val k: DoubleArray,
    private val s: DoubleArray,
) {

    /** Absorption par canal. Copie defensive : la recherche de melange precalcule ces tableaux. */
    val absorption: DoubleArray get() = k.copyOf()

    /** Diffusion par canal. */
    val scattering: DoubleArray get() = s.copyOf()

    /** Couleur du colorant pur, telle qu'elle sort du tube. */
    fun masstone(): Rgb = fromKs(doubleArrayOf(k[0] / s[0], k[1] / s[1], k[2] / s[2]))

    /** Couleur du colorant coupe de blanc de reference, a la concentration indiquee. */
    fun tint(concentration: Double): Rgb =
        mix(listOf(this, ofMasstone(REFERENCE_WHITE)), listOf(concentration, 1 - concentration))

    companion object {
        /**
         * Le blanc de reference des teintes diluees : un blanc de titane. C'est lui qui
         * fixe l'echelle des constantes, arbitraire par nature.
         */
        val REFERENCE_WHITE: Rgb = Rgb.ofHex("#F7F5F0")

        /** Proportion de colorant dans la teinte diluee de reference : une part pour neuf de blanc. */
        const val REFERENCE_TINT_CONCENTRATION: Double = 0.1

        /** Diffusion du blanc de reference, qui sert d'unite. */
        private const val REFERENCE_SCATTERING = 1.0

        /** En deca, la diffusion devient numeriquement instable. */
        private const val MIN_SCATTERING = 1e-4

        /**
         * Colorant decrit par son seul ton de masse.
         *
         * <p>La diffusion est posee a l'unite sur les trois canaux : le modele se ramene
         * alors exactement au modele a constante unique. C'est le repli quand la teinte
         * diluee d'une huile n'est pas renseignee.</p>
         */
        fun ofMasstone(masstone: Rgb): Colorant {
            val ratio = toKs(masstone)
            val k = DoubleArray(3)
            val s = DoubleArray(3)
            for (channel in 0..2) {
                s[channel] = REFERENCE_SCATTERING
                k[channel] = ratio[channel] * REFERENCE_SCATTERING
            }
            return Colorant(k, s)
        }

        /**
         * Colorant ajuste sur deux observations : le ton de masse et la teinte diluee.
         *
         * <p>Si la teinte fournie n'est pas plus claire que le ton de masse, le systeme
         * n'a pas de solution utilisable : on retombe sur le modele a constante unique
         * plutot que de produire des constantes negatives.</p>
         */
        fun ofMasstoneAndTint(masstone: Rgb, tint: Rgb, concentration: Double): Colorant {
            val c = concentration.coerceIn(1e-3, 1 - 1e-3)
            val masstoneRatio = toKs(masstone)
            val tintRatio = toKs(tint)
            val whiteRatio = toKs(REFERENCE_WHITE)

            val k = DoubleArray(3)
            val s = DoubleArray(3)
            for (channel in 0..2) {
                val whiteAbsorption = whiteRatio[channel] * REFERENCE_SCATTERING
                val denominator = c * (masstoneRatio[channel] - tintRatio[channel])
                val numerator = (1 - c) * (tintRatio[channel] * REFERENCE_SCATTERING - whiteAbsorption)

                s[channel] = if (denominator <= 0 || numerator <= 0) {
                    // Teinte incoherente avec le ton de masse : on ne devine pas, on se replie.
                    REFERENCE_SCATTERING
                } else {
                    maxOf(numerator / denominator, MIN_SCATTERING)
                }
                k[channel] = masstoneRatio[channel] * s[channel]
            }
            return Colorant(k, s)
        }

        /**
         * Melange de plusieurs colorants. Absorption et diffusion se combinent
         * lineairement, chacune de son cote : c'est ce qui permet a un blanc tres
         * diffusant d'imposer sa loi des qu'il est present.
         */
        fun mix(colorants: List<Colorant>, weights: List<Double>): Rgb {
            require(colorants.isNotEmpty() && colorants.size == weights.size) {
                "Il faut autant de poids que de colorants, et au moins un colorant"
            }
            val total = weights.sum()
            require(total > 0) { "La somme des poids doit etre strictement positive" }

            val k = DoubleArray(3)
            val s = DoubleArray(3)
            for (i in colorants.indices) {
                val share = weights[i] / total
                val colorant = colorants[i]
                for (channel in 0..2) {
                    k[channel] += share * colorant.k[channel]
                    s[channel] += share * colorant.s[channel]
                }
            }
            return colorOf(k, s)
        }

        /**
         * Couleur d'un melange dont l'absorption et la diffusion sont deja cumulees.
         * Raccourci pour la recherche de melange, qui fait ses propres sommes.
         */
        fun colorOf(absorption: DoubleArray, scattering: DoubleArray): Rgb = fromKs(
            doubleArrayOf(
                absorption[0] / maxOf(scattering[0], MIN_SCATTERING),
                absorption[1] / maxOf(scattering[1], MIN_SCATTERING),
                absorption[2] / maxOf(scattering[2], MIN_SCATTERING),
            )
        )
    }
}
