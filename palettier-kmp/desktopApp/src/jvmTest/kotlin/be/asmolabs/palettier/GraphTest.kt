package be.asmolabs.palettier

import app.cash.sqldelight.db.SqlDriver
import be.asmolabs.palettier.ai.di.aiModule
import be.asmolabs.palettier.data.catalog.CatalogSeeder
import be.asmolabs.palettier.data.di.IO
import be.asmolabs.palettier.data.di.dataModule
import be.asmolabs.palettier.data.di.domainModule
import be.asmolabs.palettier.data.inMemoryDriver
import be.asmolabs.palettier.ui.SampledColour
import be.asmolabs.palettier.ui.app.uiModule
import be.asmolabs.palettier.ui.assistant.AssistantViewModel
import be.asmolabs.palettier.ui.catalog.CatalogViewModel
import be.asmolabs.palettier.ui.drying.DryingViewModel
import be.asmolabs.palettier.ui.mixer.MixerViewModel
import be.asmolabs.palettier.ui.palettes.PalettesViewModel
import be.asmolabs.palettier.ui.picker.PickerViewModel
import be.asmolabs.palettier.ui.projects.ProjectsViewModel
import be.asmolabs.palettier.ui.recipes.RecipesViewModel
import be.asmolabs.palettier.ui.workbench.WorkbenchViewModel
import kotlinx.coroutines.Dispatchers
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.dsl.module
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertNotNull
import kotlin.test.assertSame

/**
 * Le graphe complet s'assemble-t-il, interface comprise ?
 *
 * <p>core-data a deja son essai, mais il s'arrete aux depots. Un ViewModel dont il manque
 * un parametre ne se voit pas davantage a la compilation : Koin resout au premier appel,
 * et le premier appel arrive a l'ecran.</p>
 *
 * <p>C'est le module applicatif qui assemble, donc c'est ici que la verification a sa
 * place. Et comme uiModule est partage par les deux clients, ce qui passe ici vaut pour
 * le telephone : seul le pilote de base change d'une cible a l'autre.</p>
 */
class GraphTest {

    // Le pilote de la plateforme est remplace par une base en memoire : on verifie le
    // cablage, pas le repertoire personnel du peintre.
    private val testPlatform = module {
        single<SqlDriver> { inMemoryDriver() }
        single(IO) { Dispatchers.IO }
    }

    private val koin = startKoin {
        modules(testPlatform, dataModule, domainModule, aiModule, uiModule)
    }.koin

    @AfterTest
    fun tearDown() = stopKoin()

    @Test
    fun `chaque ecran obtient son ViewModel`() {
        assertNotNull(koin.get<WorkbenchViewModel>())
        assertNotNull(koin.get<ProjectsViewModel>())
        assertNotNull(koin.get<PalettesViewModel>())
        assertNotNull(koin.get<PickerViewModel>())
        assertNotNull(koin.get<AssistantViewModel>())
        assertNotNull(koin.get<MixerViewModel>())
        assertNotNull(koin.get<CatalogViewModel>())
        assertNotNull(koin.get<DryingViewModel>())
        assertNotNull(koin.get<RecipesViewModel>())
    }

    @Test
    fun `l'amorcage du catalogue est la, sans quoi une installation neuve resterait vide`() {
        assertNotNull(koin.get<CatalogSeeder>())
    }

    @Test
    fun `la teinte relevee est un seul objet, sinon elle ne circulerait pas`() {
        // Deux instances rendraient le pont muet : la pipette deposerait dans l'une et le
        // catalogue lirait l'autre, sans que rien ne signale l'erreur.
        assertSame(koin.get<SampledColour>(), koin.get<SampledColour>())
    }

    @Test
    fun `deux ecrans ouverts ont chacun leur ViewModel`() {
        // Un ViewModel porte l'etat d'un ecran : le partager ferait deriver deux ecrans
        // l'un sur l'autre.
        val first = koin.get<CatalogViewModel>()
        val second = koin.get<CatalogViewModel>()
        assertNotNull(first)
        assertNotNull(second)
        kotlin.test.assertNotSame(first, second)
    }
}
