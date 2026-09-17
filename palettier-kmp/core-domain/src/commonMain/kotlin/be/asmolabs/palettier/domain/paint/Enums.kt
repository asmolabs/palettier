package be.asmolabs.palettier.domain.paint

import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours

/**
 * Vitesse de sechage d'un pigment. C'est le pigment qui commande a l'huile, pas la
 * marque : le temps de reference est celui du plus lent d'un melange.
 */
enum class DryingClass(
    val label: String,
    val referenceTouchDry: Duration,
    val typicalPigments: String,
) {
    FAST("Rapide", 10.hours, "Terres d'ombre, bleu de Prusse, siccatifs naturels."),
    MEDIUM("Moyen", 20.hours, "Terres de Sienne, ocres, bleus et verts courants."),
    SLOW("Lent", 38.hours, "Noirs d'ivoire, blancs de titane, la plupart des rouges."),
    VERY_SLOW("Tres lent", 64.hours, "Cadmiums, blanc de zinc, laques organiques.");

    fun slowest(other: DryingClass): DryingClass = if (ordinal >= other.ordinal) this else other

    override fun toString() = label
}

/** Pouvoir couvrant d'une huile, tel qu'indique par les fabricants sur le tube. */
enum class Opacity(val label: String, val advice: String) {
    TRANSPARENT("Transparent", "Ideal pour les glacis et les filtres : la couche du dessous reste visible."),
    SEMI_TRANSPARENT("Semi-transparent", "Bon compromis pour un jus ou un fondu leger."),
    SEMI_OPAQUE("Semi-opaque", "Couvre en deux passes, garde un peu de profondeur."),
    OPAQUE("Opaque", "Couvre en une passe : pour les aplats et les points lumineux.");

    override fun toString() = label
}

/** Epaisseur de la couche deposee : premier facteur du temps de sechage apres le pigment. */
enum class LayerThickness(val label: String, val factor: Double) {
    GLAZE("Glacis / voile", 0.30),
    THIN("Couche fine", 0.60),
    NORMAL("Couche normale", 1.00),
    THICK("Couche chargee", 1.90),
    IMPASTO("Empatement", 3.40);

    override fun toString() = label
}

/** Ventilation de l'atelier. */
enum class Ventilation(val label: String, val factor: Double) {
    CONFINED("Boite fermee / vitrine", 1.30),
    NORMAL("Piece normale", 1.00),
    GOOD("Bien aere ou courant d'air", 0.82);

    override fun toString() = label
}

/**
 * Diluant ou medium ajoute a l'huile.
 *
 * @param dryingFactorAtFullRatio multiplicateur du temps de sechage a la proportion
 *                                maximale utile ; interpole lineairement en deca
 * @param fatDirection            -1 pour un diluant qui maigrit la couche, +1 pour une
 *                                huile qui l'engraisse, 0 pour un apport negligeable
 */
enum class Medium(
    val label: String,
    private val dryingFactorAtFullRatio: Double,
    val maxUsefulRatio: Double,
    private val fatDirection: Int,
    val advice: String,
) {
    NONE("Pur (sortie de tube)", 1.00, 0.00, 0, "Pate epaisse : reserve aux empatements et aux textures."),
    ODORLESS_THINNER("Diluant inodore", 0.55, 0.85, -1, "Le standard pour les jus et les fondus sur figurine."),
    WHITE_SPIRIT("White spirit", 0.48, 0.85, -1, "Plus agressif : verifier la tenue du vernis en dessous."),
    TURPENTINE("Essence de terebenthine", 0.50, 0.85, -1, "Evapore vite, odeur forte, bonne accroche."),
    ALKYD_MEDIUM("Medium alkyde (type Liquin)", 0.35, 0.50, 1, "Accelere fortement et lisse le fondu."),
    LINSEED_OIL("Huile de lin", 1.90, 0.40, 1, "Rallonge le temps ouvert, jaunit legerement en couche epaisse."),
    WALNUT_OIL("Huile de noix", 2.20, 0.40, 1, "Temps ouvert tres long, ne jaunit presque pas."),
    STAND_OIL("Huile stand", 2.60, 0.30, 1, "Tres lent, tres lissant : pour les fondus longs."),
    COBALT_DRIER("Siccatif au cobalt", 0.28, 0.05, 0, "Quelques gouttes suffisent ; au-dela, la couche craquelle.");

    /** Multiplicateur de sechage pour une proportion de medium donnee. */
    fun dryingFactor(ratio: Double): Double {
        val r = ratio.coerceIn(0.0, 1.0)
        val reference = maxOf(maxUsefulRatio, 1e-6)
        val progress = minOf(r / reference, 1.0)
        return 1.0 + (dryingFactorAtFullRatio - 1.0) * progress
    }

    /**
     * Part de liant de la couche, la peinture pure valant 1. C'est le nombre que compare
     * la regle du gras sur maigre.
     */
    fun fatness(ratio: Double): Double = 1.0 + fatDirection * ratio.coerceIn(0.0, 1.0)

    override fun toString() = label
}

