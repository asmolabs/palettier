package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.port.SettingsRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine

/** Les moteurs que l'application sait interroger. */
enum class AiProvider(val label: String) {

    /** Aucun : l'assistant est un supplement, et tout le reste marche sans lui. */
    NONE("Aucun"),

    /** Sur la machine meme, sans compte ni cle : rien ne sort du poste. */
    OLLAMA("Ollama (local)"),
    OPENAI("OpenAI"),
    GEMINI("Google Gemini");

    /** Vrai si le moteur exige une cle, donc l'envoi des photos a un service distant. */
    val needsKey: Boolean get() = this == OPENAI || this == GEMINI

    companion object {
        fun byName(name: String?): AiProvider =
            entries.firstOrNull { it.name.equals(name, ignoreCase = true) } ?: NONE
    }
}

/**
 * Le reglage de l'assistant.
 *
 * <p>Du temps de Spring, tout cela vivait dans application.yml et demandait un
 * redemarrage. Ici c'est un reglage de l'application, change a chaud : comparer deux
 * modeles sur le meme sujet est un geste d'essai, pas une decision a graver.</p>
 */
data class AiSettings(
    val provider: AiProvider = AiProvider.NONE,
    /** Vide pour l'adresse par defaut du moteur choisi. */
    val baseUrl: String = "",
    val apiKey: String = "",
    /** Vide pour le modele par defaut du moteur choisi. */
    val model: String = "",
) {
    /** Vrai quand le reglage suffit a interroger un moteur. */
    val usable: Boolean get() = provider != AiProvider.NONE &&
        (!provider.needsKey || apiKey.isNotBlank())
}

/**
 * Les reglages de l'assistant, ranges avec les autres.
 *
 * <p>La cle d'un service distant est ecrite en clair dans la base locale, comme le reste.
 * C'est un choix a connaitre : la base vit dans le repertoire de l'utilisateur, et rien
 * ne la chiffre. Elle n'est en revanche pas exportee avec les sauvegardes -- une
 * sauvegarde se transporte, une cle non.</p>
 */
class AiSettingsStore(private val settings: SettingsRepository) {

    fun observe(): Flow<AiSettings> = combine(
        settings.observe(PROVIDER),
        settings.observe(BASE_URL),
        settings.observe(API_KEY),
        settings.observe(MODEL),
    ) { provider, baseUrl, apiKey, model ->
        AiSettings(AiProvider.byName(provider), baseUrl.orEmpty(), apiKey.orEmpty(), model.orEmpty())
    }

    suspend fun current(): AiSettings = AiSettings(
        provider = AiProvider.byName(settings.get(PROVIDER)),
        baseUrl = settings.get(BASE_URL).orEmpty(),
        apiKey = settings.get(API_KEY).orEmpty(),
        model = settings.get(MODEL).orEmpty(),
    )

    suspend fun save(value: AiSettings) {
        settings.put(PROVIDER, value.provider.name)
        write(BASE_URL, value.baseUrl)
        write(API_KEY, value.apiKey)
        write(MODEL, value.model)
    }

    /** Un champ vide s'efface plutot que de ranger du vide : le defaut reprend la main. */
    private suspend fun write(key: String, value: String) {
        if (value.isBlank()) settings.remove(key) else settings.put(key, value.trim())
    }

    companion object {
        const val PROVIDER = "ai.provider"
        const val BASE_URL = "ai.base-url"
        const val API_KEY = "ai.api-key"
        const val MODEL = "ai.model"
    }
}
