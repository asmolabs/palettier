package be.asmolabs.palettier.ai

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject

/** Une photo soumise au modele, deja reduite. */
data class PhotoInput(val data: ByteArray, val caption: String) {
    override fun equals(other: Any?) = other is PhotoInput && caption == other.caption &&
        data.contentEquals(other.data)
    override fun hashCode() = caption.hashCode() * 31 + data.contentHashCode()
}

/**
 * Ce qu'on demande a un moteur : deux textes, des images, et un schema a respecter.
 *
 * <p>Le schema est donne sous sa forme canonique. Chaque moteur le retaille a sa mesure
 * -- c'est son affaire, pas celle de l'appelant.</p>
 *
 * @param maxTokens plafond de generation. Un plan a six zones fait plusieurs milliers de
 *                  mots de JSON : avec la limite par defaut de certains moteurs, la
 *                  reponse est coupee en plein objet et rien n'est exploitable.
 */
data class JsonRequest(
    val system: String,
    val user: String,
    val schema: JsonObject,
    val photos: List<PhotoInput> = emptyList(),
    val model: String? = null,
    val maxTokens: Int = 16384,
)

/**
 * Un moteur de conversation capable de rendre du JSON conforme a un schema.
 *
 * <p>Les trois moteurs vises -- Ollama, OpenAI, Gemini -- savent tous contraindre leur
 * sortie, mais aucun de la meme facon. C'est precisement ce que cette interface cache :
 * ailleurs dans le code, on demande du JSON conforme, on ne le negocie pas.</p>
 *
 * <p>Elle rend une chaine et non un objet : deux usages s'en servent deja, le plan de
 * peinture et la lecture d'etiquettes, et rien ne dit qu'ils seront les derniers.</p>
 */
interface ChatEngine {
    val name: String

    /** @throws PlanUnavailable quand le moteur ne repond pas */
    suspend fun ask(request: JsonRequest): String
}

/** Le moteur n'a pas rendu de reponse exploitable. */
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
internal fun String.toPlanDraft(json: Json): PlanDraft =
    runCatching { json.decodeFromString<PlanDraft>(this) }.getOrElse {
        throw PlanUnavailable(
            "Le modele n'a pas produit de plan exploitable. Essayez un modele plus capable, " +
                "ou moins de zones en precisant le sujet.",
            it,
        )
    }
