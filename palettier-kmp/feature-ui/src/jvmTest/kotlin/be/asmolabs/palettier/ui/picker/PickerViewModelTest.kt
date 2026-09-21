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

    private fun model() = PickerViewModel(imageDecoder(), FakeCatalog(catalogue), palettes(), shared, ColorMixService())

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
    fun `une photo chargee rend ses teintes et propose les tubes les plus proches`() = runTest(dispatcher) {
        val model = model()

        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        val state = model.await { it.closest.isNotEmpty() }

        assertTrue(!state.loading)
        assertEquals(2, state.dominant.size)
        assertNotNull(state.selectedHex)
        // Le tube le plus proche de la teinte dominante est celui de meme couleur.
        assertEquals("Flesh Tint", state.closest.first().paint.name)
        assertTrue(state.closest.first().deltaE < 1.0)
    }

    @Test
    fun `choisir une autre teinte change les tubes proposes`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72", "#4A3427")))
        model.await { it.closest.isNotEmpty() }

        model.onIntent(PickerIntent.Select("#4A3427"))
        // Les propositions ont ete videes le temps du calcul : quand elles reviennent,
        // elles portent bien sur la nouvelle teinte.
        val state = model.await { it.selectedHex == "#4A3427" && it.closest.isNotEmpty() }

        assertEquals("Burnt Umber", state.closest.first().paint.name)
    }

    @Test
    fun `l'etagere se filtre, et le tube absent disparait des propositions`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#B22222")))
        val owned = model.await { it.closest.isNotEmpty() }

        // Par defaut on ne propose que ce qu'on possede.
        assertTrue(owned.closest.none { it.paint.name == "Cadmium Red" })

        model.onIntent(PickerIntent.ChooseScope(SearchScope.Catalogue))
        val all = model.await {
            it.scope == SearchScope.Catalogue && it.closest.any { match -> match.paint.name == "Cadmium Red" }
        }
        assertEquals("Cadmium Red", all.closest.first().paint.name)
    }

    @Test
    fun `chercher dans une palette ne propose plus que ses tubes`() = runTest(dispatcher) {
        val model = model()
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        val shelf = model.await { it.closest.isNotEmpty() }

        // Sur l'etagere, c'est la carnation qui gagne : elle est de la meme couleur.
        assertEquals("Flesh Tint", shelf.closest.first().paint.name)
        val zorn = model.await { it.scopes.any { scope -> scope.label == "Zorn" } }
            .scopes.first { it.label == "Zorn" }

        model.onIntent(PickerIntent.ChooseScope(zorn))
        val inPalette = model.await { it.scope == zorn && it.closest.isNotEmpty() }

        // Dans la palette, la seule reponse possible est celle qu'on a sortie sur la table,
        // meme si elle est loin : une recette faisable plutot qu'une recette juste.
        assertEquals(listOf("Burnt Umber"), inPalette.closest.map { it.paint.name })
    }

    @Test
    fun `une palette vide le dit, au lieu de ne rien proposer sans raison`() = runTest(dispatcher) {
        val palettes = be.asmolabs.palettier.ui.FakePaletteRepository(
            listOf(be.asmolabs.palettier.domain.palette.Palette(id = 9, name = "A composer"))
        )
        val model = PickerViewModel(imageDecoder(), FakeCatalog(catalogue), palettes, shared, ColorMixService())
        model.onIntent(PickerIntent.Load(image("#C98F72")))
        model.await { it.closest.isNotEmpty() }

        val empty = model.await { it.scopes.any { scope -> scope.label == "A composer" } }
            .scopes.first { it.label == "A composer" }
        model.onIntent(PickerIntent.ChooseScope(empty))

        val state = model.await { it.scope == empty && it.emptyScope }
        assertTrue(state.closest.isEmpty())
    }

    @Test
    fun `designer un gris neutre corrige la dominante de la photo`() = runTest(dispatcher) {
        val model = model()
        // Une carnation et un blanc de socle, tous deux photographies sous lampe chaude.
        model.onIntent(PickerIntent.Load(image("#D9A070", "#E8D2A8")))
        model.await { it.closest.isNotEmpty() }
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
        model.await { it.closest.isNotEmpty() }

        // Au milieu de la bande du milieu, loin des frontieres.
        model.onIntent(PickerIntent.PickAt(100, 50))
        val state = model.await { it.selectedHex == "#4A3427" && it.closest.isNotEmpty() }

        assertEquals(PickPoint(100, 50), state.pickedAt)
        assertEquals("Burnt Umber", state.closest.first().paint.name)
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
        val before = model.await { it.closest.isNotEmpty() }

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
        model.await { it.closest.isNotEmpty() }

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
        model.await { it.closest.isNotEmpty() }

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
        model.await { it.closest.isNotEmpty() }

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
        model.await { it.closest.isNotEmpty() }

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
        model.await { it.closest.isNotEmpty() }

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
        model.await { it.closest.isNotEmpty() }
        model.onIntent(PickerIntent.FindMixes)
        model.await { it.suggestions.isNotEmpty() }

        model.onIntent(PickerIntent.PickAt(150, 50))
        val after = model.await { it.selectedHex == "#4A3427" }

        assertTrue(after.suggestions.isEmpty(), "une recette pour la teinte precedente serait fausse")
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
