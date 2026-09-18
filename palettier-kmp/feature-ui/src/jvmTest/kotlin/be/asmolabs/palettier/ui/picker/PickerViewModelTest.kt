package be.asmolabs.palettier.ui.picker

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.image.imageDecoder
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
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

    private fun model() = PickerViewModel(imageDecoder(), FakeCatalog(catalogue))

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

        model.onIntent(PickerIntent.OnlyInStock(false))
        val all = model.await { !it.onlyInStock && it.closest.any { match -> match.paint.name == "Cadmium Red" } }
        assertEquals("Cadmium Red", all.closest.first().paint.name)
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
    fun `un fichier illisible le dit, sans faire tomber l'ecran`() = runTest(dispatcher) {
        val model = model()

        model.onIntent(PickerIntent.Load("pas une image".encodeToByteArray()))
        val state = model.await { it.error != null }

        assertNotNull(state.error)
        assertTrue(state.dominant.isEmpty())
    }
}
