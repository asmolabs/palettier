package be.asmolabs.palettier.ui.catalog

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class CatalogViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private fun paint(
        id: Long, brand: String, name: String, hex: String,
        code: String = "", legacy: String = "", pigments: Set<String> = emptySet(),
        inStock: Boolean = true,
    ) = Paint(
        id = id, brand = brand, name = name, hexColor = hex, code = code, legacyCode = legacy,
        pigments = pigments, inStock = inStock,
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.MEDIUM, tintingStrength = 0.8,
    )

    private val catalogue = listOf(
        paint(1, "Winsor & Newton", "Burnt Umber", "#4A3427", code = "076", pigments = setOf("PBr7")),
        paint(2, "Winsor & Newton", "Titanium White", "#F4F2EC", code = "644", pigments = setOf("PW6")),
        paint(3, "Abteilung 502", "Terre brulee", "#4C3628", code = "AKABT004", legacy = "ABT004"),
        paint(4, "Gamblin", "Cadmium Red", "#B22222", pigments = setOf("PR108"), inStock = false),
    )

    private class FakeCatalog(paints: List<Paint>) : PaintCatalogRepository {
        val paints = MutableStateFlow(paints)
        override fun observeAll(): Flow<List<Paint>> = this.paints
        override suspend fun all() = paints.value
        override suspend fun inStock() = paints.value.filter { it.inStock }
        override suspend fun findById(id: Long) = paints.value.firstOrNull { it.id == id }
        override suspend fun findByNaturalKey(brand: String, name: String) =
            paints.value.firstOrNull { it.brand == brand && it.name == name }
        override suspend fun save(paint: Paint) = paint
        override suspend fun setOwned(paint: Paint, owned: Boolean) {
            paints.value = paints.value.map { if (it.id == paint.id) it.copy(inStock = owned) else it }
        }
        override suspend fun recordTint(paint: Paint, tint: Rgb) {}
        override suspend fun countOwned() = paints.value.count { it.inStock }.toLong()
    }

    private suspend fun kotlinx.coroutines.CoroutineScope.model(catalog: FakeCatalog): CatalogViewModel {
        val model = CatalogViewModel(catalog)
        launch { model.state.collect { } }
        model.state.first { !it.loading }
        return model
    }

    @Test
    fun `le catalogue arrive classe par marque et nom`() = runTest(dispatcher) {
        val model = backgroundScope.model(FakeCatalog(catalogue))

        assertEquals(4, model.state.value.total)
        assertEquals(
            listOf("Terre brulee", "Cadmium Red", "Burnt Umber", "Titanium White"),
            model.state.value.rows.map { it.paint.name },
        )
    }

    @Test
    fun `la recherche porte aussi sur les pigments et l'ancienne reference`() = runTest(dispatcher) {
        val model = backgroundScope.model(FakeCatalog(catalogue))

        model.onIntent(CatalogIntent.Search("PBr7"))
        advanceUntilIdle()
        assertEquals(listOf("Burnt Umber"), model.state.value.rows.map { it.paint.name })

        // Un tube achete avant la renumerotation porte encore son etiquette d'origine :
        // c'est sous celle-la qu'on le cherche.
        model.onIntent(CatalogIntent.Search("ABT004"))
        advanceUntilIdle()
        assertEquals(listOf("Terre brulee"), model.state.value.rows.map { it.paint.name })
    }

    @Test
    fun `l'etagere se filtre`() = runTest(dispatcher) {
        val model = backgroundScope.model(FakeCatalog(catalogue))

        model.onIntent(CatalogIntent.OnlyInStock(true))
        advanceUntilIdle()

        assertEquals(3, model.state.value.rows.size)
        assertTrue(model.state.value.rows.none { it.paint.name == "Cadmium Red" })
    }

    @Test
    fun `viser une teinte reclasse par ecart`() = runTest(dispatcher) {
        val model = backgroundScope.model(FakeCatalog(catalogue))

        model.onIntent(CatalogIntent.AimAt("#4A3427"))
        advanceUntilIdle()

        val rows = model.state.value.rows
        assertTrue(model.state.value.sortedByDistance)
        assertEquals("Burnt Umber", rows.first().paint.name)
        // Les deux terres se suivent, le blanc ferme la marche.
        assertEquals("Titanium White", rows.last().paint.name)
        assertTrue(rows.first().deltaE!! < rows.last().deltaE!!)
    }

    @Test
    fun `declarer un tube possede se voit tout de suite dans la liste`() = runTest(dispatcher) {
        val catalog = FakeCatalog(catalogue)
        val model = backgroundScope.model(catalog)

        val absent = catalogue.first { !it.inStock }
        model.onIntent(CatalogIntent.SetOwned(absent, true))
        advanceUntilIdle()

        // C'est le flux de la base qui le rapporte, pas un rafraichissement manuel.
        model.onIntent(CatalogIntent.OnlyInStock(true))
        advanceUntilIdle()
        assertEquals(4, model.state.value.rows.size)
    }
}
