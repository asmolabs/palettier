package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

/**
 * Le moteur local : Ollama.
 *
 * <p>Le schema est passe dans le champ {@code format}, qui contraint le decodage : le
 * jeton qui casserait le JSON devient impossible a produire. C'est la difference de
 * nature entre une consigne, qu'on suit a peu pres, et une contrainte. Le portage Java a
 * mis une semaine a comprendre que l'une ne vaut pas l'autre ; autant partir du bon
 * cote.</p>
 *
 * <p>La fenetre de contexte et le plafond de generation sont poses explicitement. Les
 * valeurs par defaut d'Ollama sont trop courtes pour une image et un plan complet a la
 * fois -- la reponse se retrouve coupee.</p>
 */
class OllamaEngine(
    private val client: HttpClient,
    private val baseUrl: String = DEFAULT_URL,
    private val defaultModel: String = DEFAULT_MODEL,
) : ChatEngine {

    override val name = "ollama"

    private val json = Json { ignoreUnknownKeys = true }

    @OptIn(ExperimentalEncodingApi::class)
    override suspend fun ask(request: JsonRequest): String {
        val body = buildJsonObject {
            put("model", request.model?.takeIf { it.isNotBlank() } ?: defaultModel)
            put("stream", false)
            // Le schema, et non une consigne : c'est lui qui rend le JSON invalide
            // impossible plutot qu'improbable.
            put("format", request.schema)
            putJsonObject("options") {
                put("temperature", 0.2)
                put("num_ctx", 32768)
                put("num_predict", request.maxTokens)
            }
            put("messages", buildJsonArray {
                add(buildJsonObject {
                    put("role", "system")
                    put("content", request.system)
                })
                add(buildJsonObject {
                    put("role", "user")
                    put("content", request.user)
                    if (request.photos.isNotEmpty()) {
                        put("images", buildJsonArray {
                            request.photos.forEach { add(JsonPrimitive(Base64.encode(it.data))) }
                        })
                    }
                })
            })
        }

        val response: HttpResponse = client.post("${baseUrl.trimEnd('/')}/api/chat") {
            contentType(ContentType.Application.Json)
            setBody(body.toString())
        }
        if (!response.status.isSuccess()) {
            throw PlanUnavailable("Ollama a refuse la demande (${response.status.value}).")
        }

        val content = runCatching {
            json.parseToJsonElement(response.bodyAsText())
                .jsonObject["message"]?.jsonObject?.get("content")?.jsonPrimitive?.content
        }.getOrNull()
            ?: throw PlanUnavailable("Reponse d'Ollama inattendue : pas de contenu.")


        return content
    }

    companion object {
        const val DEFAULT_URL = "http://localhost:11434"
        const val DEFAULT_MODEL = "gemma4:e4b"
    }
}
