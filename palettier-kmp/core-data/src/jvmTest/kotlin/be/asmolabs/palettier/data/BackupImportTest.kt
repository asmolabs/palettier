package be.asmolabs.palettier.data

import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.workbench.WorkbenchService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import java.nio.file.Path
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.days
import kotlin.time.Instant

/**
 * L'import d'une vraie sauvegarde, produite par l'application Java.
 *
 * <p>C'est le pont entre les deux applications, et la raison pour laquelle aucun
 * convertisseur H2 vers SQLite n'a eu a etre ecrit. L'archive de cet essai n'est pas
 * fabriquee a la main : elle sort du BackupService Java, catalogue complet compris.</p>
 */
class BackupImportTest {

    private val database = PalettierDatabase(inMemoryDriver())
    private val catalog = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
    private val palettes = SqlDelightPaletteRepository(database, catalog, Dispatchers.Default)
    private val projects = SqlDelightProjectRepository(database, catalog, palettes, Dispatchers.Default)
    private val recipes = SqlDelightRecipeRepository(database, Dispatchers.Default)
    private val importer = BackupImporter(catalog, palettes, projects, recipes)

    private fun archive() = ZipBackupArchive(
        Path.of(requireNotNull(javaClass.classLoader.getResource("sauvegarde-reelle.zip")) {
            "archive d'essai absente"
        }.toURI())
    )

    @Test
    fun `le catalogue entier arrive, pigments compris`() = runTest {
        val report = archive().use { importer.import(it) }

        assertEquals(680, report.paintsCreated)
        assertEquals(0, report.paintsUpdated)
        assertEquals(680, catalog.all().size)

        val umber = catalog.findByNaturalKey("Winsor & Newton", "Burnt Umber")
        assertNotNull(umber)
        assertEquals(setOf("PBr7"), umber.pigments)
        assertEquals(DryingClass.FAST, umber.dryingClass)
    }

    @Test
    fun `les corrections du peintre survivent au passage`() = runTest {
        archive().use { importer.import(it) }

        // Cinq gammes vendent un "Burnt Umber" : c'est la marque et le nom ensemble qui
        // designent un tube, jamais le nom seul. La teinte relevee au moment de produire
        // l'archive l'a ete sur celui de Gamblin.
        val gamblin = catalog.findByNaturalKey("Gamblin", "Burnt Umber")!!
        assertEquals("#C9B9AC", gamblin.tintHex, "la teinte diluee relevee")
        assertTrue(gamblin.inStock)

        // Et elle fait bien basculer le melange vers Kubelka-Munk a deux constantes.
        assertTrue(gamblin.colorant().scattering.any { it != 1.0 })

        // Les autres gardent la leur : une correction ne deborde pas sur ses homonymes.
        assertEquals("#A59CA9", catalog.findByNaturalKey("Winsor & Newton", "Burnt Umber")!!.tintHex)
    }

    @Test
    fun `le projet arrive avec ses zones, ses couches et sa palette`() = runTest {
        val report = archive().use { importer.import(it) }

        assertEquals(1, report.projectsAdded)
        assertTrue(report.palettesAdded >= 1)

        val project = projects.all().single { it.name == "Grognard de reference" }
        assertEquals("Buste de grognard", project.subject)
        assertEquals("Palette courte.", project.approach)
        assertEquals(listOf("Visage"), project.zones.map { it.name })

        // Cinq marches de l'echelle plus une variation locale.
        val layers = project.zones.single().layers
        assertEquals(6, layers.size)
        assertEquals(1, layers.count { it.kind == be.asmolabs.palettier.domain.project.ProjectLayer.Kind.ACCENT })
        assertTrue(project.paints.isNotEmpty(), "les tubes figes du projet")
        assertNotNull(project.palette)
    }

    @Test
    fun `l'avancement de la piece n'est pas perdu`() = runTest {
        archive().use { importer.import(it) }

        val applied = projects.all().single { it.name == "Grognard de reference" }
            .zones.single().layers.first { it.isApplied }.applied

        assertNotNull(applied)
        assertEquals(Instant.parse("2026-02-14T18:30:00Z"), applied.at)
        assertEquals(24.0, applied.workshop.temperatureCelsius)
        assertEquals(40.0, applied.workshop.relativeHumidity)
        assertEquals(Ventilation.GOOD, applied.workshop.ventilation)
        assertEquals(DryingClass.VERY_SLOW, applied.dryingClass)
    }

    @Test
    fun `les photos rangees a cote du JSON sont recuperees`() = runTest {
        val report = archive().use { importer.import(it) }

        assertEquals(1, report.photos)
        assertTrue(report.warnings.isEmpty(), report.warnings.toString())

        val project = projects.all().single { it.name == "Grognard de reference" }
        val withPhotos = projects.findWithPhotos(project.id!!)!!
        assertEquals(listOf("piece.png"), withPhotos.photos.map { it.caption })
        assertTrue(withPhotos.photos.single().data.isNotEmpty())
    }

    @Test
    fun `reimporter la meme archive ne duplique rien`() = runTest {
        archive().use { importer.import(it) }
        val second = archive().use { importer.import(it) }

        // Le catalogue se met a jour, le travail en cours est laisse intact.
        assertEquals(680, second.paintsUpdated)
        assertEquals(0, second.paintsCreated)
        assertEquals(0, second.projectsAdded)
        assertTrue(second.skipped > 0)

        assertEquals(680, catalog.all().size)
        assertEquals(1, projects.all().count { it.name == "Grognard de reference" })
    }

    @Test
    fun `l'etabli fonctionne sur les donnees restaurees`() = runTest {
        archive().use { importer.import(it) }

        // La boucle complete : archive Java, base SQLite, domaine Kotlin.
        val posed = Instant.parse("2026-02-14T18:30:00Z")
        val bench = WorkbenchService().bench(projects.all(), posed + 30.days)
        val zone = bench.pieces.single().zones.single()

        assertTrue(zone.readyNow, "la couche de fevrier est seche depuis longtemps")
        assertEquals(1, zone.applied)
        assertEquals(6, zone.total)
    }
}
