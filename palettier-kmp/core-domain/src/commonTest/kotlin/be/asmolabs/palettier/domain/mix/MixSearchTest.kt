package be.asmolabs.palettier.domain.mix

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.paint.Paint
import kotlin.math.abs
import kotlin.math.round
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.TimeSource

/**
 * Traduction fidele de MixSearchTest cote Java, jugee sur le meme catalogue de 680
 * huiles et les memes dix teintes reelles.
 */
class MixSearchTest {

    private val service = ColorMixService()
    private val catalog: List<Paint> = CATALOGUE

    /** Un echantillon de teintes reelles : carnations, terrains, blindages, tissus. */
    private val targets = listOf(
        "#C98F72", "#8A4A3C", "#E0B49A", "#6B5A42", "#4A5560",
        "#9A8F80", "#3E4A3A", "#B29B62", "#7A3A22", "#2C3E50",
    )

    private fun find(name: String): Paint =
        catalog.first { it.brand == "Winsor & Newton" && it.name == name }

    @Test
    fun `une couleur du catalogue est retrouvee a l'identique par un seul tube`() {
        val reference = catalog.first { it.name == "Burnt Umber" }

        val best = service.suggestMixes(reference.color, catalog, 1).first()

        assertTrue(best.deltaE < 0.5, "ecart ${best.deltaE}")
        assertEquals(1, best.parts.size)
    }

    @Test
    fun `chaque proposition annonce l'ecart du dosage propose`() {
        for (hex in targets) {
            val target = Rgb.ofHex(hex)
            for (suggestion in service.suggestMixes(target, catalog, 5)) {
                val actual = service.mix(suggestion.parts).color
                assertTrue(
                    abs(deltaE2000(target, actual) - suggestion.deltaE) < 0.01,
                    "$hex : ecart annonce pour ${suggestion.describe()}",
                )
            }
        }
    }

    @Test
    fun `les dosages proposes sont des rapports d'entiers simples`() {
        for (hex in targets) {
            for (suggestion in service.suggestMixes(Rgb.ofHex(hex), catalog, 5)) {
                for (part in suggestion.parts) {
                    assertEquals(round(part.parts), part.parts)
                    assertTrue(part.parts in 1.0..30.0)
                }
            }
        }
    }

    @Test
    fun `le nombre de tubes demande est respecte`() {
        for (hex in targets) {
            for (max in 1..5) {
                service.suggestMixes(Rgb.ofHex(hex), catalog, 8, max).forEach {
                    assertTrue(it.parts.size <= max, "$hex avec $max tubes")
                }
            }
        }
    }

    @Test
    fun `une palette courte tire profit des tubes supplementaires`() {
        // Trois primaires, un blanc et une terre : le cas ou trois tubes ne suffisent pas.
        val shortPalette = listOf(
            find("Cadmium Yellow Pale"), find("Permanent Rose"),
            find("Winsor Blue (Green Shade)"), find("Titanium White"), find("Burnt Umber"),
        )

        var withThree = 0.0
        var withFour = 0.0
        for (hex in targets) {
            withThree += service.suggestMixes(Rgb.ofHex(hex), shortPalette, 1, 3).first().deltaE
            withFour += service.suggestMixes(Rgb.ofHex(hex), shortPalette, 1, 4).first().deltaE
        }

        assertTrue(withFour < withThree, "trois tubes : $withThree, quatre tubes : $withFour")
    }

    @Test
    fun `la liste propose des combinaisons differentes, pas des variantes de dosage`() {
        val suggestions = service.suggestMixes(Rgb.ofHex("#C98F72"), catalog, 8)

        val combinations = suggestions.map { s -> s.parts.map { it.paint.displayName }.sorted() }
        assertEquals(combinations.size, combinations.toSet().size, combinations.toString())
    }

    @Test
    fun `autoriser un tube de plus ne degrade jamais le resultat`() {
        for (hex in targets) {
            val target = Rgb.ofHex(hex)
            var previous = Double.MAX_VALUE
            for (max in 1..5) {
                val current = service.suggestMixes(target, catalog, 1, max).first().deltaE
                assertTrue(current <= previous + 1e-9, "cible $hex avec $max tubes")
                previous = current
            }
        }
    }

    @Test
    fun `les propositions sont classees par palier perceptuel`() {
        val suggestions = service.suggestMixes(Rgb.ofHex("#6B5A42"), catalog, 8)

        // A l'interieur d'un palier de 0,5 l'oeil ne fait pas la difference : l'ordre y
        // est decide par la simplicite du melange, pas par la troisieme decimale.
        val tiers = suggestions.map { (it.deltaE / 0.5).toInt() }
        assertEquals(tiers.sorted(), tiers, tiers.toString())
    }

    @Test
    fun `a ecart imperceptible, le melange le plus simple passe devant`() {
        val reference = catalog.first { it.name == "Yellow Ochre" }

        val best = service.suggestMixes(reference.color, catalog, 5).first()

        assertEquals(1, best.parts.size, best.describe())
        assertTrue(best.deltaE < 0.5)
    }

    @Test
    fun `le nombre de tubes s'adapte a ce qui est disponible`() {
        assertEquals(5, ColorMixService.recommendedMaxPaints(4))
        assertEquals(5, ColorMixService.recommendedMaxPaints(8))
        assertEquals(4, ColorMixService.recommendedMaxPaints(9))
        assertEquals(4, ColorMixService.recommendedMaxPaints(24))
        assertEquals(3, ColorMixService.recommendedMaxPaints(25))
        assertEquals(3, ColorMixService.recommendedMaxPaints(439))
    }

    @Test
    fun `sans consigne, une palette courte recoit des melanges plus riches que le catalogue`() {
        val shortPalette = listOf(
            find("Cadmium Yellow Pale"), find("Permanent Rose"),
            find("Winsor Blue (Green Shade)"), find("Titanium White"), find("Burnt Umber"),
        )

        val onPalette = service.suggestMixes(Rgb.ofHex("#6B5A42"), shortPalette, 1).first().parts.size
        val onCatalogue = service.suggestMixes(Rgb.ofHex("#6B5A42"), catalog, 1).first().parts.size

        assertTrue(onPalette > onCatalogue, "palette : $onPalette tubes, catalogue : $onCatalogue")
        assertTrue(onCatalogue <= 3)
    }

    @Test
    fun `cout de la recherche sur le catalogue complet`() {
        val start = TimeSource.Monotonic.markNow()
        var worst = 0.0
        for (hex in targets) {
            worst = maxOf(worst, service.suggestMixes(Rgb.ofHex(hex), catalog, 8).first().deltaE)
        }
        val millis = start.elapsedNow().inWholeMilliseconds / targets.size

        println("recherche : ${catalog.size} tubes, $millis ms par cible, pire ecart $worst")
        assertTrue(millis < 4_000, "$millis ms par cible")
    }
}
