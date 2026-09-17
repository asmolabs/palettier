package be.asmolabs.palettier.data

import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/** Le catalogue, contre une vraie base SQLite en memoire. */
class PaintCatalogRepositoryTest {

    private val database = PalettierDatabase(inMemoryDriver())
    private val repository = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)

    private fun umber(inStock: Boolean = true) = Paint(
        brand = "Winsor & Newton", name = "Burnt Umber", code = "076",
        hexColor = "#4A3427", tintHex = "#C9B9AC",
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.FAST, tintingStrength = 0.8,
        pigments = setOf("PBr7"), inStock = inStock, pigmentsVerified = true,
    )

    @Test
    fun `un tube enregistre se relit a l'identique, pigments compris`() = runTest {
        val saved = repository.save(umber())
        assertNotNull(saved.id)

        val read = repository.findById(saved.id!!)
        assertNotNull(read)
        assertEquals("Burnt Umber", read.name)
        assertEquals(setOf("PBr7"), read.pigments)
        assertEquals("#C9B9AC", read.tintHex)
        assertEquals(DryingClass.FAST, read.dryingClass)
        assertEquals(Opacity.SEMI_OPAQUE, read.opacity)
        assertTrue(read.pigmentsVerified)
        assertTrue(read.inStock)
    }

    @Test
    fun `la teinte diluee relue fait bien basculer le melange a deux constantes`() = runTest {
        val saved = repository.save(umber())
        val read = repository.findById(saved.id!!)!!

        // Un colorant a deux constantes n'a pas la meme diffusion sur les trois canaux ;
        // celui a constante unique l'a a 1 partout. C'est ce qui distingue les deux.
        val scattering = read.colorant().scattering
        assertTrue(scattering.any { it != 1.0 }, scattering.toList().toString())
    }

    @Test
    fun `l'etagere ne rend que ce qu'on possede`() = runTest {
        repository.save(umber(inStock = true))
        repository.save(umber(inStock = false).copy(name = "Raw Umber"))

        assertEquals(2, repository.all().size)
        assertEquals(listOf("Burnt Umber"), repository.inStock().map { it.name })
        assertEquals(1, repository.countOwned())
    }

    @Test
    fun `declarer un tube possede se voit tout de suite`() = runTest {
        val raw = repository.save(umber(inStock = false).copy(name = "Raw Umber"))

        repository.setOwned(raw, true)

        assertEquals(1, repository.countOwned())
    }

    @Test
    fun `relever une teinte sur un ecouvillon la consigne`() = runTest {
        val saved = repository.save(umber().copy(tintHex = null))
        val id = saved.id!!
        assertEquals(null, repository.findById(id)!!.tintHex)

        repository.recordTint(saved, Rgb.ofHex("#D2C3B4"))

        assertEquals("#D2C3B4", repository.findById(id)!!.tintHex)
    }

    @Test
    fun `le flux rend l'etat courant sans qu'on le redemande`() = runTest {
        repository.save(umber())

        // C'est tout l'interet du Flow : l'ecran s'abonne une fois, la base le previent.
        val seen = repository.observeAll().first()
        assertEquals(listOf("Burnt Umber"), seen.map { it.name })
        assertEquals(setOf("PBr7"), seen.first().pigments)
    }
}
