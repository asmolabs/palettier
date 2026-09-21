package be.asmolabs.palettier.ui.assistant

import be.asmolabs.palettier.ai.AiProvider
import be.asmolabs.palettier.ai.AiSettings
import be.asmolabs.palettier.ai.AiSettingsStore
import be.asmolabs.palettier.ai.ChatEngine
import be.asmolabs.palettier.ai.ChatEngines
import be.asmolabs.palettier.ai.LayerDraft
import be.asmolabs.palettier.ai.PaintingPlanService
import be.asmolabs.palettier.ai.PlanDraft
import be.asmolabs.palettier.ai.PlanEnricher
import be.asmolabs.palettier.ai.JsonRequest
import be.asmolabs.palettier.ai.PlanUnavailable
import be.asmolabs.palettier.ai.ZoneDraft
import be.asmolabs.palettier.domain.image.PixelMap
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.SettingsRepository
import be.asmolabs.palettier.image.ImageDecoder
import be.asmolabs.palettier.ui.FakePaletteRepository
import be.asmolabs.palettier.ui.testPaint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.serialization.json.Json
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * L'assistant vu de l'ecran.
 *
 * <p>Ce qui compte ici n'est pas le plan lui-meme -- il a ses propres essais -- mais ce
 * que l'ecran devient quand le moteur manque, quand il repond, et quand il echoue.</p>
 */
@OptIn(ExperimentalCoroutinesApi::class)
class AssistantViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private class InMemorySettings : SettingsRepository {
        private val values = MutableStateFlow(emptyMap<String, String>())
        override fun observe(key: String): Flow<String?> = values.map { it[key] }
        override suspend fun get(key: String) = values.value[key]
        override suspend fun put(key: String, value: String) { values.value = values.value + (key to value) }
        override suspend fun remove(key: String) { values.value = values.value - key }
    }

    private object Plain : ImageDecoder {
        override suspend fun decode(bytes: ByteArray) = PixelMap(4, 4, IntArray(16) { 0xFF808080.toInt() })
        override suspend fun scaleTo(bytes: ByteArray, maxEdge: Int) = bytes
    }

    private val zorn = Palette(
        id = 1, name = "Zorn",
        paints = listOf(
            testPaint(1, "Titanium White", "#F4F2EC"),
            testPaint(2, "Ivory Black", "#221F1C"),
            testPaint(3, "Yellow Ochre", "#B07C2A"),
            testPaint(4, "Cadmium Red", "#B3232A"),
        ),
    )

    private val draft = PlanDraft(
        approach = "Palette courte.",
        zones = listOf(
            ZoneDraft(
                name = "Visage", material = "Peau", note = "",
                base = LayerDraft("#C98F72", "Glacis", ""),
                shadow1 = LayerDraft("#8A5F4A", "Glacis", ""),
                shadow2 = LayerDraft("#5A3B2E", "Glacis", ""),
                highlight1 = LayerDraft("#E0B49A", "Glacis", ""),
                highlight2 = LayerDraft("#F2D6C0", "Glacis", ""),
            )
        ),
    )

    private fun model(engine: ChatEngine?, repository: SettingsRepository = InMemorySettings()): AssistantViewModel {
        val store = AiSettingsStore(repository)
        val service = PaintingPlanService(ChatEngines { engine }, PlanEnricher(ColorMixService()), Plain)
        return AssistantViewModel(service, store, FakePaletteRepository(listOf(zorn)))
    }

    private fun answering(reply: PlanDraft) = object : ChatEngine {
        override val name = "essai"
        var asked: JsonRequest? = null
        override suspend fun ask(request: JsonRequest): String {
            asked = request
            return Json.encodeToString(reply)
        }
    }

    @Test
    fun `sans moteur choisi, la demande n'est pas proposee`() = runTest(dispatcher) {
        val model = model(null)
        advanceUntilIdle()

        assertEquals(AiProvider.NONE, model.state.value.settings.provider)
        assertFalse(model.state.value.canAsk, "rien a demander tant qu'aucun moteur n'est choisi")
    }

    @Test
    fun `la premiere palette est proposee d'emblee`() = runTest(dispatcher) {
        val model = model(null)
        advanceUntilIdle()

        assertEquals(1L, model.state.value.paletteId)
        assertEquals("Zorn", model.state.value.palette?.name)
    }

    @Test
    fun `un sujet et un moteur suffisent a demander un plan`() = runTest(dispatcher) {
        val model = model(answering(draft))
        advanceUntilIdle()

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste de grognard"))
        advanceUntilIdle()
        assertTrue(model.state.value.canAsk)

        model.onIntent(AssistantIntent.Ask)
        advanceUntilIdle()

        val plan = assertNotNull(model.state.value.plan)
        assertEquals("buste de grognard", plan.subject)
        assertEquals("Zorn", plan.paletteName)
        // Le melange est calcule par l'application, pas demande au modele.
        assertNotNull(plan.zones.single().base?.recipe)
        assertFalse(model.state.value.working)
    }

    @Test
    fun `une photo suffit aussi, sans sujet ecrit`() = runTest(dispatcher) {
        val engine = answering(draft)
        val model = model(engine)
        advanceUntilIdle()

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.AttachFigurine(byteArrayOf(1, 2, 3)))
        advanceUntilIdle()
        assertTrue(model.state.value.canAsk)

        model.onIntent(AssistantIntent.Ask)
        advanceUntilIdle()

        assertEquals(1, engine.asked!!.photos.size)
        assertNotNull(model.state.value.plan)
    }

    @Test
    fun `un moteur qui echoue laisse l'ecran utilisable`() = runTest(dispatcher) {
        val brise = object : ChatEngine {
            override val name = "casse"
            override suspend fun ask(request: JsonRequest): String =
                throw PlanUnavailable("Le modele n'a pas produit de plan exploitable.")
        }
        val model = model(brise)
        advanceUntilIdle()

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste"))
        advanceUntilIdle()
        model.onIntent(AssistantIntent.Ask)
        advanceUntilIdle()

        assertFalse(model.state.value.working)
        assertNull(model.state.value.plan)
        assertTrue("plan exploitable" in model.state.value.error!!)
        assertTrue(model.state.value.canAsk, "on doit pouvoir reessayer")
    }

    @Test
    fun `une panne de reseau est dite, et non avalee`() = runTest(dispatcher) {
        val muet = object : ChatEngine {
            override val name = "muet"
            override suspend fun ask(request: JsonRequest): String =
                throw IllegalStateException("connexion refusee")
        }
        val model = model(muet)
        advanceUntilIdle()

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste"))
        advanceUntilIdle()
        model.onIntent(AssistantIntent.Ask)
        advanceUntilIdle()

        assertTrue("connexion refusee" in model.state.value.error!!)
    }

    @Test
    fun `le reglage se retient d'une session a l'autre`() = runTest(dispatcher) {
        val repository = InMemorySettings()
        val premier = model(answering(draft), repository)
        advanceUntilIdle()
        premier.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.GEMINI, apiKey = "cle")))
        advanceUntilIdle()

        val second = model(answering(draft), repository)
        advanceUntilIdle()
        assertEquals(AiProvider.GEMINI, second.state.value.settings.provider)
        assertEquals("cle", second.state.value.settings.apiKey)
    }
}
