package be.asmolabs.palettier.data

import be.asmolabs.palettier.data.backup.BackupExporter
import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Ventilation
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.deleteRecursively
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.time.Instant

/**
 * Aller-retour complet : une archive Java entre, une archive Palettier sort, et la
 * seconde se relit comme la premiere.
 *
 * <p>Sans l'export, la migration serait un aller simple. Cet essai est ce qui garantit
 * qu'on peut revenir en arriere.</p>
 */
@OptIn(kotlin.io.path.ExperimentalPathApi::class)
class BackupRoundTripTest {

    private val directory: Path = Files.createTempDirectory("palettier-export")

    @AfterTest
    fun tearDown() = directory.deleteRecursively()

    private class Installation {
        val database = PalettierDatabase(inMemoryDriver())
        val catalog = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
        val palettes = SqlDelightPaletteRepository(database, catalog, Dispatchers.Default)
        val projects = SqlDelightProjectRepository(database, catalog, palettes, Dispatchers.Default)
        val recipes = SqlDelightRecipeRepository(database, Dispatchers.Default)
        val importer = BackupImporter(catalog, palettes, projects, recipes)
        val exporter = BackupExporter(catalog, palettes, projects, recipes)
    }

    private fun javaArchive() = ZipBackupArchive(
        Path.of(requireNotNull(javaClass.classLoader.getResource("sauvegarde-reelle.zip")).toURI())
    )

    @Test
    fun `ce qui sort se relit, et rend exactement ce qui etait entre`() = runTest {
        // Premiere installation : on restaure l'archive produite par l'application Java.
        val first = Installation()
        javaArchive().use { first.importer.import(it) }

        val archive = directory.resolve("sortie.zip")
        val report = ZipBackupWriter(archive).use { first.exporter.export(it) }

        assertEquals(680, report.paints)
        assertEquals(1, report.projects)
        assertEquals(1, report.photos)
        assertTrue(Files.size(archive) > 0)

        // Seconde installation, vierge : elle relit ce que la premiere a ecrit.
        val second = Installation()
        ZipBackupArchive(archive).use { second.importer.import(it) }

        assertEquals(680, second.catalog.all().size)
        assertEquals(first.palettes.all().map { it.name }.sorted(), second.palettes.all().map { it.name }.sorted())
        assertEquals(first.recipes.all().map { it.name }.sorted(), second.recipes.all().map { it.name }.sorted())

        val project = second.projects.all().single()
        assertEquals("Grognard de reference", project.name)
        assertEquals(6, project.zones.single().layers.size)
    }

    @Test
    fun `la pose d'une couche traverse l'aller-retour intacte`() = runTest {
        val first = Installation()
        javaArchive().use { first.importer.import(it) }

        val archive = directory.resolve("pose.zip")
        ZipBackupWriter(archive).use { first.exporter.export(it) }

        val second = Installation()
        ZipBackupArchive(archive).use { second.importer.import(it) }

        val applied = second.projects.all().single().zones.single().layers.first { it.isApplied }.applied
        assertNotNull(applied)
        assertEquals(Instant.parse("2026-02-14T18:30:00Z"), applied.at)
        assertEquals(24.0, applied.workshop.temperatureCelsius)
        assertEquals(Ventilation.GOOD, applied.workshop.ventilation)
        assertEquals(DryingClass.VERY_SLOW, applied.dryingClass)
    }

    @Test
    fun `les corrections du catalogue et les photos suivent aussi`() = runTest {
        val first = Installation()
        javaArchive().use { first.importer.import(it) }

        val archive = directory.resolve("corrections.zip")
        ZipBackupWriter(archive).use { first.exporter.export(it) }

        val second = Installation()
        ZipBackupArchive(archive).use { second.importer.import(it) }

        // La teinte diluee relevee sur le Burnt Umber de Gamblin.
        assertEquals("#C9B9AC", second.catalog.findByNaturalKey("Gamblin", "Burnt Umber")!!.tintHex)

        val project = second.projects.all().single()
        val photos = second.projects.findWithPhotos(project.id!!)!!.photos
        assertEquals(listOf("piece.png"), photos.map { it.caption })
        assertTrue(photos.single().data.isNotEmpty())
    }

    @Test
    fun `l'archive produite porte le format que l'application Java sait lire`() = runTest {
        val first = Installation()
        javaArchive().use { first.importer.import(it) }

        val archive = directory.resolve("format.zip")
        ZipBackupWriter(archive).use { first.exporter.export(it) }

        java.util.zip.ZipFile(archive.toFile()).use { zip ->
            val document = zip.getEntry(BackupExporter.DOCUMENT)
            assertNotNull(document, "le document principal doit s'appeler ${BackupExporter.DOCUMENT}")

            val json = zip.getInputStream(document).readAllBytes().decodeToString()
            assertTrue("\"formatVersion\": 1" in json, "version de format annoncee")
            // Les tubes sont designes par marque et nom, jamais par identifiant.
            assertTrue("\"brand\"" in json)
            assertTrue("\"id\":" !in json, "aucun identifiant de base ne doit sortir")

            // Les images sont rangees a cote du JSON, pas encodees dedans.
            val images = zip.entries().asSequence().filter { it.name.startsWith(BackupExporter.IMAGES + "/") }
            assertEquals(1, images.count())
        }
    }
}
