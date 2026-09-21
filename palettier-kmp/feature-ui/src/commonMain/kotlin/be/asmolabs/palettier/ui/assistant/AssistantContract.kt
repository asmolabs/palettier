package be.asmolabs.palettier.ui.assistant

import be.asmolabs.palettier.ai.AiSettings
import be.asmolabs.palettier.ai.Identification
import be.asmolabs.palettier.ai.ModelInfo
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.plan.PaintingPlan

/** Une photo jointe, telle que l'ecran la montre : on n'affiche pas ses octets. */
data class AttachedPhoto(val caption: String, val bytes: ByteArray) {
    override fun equals(other: Any?) = other is AttachedPhoto &&
        caption == other.caption && bytes.contentEquals(other.bytes)
    override fun hashCode() = caption.hashCode() * 31 + bytes.contentHashCode()
}

/**
 * Ce que l'ecran de l'assistant montre.
 *
 * <p>Le reglage du moteur fait partie de l'etat, et non d'un ecran a part : sans moteur
 * choisi, la page n'a rien d'autre a proposer que ce choix.</p>
 */
data class AssistantUiState(
    val settings: AiSettings = AiSettings(),
    val palettes: List<Palette> = emptyList(),
    val paletteId: Long? = null,
    val subject: String = "",
    val maxPaints: Int = 3,
    /** La piece telle qu'elle est aujourd'hui. */
    val figurine: AttachedPhoto? = null,
    /** Ce que l'on cherche a obtenir. */
    val references: List<AttachedPhoto> = emptyList(),
    val working: Boolean = false,
    val plan: PaintingPlan? = null,
    val error: String? = null,

    /** Ce qui est installe sur ce poste, quand le moteur local le permet. */
    val installed: List<ModelInfo> = emptyList(),

    /** Les tubes lus sur une photo d'etagere, en attente de la decision du peintre. */
    val tubes: List<Identification> = emptyList(),
    val reading: Boolean = false,

    /** Vrai le temps que le plan devienne un projet, photos comprises. */
    val saving: Boolean = false,
    /** Ce que le dernier enregistrement a donne, nomme pour qu'on puisse le retrouver. */
    val savedMessage: String? = null,
) {
    /**
     * Le nom propose pour le projet : le sujet decrit, ecourte s'il est bavard.
     *
     * <p>Un sujet vaut mieux qu'un champ vide : on tape rarement deux fois la meme
     * description, et c'est sous elle qu'on reconnaitra la piece dans six mois.</p>
     */
    val suggestedProjectName: String
        get() = subject.trim().let {
            when {
                it.isBlank() -> "Projet sans titre"
                it.length <= 50 -> it
                else -> it.take(50).trim()
            }
        }

    val palette: Palette? get() = palettes.firstOrNull { it.id == paletteId }

    /** Vrai quand la demande a de quoi partir : un moteur, une palette, et un sujet. */
    val canAsk: Boolean get() = settings.usable && !working && palette != null &&
        (subject.isNotBlank() || figurine != null)
}

sealed interface AssistantIntent {
    data class SetSubject(val subject: String) : AssistantIntent
    data class ChoosePalette(val id: Long) : AssistantIntent
    data class SetMaxPaints(val count: Int) : AssistantIntent

    /** La plateforme a lu un fichier ; le ViewModel ne connait que des octets. */
    data class AttachFigurine(val bytes: ByteArray) : AssistantIntent
    data class AttachReference(val bytes: ByteArray) : AssistantIntent
    data object ClearPhotos : AssistantIntent

    data object Ask : AssistantIntent
    data class SaveSettings(val settings: AiSettings) : AssistantIntent

    /** Lire les etiquettes d'une photo d'etagere. */
    data class ReadTubes(val bytes: ByteArray) : AssistantIntent

    /** Le peintre confirme qu'il possede ce tube : c'est lui qui tranche, pas le modele. */
    data class KeepTube(val identification: Identification) : AssistantIntent
    data object ForgetTubes : AssistantIntent

    /**
     * Fige le plan propose en projet.
     *
     * <p>Les photos qui ont servi a l'etablir partent avec lui : sans elles, on relit dans
     * six mois un plan dont on ne sait plus a quoi il s'appliquait.</p>
     */
    data class SaveProject(val name: String) : AssistantIntent
}
