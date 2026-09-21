package be.asmolabs.palettier.ai

/** Une photo soumise au modele, deja reduite. */
data class PhotoInput(val data: ByteArray, val caption: String) {
    override fun equals(other: Any?) = other is PhotoInput && caption == other.caption &&
        data.contentEquals(other.data)
    override fun hashCode() = caption.hashCode() * 31 + data.contentHashCode()
}

/** Ce qu'on demande a un moteur : deux textes, des images, et un schema a respecter. */
data class PlanRequest(
    val system: String,
    val user: String,
    val photos: List<PhotoInput> = emptyList(),
    val model: String? = null,
)

/**
 * Un moteur de conversation capable de rendre du JSON conforme a un schema.
 *
 * <p>Les trois moteurs vises -- Ollama, OpenAI, Gemini -- savent tous contraindre leur
 * sortie, mais aucun de la meme facon. C'est precisement ce que cette interface cache :
 * ailleurs dans le code, un plan se demande, il ne se negocie pas.</p>
 */
interface ChatEngine {
    val name: String

    /** @throws PlanUnavailable quand le moteur ne rend pas de plan exploitable */
    suspend fun draft(request: PlanRequest): PlanDraft
}

/** Le moteur n'a pas rendu de plan exploitable. */
class PlanUnavailable(message: String, cause: Throwable? = null) : Exception(message, cause)

/** Le type des photos soumises, celui que produit la reduction. */
const val PHOTO_MIME = "image/jpeg"

/**
 * Le texte rendu par un moteur, relu comme un plan.
 *
 * <p>Meme sous contrainte de schema, un modele a mode de reflexion repond parfois a cote,
 * en texte libre. La contrainte rend le JSON invalide improbable, pas impossible : c'est
 * ici qu'on le constate, une fois pour les trois moteurs.</p>
 */
internal fun String.toPlanDraft(json: kotlinx.serialization.json.Json): PlanDraft =
    runCatching { json.decodeFromString<PlanDraft>(this) }.getOrElse {
        throw PlanUnavailable(
            "Le modele n'a pas produit de plan exploitable. Essayez un modele plus capable, " +
                "ou moins de zones en precisant le sujet.",
            it,
        )
    }
