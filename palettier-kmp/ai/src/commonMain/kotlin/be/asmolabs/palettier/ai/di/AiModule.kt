package be.asmolabs.palettier.ai.di

import be.asmolabs.palettier.ai.AiSettingsStore
import be.asmolabs.palettier.ai.ChatEngines
import be.asmolabs.palettier.ai.ModelCatalog
import be.asmolabs.palettier.ai.PaintingPlanService
import be.asmolabs.palettier.ai.PlanEnricher
import be.asmolabs.palettier.ai.TubeRecognitionService
import be.asmolabs.palettier.ai.SettingsChatEngines
import be.asmolabs.palettier.image.imageDecoder
import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import org.koin.dsl.module

/**
 * L'assistant, assemble.
 *
 * <p>Le module se declare toujours, meme sans moteur choisi : c'est {@link ChatEngines}
 * qui repond "aucun", et l'ecran le dit au peintre. Un module absent aurait oblige le
 * reste du graphe a savoir si l'assistant existe.</p>
 */
val aiModule = module {

    single {
        HttpClient {
            install(HttpTimeout) {
                // Un modele local qui lit une photo prend des minutes, pas des secondes.
                requestTimeoutMillis = 15 * 60 * 1000
                connectTimeoutMillis = 30 * 1000
                socketTimeoutMillis = 15 * 60 * 1000
            }
        }
    }

    single { AiSettingsStore(get()) }
    single<ChatEngines> { SettingsChatEngines(get(), get()) }
    single { PlanEnricher(get()) }
    single { PaintingPlanService(get(), get(), imageDecoder()) }
    single { TubeRecognitionService(get(), get(), imageDecoder()) }
    single { ModelCatalog(get(), get()) }
}
