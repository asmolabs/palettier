package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.port.SettingsRepository
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respondOk
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** Le reglage de l'assistant, et le moteur qu'il designe. */
class AiSettingsTest {

    /** Des reglages en memoire : la persistance a ses propres essais. */
    private class InMemory : SettingsRepository {
        private val values = MutableStateFlow(emptyMap<String, String>())

        override fun observe(key: String): Flow<String?> = values.map { it[key] }
        override suspend fun get(key: String) = values.value[key]
        override suspend fun put(key: String, value: String) {
            values.value = values.value + (key to value)
        }
        override suspend fun remove(key: String) {
            values.value = values.value - key
        }

        val keys: Set<String> get() = values.value.keys
    }

    @Test
    fun `sans rien de range, aucun moteur n'est choisi`() = runTest {
        val settings = AiSettingsStore(InMemory()).current()
        assertEquals(AiProvider.NONE, settings.provider)
        assertFalse(settings.usable)
    }

    @Test
    fun `un champ laisse vide s'efface plutot que de ranger du vide`() = runTest {
        val store = InMemory()
        AiSettingsStore(store).save(
            AiSettings(AiProvider.OLLAMA, baseUrl = "", apiKey = "", model = "gemma4:e4b")
        )

        assertEquals(setOf(AiSettingsStore.PROVIDER, AiSettingsStore.MODEL), store.keys)
    }

    @Test
    fun `un moteur distant sans cle n'est pas utilisable`() {
        assertFalse(AiSettings(AiProvider.OPENAI).usable)
        assertTrue(AiSettings(AiProvider.OPENAI, apiKey = "cle").usable)
        // Ollama tourne sur la machine meme : rien a presenter.
        assertTrue(AiSettings(AiProvider.OLLAMA).usable)
    }

    @Test
    fun `le moteur suit le reglage, sans redemarrage`() = runTest {
        val repository = InMemory()
        val store = AiSettingsStore(repository)
        val engines = SettingsChatEngines(store, HttpClient(MockEngine { respondOk() }))

        assertNull(engines.current(), "aucun moteur tant que rien n'est choisi")

        store.save(AiSettings(AiProvider.OLLAMA))
        assertIs<OllamaEngine>(engines.current())

        store.save(AiSettings(AiProvider.GEMINI, apiKey = "cle"))
        assertIs<GeminiEngine>(engines.current())

        // La cle retiree, le moteur distant redevient indisponible plutot que d'echouer
        // a chaque demande.
        store.save(AiSettings(AiProvider.GEMINI, apiKey = ""))
        assertNull(engines.current())
    }

    @Test
    fun `le reglage se relit tel qu'il a ete range`() = runTest {
        val store = AiSettingsStore(InMemory())
        val saved = AiSettings(AiProvider.OPENAI, "https://exemple.test/v1", "cle", "gpt-5")
        store.save(saved)
        assertEquals(saved, store.current())
    }
}
