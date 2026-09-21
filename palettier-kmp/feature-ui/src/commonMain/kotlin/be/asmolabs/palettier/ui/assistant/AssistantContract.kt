package be.asmolabs.palettier.ui.assistant

import be.asmolabs.palettier.ai.AiSettings
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
) {
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
}
