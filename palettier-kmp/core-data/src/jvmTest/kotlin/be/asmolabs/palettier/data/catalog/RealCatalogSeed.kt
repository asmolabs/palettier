package be.asmolabs.palettier.data.catalog

import be.asmolabs.palettier.data.SqlDelightPaintCatalogRepository
import be.asmolabs.palettier.data.SqlDelightPaletteRepository
import be.asmolabs.palettier.data.SqlDelightRecipeRepository
import be.asmolabs.palettier.data.desktopDriver
import be.asmolabs.palettier.db.PalettierDatabase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import java.nio.file.Path
import kotlin.test.Test

/**
 * Passe le seeder sur une vraie base et raconte ce qu'il a fait.
 *
 * <p>Sert a voir ce qu'un demarrage changerait avant de le laisser arriver, en particulier
 * quelles fiches perimees seraient retirees. Ce n'est pas un test : sans base designee, il
 * ne fait rien.</p>
 *
 * <pre>
 * ./gradlew :core-data:jvmTest --tests '*RealCatalogSeed*' \
 *     -Dpalettier.seed.home=/chemin/vers/un/dossier/contenant/palettier.db
 * </pre>
 */
class RealCatalogSeed {

    @Test
    fun seed() {
        val home = System.getProperty("palettier.seed.home")?.let { Path.of(it) } ?: return

        val database = PalettierDatabase(desktopDriver(home))
        val paints = SqlDelightPaintCatalogRepository(database, Dispatchers.Default)
        val palettes = SqlDelightPaletteRepository(database, paints, Dispatchers.Default)
        val recipes = SqlDelightRecipeRepository(database, Dispatchers.Default)

        runBlocking {
            val before = paints.all()
            val report = CatalogSeeder(paints, palettes, recipes).seed()
            val after = paints.all()

            println("AMORCAGE de $home")
            println("  tubes avant : ${before.size}, apres : ${after.size}")
            println("  crees ${report.paintsCreated}, rafraichis ${report.paintsRefreshed}")
            println("  perimes retires ${report.paintsRemoved}, epargnes car cites ${report.paintsKeptInUse}")
            report.paintsKeptCalibrated.forEach { println("  EPARGNE car mesure : $it") }
            println("  teintes diluees rattrapees ${report.tintsBackfilled}")
            println("  palettes ajoutees ${report.palettesAdded}, recettes ajoutees ${report.recipesAdded}")

            val keys = after.map { it.brand to it.name }.toSet()
            before.filterNot { (it.brand to it.name) in keys }
                .forEach { println("  RETIRE : ${it.brand} - ${it.name}") }
        }
    }
}
