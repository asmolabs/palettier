package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

/**
 * Le moteur de Google : Gemini.
 *
 * <p>Troisieme facon de dire la meme chose. Le schema se donne dans {@code
 * generationConfig.responseSchema}, avec un type de reponse JSON, et il doit tenir dans
 * le sous-ensemble OpenAPI que Gemini accepte : ni motif, ni champ supplementaire. C'est
 * {@link PlanSchema#gemini}, et la demande entiere serait rejetee sans cette taille.</p>
 *
 * <p>La consigne de systeme est un champ a part, {@code systemInstruction}, et non un
 * message de plus. Les images voyagent en pieces jointes de la meme requete.</p>
 */
class GeminiEngine(
    private val client: HttpClient,
    private val apiKey: String,
    private val baseUrl: String = DEFAULT_URL,
    private val defaultModel: String = DEFAULT_MODEL,
) : ChatEngine {

    override val name = "gemini"

    private val json = Json { ignoreUnknownKeys = true }

    @OptIn(ExperimentalEncodingApi::class)
    override suspend fun draft(request: PlanRequest): PlanDraft {
        val model = request.model?.takeIf { it.isNotBlank() } ?: defaultModel

        val body = buildJsonObject {
            putJsonObject("systemInstruction") {
                putJsonArray("parts") { add(buildJsonObject { put("text", request.system) }) }
            }
            putJsonArray("contents") {
                add(buildJsonObject {
                    put("role", "user")
                    putJsonArray("parts") {
                        add(buildJsonObject { put("text", request.user) })
                        request.photos.forEach { photo ->
                            add(buildJsonObject {
                                putJsonObject("inline_data") {
                                    put("mime_type", PHOTO_MIME)
                                    put("data", Base64.encode(photo.data))
                                }
                            })
                        }
                    }
                })
            }
            putJsonObject("generationConfig") {
                put("temperature", 0.2)
                put("maxOutputTokens", MAX_TOKENS)
                put("responseMimeType", "application/json")
                put("responseSchema", PlanSchema.gemini)
            }
        }

        val response: HttpResponse = client.post("${baseUrl.trimEnd('/')}/models/$model:generateContent") {
            contentType(ContentType.Application.Json)
            // La cle passe en entete plutot qu'en parametre d'URL : une URL se retrouve
            // dans les journaux, pas un entete.
            header("x-goog-api-key", apiKey)
            setBody(body.toString())
        }
        if (!response.status.isSuccess()) {
            throw PlanUnavailable("Gemini a refuse la demande (${response.status.value}).")
        }

        val content = runCatching {
            json.parseToJsonElement(response.bodyAsText())
                .jsonObject["candidates"]?.jsonArray?.firstOrNull()
                ?.jsonObject?.get("content")?.jsonObject?.get("parts")?.jsonArray
                ?.mapNotNull { it.jsonObject["text"]?.jsonPrimitive?.content }
                ?.joinToString("")
                ?.takeIf { it.isNotBlank() }
        }.getOrNull()
            ?: throw PlanUnavailable("Reponse de Gemini inattendue : pas de contenu.")

        return content.toPlanDraft(json)
    }

    companion object {
        const val DEFAULT_URL = "https://generativelanguage.googleapis.com/v1beta"
        const val DEFAULT_MODEL = "gemini-2.5-pro"
        const val MAX_TOKENS = 16384
    }
}
