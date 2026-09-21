package be.asmolabs.palettier.data

import be.asmolabs.palettier.data.backup.BackupExporter
import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.db.PalettierDatabase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.Test
import kotlin.test.assertTrue

/** Outil : ecrit une archive Kotlin pour que l'application Java essaie de la relire. */
class ExportForJavaTest {

    @Test
    fun writeArchiveForJava() = runTest {
        val database = PalettierDatabase(inMemoryDriver())
        val catalog = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
        val palettes = SqlDelightPaletteRepository(database, catalog, Dispatchers.Default)
        val projects = SqlDelightProjectRepository(database, catalog, palettes, Dispatchers.Default)
        val recipes = SqlDelightRecipeRepository(database, Dispatchers.Default)

        ZipBackupArchive(
            Path.of(requireNotNull(javaClass.classLoader.getResource("sauvegarde-reelle.zip")).toURI())
        ).use { BackupImporter(catalog, palettes, projects, recipes).import(it) }

        val target = Path.of("build/sortie-kotlin.zip")
        val report = ZipBackupWriter(target).use {
            BackupExporter(catalog, palettes, projects, recipes).export(it)
        }

        assertTrue(Files.exists(target))
        println("ARCHIVE KOTLIN ${report.paints} huiles, ${report.projects} projets, " +
            "${report.photos} photos -> ${target.toAbsolutePath()}")
    }
}
