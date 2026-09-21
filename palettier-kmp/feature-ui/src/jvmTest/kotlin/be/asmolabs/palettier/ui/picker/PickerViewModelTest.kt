package be.asmolabs.palettier.ui.picker

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.image.imageDecoder
import be.asmolabs.palettier.ui.SearchScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withContext
import java.awt.Color
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class PickerViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private fun paint(id: Long, name: String, hex: String, inStock: Boolean = true) = Paint(
        id = id, brand = "W&N", name = name, hexColor = hex, inStock = inStock,
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.MEDIUM, tintingStrength = 0.8,
    )

    private val catalogue = listOf(
        paint(1, "Flesh Tint", "#C98F72"),
        paint(2, "Burnt Umber", "#4A3427"),
        paint(3, "Titanium White", "#F4F2EC"),
        paint(4, "Cadmium Red", "#B22222", inStock = false),
    )

    private class FakeCatalog(val paints: List<Paint>) : PaintCatalogRepository {
        override fun observeAll(): Flow<List<Paint>> = MutableStateFlow(paints)
        override suspend fun all() = paints
        override suspend fun inStock() = paints.filter { it.inStock }
        override suspend fun findById(id: Long) = paints.firstOrNull { it.id == id }
        override suspend fun findByNaturalKey(brand: String, name: String) = null
        override suspend fun save(paint: Paint) = paint
        override suspend fun setOwned(paint: Paint, owned: Boolean) {}
        override suspend fun recordTint(paint: Paint, tint: Rgb) {}
        override suspend fun recordMasstone(paint: Paint, measured: Rgb) {}
        override suspend fun countOwned() = paints.count { it.inStock }.toLong()
        // Catalogue fige : cet ecran ne l'ecrit pas.
        override suspend fun saveAll(paints: List<Paint>) {}
        override suspend fun count() = paints.size.toLong()
        override suspend fun refreshIdentity(paint: Paint) {}
        override suspend fun idsInUse(): Set<Long> = emptySet()
        override suspend fun delete(paint: Paint) {}
    }

    private fun image(vararg hex: String): ByteArray {
        val image = BufferedImage(200, 100, BufferedImage.TYPE_INT_RGB)
        val g = image.createGraphics()
        val band = 200 / hex.size
        hex.forEachIndexed { i, colour ->
            g.color = Color.decode(colour)
            g.fillRect(i * band, 0, band, 100)
        }
        g.dispose()
        return ByteArrayOutputStream().also { ImageIO.write(image, "png", it) }.toByteArray()
    }

    /** Une palette qui ne contient que la terre d'ombre : de quoi voir la portee mordre. */
    private fun palettes() = be.asmolabs.palettier.ui.FakePaletteRepository(
        listOf(
            be.asmolabs.palettier.domain.palette.Palette(
                id = 7,
                name = "Zorn",
                paints = listOf(catalogue[1]),
            )
        )
    )

    private val shared = be.asmolabs.palettier.ui.SampledColour()

    private val projects = be.asmolabs.palettier.ui.FakeProjectRepository()

    private fun model() = PickerViewModel(
        imageDecoder(), FakeCatalog(catalogue), palettes(), shared, ColorMixService(),
        projects, be.asmolabs.palettier.domain.plan.SampledPlanService(ColorMixService()),
    )

    /**
     * Attend que l'etat remplisse une condition.
     *
     * <p>advanceUntilIdle ne suffit pas ici : le decodage bascule sur Dispatchers.IO, un
     * vrai repartiteur que l'horloge virtuelle du test ne commande pas. On attend donc le
     * resultat plutot que de faire semblant d'avancer le temps.</p>
     */
    private suspend fun PickerViewModel.await(condition: (PickerUiState) -> Boolean): PickerUiState =
        withContext(Dispatchers.Default) { state.first(condition) }

    @Test
    fun `une photo chargee rend ses teintes dominantes`() = runTest(dispatcher) {
        val model = model()

        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        val state = model.await { it.dominant.isNotEmpty() }

        assertTrue(!state.loading)
        assertEquals(2, state.dominant.size)
        assertEquals("#C98F72", state.selectedHex)
        // Rien de plus n'arrive tout seul : la recette se demande.
        assertTrue(state.suggestions.isEmpty())
    }

    @Test
    fun `un melange a un tube rend le tube le plus proche, quand c'est ce qu'on veut`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        model.await { it.dominant.isNotEmpty() }

        // Chercher un tube seul est un cas particulier du melange, pas une liste a part.
        model.onIntent(PickerIntent.SetMaxPaints(1))
        model.onIntent(PickerIntent.FindMixes)
        val state = model.await { it.suggestions.isNotEmpty() }

        val best = state.suggestions.first()
        assertEquals(1, best.parts.size)
        assertEquals("Flesh Tint", best.parts.single().paint.name)
        assertTrue(best.deltaE < 1.0)
    }

    @Test
    fun `choisir une autre teinte repose la question, et la reponse change`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        model.await { it.dominant.isNotEmpty() }
        model.onIntent(PickerIntent.SetMaxPaints(1))
        model.onIntent(PickerIntent.FindMixes)
        model.await { it.suggestions.isNotEmpty() }

        model.onIntent(PickerIntent.Select("#4A3427"))
        // La reponse precedente est oubliee : elle portait sur une autre teinte.
        assertTrue(model.await { it.selectedHex == "#4A3427" }.suggestions.isEmpty())

        model.onIntent(PickerIntent.FindMixes)
        val state = model.await { it.suggestions.isNotEmpty() }
        assertEquals("Burnt Umber", state.suggestions.first().parts.single().paint.name)
    }

    @Test
    fun `l'etagere borne la recherche, et le tube absent n'entre dans aucun melange`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#B22222")))
        model.await { it.dominant.isNotEmpty() }
        model.onIntent(PickerIntent.FindMixes)

        // Par defaut on ne cherche que dans ce qu'on possede.
        val owned = model.await { it.suggestions.isNotEmpty() }
        assertTrue(owned.suggestions.none { s -> s.parts.any { it.paint.name == "Cadmium Red" } })

        model.onIntent(PickerIntent.ChooseScope(SearchScope.Catalogue))
        model.onIntent(PickerIntent.FindMixes)
        val all = model.await { it.scope == SearchScope.Catalogue && it.suggestions.isNotEmpty() }
        assertTrue(all.suggestions.any { s -> s.parts.any { it.paint.name == "Cadmium Red" } })
    }

    @Test
    fun `chercher dans une palette ne melange plus que ses tubes`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.dominant.isNotEmpty() }

        val zorn = model.await { it.scopes.any { scope -> scope.label == "Zorn" } }
            .scopes.first { it.label == "Zorn" }
        model.onIntent(PickerIntent.ChooseScope(zorn))
        model.onIntent(PickerIntent.FindMixes)
        val inPalette = model.await { it.scope == zorn && it.suggestions.isNotEmpty() }

        // La palette ne contient que la terre d'ombre : la reponse est faisable, pas juste.
        assertTrue(
            inPalette.suggestions.all { s -> s.parts.all { it.paint.name == "Burnt Umber" } },
            inPalette.suggestions.map { s -> s.parts.map { it.paint.name } }.toString(),
        )
    }

    @Test
    fun `une palette vide le dit, au lieu de ne rien proposer sans raison`() = runTest(dispatcher) {
        val palettes = be.asmolabs.palettier.ui.FakePaletteRepository(
            listOf(be.asmolabs.palettier.domain.palette.Palette(id = 9, name = "A composer"))
        )
        val model = PickerViewModel(
            imageDecoder(), FakeCatalog(catalogue), palettes, shared, ColorMixService(),
            be.asmolabs.palettier.ui.FakeProjectRepository(), be.asmolabs.palettier.domain.plan.SampledPlanService(ColorMixService()),
        )
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.dominant.isNotEmpty() }

        val empty = model.await { it.scopes.any { scope -> scope.label == "A composer" } }
            .scopes.first { it.label == "A composer" }
        model.onIntent(PickerIntent.ChooseScope(empty))

        val state = model.await { it.scope == empty && it.emptyScope }
        assertTrue(state.suggestions.isEmpty())
    }

    @Test
    fun `designer un gris neutre corrige la dominante de la photo`() = runTest(dispatcher) {
        val model = model()
        // Une carnation et un blanc de socle, tous deux photographies sous lampe chaude.
        model.onIntent(PickerIntent.Load(image("#D9A070", "#E8D2A8")))
        model.await { it.dominant.isNotEmpty() }
        model.onIntent(PickerIntent.Select("#D9A070"))
        assertNull(model.await { it.selectedHex == "#D9A070" }.correctedHex, "sans point gris, rien n'est corrige")

        model.onIntent(PickerIntent.UseAsGrey("#E8D2A8"))
        val corrected = model.await { it.correctedHex != null }.correctedHex
        assertNotNull(corrected)
        // La correction retire la chaleur : l'ecart rouge-bleu se resserre.
        val before = Rgb.ofHex("#D9A070")
        val after = Rgb.ofHex(corrected)
        assertTrue((after.r - after.b) < (before.r - before.b), "avant ${before.toHex()}, apres $corrected")
    }


    @Test
    fun `un clic sur la photo releve la teinte de ce point, pas une dominante`() = runTest(dispatcher) {
        val model = model()
        // Trois bandes de 66 pixels : l'ocre, la terre d'ombre, le blanc.
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427", "#F4F2EC")))
        model.await { it.dominant.isNotEmpty() }

        // Au milieu de la bande du milieu, loin des frontieres.
        model.onIntent(PickerIntent.PickAt(100, 50))
        val state = model.await { it.selectedHex == "#4A3427" }

        assertEquals(PickPoint(100, 50), state.pickedAt)
        // La teinte relevee est celle du pixel vise, pas une dominante du lot.
        assertEquals("#4A3427", state.workingHex)
    }

    @Test
    fun `la photo ouverte est gardee telle quelle, pour etre affichee`() = runTest(dispatcher) {
        val model = model()
        val bytes = image("#C98F72", "#4A3427")

        model.onIntent(PickerIntent.Load(bytes))
        val state = model.await { it.image != null }

        assertEquals(200, state.image?.width)
        assertEquals(100, state.image?.height)
        // Les octets d'origine, non reencodes : c'est sur eux que porte la mesure.
        assertTrue(bytes.contentEquals(state.image?.bytes))
    }

    @Test
    fun `un clic hors de l'image ne releve rien`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        val before = model.await { it.dominant.isNotEmpty() }

        model.onIntent(PickerIntent.PickAt(500, 500))
        // Le releve hors bornes ne touche a aucun repartiteur : laisser tourner la boucle
        // suffit a le mener a son terme, et prouve donc qu'il n'a rien fait.
        advanceUntilIdle()

        // Rien n'a bouge : ni la teinte retenue, ni le repere.
        val after = model.state.value
        assertEquals(before.selectedHex, after.selectedHex)
        assertNull(after.pickedAt)
    }

    @Test
    fun `le point neutre s'arme, se prend au clic, et se desarme aussitot`() = runTest(dispatcher) {
        val model = model()
        // Une carnation sous lampe chaude, et un blanc de socle qui en subit la dominante.
        model.onIntent(PickerIntent.Load(image("#D9A070", "#E8D2A8")))
        model.await { it.dominant.isNotEmpty() }

        model.onIntent(PickerIntent.PickAt(50, 50))
        model.await { it.selectedHex == "#D9A070" }

        model.onIntent(PickerIntent.PickGrey(true))
        assertTrue(model.state.value.pickingGrey)

        // Le clic suivant designe le gris, et ne change pas la teinte visee.
        model.onIntent(PickerIntent.PickAt(150, 50))
        val state = model.await { it.greyPointHex != null && it.correctedHex != null }

        assertEquals("#E8D2A8", state.greyPointHex)
        assertEquals("#D9A070", state.selectedHex)
        assertTrue(!state.pickingGrey, "un point neutre se designe, il ne s'enchaine pas")
    }

    @Test
    fun `le rayon moyenne autour du clic, et un rayon nul releve le seul pixel`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        model.await { it.dominant.isNotEmpty() }

        // A cheval sur la frontiere des deux aplats, avec un large rayon.
        model.onIntent(PickerIntent.SetRadius(10))
        model.onIntent(PickerIntent.PickAt(100, 50))
        val mixed = model.await { it.pickedAt != null }.selectedHex
        assertNotNull(mixed)
        assertTrue(mixed != "#C98F72" && mixed != "#4A3427", "a cheval, le releve est un melange : $mixed")

        // Le meme point a rayon nul rend le pixel, sans melange.
        model.onIntent(PickerIntent.SetRadius(0))
        model.onIntent(PickerIntent.PickAt(100, 50))
        assertEquals("#4A3427", model.await { it.selectedHex == "#4A3427" }.selectedHex)
    }

    @Test
    fun `le rayon choisi survit a l'ouverture d'une autre photo`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.SetRadius(12))
        model.onIntent(PickerIntent.Load(image("#C98F72")))

        assertEquals(12, model.await { it.image != null }.sampleRadius)
    }

    @Test
    fun `la teinte relevee part vers les autres ecrans, corrigee si elle l'a ete`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#D9A070", "#E8D2A8")))
        model.await { it.dominant.isNotEmpty() }

        model.onIntent(PickerIntent.PickAt(50, 50))
        model.await { it.selectedHex == "#D9A070" }
        assertEquals("#D9A070", shared.colour.value?.toHex())

        // Une fois la dominante annulee, c'est la teinte corrigee qui vaut : c'est elle
        // que le catalogue et le melangeur doivent recevoir, pas celle du capteur.
        model.onIntent(PickerIntent.UseAsGrey("#E8D2A8"))
        val corrected = model.await { it.correctedHex != null }.correctedHex
        assertEquals(corrected, shared.colour.value?.toHex())
    }

    @Test
    fun `la pipette propose un melange, et seulement quand on le demande`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.dominant.isNotEmpty() }

        // Rien n'arrive tout seul : la recherche coute, et on clique souvent avant de
        // retenir un point.
        assertTrue(model.state.value.suggestions.isEmpty())

        model.onIntent(PickerIntent.FindMixes)
        val found = model.await { it.suggestions.isNotEmpty() }

        assertTrue(!found.searching)
        assertTrue(found.suggestions.first().parts.isNotEmpty())
    }

    @Test
    fun `le nombre de tubes borne le melange propose`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#8A6A55")))
        model.await { it.dominant.isNotEmpty() }

        model.onIntent(PickerIntent.SetMaxPaints(1))
        model.onIntent(PickerIntent.FindMixes)
        val single = model.await { it.suggestions.isNotEmpty() }
        assertTrue(
            single.suggestions.all { it.parts.size == 1 },
            "un seul tube demande : ${single.suggestions.map { s -> s.parts.size }}",
        )

        // Changer le nombre repose la question : la reponse se refait sans qu'on la
        // redemande, puisqu'on en avait deja demande une.
        model.onIntent(PickerIntent.SetMaxPaints(3))
        val wider = model.await { it.suggestions.isNotEmpty() && it.suggestions.any { s -> s.parts.size > 1 } }
        assertTrue(wider.suggestions.any { it.parts.size > 1 })
    }

    @Test
    fun `un nouveau releve oublie le melange, qui repondait a une autre question`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        model.await { it.dominant.isNotEmpty() }
        model.onIntent(PickerIntent.FindMixes)
        model.await { it.suggestions.isNotEmpty() }

        model.onIntent(PickerIntent.PickAt(150, 50))
        val after = model.await { it.selectedHex == "#4A3427" }

        assertTrue(after.suggestions.isEmpty(), "une recette pour la teinte precedente serait fausse")
    }

    @Test
    fun `des releves groupes deviennent un projet, ses groupes en zones`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427", "#E8D2BE")))
        model.await { it.dominant.isNotEmpty() }
        model.await { it.palettes.isNotEmpty() }

        model.onIntent(PickerIntent.CreateGroup("Peau"))
        // Trois points de la meme carnation : une ombre, une base, une lumiere.
        listOf(30, 100, 180).forEach { x ->
            model.onIntent(PickerIntent.PickAt(x, 50))
            model.await { it.pickedAt?.x == x }
            model.onIntent(PickerIntent.AddSampleToGroup)
        }
        val ready = model.await { it.group?.samples?.size == 3 }
        assertTrue(ready.hasSamples)

        model.onIntent(PickerIntent.CreateProject("Grognard"))
        val done = model.await { it.groupMessage?.contains("cree") == true }

        assertTrue(!done.savingProject)
        val project = projects.saved.value.single()
        assertEquals("Grognard", project.name)
        assertEquals(listOf("Peau"), project.zones.map { it.name })
        // Les roles viennent de la clarte des points, pas de l'ordre de prelevement.
        assertEquals(
            listOf("Ombre 1", "Base", "Lumiere 1"),
            project.zones.first().layers.map { it.role },
        )
    }

    @Test
    fun `sans le moindre point, le projet est refuse et rien ne s'enregistre`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.dominant.isNotEmpty() }

        model.onIntent(PickerIntent.CreateGroup("Peau"))
        model.onIntent(PickerIntent.CreateProject("Grognard"))
        val state = model.await { it.groupMessage?.contains("Aucun point") == true }

        assertTrue(projects.saved.value.isEmpty())
        assertTrue(!state.savingProject)
    }

    @Test
    fun `un nom de projet deja pris est numerote plutot que refuse`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.dominant.isNotEmpty() }
        model.await { it.palettes.isNotEmpty() }

        repeat(2) {
            model.onIntent(PickerIntent.CreateGroup("Peau $it"))
            model.onIntent(PickerIntent.PickAt(100, 50))
            model.await { it.pickedAt != null }
            model.onIntent(PickerIntent.AddSampleToGroup)
            model.await { it.group?.samples?.isNotEmpty() == true }
            model.onIntent(PickerIntent.CreateProject("Grognard"))
            model.await { s -> s.groupMessage?.contains("cree") == true }
        }

        assertEquals(listOf("Grognard", "Grognard 2"), projects.saved.value.map { it.name })
    }

    @Test
    fun `un groupe sans nom est refuse, et un homonyme aussi`() = runTest(dispatcher) {
        val model = model()

        model.onIntent(PickerIntent.CreateGroup("   "))
        assertEquals("Donnez un nom au groupe.", model.state.value.groupMessage)
        assertTrue(model.state.value.groups.isEmpty())

        model.onIntent(PickerIntent.CreateGroup("Peau"))
        model.onIntent(PickerIntent.CreateGroup("peau"))
        assertEquals("Un groupe porte deja ce nom.", model.state.value.groupMessage)
        assertEquals(1, model.state.value.groups.size)
    }

    @Test
    fun `retirer un releve n'emporte pas son jumeau de meme teinte`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.dominant.isNotEmpty() }

        model.onIntent(PickerIntent.CreateGroup("Peau"))
        // Deux points pris au meme endroit : la meme teinte, deux fois.
        repeat(2) {
            model.onIntent(PickerIntent.PickAt(100, 50))
            model.onIntent(PickerIntent.AddSampleToGroup)
        }
        val two = model.await { it.group?.samples?.size == 2 }
        val hex = two.group!!.samples.first().toHex()

        model.onIntent(PickerIntent.RemoveSample("Peau", hex))
        val one = model.await { it.group?.samples?.size == 1 }
        assertEquals(hex, one.group!!.samples.single().toHex())
    }

    @Test
    fun `un fichier illisible le dit, sans faire tomber l'ecran`() = runTest(dispatcher) {
        val model = model()

        model.onIntent(PickerIntent.Load("pas une image".encodeToByteArray()))
        val state = model.await { it.error != null }

        assertNotNull(state.error)
        assertTrue(state.dominant.isEmpty())
    }
}
