package be.asmolabs.palettier.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.NavigationDrawerItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import be.asmolabs.palettier.data.catalog.CatalogSeeder
import be.asmolabs.palettier.ui.app.Section
import be.asmolabs.palettier.ui.app.atelier
import be.asmolabs.palettier.ui.assistant.AssistantIntent
import be.asmolabs.palettier.ui.assistant.AssistantScreen
import be.asmolabs.palettier.ui.assistant.AssistantViewModel
import be.asmolabs.palettier.ui.catalog.CatalogScreen
import be.asmolabs.palettier.ui.catalog.CatalogViewModel
import be.asmolabs.palettier.ui.drying.DryingScreen
import be.asmolabs.palettier.ui.drying.DryingViewModel
import be.asmolabs.palettier.ui.mixer.MixerScreen
import be.asmolabs.palettier.ui.mixer.MixerViewModel
import be.asmolabs.palettier.ui.palettes.PalettesScreen
import be.asmolabs.palettier.ui.palettes.PalettesViewModel
import be.asmolabs.palettier.ui.picker.PickerIntent
import be.asmolabs.palettier.ui.picker.PickerScreen
import be.asmolabs.palettier.ui.picker.PickerViewModel
import be.asmolabs.palettier.ui.projects.ProjectsIntent
import be.asmolabs.palettier.ui.projects.ProjectsScreen
import be.asmolabs.palettier.ui.projects.ProjectsViewModel
import be.asmolabs.palettier.ui.recipes.RecipesScreen
import be.asmolabs.palettier.ui.recipes.RecipesViewModel
import be.asmolabs.palettier.ui.workbench.WorkbenchScreen
import be.asmolabs.palettier.ui.workbench.WorkbenchViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel

/**
 * Le client Android.
 *
 * <p>Les ecrans sont ceux du bureau, sans copie : ce qui change est la navigation -- un
 * tiroir plutot qu'une colonne, parce qu'un telephone n'a pas 248 points a donner en
 * permanence -- et la facon de choisir une photo, qui passe par le selecteur du systeme
 * au lieu d'une boite de dialogue.</p>
 */
class MainActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            MaterialTheme(colorScheme = atelier) {
                val photos = rememberPhotoPicker()
                var section by remember { mutableStateOf(Section.entries.first()) }
                val drawer = rememberDrawerState(DrawerValue.Closed)
                val scope = rememberCoroutineScope()

                // L'amorcage avant tout le reste : sur une installation neuve, c'est lui
                // qui met les 680 huiles en base. Le catalogue etant compile dans
                // l'application, un telephone n'a rien a importer pour commencer.
                val seeder = koinInject<CatalogSeeder>()
                var seeding by remember { mutableStateOf(true) }
                androidx.compose.runtime.LaunchedEffect(Unit) {
                    withContext(Dispatchers.Default) { seeder.seed() }
                    seeding = false
                }

                ModalNavigationDrawer(
                    drawerState = drawer,
                    drawerContent = {
                        ModalDrawerSheet {
                            Column(Modifier.padding(12.dp)) {
                                Text(
                                    "Palettier",
                                    style = MaterialTheme.typography.titleMedium,
                                    modifier = Modifier.padding(16.dp),
                                )
                                Section.entries.forEach { entry ->
                                    NavigationDrawerItem(
                                        label = { Text(entry.title) },
                                        selected = entry == section,
                                        onClick = {
                                            section = entry
                                            scope.launch { drawer.close() }
                                        },
                                    )
                                }
                            }
                        }
                    },
                ) {
                    Scaffold(
                        topBar = {
                            TopAppBar(
                                title = { Text(section.title) },
                                navigationIcon = {
                                    TextButton({ scope.launch { drawer.open() } }) { Text("≡") }
                                },
                            )
                        },
                    ) { padding ->
                        Surface(Modifier.fillMaxSize().padding(padding)) {
                            if (seeding) {
                                Box(Modifier.fillMaxSize()) {
                                    CircularProgressIndicator(Modifier.size(28.dp))
                                }
                            } else {
                                Content(section, photos)
                            }
                        }
                    }
                }
            }
        }
    }
}

/**
 * Les ecrans, les memes que sur le bureau.
 *
 * <p>Ce qui n'existe pas encore ici est passe a vide plutot qu'invente : l'export PDF
 * demande une bibliotheque que cette cible ne porte pas, et la sauvegarde demande le
 * selecteur de documents du systeme. Les deux viendront ; mieux vaut un bouton sans effet
 * qu'un ecran different de celui du bureau.</p>
 */
@androidx.compose.runtime.Composable
private fun Content(section: Section, photos: PhotoPicker) {
    val scope = rememberCoroutineScope()

    when (section) {
        Section.Today -> {
            val model: WorkbenchViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            WorkbenchScreen(state)
        }

        Section.Projects -> {
            val model: ProjectsViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            ProjectsScreen(
                state,
                model::onIntent,
                onChoosePhoto = { role ->
                    scope.launch { photos.pick()?.let { model.onIntent(ProjectsIntent.AddPhoto(it, role)) } }
                },
                onCompare = {
                    scope.launch { photos.pick()?.let { model.onIntent(ProjectsIntent.CompareWithPlan(it)) } }
                },
                onExport = {},
            )
        }

        Section.Palettes -> {
            val model: PalettesViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            PalettesScreen(state, model::onIntent)
        }

        Section.Picker -> {
            val model: PickerViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            PickerScreen(
                state,
                model::onIntent,
                onChooseImage = {
                    scope.launch { photos.pick()?.let { model.onIntent(PickerIntent.Load(it)) } }
                },
            )
        }

        Section.Assistant -> {
            val model: AssistantViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            AssistantScreen(
                state,
                model::onIntent,
                onChooseFigurine = {
                    scope.launch { photos.pick()?.let { model.onIntent(AssistantIntent.AttachFigurine(it)) } }
                },
                onChooseReference = {
                    scope.launch { photos.pick()?.let { model.onIntent(AssistantIntent.AttachReference(it)) } }
                },
                onChooseShelf = {
                    scope.launch { photos.pick()?.let { model.onIntent(AssistantIntent.ReadTubes(it)) } }
                },
                onExport = {},
            )
        }

        Section.Mixer -> {
            val model: MixerViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            MixerScreen(state, model::onIntent)
        }

        Section.Catalogue -> {
            val model: CatalogViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            CatalogScreen(state, model::onIntent)
        }

        Section.Drying -> {
            val model: DryingViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            DryingScreen(state, model::onIntent)
        }

        Section.Recipes -> {
            val model: RecipesViewModel = koinViewModel()
            val state by model.state.collectAsStateWithLifecycle()
            RecipesScreen(state, model::onIntent)
        }

        Section.Settings -> Box(Modifier.fillMaxSize().padding(20.dp)) {
            Text(
                "La sauvegarde passera par le selecteur de documents du systeme. " +
                    "En attendant, le catalogue est livre avec l'application : " +
                    "rien n'est a importer pour commencer.",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}
