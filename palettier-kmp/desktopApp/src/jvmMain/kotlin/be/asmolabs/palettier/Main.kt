package be.asmolabs.palettier

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import be.asmolabs.palettier.ai.di.aiModule
import be.asmolabs.palettier.data.catalog.CatalogSeeder
import be.asmolabs.palettier.data.di.dataModule
import be.asmolabs.palettier.data.di.domainModule
import be.asmolabs.palettier.data.di.platformModule
import be.asmolabs.palettier.image.imageDecoder
import be.asmolabs.palettier.ui.assistant.AssistantIntent
import be.asmolabs.palettier.ui.assistant.AssistantScreen
import be.asmolabs.palettier.ui.assistant.AssistantViewModel
import be.asmolabs.palettier.ui.catalog.CatalogScreen
import be.asmolabs.palettier.ui.export.PdfPlanExporter
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
import be.asmolabs.palettier.ui.recipes.RecipesScreen
import be.asmolabs.palettier.ui.settings.SettingsScreen
import be.asmolabs.palettier.ui.settings.SettingsUiState
import be.asmolabs.palettier.ui.recipes.RecipesViewModel
import be.asmolabs.palettier.ui.projects.ProjectsViewModel
import be.asmolabs.palettier.ui.workbench.WorkbenchScreen
import be.asmolabs.palettier.ui.workbench.WorkbenchViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
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
    var section by remember { mutableStateOf(Section.entries.first()) }
    var navExpanded by remember { mutableStateOf(true) }

    Window(
        onCloseRequest = ::exitApplication,
        title = "Palettier",
        state = rememberWindowState(width = 1280.dp, height = 860.dp),
        // On peint d'une main, la souris tient souvent autre chose. Passer du releve au
        // melange doit pouvoir se faire sans lacher le pinceau.
        onPreviewKeyEvent = { event ->
            val held = event.isMetaPressed || event.isCtrlPressed
            val digit = DIGITS.indexOf(event.key)
            val chosen = Section.entries.firstOrNull { it.shortcut == digit }
            when {
                event.type != KeyEventType.KeyDown || !held -> false
                // Plier le menu rend la largeur a l'ecran : une photo au zoom la reclame.
                event.key == Key.B -> {
                    navExpanded = !navExpanded
                    true
                }
                chosen != null -> {
                    section = chosen
                    true
                }
                else -> false
            }
        },
    ) {
        KoinApplication(application = {
            modules(platformModule, dataModule, domainModule, aiModule, uiModule)
        }) {
            MaterialTheme(colorScheme = atelier) {
                // L'amorcage avant tout le reste : sur une installation neuve, c'est lui
                // qui met les 680 huiles, les palettes et les recettes d'exemple en base.
                // Idempotent, donc appele sans condition a chaque demarrage.
                val seeder = koinInject<CatalogSeeder>()
                LaunchedEffect(Unit) {
                    val report = withContext(Dispatchers.Default) { seeder.seed() }
                    if (report.touchedAnything) {
                        println(
                            "Amorcage : ${report.paintsCreated} tubes, " +
                                "${report.palettesAdded} palettes, ${report.recipesAdded} recettes, " +
                                "${report.paintsRefreshed} fiches rafraichies, " +
                                "${report.paintsRemoved} perimees retirees, " +
                                "${report.tintsBackfilled} teintes diluees rattrapees"
                        )
                    }
                    report.unknownPigments.takeIf { it.isNotEmpty() }?.let {
                        println("Pigments cites par le catalogue mais absents de la table : ${it.joinToString()}")
                    }
                    // Nommees, et pas seulement comptees : ce sont des fiches que plus
                    // aucun fichier de gamme ne decrit, et le peintre est seul a savoir
                    // s'il veut les garder.
                    report.paintsKeptCalibrated.takeIf { it.isNotEmpty() }?.let {
                        println(
                            "Fiches perimees gardees parce qu'elles portent votre mesure : " +
                                it.joinToString()
                        )
                    }
                }

                val model: WorkbenchViewModel = koinViewModel()
                val state by model.state.collectAsStateWithLifecycle()
                val scope = rememberCoroutineScope()
                val importer = koinInject<be.asmolabs.palettier.data.backup.BackupImporter>()
                val exporter = koinInject<be.asmolabs.palettier.data.backup.BackupExporter>()
                val import = remember { ImportState(importer, exporter, scope) }

                val projectsModel: ProjectsViewModel = koinViewModel()
                val projectsState by projectsModel.state.collectAsStateWithLifecycle()
                val catalogModel: CatalogViewModel = koinViewModel()
                val catalogState by catalogModel.state.collectAsStateWithLifecycle()
                val pickerModel: PickerViewModel = koinViewModel()
                val pickerState by pickerModel.state.collectAsStateWithLifecycle()
                val palettesModel: PalettesViewModel = koinViewModel()
                val palettesState by palettesModel.state.collectAsStateWithLifecycle()
                val mixerModel: MixerViewModel = koinViewModel()
                val mixerState by mixerModel.state.collectAsStateWithLifecycle()
                val dryingModel: DryingViewModel = koinViewModel()
                val dryingState by dryingModel.state.collectAsStateWithLifecycle()
                val recipesModel: RecipesViewModel = koinViewModel()
                val recipesState by recipesModel.state.collectAsStateWithLifecycle()
                val assistantModel: AssistantViewModel = koinViewModel()
                val assistantState by assistantModel.state.collectAsStateWithLifecycle()
                Surface(Modifier.fillMaxSize()) {
                    Row(Modifier.fillMaxSize()) {
                        Sidebar(section, navExpanded, { navExpanded = !navExpanded }) { section = it }

                        when (section) {
                            Section.Today -> WorkbenchScreen(state, Modifier.weight(1f))
                            Section.Projects -> ProjectsScreen(
                                projectsState,
                                projectsModel::onIntent,
                                onChoosePhoto = { role ->
                                    pickPhoto()?.let {
                                        projectsModel.onIntent(ProjectsIntent.AddPhoto(it, role))
                                    }
                                },
                                onCompare = {
                                    // La comparaison lit le fichier d'origine, pas la photo
                                    // rangee : celle-ci est reduite, ses couleurs ont bouge.
                                    pickPhoto()?.let {
                                        projectsModel.onIntent(ProjectsIntent.CompareWithPlan(it))
                                    }
                                },
                                onExport = {
                                    projectsState.plan?.let { plan ->
                                        exportPlan(plan, projectsState.selected?.name.orEmpty())
                                    }
                                },
                                modifier = Modifier.weight(1f),
                            )
                            Section.Palettes ->
                                PalettesScreen(palettesState, palettesModel::onIntent, Modifier.weight(1f))
                            Section.Picker -> PickerScreen(
                                pickerState,
                                pickerModel::onIntent,
                                onChooseImage = {
                                    // Le choix du fichier appartient a la plateforme ;
                                    // le ViewModel ne connait que des octets.
                                    pickPhoto()?.let { pickerModel.onIntent(PickerIntent.Load(it)) }
                                },
                                Modifier.weight(1f),
                            )
                            Section.Assistant -> AssistantScreen(
                                assistantState,
                                assistantModel::onIntent,
                                onChooseFigurine = {
                                    pickPhoto()?.let { assistantModel.onIntent(AssistantIntent.AttachFigurine(it)) }
                                },
                                onChooseReference = {
                                    pickPhoto()?.let { assistantModel.onIntent(AssistantIntent.AttachReference(it)) }
                                },
                                onChooseShelf = {
                                    pickPhoto()?.let { assistantModel.onIntent(AssistantIntent.ReadTubes(it)) }
                                },
                                onExport = {
                                    assistantState.plan?.let { plan ->
                                        exportPlan(plan, assistantState.suggestedProjectName)
                                    }
                                },
                                modifier = Modifier.weight(1f),
                            )
                            Section.Mixer -> MixerScreen(mixerState, mixerModel::onIntent, Modifier.weight(1f))
                            Section.Catalogue ->
                                CatalogScreen(catalogState, catalogModel::onIntent, Modifier.weight(1f))
                            Section.Drying -> DryingScreen(dryingState, dryingModel::onIntent, Modifier.weight(1f))
                            Section.Recipes ->
                                RecipesScreen(recipesState, recipesModel::onIntent, Modifier.weight(1f))
                            Section.Settings -> SettingsScreen(
                                SettingsUiState(busy = import.busy, message = import.message),
                                onExport = import::exportTo,
                                onImport = import::choose,
                                Modifier.weight(1f),
                            )
                        }
                    }
                }
            }
        }
    }
}

