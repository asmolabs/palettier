package be.asmolabs.palettier.ui.projects

import androidx.compose.foundation.background
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.graphics.decodeToImageBitmap
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectPhoto

/**
 * Les pieces en cours et leur plan.
 *
 * <p>Cocher une couche est le geste central de cet ecran : c'est lui qui fait courir le
 * sechage, et sans lui l'etabli n'a jamais rien de nouveau a raconter.</p>
 */
@Composable
fun ProjectsScreen(
    state: ProjectsUiState,
    onIntent: (ProjectsIntent) -> Unit,
    onChoosePhoto: (ProjectPhoto.Role) -> Unit,
    onCompare: () -> Unit,
    onExport: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(modifier.fillMaxSize()) {
        when {
            state.loading -> Box(Modifier.fillMaxSize(), Alignment.Center) { CircularProgressIndicator() }

            state.projects.isEmpty() -> Box(Modifier.fillMaxSize().padding(24.dp), Alignment.Center) {
                Text(
                    "Aucun projet. Importez une sauvegarde pour retrouver vos pieces.",
                    style = MaterialTheme.typography.bodyLarge,
                )
            }

            else -> Row(Modifier.fillMaxSize()) {
                ProjectList(state, onIntent, Modifier.width(260.dp).fillMaxHeight())
                Detail(state, onIntent, onChoosePhoto, onCompare, onExport, Modifier.weight(1f).fillMaxHeight())
            }
        }
    }
}

@Composable
private fun ProjectList(state: ProjectsUiState, onIntent: (ProjectsIntent) -> Unit, modifier: Modifier) {
    Surface(modifier, color = MaterialTheme.colorScheme.surfaceVariant) {
        LazyColumn(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            items(state.projects, key = { it.id ?: it.name.hashCode().toLong() }) { project ->
                ProjectRow(project, project.id == state.selected?.id) { onIntent(ProjectsIntent.Select(project)) }
            }
        }
    }
}

@Composable
private fun ProjectRow(project: Project, selected: Boolean, onClick: () -> Unit) {
    val coats = project.zones.sumOf { it.layers.size }
    val applied = project.zones.sumOf { zone -> zone.layers.count { it.isApplied } }

    Surface(
        Modifier.fillMaxWidth().clickable(onClick = onClick),
        color = if (selected) MaterialTheme.colorScheme.primary.copy(alpha = 0.18f) else Color.Transparent,
        shape = RoundedCornerShape(6.dp),
    ) {
        Column(Modifier.padding(horizontal = 10.dp, vertical = 8.dp)) {
            Text(project.name, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
            Text(
                "${project.zones.size} zones  -  $applied couches posees sur $coats",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

@Composable
private fun Detail(
    state: ProjectsUiState,
    onIntent: (ProjectsIntent) -> Unit,
    onChoosePhoto: (ProjectPhoto.Role) -> Unit,
    onCompare: () -> Unit,
    onExport: () -> Unit,
    modifier: Modifier,
) {
    val project = state.selected ?: return

    LazyColumn(modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        item { Header(state, onIntent, onExport) }
        item { PhotosCard(state, onIntent, onChoosePhoto, onCompare) }
        if (state.comparison.isNotEmpty() || state.comparing) item { ComparisonCard(state, onIntent) }

        if (state.risks.isNotEmpty()) item { CrackingCard(state) }
        if (state.missing.isNotEmpty()) item { ShelfCard(state) }
        if (state.computing) item { Row { CircularProgressIndicator(Modifier.size(18.dp)) } }

        items(state.zones) { zone -> ZoneCard(zone, onIntent) }
    }
}

/** L'en-tete : ce que la piece est, et les trois gestes qui la tiennent a jour. */
@Composable
private fun Header(state: ProjectsUiState, onIntent: (ProjectsIntent) -> Unit, onExport: () -> Unit) {
    val project = state.selected ?: return
    var renaming by remember(project.id) { mutableStateOf(false) }
    var name by remember(project.id) { mutableStateOf(project.name) }
    var confirming by remember(project.id) { mutableStateOf(false) }

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (renaming) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Nom du projet") },
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                )
                TextButton({
                    onIntent(ProjectsIntent.Rename(name))
                    renaming = false
                }) { Text("Valider") }
                TextButton({
                    name = project.name
                    renaming = false
                }) { Text("Annuler") }
            }
        } else {
            Text(project.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
        }

        Text(project.subject, style = MaterialTheme.typography.bodySmall)
        if (project.approach.isNotBlank()) {
            Text(project.approach, style = MaterialTheme.typography.bodySmall)
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (!renaming) TextButton({ renaming = true }) { Text("Renommer") }

            // Le plan sert a l'etabli, ou l'on ne consulte pas un ecran les mains grasses
            // de diluant. Propose seulement une fois le plan calcule : sans lui, il n'y a
            // rien a mettre sur papier.
            if (state.plan != null) TextButton(onExport) { Text("Exporter en PDF") }

            // La suppression demande confirmation : elle emporte les couches posees, et
            // rien dans l'application ne les rendrait.
            if (confirming) {
                TextButton({
                    onIntent(ProjectsIntent.Delete)
                    confirming = false
                }) { Text("Confirmer la suppression") }
                TextButton({ confirming = false }) { Text("Annuler") }
            } else {
                TextButton({ confirming = true }) { Text("Supprimer") }
            }
        }

        // Proposee seulement quand elle a lieu d'etre : une palette remaniee depuis que le
        // plan a ete etabli.
        if (state.paletteDiverges) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    "La palette a change depuis ce plan.",
                    style = MaterialTheme.typography.bodySmall,
                )
                TextButton({ onIntent(ProjectsIntent.SyncPalette) }) { Text("Reprendre la palette actuelle") }
            }
        }

        state.message?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
    }
}

