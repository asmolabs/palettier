package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.plan.PaintingPlan
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/**
 * Ce que le service fait de la reponse du modele, sans moteur.
 *
 * <p>C'est la moitie du travail qui ne depend d'aucun serveur : combler une couche
 * oubliee, ecarter une variation vide, encaisser une couleur mal formee.</p>
 */
class PlanEnricherTest {

    private val enricher = PlanEnricher(ColorMixService())

    private fun paint(id: Long, name: String, hex: String, drying: DryingClass = DryingClass.MEDIUM) =
        Paint(id = id, brand = "W&N", name = name, hexColor = hex,
            opacity = Opacity.SEMI_OPAQUE, dryingClass = drying, tintingStrength = 0.85)

    private val palette = Palette(
        id = 1, name = "Essai",
        paints = listOf(
            paint(1, "Titanium White", "#F4F2EC", DryingClass.SLOW),
            paint(2, "Burnt Umber", "#4A3427", DryingClass.FAST),
            paint(3, "Cadmium Red", "#B22222", DryingClass.VERY_SLOW),
        ),
    )

    private fun hex(value: String) = LayerDraft(value, "Glacis", "")

    private suspend fun only(
        base: LayerDraft? = hex("#C98F72"),
        shadow1: LayerDraft? = hex("#8A5F4A"),
        shadow2: LayerDraft? = hex("#5A3B2E"),
        highlight1: LayerDraft? = hex("#E0B49A"),
        highlight2: LayerDraft? = hex("#F2D8C4"),
        vararg accents: AccentDraft,
    ): PaintingPlan.Zone = enricher.enrich(
        "Buste", palette,
        PlanDraft(
            "Approche",
            listOf(
                ZoneDraft(
                    "Visage", "Peau", "", base, shadow1, shadow2, highlight1, highlight2,
                    accents.getOrNull(0), accents.getOrNull(1), accents.getOrNull(2),
                )
            ),
        ),
        maxPaints = 3,
    ).zones.first()

    @Test
    fun `les cinq marches de l'echelle sont toujours presentes et nommees`() = runTest {
        assertEquals(
            listOf("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2"),
            only().layers().map { it.role },
        )
    }

    @Test
    fun `une couche oubliee par le modele est interpolee, pas inventee`() = runTest {
        // Ombre 1 absente : elle doit tomber entre la base et l'ombre profonde.
        val zone = only(shadow1 = null)
        val interpolated = zone.shadows.first()

        assertEquals("Ombre 1", interpolated.role)
        assertTrue("interpole" in interpolated.note)

        val base = Rgb.ofHex("#C98F72")
        val deep = Rgb.ofHex("#5A3B2E")
        val got = interpolated.target
        assertTrue(got.r in minOf(base.r, deep.r)..maxOf(base.r, deep.r))
        assertTrue(got.g in minOf(base.g, deep.g)..maxOf(base.g, deep.g))
        assertTrue(got.b in minOf(base.b, deep.b)..maxOf(base.b, deep.b))
    }

    @Test
    fun `sans rien pour interpoler, la couche retombe sur la base et le dit`() = runTest {
        val zone = only(shadow1 = null, shadow2 = null)

        zone.shadows.forEach {
            assertEquals(Rgb.ofHex("#C98F72"), it.target)
            assertTrue("absente" in it.note)
        }
    }

    @Test
    fun `une variation locale vide est ecartee, une variation sans nom en recoit un`() = runTest {
        val zone = only(
            accents = arrayOf(
                AccentDraft("Rougeur des pommettes", "#C97A62", "Glacis", ""),
                AccentDraft("Sans couleur", "", "Glacis", ""),
                AccentDraft("", "#8FA3B0", "Filtre", ""),
            )
        )

        assertEquals(listOf("Rougeur des pommettes", "Variation 2"), zone.accents.map { it.role })
    }

    @Test
    fun `une couleur mal formee ne fait pas echouer le plan`() = runTest {
        val zone = only(base = hex("brun chaud"))

        assertEquals(Rgb(0.5, 0.5, 0.5), zone.base!!.target)
        assertEquals(5, zone.layers().size)
    }

    @Test
    fun `chaque couche porte un melange calcule et un ecart mesure`() = runTest {
        val zone = only()

        (zone.layers() + zone.accents).forEach {
            assertNotNull(it.recipe, "melange pour ${it.role}")
            assertTrue(it.deltaE >= 0)
        }
    }
}
