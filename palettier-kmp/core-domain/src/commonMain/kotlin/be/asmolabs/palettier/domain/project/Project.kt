package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.palette.Palette
import kotlin.time.Clock
import kotlin.time.Instant

/**
 * Une couche du projet : le role tenu, la couleur visee, la technique.
 *
 * <p>Aucun dosage n'est stocke. La couleur visee est la decision ; le melange qui y
 * conduit est un calcul, refait a chaque ouverture avec la palette du moment.</p>
 *
 * <p>La pose, elle, est un fait et non une decision. Les conditions de l'atelier et la
 * vitesse de sechage du melange sont figees avec elle : l'huile a seche dans l'atelier
 * qu'elle a connu, avec les tubes du moment, et ni un coup de chauffage ni un remaniement
 * de palette ne doivent reecrire le passe.</p>
 */
data class ProjectLayer(
    val id: Long? = null,
    /** "Ombre 2", "Base", "Lumiere 1"... */
    val role: String,
    val targetHex: String,
    val technique: String = "",
    val note: String = "",
    val kind: Kind = Kind.LADDER,
    val applied: AppliedCoat? = null,
) {
    /**
     * A quoi sert la couche. Le nom ne suffit pas a le dire : une variation locale
     * s'appelle "rougeur des pommettes", ce qu'aucune convention de nommage ne permet de
     * reconnaitre.
     */
    enum class Kind {
        /** Une marche de l'echelle des valeurs : ombre, base ou lumiere. */
        LADDER,

        /** Une couleur locale, a peu pres a la valeur de la base. */
        ACCENT,
    }

    val target: Rgb get() = Rgb.ofHex(targetHex)
    val isApplied: Boolean get() = applied != null

    fun markApplied(
        at: Instant,
        workshop: Workshop,
        dryingClass: DryingClass,
    ): ProjectLayer = copy(applied = AppliedCoat(at, workshop, dryingClass))

    /** Annule la pose : la couche redevient a peindre. Sert a corriger une fausse manoeuvre. */
    fun clearApplied(): ProjectLayer = copy(applied = null)
}

/**
 * La pose d'une couche, avec ce qui a prevalu ce jour-la.
 *
 * <p>Un type a part plutot que cinq champs optionnels : une couche est posee ou elle ne
 * l'est pas, et l'un ne va jamais sans les autres.</p>
 */
data class AppliedCoat(
    val at: Instant,
    val workshop: Workshop,
    /** Vitesse de sechage du melange effectivement pose. */
    val dryingClass: DryingClass,
) {
    companion object {
        /** Repli pour une pose restauree d'une archive qui ne portait pas ses conditions. */
        fun of(at: Instant, temperature: Double?, humidity: Double?, ventilation: Ventilation?, drying: DryingClass?) =
            AppliedCoat(
                at,
                Workshop(
                    temperature ?: Workshop.standard().temperatureCelsius,
                    humidity ?: Workshop.standard().relativeHumidity,
                    ventilation ?: Workshop.standard().ventilation,
                ),
                drying ?: DryingClass.MEDIUM,
            )
    }
}

/** Une partie du sujet dans un projet : le visage, la cape, le ceinturon. */
data class ProjectZone(
    val id: Long? = null,
    val name: String,
    val material: String = "",
    val note: String = "",
    val layers: List<ProjectLayer> = emptyList(),
)

/** Une photo du projet : la piece, une reference, un etat d'avancement. */
data class ProjectPhoto(
    val id: Long? = null,
    val data: ByteArray,
    val role: Role = Role.PIECE,
    val caption: String = "",
    val addedAt: Instant = Clock.System.now(),
) {
    enum class Role(val label: String) {
        PIECE("La piece"),
        REFERENCE("Reference"),
        PROGRESS("Avancement"),
    }

    // ByteArray casse l'egalite structurelle d'une data class : deux photos sont la meme
    // si elles ont le meme identifiant, pas les memes octets.
    override fun equals(other: Any?) = other is ProjectPhoto && id != null && id == other.id
    override fun hashCode() = id?.hashCode() ?: 0
}

/**
 * Une piece en cours, avec le plan de peinture retenu pour elle.
 *
 * <p>Ce qui est conserve, ce sont les decisions : le decoupage en zones, la couleur visee
 * pour chaque couche, la technique. Les dosages n'y sont pas -- ils se recalculent --
 * mais les tubes avec lesquels ils se calculent, si.</p>
 *
 * <p>Le projet garde sa propre liste de tubes, figee a l'enregistrement, et non un renvoi
 * vers la palette. C'est ce qui fait qu'un plan reste reproductible : retirer un tube
 * d'une palette ne change rien aux projets deja etablis.</p>
 */
data class Project(
    val id: Long? = null,
    val name: String,
    /** Description du sujet, telle qu'elle a servi a etablir le plan. */
    val subject: String = "",
    /** Parti pris general retenu pour la piece. */
    val approach: String = "",
    val notes: String = "",
    /** La palette d'origine, gardee pour memoire et pour pouvoir resynchroniser. */
    val palette: Palette? = null,
    /** Les tubes reellement disponibles quand le plan a ete etabli. */
    val paints: List<Paint> = emptyList(),
    val zones: List<ProjectZone> = emptyList(),
    /**
     * Photos du projet. Vide tant qu'on ne les a pas demandees : ce sont des images
     * entieres, et la plupart des ecrans n'en ont que faire.
     */
    val photos: List<ProjectPhoto> = emptyList(),
    val createdAt: Instant = Clock.System.now(),
) {
    /**
     * Les tubes a employer pour les calculs : ceux figes avec le projet, et a defaut ceux
     * de la palette.
     */
    fun effectivePaints(): List<Paint> = paints.ifEmpty { palette?.paints.orEmpty() }

    /**
     * Vrai si la palette a ete remaniee depuis l'enregistrement. L'ecart n'est pas une
     * erreur : c'est une information, a l'utilisateur de decider s'il veut en profiter.
     */
    fun divergesFromPalette(): Boolean =
        palette != null && paints.isNotEmpty() && paints.map { it.id } != palette.paints.map { it.id }
}
