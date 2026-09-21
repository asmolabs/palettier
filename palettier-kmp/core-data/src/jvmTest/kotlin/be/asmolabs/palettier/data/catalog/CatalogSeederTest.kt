package be.asmolabs.palettier.data.catalog

import be.asmolabs.palettier.data.SqlDelightPaintCatalogRepository
import be.asmolabs.palettier.data.SqlDelightPaletteRepository
import be.asmolabs.palettier.data.SqlDelightRecipeRepository
import be.asmolabs.palettier.data.inMemoryDriver
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * L'amorcage de la base au premier demarrage.
 *
 * <p>C'est ce qui repond au reproche d'origine : une installation neuve ne montrait rien,
 * parce que les 680 huiles n'existaient que dans les ressources du module Java.</p>
 */
class CatalogSeederTest {

    private val database = PalettierDatabase(inMemoryDriver())
    private val paints = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
    private val palettes = SqlDelightPaletteRepository(database, paints, Dispatchers.Default)
    private val recipes = SqlDelightRecipeRepository(database, Dispatchers.Default)
    private val seeder = CatalogSeeder(paints, palettes, recipes)

    @Test
    fun `une base neuve recoit le catalogue, les palettes et les recettes`() = runTest {
        val report = seeder.seed()

        assertEquals(680, report.paintsCreated)
        assertEquals(3, report.palettesAdded)
        assertEquals(2, report.recipesAdded)
        assertEquals(emptyList(), report.unknownPigments)

        assertEquals(680, paints.all().size)
        assertEquals(3, palettes.all().size)
        assertEquals(2, recipes.all().size)
    }

    @Test
    fun `le deuxieme demarrage ne touche a rien`() = runTest {
        seeder.seed()
        val second = seeder.seed()

        assertFalse(second.touchedAnything, "amorcage non idempotent : $second")
        assertEquals(680, paints.all().size)
        assertEquals(3, palettes.all().size)
    }

    @Test
    fun `les palettes de reference arrivent garnies`() = runTest {
        seeder.seed()

        val zorn = palettes.all().single { it.name.startsWith("Palette Zorn") }
        assertEquals(4, zorn.paints.size, "quatre tubes, c'est tout l'interet de cette palette")
        assertEquals(
            DryingClass.VERY_SLOW,
            zorn.slowestDryingClass(),
            "le cadmium ecarlate commande le planning de la seance",
        )
    }

    @Test
    fun `une gamme completee arrive chez un peintre deja installe`() = runTest {
        seeder.seed()
        val removed = paints.all().first { it.brand == "Gamblin" }
        paints.delete(removed)
        assertEquals(679, paints.all().size)

        val report = seeder.seed()

        assertEquals(1, report.paintsCreated, "le tube manquant est repose")
        assertNotNull(paints.findByNaturalKey(removed.brand, removed.name))
    }

    @Test
    fun `une fiche corrigee par le fabricant est rafraichie, le releve du peintre garde`() = runTest {
        seeder.seed()
        val before = assertNotNull(paints.findByNaturalKey("Winsor & Newton", "Burnt Umber"))

        // Ce que le fabricant dit : fausse, a corriger. Et ce que le peintre a releve sur
        // son ecouvillon : a garder, c'est son travail.
        paints.refreshIdentity(
            before.copy(
                code = "0000",
                pigments = setOf("PX999"),
                opacity = Opacity.TRANSPARENT,
                dryingClass = DryingClass.VERY_SLOW,
            )
        )
        paints.setOwned(before, false)

        val report = seeder.seed()
        assertTrue(report.paintsRefreshed >= 1)

        val after = assertNotNull(paints.findByNaturalKey("Winsor & Newton", "Burnt Umber"))
        assertEquals(before.code, after.code, "la reference revient du fichier de gamme")
        assertEquals(setOf("PBr7"), after.pigments)
        assertEquals(DryingClass.FAST, after.dryingClass)
        assertFalse(after.inStock, "la possession appartient au peintre, pas au fichier")
    }

    @Test
    fun `une fiche perimee est retiree, sauf si une palette s'en sert`() = runTest {
        seeder.seed()

        // Deux fiches qui ne figurent dans aucun fichier de gamme, sous une marque livree :
        // ce sont des restes d'une version anterieure du catalogue.
        val orphan = paints.save(
            Paint(brand = "Gamblin", name = "Teinte retiree du catalogue", hexColor = "#804020")
        )
        val cited = paints.save(
            Paint(brand = "Gamblin", name = "Teinte retiree mais citee", hexColor = "#204080")
        )
        palettes.save(assertNotNull(palettes.all().firstOrNull()).withPaint(cited))

        val report = seeder.seed()

        assertEquals(1, report.paintsRemoved)
        assertEquals(1, report.paintsKeptInUse)
        assertNull(paints.findById(orphan.id!!), "l'orpheline est retiree")
        assertNotNull(paints.findById(cited.id!!), "celle que la palette cite est epargnee")
    }

    @Test
    fun `un tube saisi par le peintre n'est jamais retire`() = runTest {
        seeder.seed()
        val mine = paints.save(
            Paint(
                brand = "Gamblin",
                name = "Melange que j'ai fabrique",
                hexColor = "#553311",
                userAdded = true,
            )
        )

        val report = seeder.seed()

        assertEquals(0, report.paintsRemoved)
        assertNotNull(paints.findById(mine.id!!))
    }

    @Test
    fun `une fiche perimee qui porte une mesure du peintre est epargnee`() = runTest {
        seeder.seed()

        // Trois fiches qui ne figurent plus dans aucun fichier de gamme. La premiere est
        // nue, les deux autres portent quelque chose que les fichiers ne rendraient pas.
        val nue = paints.save(
            Paint(brand = "Gamblin", name = "Teinte retiree, jamais mesuree", hexColor = "#804020")
        )
        val mesuree = paints.save(
            Paint(
                brand = "Gamblin",
                name = "Teinte retiree, mais calibree",
                hexColor = "#804020",
                tintHex = "#D9B9A4",
            )
        )
        val annotee = paints.save(
            Paint(
                brand = "Gamblin",
                name = "Teinte retiree, mais annotee",
                hexColor = "#804020",
                notes = "le tube de 2019, plus epais que le neuf",
            )
        )

        val report = seeder.seed()

        assertEquals(1, report.paintsRemoved)
        assertNull(paints.findById(nue.id!!), "sans mesure, la fiche perimee s'en va")

        // Une teinte relevee sur un ecouvillon ne figure dans aucun fichier : la perdre,
        // c'est la perdre pour de bon.
        assertNotNull(paints.findById(mesuree.id!!))
        assertNotNull(paints.findById(annotee.id!!))
        assertEquals(2, report.paintsKeptCalibrated.size)
        assertTrue(report.paintsKeptCalibrated.any { it.contains("calibree") }, report.paintsKeptCalibrated.toString())
    }

    @Test
    fun `la teinte diluee est rattrapee sur une base qui n'en a pas`() = runTest {
        seeder.seed()
        // Une base d'avant le modele a deux constantes : la teinte diluee y est absente.
        val known = paints.all().first { it.tintHex != null }
        val id = assertNotNull(known.id)
        database.paintQueries.setTint(null, id)
        assertNull(assertNotNull(paints.findById(id)).tintHex)

        val report = seeder.seed()

        assertEquals(1, report.tintsBackfilled)
        assertEquals(known.tintHex, assertNotNull(paints.findById(id)).tintHex)
    }
}
