package be.asmolabs.palettier

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import be.asmolabs.palettier.data.di.dataModule
import be.asmolabs.palettier.data.di.domainModule
import be.asmolabs.palettier.data.di.platformModule
import be.asmolabs.palettier.ui.workbench.WorkbenchScreen
import be.asmolabs.palettier.ui.workbench.WorkbenchViewModel
import org.koin.compose.KoinApplication
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
                WorkbenchScreen(state)
            }
        }
    }
}

/** Les ViewModels : declares ici, parce que c'est l'application qui sait ce qu'elle affiche. */
private val uiModule = module {
    factory { WorkbenchViewModel(get(), get(), get(), get()) }
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
