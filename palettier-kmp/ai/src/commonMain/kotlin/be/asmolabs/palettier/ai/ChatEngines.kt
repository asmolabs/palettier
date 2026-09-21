package be.asmolabs.palettier.ai

import io.ktor.client.HttpClient

/**
 * Le moteur a interroger maintenant.
 *
 * <p>Le reglage change a chaud : le moteur ne peut donc pas etre resolu une fois pour
 * toutes au demarrage. C'est une question posee a chaque demande, et la reponse peut
 * etre "aucun" -- auquel cas l'assistant se tait et le reste de l'application ne change
 * pas d'un pouce.</p>
 */
fun interface ChatEngines {
    suspend fun current(): ChatEngine?
}

/** Le moteur qu'indiquent les reglages du peintre. */
class SettingsChatEngines(
    private val store: AiSettingsStore,
    private val client: HttpClient,
) : ChatEngines {

    override suspend fun current(): ChatEngine? {
        val settings = store.current()
        if (!settings.usable) return null

        return when (settings.provider) {
            AiProvider.NONE -> null
            AiProvider.OLLAMA -> OllamaEngine(
                client,
                settings.baseUrl.ifBlank { OllamaEngine.DEFAULT_URL },
                settings.model.ifBlank { OllamaEngine.DEFAULT_MODEL },
            )
            AiProvider.OPENAI -> OpenAiEngine(
                client,
                settings.apiKey,
                settings.baseUrl.ifBlank { OpenAiEngine.DEFAULT_URL },
                settings.model.ifBlank { OpenAiEngine.DEFAULT_MODEL },
            )
            AiProvider.GEMINI -> GeminiEngine(
                client,
                settings.apiKey,
                settings.baseUrl.ifBlank { GeminiEngine.DEFAULT_URL },
                settings.model.ifBlank { GeminiEngine.DEFAULT_MODEL },
            )
        }
    }
}
