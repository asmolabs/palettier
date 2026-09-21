package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
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
 * Le moteur distant d'OpenAI, et tout service qui en parle le dialecte.
 *
 * <p>La contrainte de generation existe ici aussi, sous un autre nom : {@code
 * response_format} en mode {@code json_schema}, avec {@code strict}. C'est pour lui que
 * {@link PlanSchema#strict} existe -- OpenAI ne garantit la conformite que si aucun champ
 * n'est facultatif, ce qui se dit en admettant le nul plutot qu'en omettant.</p>
 *
 * <p>Les images voyagent en URL de donnees dans le message lui-meme, et non a cote. Le
 * modele doit etre capable de les lire : demander un plan d'apres photo a un modele
 * textuel ne donne rien d'utile.</p>
 */
class OpenAiEngine(
    private val client: HttpClient,
    private val apiKey: String,
    private val baseUrl: String = DEFAULT_URL,
    private val defaultModel: String = DEFAULT_MODEL,
) : ChatEngine {

    override val name = "openai"

    private val json = Json { ignoreUnknownKeys = true }

    @OptIn(ExperimentalEncodingApi::class)
    override suspend fun ask(request: JsonRequest): String {
        val body = buildJsonObject {
            put("model", request.model?.takeIf { it.isNotBlank() } ?: defaultModel)
            put("max_completion_tokens", request.maxTokens)
            putJsonObject("response_format") {
                put("type", "json_schema")
                putJsonObject("json_schema") {
                    put("name", "reponse")
                    put("strict", true)
                    put("schema", Schemas.strict(request.schema))
                }
            }
            putJsonArray("messages") {
                add(buildJsonObject {
                    put("role", "system")
                    put("content", request.system)
                })
                add(buildJsonObject {
                    put("role", "user")
                    put("content", buildJsonArray {
                        add(buildJsonObject {
                            put("type", "text")
                            put("text", request.user)
                        })
                        request.photos.forEach { photo ->
                            add(buildJsonObject {
                                put("type", "image_url")
                                putJsonObject("image_url") {
                                    put("url", "data:$PHOTO_MIME;base64,${Base64.encode(photo.data)}")
                                }
                            })
                        }
                    })
                })
            }
        }

        val response: HttpResponse = client.post("${baseUrl.trimEnd('/')}/chat/completions") {
            contentType(ContentType.Application.Json)
            header(HttpHeaders.Authorization, "Bearer $apiKey")
            setBody(body.toString())
        }
        if (!response.status.isSuccess()) {
            throw PlanUnavailable("OpenAI a refuse la demande (${response.status.value}).")
        }

        val content = runCatching {
            json.parseToJsonElement(response.bodyAsText())
                .jsonObject["choices"]?.jsonArray?.firstOrNull()
                ?.jsonObject?.get("message")?.jsonObject?.get("content")?.jsonPrimitive?.content
        }.getOrNull()
            ?: throw PlanUnavailable("Reponse d'OpenAI inattendue : pas de contenu.")

        return content
    }

    companion object {
        const val DEFAULT_URL = "https://api.openai.com/v1"
        const val DEFAULT_MODEL = "gpt-5"
    }
}
