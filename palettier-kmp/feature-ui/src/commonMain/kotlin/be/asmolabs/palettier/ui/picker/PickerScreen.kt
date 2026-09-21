package be.asmolabs.palettier.ui.picker

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.decodeToImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.ui.component.Chip
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt

/**
 * La pipette : relever une teinte sur une photo, et voir avec quoi la faire.
 *
 * <p>Trois precautions y sont visibles. La mesure porte sur le fichier d'origine, jamais
 * sur l'image telle qu'elle est affichee -- le zoom interpole, et relever dessus
 * reviendrait a mesurer le travail de la carte graphique. Le clic moyenne un petit carre
 * plutot qu'un pixel seul, sans quoi le bruit du capteur deciderait a la place du
 * peintre. Et une photo prise sous lampe chaude tire au jaune : designer un point cense
 * etre gris annule cette dominante.</p>
 */
@Composable
fun PickerScreen(
    state: PickerUiState,
    onIntent: (PickerIntent) -> Unit,
    onChooseImage: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(modifier.fillMaxSize()) {
        Row(Modifier.fillMaxSize().padding(20.dp), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
            Column(Modifier.weight(1.4f), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    TextButton(onChooseImage) { Text("Choisir une photo") }
                    if (state.loading) CircularProgressIndicator(Modifier.size(18.dp))
                    state.error?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
                }
                Photo(state, onIntent, Modifier.weight(1f))
            }

            Column(
                Modifier.weight(1f).verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                if (state.dominant.isNotEmpty()) {
                    Swatches(state, onIntent)
                    HorizontalDivider()
                    Working(state, onIntent)
                    HorizontalDivider()
                    Mixes(state, onIntent)
                    HorizontalDivider()
                }
                if (state.image != null || state.groups.isNotEmpty()) {
                    Groups(state, onIntent)
                } else if (!state.loading) {
                    Text(
                        "Ouvrez une photo, puis cliquez dessus pour relever une teinte. " +
                            "La mesure se fait sur votre fichier d'origine.",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        }
    }
}

/**
 * La photo, son zoom, et le clic qui releve.
 *
 * <p>Le facteur d'ajustement se recalcule avec la taille disponible : une fenetre
 * redimensionnee ne laisse pas la photo a moitie dehors. Au-dela de la taille reelle, le
 * lissage est coupe -- on veut voir les pixels, pas un flou d'interpolation, sans quoi on
 * releverait une couleur que le fichier ne contient pas.</p>
 */
@Composable
private fun Photo(state: PickerUiState, onIntent: (PickerIntent) -> Unit, modifier: Modifier) {
    val image = state.image
    if (image == null) {
        Box(modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            Text("Aucune photo ouverte.", style = MaterialTheme.typography.bodySmall)
        }
        return
    }

    // Le decodage d'affichage est refait quand la photo change, pas a chaque recomposition.
    val bitmap = remember(image) { runCatching { image.bytes.decodeToImageBitmap() }.getOrNull() }
    if (bitmap == null) {
        Box(modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
            Text("Cette image se lit mais ne s'affiche pas.", style = MaterialTheme.typography.bodySmall)
        }
        return
    }

    // null tant que le peintre n'a pas choisi : l'echelle suit alors la fenetre.
    var chosenScale: Float? by remember(image) { mutableStateOf(null) }
    val density = LocalDensity.current

    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        BoxWithConstraints(
            Modifier
                .weight(1f)
                .fillMaxWidth()
                .background(MaterialTheme.colorScheme.surfaceVariant, RoundedCornerShape(6.dp)),
        ) {
            val viewport = with(density) { Pair(maxWidth.toPx(), maxHeight.toPx()) }
            val fit = min(viewport.first / image.width, viewport.second / image.height)
                .coerceIn(MIN_SCALE, MAX_SCALE)
            val scale = chosenScale ?: fit

            val shownWidth = image.width * scale
            val shownHeight = image.height * scale

            Box(
                Modifier
                    .fillMaxSize()
                    .horizontalScroll(rememberScrollState())
                    .verticalScroll(rememberScrollState()),
                contentAlignment = Alignment.Center,
            ) {
                Box(
                    Modifier
                        .size(with(density) { shownWidth.toDp() }, with(density) { shownHeight.toDp() })
                        .pointerInput(image, scale, state.sampleRadius) {
                            detectTapGestures { offset ->
                                // De l'ecran vers les coordonnees de l'image, quel que soit le zoom.
                                val x = (offset.x / scale).roundToInt()
                                val y = (offset.y / scale).roundToInt()
                                onIntent(PickerIntent.PickAt(x, y))
                            }
                        },
                ) {
                    Image(
                        bitmap = bitmap,
                        contentDescription = "La photo a relever",
                        modifier = Modifier.fillMaxSize(),
                        // Au-dela de la taille reelle : les pixels, pas un flou.
                        filterQuality = if (scale > 1f) FilterQuality.None else FilterQuality.Medium,
                    )
                    state.pickedAt?.let { point ->
                        Marker(point, scale, state.sampleRadius)
                    }
                }
            }
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            val viewportScale = chosenScale
            TextButton({
                chosenScale = ((viewportScale ?: 1f) / ZOOM_STEP).coerceIn(MIN_SCALE, MAX_SCALE)
            }) { Text("−") }
            TextButton({
                chosenScale = ((viewportScale ?: 1f) * ZOOM_STEP).coerceIn(MIN_SCALE, MAX_SCALE)
            }) { Text("+") }
            TextButton({ chosenScale = null }) { Text("Ajuster") }
            TextButton({ chosenScale = 1f }) { Text("100 %") }
            Text(
                viewportScale?.let { "%.0f %%".format(it * 100) } ?: "ajuste",
                style = MaterialTheme.typography.bodySmall,
            )

            Spacer(Modifier.width(12.dp))
            Text("Rayon", style = MaterialTheme.typography.bodySmall)
            listOf(0, 1, 3, 6, 12).forEach { radius ->
                Chip("$radius", radius == state.sampleRadius) { onIntent(PickerIntent.SetRadius(radius)) }
            }
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Chip(
                if (state.pickingGrey) "Cliquez un point gris sur la photo" else "Definir le point neutre",
                state.pickingGrey,
            ) { onIntent(PickerIntent.PickGrey(!state.pickingGrey)) }
            if (state.greyPointHex != null) {
                TextButton({ onIntent(PickerIntent.UseAsGrey(null)) }) { Text("Retirer") }
            }
        }
    }
}

/** Le cercle qui montre ou le releve a ete pris, et sur quelle etendue. */
@Composable
private fun Marker(point: PickPoint, scale: Float, radius: Int) {
    val ring = MaterialTheme.colorScheme.primary
    Canvas(Modifier.fillMaxSize()) {
        drawCircle(
            color = ring,
            radius = max(MIN_MARKER_PX, radius * scale),
            center = androidx.compose.ui.geometry.Offset(point.x * scale, point.y * scale),
            style = Stroke(width = 2f),
        )
    }
}

/** Les teintes relevees, de la plus presente a la moins presente. */
@Composable
private fun Swatches(state: PickerUiState, onIntent: (PickerIntent) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text("Teintes relevees", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            state.dominant.forEach { dominant ->
                val hex = dominant.color.toHex()
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Swatch(
                        hex,
                        size = 46,
                        selected = hex == state.selectedHex,
                        onClick = { onIntent(PickerIntent.Select(hex)) },
                    )
                    Text("%.0f %%".format(dominant.share * 100), style = MaterialTheme.typography.labelSmall)
                }
            }
        }
    }
}

