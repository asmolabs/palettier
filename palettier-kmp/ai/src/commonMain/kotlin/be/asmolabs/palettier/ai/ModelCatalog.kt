package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.preparePost
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsChannel
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.utils.io.readUTF8Line
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long
import kotlinx.serialization.json.put

/**
 * Un modele installe.
 *
 * @param capabilities ce que le modele sait faire. La presence de "vision" est decisive :
 *                     sans elle, l'assistant ne peut pas lire vos photos.
 */
data class ModelInfo(val name: String, val sizeBytes: Long, val capabilities: List<String>) {

    val supportsVision: Boolean get() = "vision" in capabilities

    val sizeLabel: String get() = if (sizeBytes <= 0) "" else {
        val go = sizeBytes / 1e9
        "${(go * 10).toLong() / 10.0} Go"
    }
}

/** Avancement d'un telechargement. */
data class PullProgress(val status: String, val completed: Long = 0, val total: Long = 0) {

    /** Fraction telechargee, ou -1 quand le serveur ne l'indique pas encore. */
    val fraction: Double get() = if (total > 0) completed.toDouble() / total else -1.0

    val done: Boolean get() = status == "success"
}

/**
 * Dialogue avec le serveur Ollama local : ce qui est installe, et comment en installer
 * davantage.
 *
 * <p>Ollama publie tout cela sans authentification, sur la machine meme. Les autres
 * moteurs exposent des catalogues lies a un compte : pour eux, le nom du modele reste une
 * saisie libre, et ce catalogue-ci se declare simplement injoignable.</p>
 */
class ModelCatalog(
    private val client: HttpClient,
    private val store: AiSettingsStore,
) {

    private val json = Json { ignoreUnknownKeys = true }

    /** Vrai quand le moteur actif permet de lister et d'installer des modeles. */
    suspend fun isManageable(): Boolean = store.current().provider == AiProvider.OLLAMA

    private suspend fun baseUrl(): String =
        store.current().baseUrl.ifBlank { OllamaEngine.DEFAULT_URL }.trimEnd('/')

    /** Vrai si le serveur repond. */
    suspend fun reachable(): Boolean = isManageable() && get("/api/tags") != null

    /** Modeles installes, avec leurs capacites. Liste vide si le serveur ne repond pas. */
    suspend fun installed(): List<ModelInfo> {
        val tags = if (isManageable()) get("/api/tags") else null
        val models = tags?.jsonObject?.get("models")?.jsonArray ?: return emptyList()

        return models.mapNotNull { entry ->
            val name = entry.jsonObject["name"]?.jsonPrimitive?.content.orEmpty()
            if (name.isBlank()) null else ModelInfo(
                name = name,
                sizeBytes = entry.jsonObject["size"]?.jsonPrimitive?.long ?: 0L,
                capabilities = capabilitiesOf(name),
            )
        }.sortedBy { it.name.lowercase() }
    }

    private suspend fun capabilitiesOf(model: String): List<String> {
        val details = post("/api/show", buildJsonObject { put("model", model) }.toString())
        return details?.jsonObject?.get("capabilities")?.jsonArray
            ?.map { it.jsonPrimitive.content }
            ?: emptyList()
    }

    /**
     * Telecharge un modele et rend compte de l'avancement au fur et a mesure.
     *
     * <p>Ollama repond en flux : une ligne JSON par etape. On les transmet telles quelles
     * plutot que d'attendre la fin, sans quoi l'ecran resterait fige pendant plusieurs
     * gigaoctets.</p>
     */
    fun pull(model: String): Flow<PullProgress> = flow {
        if (!isManageable() || model.isBlank()) {
            emit(PullProgress("Le telechargement n'est possible qu'avec Ollama."))
            return@flow
        }

        val body = buildJsonObject {
            put("model", model.trim())
            put("stream", true)
        }.toString()

        client.preparePost("${baseUrl()}/api/pull") {
            contentType(ContentType.Application.Json)
            setBody(body)
        }.execute { response ->
            if (!response.status.isSuccess()) {
                emit(PullProgress("refuse par le serveur (${response.status.value})"))
                return@execute
            }
            val channel = response.bodyAsChannel()
            while (true) {
                val line = channel.readUTF8Line() ?: break
                if (line.isBlank()) continue
                val step = runCatching { json.parseToJsonElement(line).jsonObject }.getOrNull() ?: continue

                step["error"]?.jsonPrimitive?.content?.let {
                    emit(PullProgress(it))
                    return@execute
                }
                emit(
                    PullProgress(
                        status = step["status"]?.jsonPrimitive?.content.orEmpty(),
                        completed = step["completed"]?.jsonPrimitive?.long ?: 0L,
                        total = step["total"]?.jsonPrimitive?.long ?: 0L,
                    )
                )
            }
        }
    }

    // --- Acces HTTP --------------------------------------------------------

    private suspend fun get(path: String): JsonElement? = read { client.get("${baseUrl()}$path") }

    private suspend fun post(path: String, body: String): JsonElement? = read {
        client.post("${baseUrl()}$path") {
            contentType(ContentType.Application.Json)
            setBody(body)
        }
    }

    /** Un serveur eteint n'est pas un incident : c'est le cas normal avant installation. */
    private suspend fun read(call: suspend () -> HttpResponse): JsonElement? = runCatching {
        val response = call()
        if (response.status.isSuccess()) json.parseToJsonElement(response.bodyAsText()) else null
    }.getOrNull()
}
