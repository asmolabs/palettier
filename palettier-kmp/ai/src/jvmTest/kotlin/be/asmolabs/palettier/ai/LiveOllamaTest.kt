package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.image.imageDecoder
import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * L'essai qui parle vraiment a Ollama.
 *
 * <p>Il est eteint par defaut, et c'est delibere : une suite de tests ne doit pas dependre
 * d'un serveur allume ni prendre plusieurs minutes. Il s'allume avec
 * {@code PALETTIER_OLLAMA=1 ./gradlew :ai:jvmTest}, et sert a verifier ce qu'aucun serveur
 * simule ne peut prouver -- qu'un vrai modele, sous contrainte de schema, rend un plan
 * complet et non un JSON coupe en plein objet.</p>
 *
 * <p>C'est precisement ce qui avait coute une semaine du cote Java : la contrainte remise
 * au moteur n'est pas la meme chose qu'une consigne ajoutee au prompt.</p>
 */
class LiveOllamaTest {

    private val enabled = System.getenv("PALETTIER_OLLAMA") == "1"
    private val model = System.getenv("PALETTIER_OLLAMA_MODEL") ?: OllamaEngine.DEFAULT_MODEL

    private val zorn = Palette(
        id = 1, name = "Zorn",
        paints = listOf(
            Paint(brand = "Gamblin", name = "Titanium White", hexColor = "#F4F2EC", pigments = setOf("PW6")),
            Paint(brand = "Gamblin", name = "Ivory Black", hexColor = "#221F1C", pigments = setOf("PBk9")),
            Paint(brand = "Gamblin", name = "Yellow Ochre", hexColor = "#B07C2A", pigments = setOf("PY43")),
            Paint(brand = "Gamblin", name = "Cadmium Red", hexColor = "#B3232A", pigments = setOf("PR108")),
        ),
    )

    private fun client() = HttpClient {
        install(HttpTimeout) {
            requestTimeoutMillis = 15 * 60 * 1000
            connectTimeoutMillis = 30 * 1000
            socketTimeoutMillis = 15 * 60 * 1000
        }
    }

    @Test
    fun `un vrai modele rend un plan complet sous contrainte de schema`() {
        if (!enabled) return
        runBlocking {
            val engine = OllamaEngine(client(), defaultModel = model)
            val service = PaintingPlanService(
                ChatEngines { engine },
                PlanEnricher(ColorMixService()),
                imageDecoder(),
            )

            val plan = service.plan("buste de grognard napoleonien, manteau bleu et bonnet a poil", zorn, 3)

            assertTrue(plan.zones.isNotEmpty(), "le modele doit decouper le sujet en zones")
            plan.zones.forEach { zone ->
                assertTrue(zone.name.isNotBlank())
                // Une reponse coupee se reconnait ici : les dernieres zones arrivent vides.
                assertTrue(zone.base != null, "zone ${zone.name} sans couche de base")
                assertTrue(zone.shadows.size == 2, "zone ${zone.name} : echelle d'ombres incomplete")
                assertTrue(zone.highlights.size == 2, "zone ${zone.name} : echelle de lumieres incomplete")
            }

            // Le melange est calcule par l'application : chaque couche doit en porter un.
            val sansMelange = plan.zones.flatMap { it.layers() }.count { it.recipe == null }
            assertTrue(sansMelange == 0, "$sansMelange couches sans melange calcule")
        }
    }

    @Test
    fun `le catalogue local liste ce qui est installe`() {
        if (!enabled) return
        runBlocking {
            val store = AiSettingsStore(EnvSettings)
            val installed = ModelCatalog(client(), store).installed()
            assertTrue(installed.isNotEmpty(), "aucun modele installe sur ce poste ?")
        }
    }

    /** Des reglages figes sur Ollama : cet essai ne touche pas a la base du peintre. */
    private object EnvSettings : be.asmolabs.palettier.domain.port.SettingsRepository {
        private val values = mapOf(AiSettingsStore.PROVIDER to AiProvider.OLLAMA.name)
        override fun observe(key: String) = kotlinx.coroutines.flow.flowOf(values[key])
        override suspend fun get(key: String) = values[key]
        override suspend fun put(key: String, value: String) = Unit
        override suspend fun remove(key: String) = Unit
    }
}
