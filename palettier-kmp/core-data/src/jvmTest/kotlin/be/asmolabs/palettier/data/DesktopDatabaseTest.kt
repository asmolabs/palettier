package be.asmolabs.palettier.data

import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.paint.Paint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import kotlin.io.path.deleteRecursively
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

/**
 * Ouvrir deux fois la meme base.
 *
 * <p>Cree sans regarder, le schema marche au premier demarrage et echoue au deuxieme sur
 * "table paint already exists" -- l'application ne s'ouvre alors plus du tout. Le defaut
 * ne se voit pas sur une base en memoire, qui est neuve a chaque essai : il a fallu
 * lancer l'application deux fois pour le rencontrer.</p>
 */
class DesktopDatabaseTest {

    @OptIn(kotlin.io.path.ExperimentalPathApi::class)
    private val directory = Files.createTempDirectory("palettier-essai").also {
        Runtime.getRuntime().addShutdownHook(Thread { it.deleteRecursively() })
    }

    @AfterTest
    @OptIn(kotlin.io.path.ExperimentalPathApi::class)
    fun tearDown() = directory.deleteRecursively()

    private fun umber() = Paint(
        brand = "Winsor & Newton", name = "Burnt Umber", hexColor = "#4A3427", pigments = setOf("PBr7"),
    )

    @Test
    fun `la base se rouvre au demarrage suivant, avec ce qu'elle contenait`() = runTest {
        val first = desktopDriver(directory)
        SqlDelightPaintCatalogRepository(PalettierDatabase(first), Dispatchers.Default).save(umber())
        first.close()

        // Le deuxieme demarrage ne doit ni echouer ni repartir de zero.
        val second = desktopDriver(directory)
        val catalog = SqlDelightPaintCatalogRepository(PalettierDatabase(second), Dispatchers.Default)

        assertEquals(listOf("Burnt Umber"), catalog.all().map { it.name })
        assertEquals(setOf("PBr7"), catalog.all().single().pigments)
        second.close()
    }

    @Test
    fun `une base neuve recoit son schema et sa version`() = runTest {
        val driver = desktopDriver(directory)
        val catalog = SqlDelightPaintCatalogRepository(PalettierDatabase(driver), Dispatchers.Default)

        assertEquals(emptyList(), catalog.all())
        catalog.save(umber())
        assertEquals(1, catalog.all().size)
        driver.close()
    }
}