/** Technique a l'huile, avec ses reglages usuels. */
enum class Technique(
    val label: String,
    val defaultMedium: Medium,
    val minRatio: Double,
    val maxRatio: Double,
    val typicalThickness: LayerThickness,
    /** Vrai si la couche precedente doit etre completement seche avant celle-ci. */
    val requiresCuredBase: Boolean,
    val tips: List<String>,
) {
    OIL_WASH(
        "Jus a l'huile", Medium.ODORLESS_THINNER, 0.80, 0.92, LayerThickness.GLAZE, true,
        listOf(
            "Le vernis brillant en dessous doit etre sec depuis 24 h au minimum.",
            "Charger le pinceau puis l'essuyer : le jus doit couler, pas s'etaler.",
            "Retirer l'exces au coton-tige a peine humide de diluant avant la prise.",
        ),
    ),
    PIN_WASH(
        "Jus capillaire (pin wash)", Medium.ODORLESS_THINNER, 0.85, 0.94, LayerThickness.GLAZE, true,
        listOf(
            "Poser la pointe du pinceau dans le creux et laisser la capillarite faire le trajet.",
            "Un seul passage par ligne : repasser dessus casse le depot.",
        ),
    ),
    FILTER(
        "Filtre", Medium.ODORLESS_THINNER, 0.88, 0.96, LayerThickness.GLAZE, true,
        listOf(
            "Tres dilue : le filtre unifie une teinte, il ne la remplace pas.",
            "Deux filtres legers valent mieux qu'un seul charge.",
        ),
    ),
    GLAZE(
        "Glacis", Medium.ODORLESS_THINNER, 0.70, 0.88, LayerThickness.GLAZE, true,
        listOf(
            "Privilegier une huile transparente : une opaque tue la profondeur.",
            "Laisser secher entre deux glacis, sinon la couche precedente se releve.",
        ),
    ),
    BLENDING(
        "Fondu / degrade", Medium.NONE, 0.00, 0.25, LayerThickness.THIN, true,
        listOf(
            "Travailler dans le temps ouvert : une fois la prise commencee, arreter.",
            "Pinceau propre et sec pour tirer la transition, essuye a chaque passage.",
        ),
    ),
    DOT_FADING(
        "Dot fading", Medium.NONE, 0.00, 0.20, LayerThickness.THIN, true,
        listOf(
            "Poser des points minuscules de plusieurs teintes, puis les tirer vers le bas.",
            "Moins de peinture que ce que l'on croit : le pinceau doit etre presque sec.",
        ),
    ),
    STREAKING_GRIME(
        "Coulures et salissures", Medium.ODORLESS_THINNER, 0.60, 0.85, LayerThickness.THIN, true,
        listOf(
            "Tracer le trait, laisser mordre une a deux minutes, puis tirer vers le bas.",
            "Suivre le sens de l'eau : verticales sur les flancs, en eventail sous les rivets.",
        ),
    ),
    OIL_RENDERING(
        "Oil Paint Rendering (OPR)", Medium.ODORLESS_THINNER, 0.50, 0.80, LayerThickness.THIN, true,
        listOf(
            "Poser clair et fonce cote a cote, puis fondre la frontiere.",
            "Construire en plusieurs seances plutot qu'en une couche chargee.",
        ),
    ),
    BASE_LAYER(
        "Aplat de base", Medium.ODORLESS_THINNER, 0.10, 0.35, LayerThickness.NORMAL, false,
        listOf(
            "A l'huile, un aplat de base reste rare : il seche lentement et bloque la suite.",
            "Utile surtout sur les grandes surfaces lisses (bustes, 1/10).",
        ),
    ),
    HIGHLIGHT(
        "Eclaircis et points lumineux", Medium.NONE, 0.00, 0.20, LayerThickness.THIN, true,
        listOf(
            "Une huile opaque tient mieux le point lumineux qu'une transparente.",
            "Le blanc de titane seche tres lentement : prevoir la couche suivante a distance.",
        ),
    );

    val defaultRatio: Double get() = (minRatio + maxRatio) / 2.0

    override fun toString() = label

    companion object {
        /**
         * Retrouve une technique par son libelle, ou rend le repli.
         *
         * <p>Les plans conservent le nom en clair plutot que la constante : il provient
         * parfois d'un modele de langage ou d'une saisie libre, et doit rester lisible
         * meme lorsqu'il ne correspond a rien de connu.</p>
         */
        fun byLabel(label: String?, fallback: Technique): Technique {
            if (label.isNullOrBlank()) return fallback
            return entries.firstOrNull { it.label.equals(label.trim(), ignoreCase = true) } ?: fallback
        }
    }
}