/**
 * Les photos de la piece.
 *
 * <p>Trois roles, parce qu'ils ne servent pas a la meme chose : la piece telle qu'elle
 * est, la reference qu'on vise, et l'avancement qu'on voudra comparer plus tard.</p>
 */
@Composable
private fun PhotosCard(
    state: ProjectsUiState,
    onIntent: (ProjectsIntent) -> Unit,
    onChoosePhoto: (ProjectPhoto.Role) -> Unit,
    onCompare: () -> Unit,
) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Photos", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                Spacer(Modifier.weight(1f))
                ProjectPhoto.Role.entries.forEach { role ->
                    TextButton({ onChoosePhoto(role) }) { Text(role.label) }
                }
                if (state.busy) CircularProgressIndicator(Modifier.size(16.dp))
            }

            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TextButton(onCompare, enabled = !state.comparing) { Text("Comparer une photo au plan") }
                if (state.comparing) CircularProgressIndicator(Modifier.size(16.dp))
            }

            if (state.photos.isEmpty()) {
                Text(
                    "Aucune photo. Celles de l'assistant sont conservees a l'enregistrement.",
                    style = MaterialTheme.typography.bodySmall,
                )
            } else {
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    state.photos.forEach { photo ->
                        PhotoTile(photo) { photo.id?.let { onIntent(ProjectsIntent.RemovePhoto(it)) } }
                    }
                }
            }
        }
    }
}

@Composable
private fun PhotoTile(photo: ProjectPhoto, onRemove: () -> Unit) {
    val bitmap = remember(photo.id) { runCatching { photo.data.decodeToImageBitmap() }.getOrNull() }

    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            Modifier.size(96.dp).background(MaterialTheme.colorScheme.surfaceVariant, RoundedCornerShape(6.dp)),
            contentAlignment = Alignment.Center,
        ) {
            if (bitmap == null) {
                Text("?", style = MaterialTheme.typography.titleMedium)
            } else {
                Image(
                    bitmap = bitmap,
                    contentDescription = photo.caption.ifBlank { photo.role.label },
                    modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(6.dp)),
                    contentScale = ContentScale.Crop,
                )
            }
        }
        Text(photo.role.label, style = MaterialTheme.typography.labelSmall)
        TextButton(onRemove) { Text("\u2715", style = MaterialTheme.typography.labelSmall) }
    }
}

/**
 * Ce qu'il y a sur la piece, confronte a ce qui etait vise.
 *
 * <p>La lecture va de la piece vers le plan. Partir des couleurs visees pour chercher la
 * plus proche sur la photo trouverait toujours quelque chose -- une photo contient des
 * milliers de teintes. Partir de ce qui occupe reellement la piece se trompe moins, et
 * signale au passage ce qui n'etait prevu nulle part.</p>
 */
@Composable
private fun ComparisonCard(state: ProjectsUiState, onIntent: (ProjectsIntent) -> Unit) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Ce que montre la photo", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                Spacer(Modifier.weight(1f))
                TextButton({ onIntent(ProjectsIntent.ForgetComparison) }) { Text("Fermer") }
            }

            state.comparison.forEach { seen ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(seen.measured.toHex())
                    seen.target?.let { Spacer(Modifier.width(4.dp)); Swatch(it.toHex()) }
                    Spacer(Modifier.width(10.dp))
                    Column(Modifier.weight(1f)) {
                        Text(seen.verdict(), style = MaterialTheme.typography.bodySmall)
                        Text(
                            if (seen.matchesPlan) "%.0f %% de l'image  -  ecart %.1f".format(seen.share * 100, seen.deltaE)
                            else "%.0f %% de l'image".format(seen.share * 100),
                            style = MaterialTheme.typography.labelSmall,
                        )
                    }
                }
            }

            Text(
                "La comparaison lit votre fichier d'origine, pas la photo rangee ici : celle-ci " +
                    "est reduite et reencodee, ses couleurs ont bouge. Meme ainsi, une photo est " +
                    "prise sous une lumiere quelconque -- l'ecart se lit comme une tendance, pas " +
                    "comme un verdict.",
                style = MaterialTheme.typography.labelSmall,
            )
        }
    }
}

