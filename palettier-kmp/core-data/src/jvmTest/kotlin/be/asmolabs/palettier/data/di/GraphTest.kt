package be.asmolabs.palettier.data.di

import app.cash.sqldelight.db.SqlDriver
import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.data.inMemoryDriver
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.PaintMatcher
import be.asmolabs.palettier.domain.palette.PaletteMixService
import be.asmolabs.palettier.domain.plan.PlanDryingService
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteMixRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.ProgressCheckService
import be.asmolabs.palettier.domain.project.SubstituteService
import be.asmolabs.palettier.domain.workbench.ReadinessWatch
import be.asmolabs.palettier.domain.workbench.WorkbenchService
import kotlinx.coroutines.Dispatchers
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.dsl.module
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertNotNull
import kotlin.test.assertSame

/**
 * Le graphe s'assemble-t-il ?
 *
 * <p>Un cablage incomplet ne se voit pas a la compilation : Koin resout au premier appel,
 * et le premier appel arrive a l'ecran. Cet essai le fait tout de suite, et sur toutes
 * les entrees a la fois.</p>
 */
class GraphTest {

    // Le pilote de la plateforme est remplace par une base en memoire : on verifie le
    // cablage, pas le repertoire personnel du peintre.
    private val testPlatform = module {
        single<SqlDriver> { inMemoryDriver() }
        single(IO) { Dispatchers.IO }
    }

    private val koin = startKoin { modules(testPlatform, dataModule, domainModule) }.koin

    @AfterTest
    fun tearDown() = stopKoin()

    @Test
    fun `tout ce que l'application demande se resout`() {
        assertNotNull(koin.get<PaintCatalogRepository>())
        assertNotNull(koin.get<PaletteRepository>())
        assertNotNull(koin.get<ProjectRepository>())
        assertNotNull(koin.get<RecipeRepository>())
        assertNotNull(koin.get<PaletteMixRepository>())
        assertNotNull(koin.get<BackupImporter>())

        assertNotNull(koin.get<DryingTimeService>())
        assertNotNull(koin.get<ColorMixService>())
        assertNotNull(koin.get<PlanDryingService>())
        assertNotNull(koin.get<WorkbenchService>())
        assertNotNull(koin.get<PaletteMixService>())
        assertNotNull(koin.get<FatOverLeanService>())
        assertNotNull(koin.get<SubstituteService>())
        assertNotNull(koin.get<ProgressCheckService>())
        assertNotNull(koin.get<PaintMatcher>())
        assertNotNull(koin.get<ReadinessWatch>())
    }

    @Test
    fun `la surveillance est unique, sans quoi elle repeterait ses annonces`() {
        // Deux instances oublieraient chacune ce que l'autre a deja dit.
        assertSame(koin.get<ReadinessWatch>(), koin.get<ReadinessWatch>())
    }

    @Test
    fun `tous les depots partagent la meme base`() {
        // Deux bases signifieraient deux fichiers, et un catalogue qui disparait d'un
        // ecran a l'autre.
        assertSame(koin.get<PalettierDatabase>(), koin.get<PalettierDatabase>())
    }
}