/** La teinte retenue, et la correction de dominante quand elle a ete demandee. */
@Composable
private fun Working(state: PickerUiState, onIntent: (PickerIntent) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            state.selectedHex?.let { Swatch(it, size = 40) }
            state.correctedHex?.let {
                Text("corrigee ->", style = MaterialTheme.typography.bodySmall)
                Swatch(it, size = 40)
            }
            Column {
                Text(state.workingHex.orEmpty(), style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
                state.greyPointHex?.let {
                    Text("dominante annulee d'apres $it", style = MaterialTheme.typography.bodySmall)
                }
                state.pickedAt?.let {
                    Text("releve en ${it.x}, ${it.y}", style = MaterialTheme.typography.bodySmall)
                }
            }
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Point gris :", style = MaterialTheme.typography.bodySmall)
            state.dominant.forEach { dominant ->
                val hex = dominant.color.toHex()
                Swatch(
                    hex,
                    size = 22,
                    selected = hex == state.greyPointHex,
                    onClick = { onIntent(PickerIntent.UseAsGrey(hex.takeIf { it != state.greyPointHex })) },
                )
            }
        }
    }
}

/**
 * Le melange a faire pour obtenir la teinte relevee.
 *
 * <p>C'est la vraie reponse de la pipette. Un tube du commerce s'approche rarement d'une
 * carnation ou d'une ombre : ce qu'on veut savoir, c'est quoi melanger et dans quelles
 * parts. La recherche se demande plutot qu'elle ne s'impose -- elle explore les dosages
 * realisables et coute, et on clique souvent plusieurs fois avant de retenir un point.</p>
 */
