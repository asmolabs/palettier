package be.asmolabs.palettier.data.backup

import kotlinx.serialization.Serializable

/**
 * La forme de la sauvegarde ecrite par l'application Java, reproduite a l'identique.
 *
 * <p>C'est le pont entre les deux applications, et la raison pour laquelle aucun
 * convertisseur H2 vers SQLite n'a eu a etre ecrit : l'ancienne exporte, la nouvelle
 * importe. Le format avait ete concu independant des entites -- "une sauvegarde doit
 * rester lisible et reimportable meme apres un remaniement du modele" -- et c'est
 * exactement ce remaniement-la qui arrive.</p>
 *
 * <p>Les dates sont declarees en texte et analysees ensuite. Le Java les ecrit en
 * ISO-8601 ; les lire comme telles evite de dependre d'un serialiseur de date dont le
 * format pourrait differer d'une version a l'autre.</p>
 */
@Serializable
data class BackupDocument(
    val formatVersion: Int = 1,
    val exportedAt: String? = null,
    val application: String? = null,
    val paints: List<BackupPaint> = emptyList(),
    val palettes: List<BackupPalette> = emptyList(),
    val projects: List<BackupProject> = emptyList(),
    val recipes: List<BackupRecipe> = emptyList(),
)

@Serializable
data class BackupPaint(
    val brand: String,
    val name: String,
    val code: String = "",
    val pigments: List<String> = emptyList(),
    val hex: String,
    val tintHex: String? = null,
    val opacity: String,
    val dryingClass: String,
    val tintingStrength: Double = 0.5,
    val owned: Boolean = false,
    val colorDerived: Boolean = false,
    val notes: String = "",
)

/** Renvoi vers un tube, par sa cle naturelle. */
@Serializable
data class BackupPaintRef(val brand: String, val name: String)

@Serializable
data class BackupPalette(
    val name: String,
    val purpose: String = "",
    val notes: String = "",
    val paints: List<BackupPaintRef> = emptyList(),
)

@Serializable
data class BackupProject(
    val name: String,
    val subject: String = "",
    val approach: String = "",
    val notes: String = "",
    val paletteName: String? = null,
    val createdAt: String? = null,
    val paints: List<BackupPaintRef> = emptyList(),
    val zones: List<BackupZone> = emptyList(),
    val photos: List<BackupPhoto> = emptyList(),
)

@Serializable
data class BackupZone(
    val name: String,
    val material: String = "",
    val note: String = "",
    val layers: List<BackupLayer> = emptyList(),
)

/**
 * Une couche : ce qui est vise, et ce qui a deja ete pose.
 *
 * <p>Les cinq derniers champs sont absents des archives ecrites avant que la pose soit
 * suivie. La couche revient alors simplement a peindre, ce qu'elle etait.</p>
 */
@Serializable
data class BackupLayer(
    val role: String,
    val targetHex: String,
    val technique: String = "",
    val note: String = "",
    val kind: String = "LADDER",
    val appliedAt: String? = null,
    val appliedTemperature: Double? = null,
    val appliedHumidity: Double? = null,
    val appliedVentilation: String? = null,
    val appliedDryingClass: String? = null,
)

/** Une photo, rangee a cote du JSON plutot qu'encodee dedans. */
@Serializable
data class BackupPhoto(
    val file: String,
    val role: String,
    val caption: String = "",
    val addedAt: String? = null,
)

@Serializable
data class BackupRecipe(
    val name: String,
    val subject: String = "",
    val notes: String = "",
    val steps: List<BackupStep> = emptyList(),
)

@Serializable
data class BackupStep(
    val technique: String,
    val paintMix: String = "",
    val medium: String,
    val mediumRatio: Double = 0.0,
    val thickness: String,
    val note: String = "",
)
