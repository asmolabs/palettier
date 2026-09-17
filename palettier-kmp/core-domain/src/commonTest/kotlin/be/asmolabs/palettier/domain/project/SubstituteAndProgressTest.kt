package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.color.Rgb
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SubstituteServiceTest {

    private val service = SubstituteService()

    private val umber = paint(1, "Burnt Umber", "#4A3427", inStock = false)
    private val rawUmber = paint(2, "Raw Umber", "#5A4A38")
    private val white = paint(3, "Titanium White", "#F4F2EC")

    @Test
    fun `un tube possede ne figure pas dans les manquants`() {
        val owned = umber.copy(inStock = true)
        assertTrue(service.missingAmong(listOf(owned), listOf(owned)).isEmpty())
    }

    @Test
    fun `un tube manquant se voit proposer le plus proche de l'etagere`() {
        val missing = service.missingAmong(listOf(umber), listOf(rawUmber, white))

        assertEquals(1, missing.size)
        val gap = missing.first()
        assertEquals("Burnt Umber", gap.paint.name)
        assertEquals("Raw Umber", gap.nearest?.name, "le blanc est bien plus loin qu'une terre")
        assertTrue("Raw Umber" in gap.verdict())
    }

    @Test
    fun `l'etagere vide se dit, elle ne provoque pas d'erreur`() {
        val missing = service.missingAmong(listOf(umber), emptyList())

        assertEquals(1, missing.size)
        assertNull(missing.first().nearest)
        assertTrue(!missing.first().isComfortable)
        assertTrue("Rien sur l'etagere" in missing.first().verdict())
    }

    @Test
    fun `un tube ne se propose jamais lui-meme`() {
        // Il est absent de l'etagere mais present dans la liste possedee : un cas qui
        // arrive des qu'on passe la meme liste des deux cotes.
        val missing = service.missingAmong(listOf(umber), listOf(umber, rawUmber))
        assertEquals("Raw Umber", missing.first().nearest?.name)
    }
}

class ProgressCheckServiceTest {

    private val service = ProgressCheckService()

    private val piece = project(
        "Grognard",
        zone("Visage", layer("Base", "#C98F72"), layer("Ombre 1", "#8A5F4A")),
    )

    @Test
    fun `une teinte posee comme prevu est reconnue, et l'ecart est nul`() {
        val observed = service.compare(
            piece,
            listOf(DominantColour(Rgb.ofHex("#C98F72"), 0.6), DominantColour(Rgb.ofHex("#8A5F4A"), 0.4)),
        )

        assertEquals("Base", observed[0].role)
        assertTrue(observed[0].deltaE < 0.001)
        assertTrue("vous y etes" in observed[0].verdict())
        assertEquals("Ombre 1", observed[1].role)
    }

    @Test
    fun `une teinte qui n'etait prevue nulle part est signalee comme telle`() {
        // Un vert franc : rien dans ce plan de carnation ne s'en approche.
        val observed = service.compare(piece, listOf(DominantColour(Rgb.ofHex("#1FA337"), 1.0)))

        assertTrue(!observed.first().matchesPlan)
        assertTrue("Rien de prevu" in observed.first().verdict())
    }

    @Test
    fun `un projet sans zone ne fait pas echouer la comparaison`() {
        val empty = Project(id = 9, name = "Vide", subject = "Rien")

        val observed = service.compare(empty, listOf(DominantColour(Rgb.ofHex("#C98F72"), 1.0)))

        assertEquals(1, observed.size)
        assertTrue(!observed.first().matchesPlan)
    }
}
