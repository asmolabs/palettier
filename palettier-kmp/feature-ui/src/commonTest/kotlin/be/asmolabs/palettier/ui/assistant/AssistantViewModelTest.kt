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
import be.asmolabs.palettier.ai.Identification
import be.asmolabs.palettier.ai.ModelCatalog
import be.asmolabs.palettier.ai.PlanUnavailable
import be.asmolabs.palettier.ai.TubeRecognitionService
import be.asmolabs.palettier.ai.ZoneDraft
import be.asmolabs.palettier.domain.image.PixelMap
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.SettingsRepository
import be.asmolabs.palettier.image.ImageDecoder
import be.asmolabs.palettier.domain.paint.PaintMatcher
import be.asmolabs.palettier.ui.FakeCatalogRepository
import be.asmolabs.palettier.ui.FakePaletteRepository
import be.asmolabs.palettier.ui.testPaint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import io.ktor.client.engine.mock.respondError
import kotlinx.serialization.json.Json
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import be.asmolabs.palettier.domain.project.ProjectPhoto
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

    private val projects = be.asmolabs.palettier.ui.FakeProjectRepository()

    private fun model(engine: ChatEngine?, repository: SettingsRepository = InMemorySettings()): AssistantViewModel {
        val store = AiSettingsStore(repository)
        val engines = ChatEngines { engine }
        val service = PaintingPlanService(engines, PlanEnricher(ColorMixService()), Plain)
        return AssistantViewModel(
            service,
            store,
            FakePaletteRepository(listOf(zorn)),
            TubeRecognitionService(engines, PaintMatcher(), Plain),
            catalogue,
            ModelCatalog(io.ktor.client.HttpClient(io.ktor.client.engine.mock.MockEngine {
                respondError(io.ktor.http.HttpStatusCode.ServiceUnavailable)
            }), store),
            projects,
            Plain,
        )
    }

    /** Le catalogue du peintre, partage par les essais qui cochent des tubes. */
    private val catalogue = FakeCatalogRepository(
        listOf(
            testPaint(10, "Burnt Umber", "#8A5A44", brand = "Winsor & Newton", inStock = false),
            testPaint(11, "Ivory Black", "#221F1C", brand = "Gamblin", inStock = false),
        )
    )

    /**
     * Attend que l'etat remplisse une condition.
     *
     * <p>advanceUntilIdle ne suffit pas : le melange se calcule en parallele et le
     * catalogue de modeles parle a un serveur, deux vrais repartiteurs que l'horloge
     * virtuelle du test ne commande pas. On attend donc le resultat plutot que de faire
     * semblant d'avancer le temps.</p>
     */
    private suspend fun AssistantViewModel.await(condition: (AssistantUiState) -> Boolean): AssistantUiState =
        withContext(Dispatchers.Default) { state.first(condition) }

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
        val state = model.await { it.palettes.isNotEmpty() }

        assertEquals(AiProvider.NONE, state.settings.provider)
        assertFalse(state.canAsk, "rien a demander tant qu'aucun moteur n'est choisi")
    }

    @Test
    fun `la premiere palette est proposee d'emblee`() = runTest(dispatcher) {
        val state = model(null).await { it.palettes.isNotEmpty() }

        assertEquals(1L, state.paletteId)
        assertEquals("Zorn", state.palette?.name)
    }

    @Test
    fun `un sujet et un moteur suffisent a demander un plan`() = runTest(dispatcher) {
        val model = model(answering(draft))
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste de grognard"))
        assertTrue(model.await { it.canAsk }.canAsk)

        model.onIntent(AssistantIntent.Ask)
        val done = model.await { it.plan != null || it.error != null }

        val plan = assertNotNull(done.plan)
        assertEquals("buste de grognard", plan.subject)
        assertEquals("Zorn", plan.paletteName)
        // Le melange est calcule par l'application, pas demande au modele.
        assertNotNull(plan.zones.single().base?.recipe)
        assertFalse(done.working)
    }

    @Test
    fun `une photo suffit aussi, sans sujet ecrit`() = runTest(dispatcher) {
        val engine = answering(draft)
        val model = model(engine)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.AttachFigurine(byteArrayOf(1, 2, 3)))
        assertTrue(model.await { it.canAsk }.canAsk)

        model.onIntent(AssistantIntent.Ask)
        val done = model.await { it.plan != null || it.error != null }

        assertEquals(1, engine.asked!!.photos.size)
        assertNotNull(done.plan)
    }

    @Test
    fun `un moteur qui echoue laisse l'ecran utilisable`() = runTest(dispatcher) {
        val brise = object : ChatEngine {
            override val name = "casse"
            override suspend fun ask(request: JsonRequest): String =
                throw PlanUnavailable("Le modele n'a pas produit de plan exploitable.")
        }
        val model = model(brise)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste"))
        model.await { it.canAsk }
        model.onIntent(AssistantIntent.Ask)
        val done = model.await { it.error != null }

        assertFalse(done.working)
        assertNull(done.plan)
        assertTrue("plan exploitable" in done.error!!)
        assertTrue(done.canAsk, "on doit pouvoir reessayer")
    }

    @Test
    fun `une panne de reseau est dite, et non avalee`() = runTest(dispatcher) {
        val muet = object : ChatEngine {
            override val name = "muet"
            override suspend fun ask(request: JsonRequest): String =
                throw IllegalStateException("connexion refusee")
        }
        val model = model(muet)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste"))
        model.await { it.canAsk }
        model.onIntent(AssistantIntent.Ask)

        assertTrue("connexion refusee" in model.await { it.error != null }.error!!)
    }

    @Test
    fun `le plan propose devient un projet, avec les photos qui l'ont etabli`() = runTest(dispatcher) {
        val model = model(answering(draft))
        model.await { it.palettes.isNotEmpty() }
        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.onIntent(AssistantIntent.SetSubject("buste de grognard"))
        model.onIntent(AssistantIntent.AttachFigurine(byteArrayOf(1, 2, 3)))
        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(4, 5, 6)))
        model.onIntent(AssistantIntent.Ask)
        model.await { it.plan != null }

        model.onIntent(AssistantIntent.SaveProject("Grognard"))
        val done = model.await { it.savedMessage != null }

        assertFalse(done.saving)
        val project = projects.saved.value.single()
        assertEquals("Grognard", project.name)
        assertEquals("buste de grognard", project.subject)
        assertEquals(listOf("Visage"), project.zones.map { it.name })
        // Les tubes de la palette sont figes : le plan doit rester reproductible.
        assertEquals(4, project.paints.size)

        // Sans les photos, on relit dans six mois un plan dont on ne sait plus a quoi il
        // s'appliquait.
        assertEquals(
            listOf(ProjectPhoto.Role.PIECE, ProjectPhoto.Role.REFERENCE),
            project.photos.map { it.role },
        )
        assertTrue(done.savedMessage!!.contains("2 photos"), done.savedMessage!!)
    }

    @Test
    fun `le nom propose vient du sujet, ecourte s'il est bavard`() = runTest(dispatcher) {
        val model = model(null)
        model.await { it.palettes.isNotEmpty() }

        assertEquals("Projet sans titre", model.state.value.suggestedProjectName)

        model.onIntent(AssistantIntent.SetSubject("  buste de grognard  "))
        assertEquals("buste de grognard", model.state.value.suggestedProjectName)

        model.onIntent(AssistantIntent.SetSubject("b".repeat(80)))
        assertEquals(50, model.state.value.suggestedProjectName.length)
    }

    @Test
    fun `sans plan propose, il n'y a rien a enregistrer`() = runTest(dispatcher) {
        val model = model(null)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.SaveProject("Grognard"))
        advanceUntilIdle()

        assertTrue(projects.saved.value.isEmpty())
        assertNull(model.state.value.savedMessage)
    }

    @Test
    fun `une photo jointe porte sa vignette, prete a etre montree`() = runTest(dispatcher) {
        val model = model(null)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.AttachFigurine(byteArrayOf(1, 2, 3)))
        val state = model.await { it.figurine != null }

        val photo = assertNotNull(state.figurine)
        assertEquals("la piece telle qu'elle est", photo.caption)
        // Les octets d'origine partent au modele, la vignette va a l'ecran.
        assertTrue(photo.bytes.contentEquals(byteArrayOf(1, 2, 3)))
        assertTrue(photo.thumbnail.isNotEmpty())
    }

    @Test
    fun `retirer une reference renumerote celles qui restent`() = runTest(dispatcher) {
        val model = model(null)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(1)))
        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(2)))
        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(3)))
        model.await { it.references.size == 3 }

        model.onIntent(AssistantIntent.RemoveReference(1))
        val left = model.await { it.references.size == 2 }

        // La premiere et la troisieme, mais numerotees 1 et 2 : garder "reference 3"
        // ferait mentir la legende que le modele recoit avec la photo.
        assertEquals(listOf("reference 1", "reference 2"), left.references.map { it.caption })
        assertEquals(listOf<Byte>(1, 3), left.references.map { it.bytes.single() })
    }

    @Test
    fun `deux fois le meme fichier restent deux references distinctes`() = runTest(dispatcher) {
        val model = model(null)
        model.await { it.palettes.isNotEmpty() }

        // Deux cadrages de la meme image : un cas normal, pas une erreur a corriger.
        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(7)))
        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(7)))
        model.await { it.references.size == 2 }

        model.onIntent(AssistantIntent.RemoveReference(0))
        val left = model.await { it.references.size == 1 }
        assertEquals("reference 1", left.references.single().caption)
    }

    @Test
    fun `la photo de la piece se retire seule, sans emporter les references`() = runTest(dispatcher) {
        val model = model(null)
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(AssistantIntent.AttachFigurine(byteArrayOf(1)))
        model.onIntent(AssistantIntent.AttachReference(byteArrayOf(2)))
        model.await { it.figurine != null && it.references.size == 1 }

        model.onIntent(AssistantIntent.RemoveFigurine)
        val left = model.await { it.figurine == null }
        assertEquals(1, left.references.size)
    }

    @Test
    fun `le reglage se retient d'une session a l'autre`() = runTest(dispatcher) {
        val repository = InMemorySettings()
        val premier = model(answering(draft), repository)
        premier.await { it.palettes.isNotEmpty() }
        premier.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.GEMINI, apiKey = "cle")))
        premier.await { it.settings.provider == AiProvider.GEMINI }

        val second = model(answering(draft), repository).await { it.settings.provider != AiProvider.NONE }
        assertEquals(AiProvider.GEMINI, second.settings.provider)
        assertEquals("cle", second.settings.apiKey)
    }

    @Test
    fun `les tubes lus ne sont pas coches d'office`() = runTest(dispatcher) {
        val lecteur = object : ChatEngine {
            override val name = "lecteur"
            override suspend fun ask(request: JsonRequest) =
                """{"tubes":["Winsor & Newton Burnt Umber","Marque inconnue Bleu de nulle part"]}"""
        }
        val model = model(lecteur)
        model.await { it.palettes.isNotEmpty() }
        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.await { it.settings.usable }

        model.onIntent(AssistantIntent.ReadTubes(byteArrayOf(1)))
        model.await { it.tubes.isNotEmpty() || it.error != null }

        assertEquals(2, model.state.value.tubes.size)
        // Rien n'est encore acquis : le peintre n'a pas tranche.
        assertTrue(catalogue.paints.value.none { it.inStock })
        assertNull(model.state.value.tubes.last().match, "aucun rapprochement de travers")

        model.onIntent(AssistantIntent.KeepTube(model.state.value.tubes.first()))
        val after = model.await { it.tubes.size == 1 }

        assertTrue(catalogue.paints.value.single { it.id == 10L }.inStock)
        assertEquals(1, after.tubes.size, "le tube confirme quitte la liste")
    }

    @Test
    fun `une photo illisible le dit, sans rien cocher`() = runTest(dispatcher) {
        val bavard = object : ChatEngine {
            override val name = "bavard"
            override suspend fun ask(request: JsonRequest) = "je vois trois tubes"
        }
        val model = model(bavard)
        model.await { it.palettes.isNotEmpty() }
        model.onIntent(AssistantIntent.SaveSettings(AiSettings(AiProvider.OLLAMA)))
        model.await { it.settings.usable }

        model.onIntent(AssistantIntent.ReadTubes(byteArrayOf(1)))
        val done = model.await { it.error != null }

        assertTrue(done.tubes.isEmpty())
        assertTrue("etiquettes sont lisibles" in done.error!!)
        assertFalse(done.reading)
    }
}