/**
 * Les sections, dans l'ordre de la barre laterale.
 *
 * <p>Le raccourci est declare par la section, et non deduit de son rang. C'etait deja la
 * regle cote JavaFX, et pour une raison qui tient : inserer une section en tete
 * deplacerait sinon tout ce qui suit sous les doigts de quelqu'un qui a pris ses
 * habitudes.</p>
 */
private enum class Section(val title: String, val shortcut: Int) {
    Today("Aujourd'hui", 0),
    Projects("Projets", 1),
    Palettes("Palettes", 2),
    Picker("Pipette", 3),
    Assistant("Assistant", 4),
    Mixer("Melangeur", 5),
    Catalogue("Catalogue", 6),
    Drying("Sechage", 7),
    Recipes("Recettes", 8),
    Settings("Parametres", 9),
}

/** Les touches chiffrees, dans l'ordre, pour retrouver le rang qu'elles designent. */
private val DIGITS = listOf(
    Key.Zero, Key.One, Key.Two, Key.Three, Key.Four,
    Key.Five, Key.Six, Key.Seven, Key.Eight, Key.Nine,
)

/**
 * La navigation : une colonne, pas dix onglets.
 *
 * <p>A dix entrees, une rangee d'onglets comprime ses libelles jusqu'a les rendre
 * illisibles et n'a nulle part ou afficher un raccourci. La colonne laisse a chaque
 * section sa ligne, sa pastille et son chiffre -- et ce chiffre s'affiche, sinon personne
 * ne le devine.</p>
 */
