package be.asmolabs.palettier.ui.projects

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.project.Project

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
                Detail(state, onIntent, Modifier.weight(1f).fillMaxHeight())
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
private fun Detail(state: ProjectsUiState, onIntent: (ProjectsIntent) -> Unit, modifier: Modifier) {
    val project = state.selected ?: return

    LazyColumn(modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        item {
            Column {
                Text(project.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Text(project.subject, style = MaterialTheme.typography.bodySmall)
                if (project.approach.isNotBlank()) {
                    Text(project.approach, style = MaterialTheme.typography.bodySmall, modifier = Modifier.padding(top = 6.dp))
                }
            }
        }

        if (state.risks.isNotEmpty()) item { CrackingCard(state) }
        if (state.missing.isNotEmpty()) item { ShelfCard(state) }
        if (state.computing) item { Row { CircularProgressIndicator(Modifier.size(18.dp)) } }

        items(state.zones) { zone -> ZoneCard(zone, onIntent) }
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

        Checkbox(
            checked = layer.isApplied,
            onCheckedChange = { onIntent(ProjectsIntent.ToggleApplied(layer.zoneIndex, layer.layerIndex)) },
        )
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
