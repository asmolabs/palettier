package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectZone
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ProjectPlannerTest {

    private val planner = ProjectPlanner(ColorMixService())

    private fun paint(id: Long, name: String, hex: String) = Paint(
        id = id, brand = "W&N", name = name, hexColor = hex,
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.MEDIUM, tintingStrength = 0.8,
    )

    private val palette = listOf(
        paint(1, "Titanium White", "#F4F2EC"),
        paint(2, "Burnt Umber", "#4A3427"),
        paint(3, "Cadmium Red", "#B22222"),
        paint(4, "Yellow Ochre", "#C8A24A"),
    )

    private fun layer(role: String, hex: String, kind: ProjectLayer.Kind = ProjectLayer.Kind.LADDER) =
        ProjectLayer(role = role, targetHex = hex, technique = "Glacis", kind = kind)

    private fun piece(paints: List<Paint> = palette) = Project(
        id = 1, name = "Grognard", subject = "Buste", approach = "Par glacis.",
        palette = Palette(id = 1, name = "Zorn", paints = paints),
        paints = paints,
        zones = listOf(
            ProjectZone(
                name = "Visage", material = "Peau",
                layers = listOf(
                    layer("Ombre 2", "#5A3B2E"),
                    layer("Ombre 1", "#8A5F4A"),
                    layer("Base", "#C98F72"),
                    layer("Lumiere 1", "#E0B49A"),
                    layer("Lumiere 2", "#F2D8C4"),
                    layer("Rougeur des pommettes", "#C97A62", ProjectLayer.Kind.ACCENT),
                ),
            )
        ),
    )

    @Test
    fun `les couches reviennent dans l'ordre du degrade, variation locale a part`() = runTest {
        val zone = planner.plan(piece()).zones.single()

        assertEquals(listOf("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2"), zone.layers().map { it.role })
        assertEquals(listOf("Rougeur des pommettes"), zone.accents.map { it.role })
        assertEquals("Base", zone.base?.role)
    }

    @Test
    fun `chaque couche recoit un melange calcule et un ecart mesure`() = runTest {
        val zone = planner.plan(piece()).zones.single()

        (zone.layers() + zone.accents).forEach { layer ->
            assertNotNull(layer.recipe, "melange pour ${layer.role}")
            assertTrue(layer.deltaE >= 0)
            // L'ecart annonce est celui du melange propose, pas d'un optimum theorique.
            assertTrue(layer.deltaE < 30, "${layer.role} : ecart ${layer.deltaE}")
        }
    }

    @Test
    fun `sans tube, le plan reste lisible et le dit`() = runTest {
        val plan = planner.plan(piece(paints = emptyList()).copy(palette = null))

        val layer = plan.zones.single().layers().first()
        assertNull(layer.recipe)
        assertEquals("palette vide", layer.reachability())
        assertEquals("0 tubes conserves avec le projet", plan.paletteName)
    }

    @Test
    fun `une palette plus riche resserre l'ecart, sans toucher aux couleurs visees`() = runTest {
        val poor = planner.plan(piece(paints = listOf(palette[1])))
        val rich = planner.plan(piece())

        val poorLayer = poor.zones.single().base!!
        val richLayer = rich.zones.single().base!!

        // La couleur visee est une decision : elle ne bouge pas. Seul le moyen d'y
        // arriver se recalcule.
        assertEquals(poorLayer.target, richLayer.target)
        assertTrue(richLayer.deltaE < poorLayer.deltaE, "${richLayer.deltaE} vs ${poorLayer.deltaE}")
    }

    @Test
    fun `le nom de la palette accompagne le plan, ou le nombre de tubes a defaut`() = runTest {
        assertEquals("Zorn", planner.plan(piece()).paletteName)
    }
}
