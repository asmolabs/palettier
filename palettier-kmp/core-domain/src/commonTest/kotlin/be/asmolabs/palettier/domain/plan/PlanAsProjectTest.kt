package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.project.ProjectLayer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PlanAsProjectTest {

    private val paints = listOf(
        Paint(
            id = 1, brand = "W&N", name = "Titanium White", hexColor = "#F4F2EC",
            opacity = Opacity.OPAQUE, dryingClass = DryingClass.SLOW, tintingStrength = 0.9,
        ),
    )
    private val palette = Palette(id = 1, name = "Zorn", paints = paints)

    private fun layer(role: String, hex: String) = PaintingPlan.Layer(
        role = role, target = Rgb.ofHex(hex), technique = "Glacis", note = "",
        recipe = null, achieved = Rgb.ofHex(hex), deltaE = 0.0,
    )

    private val plan = PaintingPlan(
        subject = "Buste de grognard",
        paletteName = "Zorn",
        approach = "Par glacis.",
        zones = listOf(
            PaintingPlan.Zone(
                name = "Peau", material = "Visage", note = "trois points",
                base = layer("Base", "#C98F72"),
                shadows = listOf(layer("Ombre 1", "#8A5F4A")),
                highlights = listOf(layer("Lumiere 1", "#E8D2BE")),
                accents = listOf(layer("Rougeur des pommettes", "#C96A5A")),
            ),
        ),
    )

    @Test
    fun `le plan devient un projet, ses zones des zones et ses couches des couches`() {
        val project = plan.asProject("Grognard", palette)

        assertEquals("Grognard", project.name)
        assertEquals("Buste de grognard", project.subject)
        assertEquals("Par glacis.", project.approach)
        assertEquals(listOf("Peau"), project.zones.map { it.name })

        val roles = project.zones.first().layers.map { it.role }
        assertEquals(listOf("Ombre 1", "Base", "Lumiere 1", "Rougeur des pommettes"), roles)
    }

    @Test
    fun `une variation locale ne se range pas dans l'echelle des valeurs`() {
        val layers = plan.asProject("Grognard", palette).zones.first().layers

        val accent = layers.first { it.role == "Rougeur des pommettes" }
        assertEquals(ProjectLayer.Kind.ACCENT, accent.kind)
        assertTrue(layers.filter { it.role != accent.role }.all { it.kind == ProjectLayer.Kind.LADDER })
    }

    @Test
    fun `les tubes sont recopies, pour que le plan reste reproductible`() {
        val project = plan.asProject("Grognard", palette)

        assertEquals(paints, project.paints)
        assertEquals(palette, project.palette)
        // Retirer un tube de la palette apres coup ne touche pas au projet deja etabli.
        assertEquals(1, project.effectivePaints().size)
    }

    @Test
    fun `sans palette, le projet garde ses zones et n'a simplement pas de tubes`() {
        val project = plan.asProject("Grognard", null)

        assertTrue(project.paints.isEmpty())
        assertEquals(1, project.zones.size)
    }

    @Test
    fun `un nom deja pris est numerote plutot que refuse`() {
        assertEquals("Grognard", uniqueName("Grognard", listOf("Cuirassier")))
        assertEquals("Grognard 2", uniqueName("Grognard", listOf("Grognard")))
        assertEquals("Grognard 3", uniqueName("Grognard", listOf("Grognard", "Grognard 2")))
        // La casse ne fait pas un nom different : deux "grognard" se ressemblent trop.
        assertEquals("Grognard 2", uniqueName("Grognard", listOf("grognard")))
        // Et un nom vide vaut mieux qu'un enregistrement refuse.
        assertEquals("Projet", uniqueName("   ", emptyList()))
    }
}
