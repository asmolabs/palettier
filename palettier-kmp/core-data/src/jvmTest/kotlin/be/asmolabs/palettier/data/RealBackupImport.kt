package be.asmolabs.palettier.data

import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.db.PalettierDatabase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import java.nio.file.Path
import kotlin.test.Test

/**
 * Relit une vraie sauvegarde dans la base du poste, sans passer par la fenetre.
 *
 * <p>Le pendant de RealBackupExporter cote Java : l'ancienne application ecrit, celle-ci
 * relit. Ce n'est pas un test -- sans archive donnee, il ne fait rien et laisse la suite
 * tranquille.</p>
 *
 * <pre>
 * ./gradlew :core-data:jvmTest --tests '*RealBackupImport*' \
 *     -Dpalettier.import.from=/chemin/vers/sauvegarde.zip
 * </pre>
 *
 * <p>La destination est ~/.palettier, comme l'application, sauf si
 * {@code palettier.import.home} en designe une autre.</p>
 */
class RealBackupImport {

    @Test
    fun readArchive() {
        val source = System.getProperty("palettier.import.from") ?: return
        val home = System.getProperty("palettier.import.home")
            ?.let { Path.of(it) }
            ?: Path.of(System.getProperty("user.home"), ".palettier")

        val database = PalettierDatabase(desktopDriver(home))
        val catalog = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
        val palettes = SqlDelightPaletteRepository(database, catalog, Dispatchers.Default)
        val projects = SqlDelightProjectRepository(database, catalog, palettes, Dispatchers.Default)
        val recipes = SqlDelightRecipeRepository(database, Dispatchers.Default)
        val importer = BackupImporter(catalog, palettes, projects, recipes)

        val report = runBlocking {
            ZipBackupArchive(Path.of(source)).use { importer.import(it) }
        }

        println(
            "RESTAURATION $source vers $home : " +
                "${report.paintsCreated} tubes ajoutes, ${report.paintsUpdated} mis a jour, " +
                "${report.palettesAdded} palettes, ${report.projectsAdded} projets, " +
                "${report.recipesAdded} recettes, ${report.photos} photos, " +
                "${report.skipped} laisses de cote"
        )
        report.warnings.forEach { println("  avertissement : $it") }
    }
}
