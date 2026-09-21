package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.port.SettingsRepository
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.respondError
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.utils.io.ByteReadChannel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Ce que le poste a d'installe.
 *
 * <p>Un serveur eteint n'est pas un incident : c'est le cas normal avant installation, et
 * le catalogue doit le dire plutot que d'echouer.</p>
 */
class ModelCatalogTest {

    private class InMemory : SettingsRepository {
        private val values = MutableStateFlow(emptyMap<String, String>())
        override fun observe(key: String): Flow<String?> = values.map { it[key] }
        override suspend fun get(key: String) = values.value[key]
        override suspend fun put(key: String, value: String) { values.value = values.value + (key to value) }
        override suspend fun remove(key: String) { values.value = values.value - key }
    }

    private suspend fun store(provider: AiProvider): AiSettingsStore =
        AiSettingsStore(InMemory()).also { it.save(AiSettings(provider, apiKey = "cle")) }

    private fun serving(vararg routes: Pair<String, String>) = HttpClient(MockEngine { request ->
        val body = routes.firstOrNull { request.url.encodedPath.endsWith(it.first) }?.second
        if (body == null) respondError(HttpStatusCode.NotFound)
        else respond(
            ByteReadChannel(body),
            HttpStatusCode.OK,
            headersOf(HttpHeaders.ContentType, "application/json"),
        )
    })

    private val tags = """
        {"models":[
          {"name":"qwen3-vl:8b","size":6000000000},
          {"name":"gemma4:e4b","size":3300000000}
        ]}
    """.trimIndent()

    @Test
    fun `les modeles installes remontent tries, avec leurs capacites`() = runTest {
        val client = serving(
            "/api/tags" to tags,
            "/api/show" to """{"capabilities":["completion","vision"]}""",
        )
        val installed = ModelCatalog(client, store(AiProvider.OLLAMA)).installed()

        assertEquals(listOf("gemma4:e4b", "qwen3-vl:8b"), installed.map { it.name })
        assertTrue(installed.first().supportsVision, "sans vision, l'assistant ne lit pas vos photos")
        assertEquals("3.3 Go", installed.first().sizeLabel)
    }

    @Test
    fun `un serveur eteint rend une liste vide, et non une erreur`() = runTest {
        val eteint = HttpClient(MockEngine { respondError(HttpStatusCode.ServiceUnavailable) })
        val catalogue = ModelCatalog(eteint, store(AiProvider.OLLAMA))

        assertEquals(emptyList(), catalogue.installed())
        assertFalse(catalogue.reachable())
    }

    @Test
    fun `un moteur distant n'a pas de catalogue a gerer`() = runTest {
        val catalogue = ModelCatalog(serving("/api/tags" to tags), store(AiProvider.OPENAI))

        assertFalse(catalogue.isManageable())
        assertEquals(emptyList(), catalogue.installed())
    }

    @Test
    fun `le telechargement rend compte etape par etape`() = runTest {
        val flux = """
            {"status":"pulling manifest"}
            {"status":"downloading","completed":500,"total":1000}
            {"status":"success"}
        """.trimIndent()
        val client = HttpClient(MockEngine {
            respond(ByteReadChannel(flux), HttpStatusCode.OK)
        })

        val etapes = ModelCatalog(client, store(AiProvider.OLLAMA)).pull("gemma4:e4b").toList()

        assertEquals(3, etapes.size)
        assertEquals(0.5, etapes[1].fraction)
        assertTrue(etapes.last().done)
        // Tant que le serveur n'annonce pas de total, il n'y a pas de fraction a montrer.
        assertEquals(-1.0, etapes.first().fraction)
    }

    @Test
    fun `une erreur de telechargement est transmise, et arrete le flux`() = runTest {
        val flux = """
            {"status":"pulling manifest"}
            {"error":"model not found"}
            {"status":"success"}
        """.trimIndent()
        val client = HttpClient(MockEngine { respond(ByteReadChannel(flux), HttpStatusCode.OK) })

        val etapes = ModelCatalog(client, store(AiProvider.OLLAMA)).pull("inconnu").toList()

        assertEquals(2, etapes.size)
        assertContains(etapes.last().status, "model not found")
        assertFalse(etapes.last().done)
    }
}