@Composable
private fun Mixes(state: PickerUiState, onIntent: (PickerIntent) -> Unit) {
    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("Le melange a faire", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)

        // Chercher dans une palette ne filtre pas un resultat : cela change la question, et
        // c'est ce qui separe une reponse theorique d'une recette faisable ce soir.
        Text("Chercher dans", style = MaterialTheme.typography.bodySmall)
        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            state.scopes.forEach { scope ->
                Chip(scope.label, scope == state.scope) { onIntent(PickerIntent.ChooseScope(scope)) }
            }
        }
        if (state.emptyScope) {
            Text("Cet ensemble ne contient aucun tube.", style = MaterialTheme.typography.bodySmall)
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("Au plus", style = MaterialTheme.typography.bodySmall)
            (1..4).forEach { count ->
                Chip(
                    if (count == 1) "un tube" else "$count tubes",
                    count == state.maxPaints,
                ) { onIntent(PickerIntent.SetMaxPaints(count)) }
            }
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            TextButton(
                { onIntent(PickerIntent.FindMixes) },
                enabled = state.workingHex != null && !state.searching && !state.emptyScope,
            ) { Text("Trouver le melange") }
            if (state.searching) CircularProgressIndicator(Modifier.size(16.dp))
        }

        state.suggestions.forEach { suggestion ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Swatch(suggestion.color.toHex(), size = 24)
                Spacer(Modifier.width(10.dp))
                Column {
                    Text(suggestion.describe(), style = MaterialTheme.typography.bodySmall)
                    Text(
                        "ecart %.1f  -  %s".format(suggestion.deltaE, verdict(suggestion.deltaE)),
                        style = MaterialTheme.typography.labelSmall,
                    )
                }
            }
        }

        if (state.suggestions.isEmpty() && !state.searching) {
            Text(
                "Un tube seul s'approche rarement d'une carnation. Pour n'en chercher " +
                    "qu'un, demandez le melange a un tube.",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

/**
 * Composer un projet a partir de releves, plutot que d'une proposition.
 *
 * <p>On nomme une partie du sujet, on y ajoute autant de points qu'on veut -- le creux de
 * l'orbite, la joue, l'arete du nez -- et l'application en deduit le degrade. Trois points
 * suffisent a obtenir une echelle, cinq donnent un modele complet.</p>
 */
@Composable
private fun Groups(state: PickerUiState, onIntent: (PickerIntent) -> Unit) {
    var name by remember { mutableStateOf("") }
    var projectName by remember { mutableStateOf("") }

    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("Groupes de releves", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("nom du groupe, par exemple Peau") },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
            TextButton({
                onIntent(PickerIntent.CreateGroup(name))
                name = ""
            }) { Text("Nouveau") }
        }

        if (state.groups.isNotEmpty()) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                state.groups.forEach { group ->
                    Chip(
                        "${group.name} (${group.samples.size})",
                        group.name == state.selectedGroup,
                    ) { onIntent(PickerIntent.SelectGroup(group.name)) }
                }
            }
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            TextButton(
                { onIntent(PickerIntent.AddSampleToGroup) },
                enabled = state.selectedGroup != null && state.workingHex != null,
            ) { Text("Ajouter le releve") }
            state.selectedGroup?.let { selected ->
                TextButton({ onIntent(PickerIntent.DeleteGroup(selected)) }) { Text("Supprimer le groupe") }
            }
        }

        // Du sombre au clair : c'est l'ordre dans lequel ces points deviendront des couches.
        state.group?.let { group ->
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                group.ordered.forEach { sample ->
                    val hex = sample.toHex()
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Swatch(hex, size = 34)
                        TextButton({ onIntent(PickerIntent.RemoveSample(group.name, hex)) }) {
                            Text("\u2715", style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }
        }

        if (state.hasSamples) {
            Text("Palette du projet", style = MaterialTheme.typography.bodySmall)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                state.palettes.forEach { palette ->
                    Chip(
                        "${palette.name} (${palette.paints.size})",
                        palette.id == state.projectPaletteId,
                    ) { palette.id?.let { onIntent(PickerIntent.ChooseProjectPalette(it)) } }
                }
            }

            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = projectName,
                    onValueChange = { projectName = it },
                    label = { Text("Nom du projet") },
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                )
                TextButton(
                    { onIntent(PickerIntent.CreateProject(projectName)) },
                    enabled = !state.savingProject,
                ) { Text("Creer un projet") }
                if (state.savingProject) CircularProgressIndicator(Modifier.size(16.dp))
            }
        }

        state.groupMessage?.let { Text(it, style = MaterialTheme.typography.bodySmall) }

        Text(
            "Relevez plusieurs points par groupe : le plus sombre deviendra l'ombre profonde, " +
                "le median la base, le plus clair le point lumineux.",
            style = MaterialTheme.typography.labelSmall,
        )
    }
}

/** Traduction de l'ecart en langage de peintre. */
private fun verdict(deltaE: Double): String = when {
    deltaE < 2 -> "identique a l'oeil"
    deltaE < 4 -> "tres proche"
    deltaE < 8 -> "proche, ecart visible cote a cote"
    deltaE < 15 -> "meme famille de teinte"
    else -> "couleur differente"
}

@Composable
private fun Swatch(hex: String, size: Int, selected: Boolean = false, onClick: (() -> Unit)? = null) {
    val rgb = Rgb.ofHex(hex)
    Box(
        Modifier
            .size(size.dp)
            .background(Color(rgb.r.toFloat(), rgb.g.toFloat(), rgb.b.toFloat()), RoundedCornerShape(5.dp))
            .then(
                if (selected) Modifier.border(2.dp, MaterialTheme.colorScheme.primary, RoundedCornerShape(5.dp))
                else Modifier
            )
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
    )
}

/** Les bornes du zoom, celles de la version JavaFX. */
private const val MIN_SCALE = 0.05f
private const val MAX_SCALE = 16f
private const val ZOOM_STEP = 1.25f

/** Un repere plus petit que cela ne se verrait pas, meme a rayon nul. */
private const val MIN_MARKER_PX = 5f
