package be.asmolabs.palettier.domain.catalog

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

/**
 * Le catalogue livre avec l'application, relu depuis les constantes generees.
 *
 * <p>Les nombres attendus sont ceux que l'application Java sortait des memes fichiers :
 * c'est ce qui prouve que le portage du chargeur n'a rien perdu en route. Ils bougeront
 * le jour ou une gamme sera completee, et ce test le signalera -- c'est voulu.</p>
 */
class CatalogLoaderTest {

    private val loaded = CatalogLoader().loadDetailed()

    @Test
    fun `les huit gammes donnent les memes 680 tubes que du temps de Java`() {
        assertEquals(680, loaded.paints.size)
        assertEquals(
            0,
            loaded.paints.groupBy { it.brand to it.name }.count { it.value.size > 1 },
            "aucun doublon marque+nom : c'est la cle naturelle, et l'index SQLite est unique dessus",
        )
    }

    @Test
    fun `aucun pigment cite n'est absent de la table`() {
        assertEquals(emptyList(), loaded.unknownPigments)
    }

    @Test
    fun `une fiche declaree est reprise telle quelle`() {
        val umber = loaded.paints.single { it.brand == "Winsor & Newton" && it.name == "Burnt Umber" }
        assertEquals(setOf("PBr7"), umber.pigments)
        assertEquals(DryingClass.FAST, umber.dryingClass, "la terre d'ombre seche vite, par son pigment")
        assertTrue(umber.pigmentsVerified)
        assertTrue(!umber.colorDerived, "sa teinte est declaree, pas deduite")
    }

    @Test
    fun `une fiche sans teinte declaree la deduit de ses pigments`() {
        val derived = loaded.paints.filter { it.colorDerived }
        assertTrue(derived.isNotEmpty(), "des gammes laissent la colonne hex vide")
        derived.forEach { paint ->
            assertTrue(paint.hexColor.startsWith("#"), "${paint.displayName} sans teinte")
        }
    }

    @Test
    fun `la teinte diluee est renseignee la ou les pigments la connaissent`() {
        val withTint = loaded.paints.count { it.tintHex != null }
        assertTrue(withTint > 400, "seulement $withTint tubes a deux constantes")
    }

    @Test
    fun `le pigment le plus lent commande le sechage du tube`() {
        val index = PigmentIndex()
        assertEquals(DryingClass.FAST, index.dryingClassFor(setOf("PBr7")))
        assertEquals(
            DryingClass.VERY_SLOW,
            index.dryingClassFor(setOf("PBr7", "PY35")),
            "une trace de cadmium suffit a tout ralentir",
        )
    }

    @Test
    fun `un code de pigment inconnu ne fait pas tomber le chargement`() {
        val index = PigmentIndex()
        assertNull(index.find("PX999"))
        assertEquals(DryingClass.MEDIUM, index.dryingClassFor(setOf("PX999")), "le repli est moyen")
    }

    @Test
    fun `le pigment le plus couvrant donne l'opacite a defaut d'indication`() {
        val index = PigmentIndex()
        val white = assertNotNull(index.find("PW6"))
        assertEquals(Opacity.OPAQUE, white.opacity)
        assertEquals(Opacity.OPAQUE, index.opacityFor(setOf("PW6", "PB27")))
    }
}
