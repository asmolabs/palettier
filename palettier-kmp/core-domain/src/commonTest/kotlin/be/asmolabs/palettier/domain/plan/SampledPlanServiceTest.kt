package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Les essais du service Java, traduits avec leurs valeurs attendues inchangees.
 *
 * <p>La palette Zorn est reconstruite ici plutot que lue en base : ce qui est verifie est
 * la deduction des roles, pas le chargement d'un catalogue.</p>
 */
class SampledPlanServiceTest {

    private val sampled = SampledPlanService(ColorMixService())

    private fun paint(id: Long, name: String, hex: String) = Paint(
        id = id, brand = "Winsor & Newton", name = name, hexColor = hex,
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.MEDIUM, tintingStrength = 0.8,
    )

    /** La palette Zorn : blanc, ocre jaune, rouge de cadmium, noir d'ivoire. */
    private val zorn = Palette(
        id = 1,
        name = "Palette Zorn",
        paints = listOf(
            paint(1, "Titanium White", "#F4F2EC"),
            paint(2, "Yellow Ochre", "#C8A24A"),
            paint(3, "Cadmium Red Pale", "#D8412F"),
            paint(4, "Ivory Black", "#2B2B2B"),
        ),
    )

    /** Cinq points d'une carnation, donnes volontairement en desordre. */
    private fun skin() = SampledPlanService.ColourGroup(
        "Peau", "Visage",
        listOf(
            Rgb.ofHex("#E8D2BE"),   // le plus clair
            Rgb.ofHex("#5A3B2E"),   // le plus sombre
            Rgb.ofHex("#C98F72"),   // median
            Rgb.ofHex("#8A5F4A"),
            Rgb.ofHex("#DCB69C"),
        ),
    )

    @Test
    fun `les roles suivent la clarte des points, pas l'ordre de prelevement`() = runTest {
        val plan = sampled.plan("Buste", zorn, listOf(skin()), 3)

        val zone = plan.zones.first()
        assertEquals(
            listOf("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2"),
            zone.layers().map { it.role },
        )
        assertEquals("#C98F72", zone.base?.target?.toHex())
        assertEquals("#5A3B2E", zone.shadows.last().target.toHex())
        assertEquals("#E8D2BE", zone.highlights.last().target.toHex())
    }

    @Test
    fun `les couches vont bien du sombre au clair`() = runTest {
        val layers = sampled.plan("Buste", zorn, listOf(skin()), 3).zones.first().layers()

        val luminances = layers.map { it.target.relativeLuminance() }
        assertEquals(luminances.sorted(), luminances, "l'echelle doit monter, pas serpenter")
    }

    @Test
    fun `chaque couche porte sa recette`() = runTest {
        val plan = sampled.plan("Buste", zorn, listOf(skin()), 3)

        assertTrue(plan.zones.first().layers().all { it.recipe != null })
    }

    @Test
    fun `chaque groupe devient une zone`() = runTest {
        val cape = SampledPlanService.ColourGroup(
            "Cape", "Laine",
            listOf(Rgb.ofHex("#3A4A6B"), Rgb.ofHex("#6A7A9B")),
        )

        val plan = sampled.plan("Buste", zorn, listOf(skin(), cape), 3)

        assertEquals(listOf("Peau", "Cape"), plan.zones.map { it.name })
    }

    @Test
    fun `un seul point donne une base, et le dit`() = runTest {
        val one = SampledPlanService.ColourGroup("Peau", samples = listOf(Rgb.ofHex("#C98F72")))

        val zone = sampled.plan("Buste", zorn, listOf(one), 3).zones.first()

        assertEquals(1, zone.layers().size)
        assertEquals("Base", zone.base?.role)
        assertTrue(zone.note.contains("Relevez une ombre et une lumiere"), zone.note)
    }

    @Test
    fun `un groupe sans releve ne fait pas de zone, et rien du tout est refuse`() = runTest {
        val empty = SampledPlanService.ColourGroup("Cape")

        // Un groupe vide est ignore, les autres passent.
        val plan = sampled.plan("Buste", zorn, listOf(skin(), empty), 3)
        assertEquals(listOf("Peau"), plan.zones.map { it.name })

        // Mais un plan sans le moindre point n'a rien a dire : il est refuse.
        assertFailsWith<IllegalArgumentException> {
            sampled.plan("Buste", zorn, listOf(empty), 3)
        }
    }

    @Test
    fun `sans palette, le plan garde les couleurs visees et n'invente aucune recette`() = runTest {
        val plan = sampled.plan("Buste", null, listOf(skin()), 3)

        val zone = plan.zones.first()
        assertEquals("sans palette", plan.paletteName)
        // La couche vaut encore : elle dit ou aller, faute de dire avec quoi.
        val base = assertNotNull(zone.base)
        assertEquals("#C98F72", base.target.toHex())
        assertEquals(null, base.recipe)
        assertEquals("palette vide", base.reachability())
    }
}
