package be.asmolabs.palettier.ui.export

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.MixSuggestion
import be.asmolabs.palettier.domain.mix.PaintPart
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.plan.PaintingPlan
import kotlinx.coroutines.test.runTest
import org.apache.pdfbox.Loader
import org.apache.pdfbox.text.PDFTextStripper
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Le document est relu plutot que decrit.
 *
 * <p>Compter des octets ne dirait pas si le plan est dessus. On rouvre donc le PDF produit
 * et on y cherche ce que le peintre doit y trouver a l'etabli : les zones, les roles, les
 * dosages en toutes lettres.</p>
 */
class PdfPlanExporterTest {

    private val exporter = PdfPlanExporter()

    private fun paint(name: String, hex: String) = Paint(
        id = 1, brand = "W&N", name = name, hexColor = hex,
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.MEDIUM, tintingStrength = 0.8,
    )

    private fun layer(role: String, hex: String, withRecipe: Boolean = true) = PaintingPlan.Layer(
        role = role,
        target = Rgb.ofHex(hex),
        technique = "Glacis",
        note = "",
        recipe = if (!withRecipe) null else MixSuggestion(
            parts = listOf(
                PaintPart(paint("Titanium White", "#F4F2EC"), 3.0),
                PaintPart(paint("Burnt Umber", "#4A3427"), 1.0),
            ),
            color = Rgb.ofHex(hex),
            deltaE = 1.4,
        ),
        achieved = Rgb.ofHex(hex),
        deltaE = 1.4,
    )

    private val plan = PaintingPlan(
        subject = "Buste de grognard",
        paletteName = "Zorn",
        approach = "Par glacis successifs, du sombre au clair.",
        zones = listOf(
            PaintingPlan.Zone(
                name = "Visage", material = "Peau", note = "trois points releves",
                base = layer("Base", "#C98F72"),
                shadows = listOf(layer("Ombre 1", "#8A5F4A")),
                highlights = listOf(layer("Lumiere 1", "#E8D2BE")),
                accents = listOf(layer("Rougeur des pommettes", "#C96A5A")),
            ),
        ),
    )

    private suspend fun textOf(plan: PaintingPlan): String =
        Loader.loadPDF(exporter.render(plan)).use { PDFTextStripper().getText(it) }

    @Test
    fun `la feuille porte le sujet, la palette et le parti pris`() = runTest {
        val text = textOf(plan)

        assertTrue(text.contains("Plan de peinture"), text)
        assertTrue(text.contains("Buste de grognard"), text)
        assertTrue(text.contains("Palette : Zorn"), text)
        assertTrue(text.contains("Par glacis successifs"), text)
    }

    @Test
    fun `chaque couche porte son dosage en toutes lettres, pas seulement sa pastille`() = runTest {
        val text = textOf(plan)

        // C'est ce qui fait qu'une feuille imprimee se suffit a elle-meme : une pastille
        // ne vaut que ce que vaut l'imprimante, un dosage se lit.
        assertTrue(text.contains("3 parts de W&N - Titanium White"), text)
        assertTrue(text.contains("Base"), text)
        assertTrue(text.contains("Ombre 1"), text)
        assertTrue(text.contains("Lumiere 1"), text)
        assertTrue(text.contains("#C98F72"), text)
        assertTrue(text.contains("ecart 1,4") || text.contains("ecart 1.4"), text)
    }

    @Test
    fun `les variations locales ont leur rubrique, hors de l'echelle des valeurs`() = runTest {
        val text = textOf(plan)

        assertTrue(text.contains("Variations locales"), text)
        assertTrue(text.contains("Rougeur des pommettes"), text)
    }

    @Test
    fun `une couche sans recette le dit, au lieu de laisser un blanc`() = runTest {
        val bare = plan.copy(
            zones = listOf(plan.zones.first().copy(base = layer("Base", "#C98F72", withRecipe = false)))
        )

        assertTrue(textOf(bare).contains("palette vide"))
    }

    @Test
    fun `un sujet vide ne laisse pas la feuille anonyme`() = runTest {
        assertTrue(textOf(plan.copy(subject = "  ")).contains("Sujet decrit par photo"))
    }

    @Test
    fun `un caractere hors du latin occidental ne fait pas perdre le document`() = runTest {
        // Les polices standard du PDF ne le codent pas : sans remplacement, l'ecriture
        // entiere echouerait et le plan serait perdu pour un guillemet.
        val exotic = plan.copy(subject = "Buste ’ grognard 中文")

        val text = textOf(exotic)
        assertTrue(text.contains("Buste ' grognard"), text)
    }

    @Test
    fun `un plan long tient sur plusieurs pages, sans couper une zone en deux`() = runTest {
        val many = plan.copy(zones = (1..12).map { plan.zones.first().copy(name = "Zone $it") })

        val document = Loader.loadPDF(exporter.render(many))
        document.use {
            assertTrue(it.numberOfPages > 1, "douze zones tiennent sur ${it.numberOfPages} page")
            val text = PDFTextStripper().getText(it)
            (1..12).forEach { index -> assertTrue(text.contains("Zone $index"), "Zone $index absente") }
        }
    }

    @Test
    fun `le document rendu est bien un PDF`() = runTest {
        val bytes = exporter.render(plan)

        assertEquals("%PDF", bytes.decodeToString(0, 4))
    }
}
