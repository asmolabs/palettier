package be.asmolabs.palettier

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.TabRow
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import be.asmolabs.palettier.data.di.dataModule
import be.asmolabs.palettier.data.di.domainModule
import be.asmolabs.palettier.data.di.platformModule
import be.asmolabs.palettier.ui.catalog.CatalogScreen
import be.asmolabs.palettier.ui.catalog.CatalogViewModel
import be.asmolabs.palettier.ui.projects.ProjectsScreen
import be.asmolabs.palettier.ui.projects.ProjectsViewModel
import be.asmolabs.palettier.ui.workbench.WorkbenchScreen
import be.asmolabs.palettier.ui.workbench.WorkbenchViewModel
import org.koin.compose.KoinApplication
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel
import org.koin.dsl.module

/**
 * Point d'entree du client de bureau.
 *
 * <p>Il ne fait que trois choses : assembler le graphe, poser une fenetre, et brancher
 * l'ecran dessus. Tout le reste appartient aux modules partages -- c'est la difference
 * avec le lanceur JavaFX, qui devait aussi faire le pont entre deux cycles de vie.</p>
 */
fun main() = application {
    Window(
        onCloseRequest = ::exitApplication,
        title = "Palettier",
        state = rememberWindowState(width = 1100.dp, height = 800.dp),
    ) {
        KoinApplication(application = {
            modules(platformModule, dataModule, domainModule, uiModule)
        }) {
            MaterialTheme(colorScheme = atelier) {
                val model: WorkbenchViewModel = koinViewModel()
                val state by model.state.collectAsStateWithLifecycle()
                val scope = rememberCoroutineScope()
                val importer = koinInject<be.asmolabs.palettier.data.backup.BackupImporter>()
                val import = remember { ImportState(importer, scope) }

                val projectsModel: ProjectsViewModel = koinViewModel()
                val projectsState by projectsModel.state.collectAsStateWithLifecycle()
                val catalogModel: CatalogViewModel = koinViewModel()
                val catalogState by catalogModel.state.collectAsStateWithLifecycle()
                var section by remember { mutableStateOf(0) }

                Surface(Modifier.fillMaxSize()) {
                    Column {
                        ImportBar(import)
                        TabRow(selectedTabIndex = section) {
                            // Aujourd'hui d'abord : c'est la question du matin.
                            Tab(section == 0, { section = 0 }) {
                                Text("Aujourd'hui", Modifier.padding(vertical = 12.dp))
                            }
                            Tab(section == 1, { section = 1 }) {
                                Text("Projets", Modifier.padding(vertical = 12.dp))
                            }
                            Tab(section == 2, { section = 2 }) {
                                Text("Catalogue", Modifier.padding(vertical = 12.dp))
                            }
                        }
                        when (section) {
                            0 -> WorkbenchScreen(state, Modifier.weight(1f))
                            1 -> ProjectsScreen(projectsState, projectsModel::onIntent, Modifier.weight(1f))
                            else -> CatalogScreen(catalogState, catalogModel::onIntent, Modifier.weight(1f))
                        }
                    }
                }
            }
        }
    }
}

/**
 * La barre d'import.
 *
 * <p>Tant qu'une sauvegarde n'a pas ete relue, la base est vide : c'est le seul chemin par
 * lequel les donnees de l'application Java entrent, et il merite d'etre visible.</p>
 */
@androidx.compose.runtime.Composable
private fun ImportBar(import: ImportState) {
    Surface(color = MaterialTheme.colorScheme.surfaceVariant) {
        Column(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Button(onClick = import::choose, enabled = !import.busy) {
                    Text("Importer une sauvegarde")
                }
                if (import.busy) CircularProgressIndicator(Modifier.size(18.dp))
            }
            import.message?.let {
                Text(it, style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 8.dp))
            }
        }
    }
}

/** Les ViewModels : declares ici, parce que c'est l'application qui sait ce qu'elle affiche. */
private val uiModule = module {
    factory { WorkbenchViewModel(get(), get(), get(), get()) }
    factory { ProjectsViewModel(get(), get(), get(), get(), get()) }
    factory { CatalogViewModel(get()) }
    single { be.asmolabs.palettier.domain.plan.ProjectPlanner(get()) }
}

/**
 * Sombre par defaut : on peint sous une lampe dirigee sur la piece, pas sur l'ecran.
 * Les teintes reprennent celles de l'application JavaFX -- ocre et terre brulee.
 */
private val atelier = darkColorScheme(
    primary = Color(0xFFE2A05F),
    onPrimary = Color(0xFF1B1410),
    surface = Color(0xFF1B1713),
    onSurface = Color(0xFFDED6CC),
    surfaceVariant = Color(0xFF252017),
    onSurfaceVariant = Color(0xFFBDB4A8),
    background = Color(0xFF14110E),
    onBackground = Color(0xFFDED6CC),
)