@androidx.compose.runtime.Composable
private fun Sidebar(
    current: Section,
    expanded: Boolean,
    onToggle: () -> Unit,
    onChoose: (Section) -> Unit,
) {
    // Repliee, la colonne garde ses pastilles : on continue de voir ou l'on est, et le
    // clic reste au meme endroit. C'est ce qui separe un menu plie d'un menu disparu.
    val width by animateDpAsState(if (expanded) 248.dp else 56.dp)

    Surface(
        Modifier.width(width).fillMaxHeight(),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(Modifier.fillMaxHeight()) {
            Row(
                Modifier.padding(start = 18.dp, top = 22.dp, end = 10.dp, bottom = 22.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (expanded) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            "Palettier",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                        )
                        Text(
                            "peinture a l'huile",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                Text(
                    if (expanded) "\u00AB" else "\u00BB",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.clickable(onClick = onToggle).padding(6.dp),
                )
            }

            Section.entries.forEach { entry ->
                NavItem(entry, entry == current, expanded) { onChoose(entry) }
            }

            Spacer(Modifier.weight(1f))

            if (expanded) {
                Text(
                    "Aucune donnee ne part ailleurs.\nTout reste dans ~/.palettier\n\u2318B replie le menu",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(18.dp),
                )
            }
        }
    }
}

@androidx.compose.runtime.Composable
private fun NavItem(section: Section, selected: Boolean, expanded: Boolean, onClick: () -> Unit) {
    val accent = MaterialTheme.colorScheme.primary
    Surface(
        color = if (selected) MaterialTheme.colorScheme.surface else androidx.compose.ui.graphics.Color.Transparent,
        modifier = Modifier.fillMaxWidth().clickable(onClick = onClick),
    ) {
        Row(
            Modifier.padding(horizontal = 18.dp, vertical = 9.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            androidx.compose.foundation.layout.Box(
                Modifier
                    .size(7.dp)
                    .background(
                        if (selected) accent else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.35f),
                        androidx.compose.foundation.shape.CircleShape,
                    )
            )
            if (expanded) {
                Spacer(Modifier.width(10.dp))
                Text(
                    section.title,
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = if (selected) FontWeight.Medium else FontWeight.Normal,
                    color = if (selected) MaterialTheme.colorScheme.onSurface
                    else MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(Modifier.weight(1f))
                Text(
                    section.shortcut.toString(),
                    style = MaterialTheme.typography.labelSmall,
                    color = if (selected) accent else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                )
            }
        }
    }
}

/**
 * Met le plan sur papier, la ou le peintre le demande.
 *
 * <p>Le rendu ne connait pas de chemin : il rend des octets, et c'est ici -- la seule
 * couche qui sache ce qu'est un dossier -- qu'on demande ou les poser.</p>
 */
private fun exportPlan(plan: be.asmolabs.palettier.domain.plan.PaintingPlan, suggested: String) {
    val dialog = java.awt.FileDialog(null as java.awt.Frame?, "Enregistrer le plan", java.awt.FileDialog.SAVE)
    dialog.file = (suggested.ifBlank { "plan" }.replace(Regex("[^A-Za-z0-9 _-]"), "")) + ".pdf"
    dialog.isVisible = true
    val directory = dialog.directory ?: return
    val name = dialog.file ?: return

    kotlinx.coroutines.runBlocking {
        val bytes = PdfPlanExporter().render(plan)
        java.nio.file.Files.write(java.nio.file.Path.of(directory, name), bytes)
    }
}

/** Le fichier choisi par le peintre, lu tel quel : reduire deplacerait les couleurs. */
private fun pickPhoto(): ByteArray? {
    val dialog = java.awt.FileDialog(null as java.awt.Frame?, "Photo a relever", java.awt.FileDialog.LOAD)
    dialog.isVisible = true
    val directory = dialog.directory ?: return null
    val name = dialog.file ?: return null
    return java.nio.file.Files.readAllBytes(java.nio.file.Path.of(directory, name))
}

/** Les ViewModels : declares ici, parce que c'est l'application qui sait ce qu'elle affiche. */
private val uiModule = module {
    // Un seul relais pour toute l'application : la pipette y depose sa mesure, le
    // catalogue, le melangeur et l'etabli la retrouvent sans se connaitre.
    single { be.asmolabs.palettier.ui.SampledColour() }
    factory { WorkbenchViewModel(get(), get(), get(), get(), get()) }
    factory { ProjectsViewModel(get(), get(), get(), get(), get(), imageDecoder(), get()) }
    factory { CatalogViewModel(get(), get()) }
    factory { PickerViewModel(imageDecoder(), get(), get(), get(), get(), get(), get()) }
    factory { PalettesViewModel(get(), get()) }
    factory { MixerViewModel(get(), get(), get(), get()) }
    factory { DryingViewModel(get()) }
    factory { RecipesViewModel(get(), get()) }
    factory { AssistantViewModel(get(), get(), get(), get(), get(), get(), get(), imageDecoder()) }
    single { be.asmolabs.palettier.domain.recipe.RecipeTimelineService(get()) }
    single { be.asmolabs.palettier.domain.plan.ProjectPlanner(get()) }
    single { be.asmolabs.palettier.domain.plan.SampledPlanService(get()) }
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