/** L'avertissement qui ne se rattrape pas : une couche maigre sur une grasse craquelle. */
@Composable
private fun CrackingCard(state: ProjectsUiState) {
    Card(Modifier.fillMaxWidth(), colors = CardDefaults.cardColors(MaterialTheme.colorScheme.errorContainer)) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Gras sur maigre", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            state.risks.forEach { risk ->
                Column {
                    Text("${risk.where} : ${risk.under} sous ${risk.over}", style = MaterialTheme.typography.bodyMedium)
                    Text(risk.explanation, style = MaterialTheme.typography.bodySmall)
                }
            }
        }
    }
}

/** Ce qui manque, et avec quoi s'en sortir ce soir. */
@Composable
private fun ShelfCard(state: ProjectsUiState) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Pas sur l'etagere", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            state.missing.forEach { gap ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(gap.paint.hexColor)
                    gap.nearest?.let { Spacer(Modifier.width(4.dp)); Swatch(it.hexColor) }
                    Spacer(Modifier.width(10.dp))
                    Column {
                        Text(gap.paint.displayName, style = MaterialTheme.typography.bodyMedium)
                        Text(gap.verdict(), style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        }
    }
}

@Composable
private fun ZoneCard(zone: ZoneRow, onIntent: (ProjectsIntent) -> Unit) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(
                if (zone.material.isBlank()) zone.name else "${zone.name} - ${zone.material}",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
            )
            zone.layers.forEachIndexed { index, layer ->
                if (index > 0) HorizontalDivider()
                LayerLine(layer, onIntent)
            }
        }
    }
}

@Composable
private fun LayerLine(layer: LayerRow, onIntent: (ProjectsIntent) -> Unit) {
    var editing by remember(layer.zoneIndex, layer.layerIndex) { mutableStateOf(false) }

    if (editing) {
        LayerEditor(layer, onIntent) { editing = false }
        return
    }

    Row(verticalAlignment = Alignment.CenterVertically) {
        Swatch(layer.targetHex)
        layer.mix?.let { Spacer(Modifier.width(4.dp)); Swatch(it.achieved.toHex()) }
        Spacer(Modifier.width(10.dp))

        Column(Modifier.weight(1f)) {
            Text(layer.role, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
            Text(
                layer.mix?.let { "${it.recipe?.describe() ?: "palette vide"}  -  ecart %.1f, ${it.reachability()}".format(it.deltaE) }
                    ?: "dosage en cours de calcul",
                style = MaterialTheme.typography.bodySmall,
            )
        }

        TextButton({ editing = true }) { Text("Modifier") }

        Checkbox(
            checked = layer.isApplied,
            onCheckedChange = { onIntent(ProjectsIntent.ToggleApplied(layer.zoneIndex, layer.layerIndex)) },
        )
    }
}

/**
 * Correction d'une couche : la couleur visee, la technique, la note.
 *
 * <p>Seule la decision se modifie. Le dosage n'apparait pas dans ce formulaire parce
 * qu'il ne se decide pas : il se calcule a partir de la couleur retenue et des tubes du
 * projet. Une couche deja posee garde sa date et son atelier -- corriger la couleur pour
 * la suite ne doit pas reecrire ce qui a deja seche.</p>
 */
@Composable
private fun LayerEditor(layer: LayerRow, onIntent: (ProjectsIntent) -> Unit, onDone: () -> Unit) {
    var hex by remember(layer) { mutableStateOf(layer.targetHex) }
    var technique by remember(layer) { mutableStateOf(layer.technique) }
    var note by remember(layer) { mutableStateOf(layer.note) }

    val valid = runCatching { Rgb.ofHex(if (hex.startsWith("#")) hex else "#$hex") }.isSuccess

    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(layer.role, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (valid) Swatch(if (hex.startsWith("#")) hex else "#$hex")
            OutlinedTextField(
                value = hex,
                onValueChange = { hex = it },
                label = { Text("Couleur visee") },
                isError = !valid,
                singleLine = true,
                modifier = Modifier.width(150.dp),
            )
            OutlinedTextField(
                value = technique,
                onValueChange = { technique = it },
                label = { Text("Technique") },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
        }

        OutlinedTextField(
            value = note,
            onValueChange = { note = it },
            label = { Text("Note") },
            modifier = Modifier.fillMaxWidth(),
        )

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            TextButton(
                {
                    onIntent(
                        ProjectsIntent.EditLayer(
                            layer.zoneIndex,
                            layer.layerIndex,
                            if (hex.startsWith("#")) hex else "#$hex",
                            technique,
                            note,
                        )
                    )
                    onDone()
                },
                enabled = valid,
            ) { Text("Valider") }
            TextButton(onDone) { Text("Annuler") }
        }
    }
}

@Composable
private fun Swatch(hex: String) {
    val rgb = Rgb.ofHex(hex)
    Box(
        Modifier.size(28.dp).background(
            Color(rgb.r.toFloat(), rgb.g.toFloat(), rgb.b.toFloat()),
            RoundedCornerShape(5.dp),
        )
    )
}
