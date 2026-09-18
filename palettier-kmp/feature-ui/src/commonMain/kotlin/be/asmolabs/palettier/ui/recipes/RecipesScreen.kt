package be.asmolabs.palettier.ui.recipes

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
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
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.recipe.RecipeTimelineService
import be.asmolabs.palettier.ui.component.Chip
import be.asmolabs.palettier.ui.component.WorkshopForm
import be.asmolabs.palettier.ui.component.formatDuration

/** Le deroule reel d'une recette : a l'huile, l'essentiel du planning est de l'attente. */
@Composable
fun RecipesScreen(state: RecipesUiState, onIntent: (RecipesIntent) -> Unit, modifier: Modifier = Modifier) {
    Surface(modifier.fillMaxSize()) {
        Row(Modifier.fillMaxSize()) {
            Surface(Modifier.width(240.dp).fillMaxHeight(), color = MaterialTheme.colorScheme.surfaceVariant) {
                LazyColumn(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    items(state.recipes, key = { it.id ?: it.name.hashCode().toLong() }) { recipe ->
                        Surface(
                            Modifier.fillMaxWidth().clickable { onIntent(RecipesIntent.Select(recipe)) },
                            color = if (recipe.id == state.selected?.id)
                                MaterialTheme.colorScheme.primary.copy(alpha = 0.18f) else Color.Transparent,
                            shape = RoundedCornerShape(6.dp),
                        ) {
                            Column(Modifier.padding(horizontal = 10.dp, vertical = 8.dp)) {
                                Text(recipe.name, style = MaterialTheme.typography.bodyMedium)
                                Text("${recipe.steps.size} etapes", style = MaterialTheme.typography.bodySmall)
                            }
                        }
                    }
                }
            }

            Column(
                Modifier.weight(1f).fillMaxHeight().padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                state.timeline?.let { Summary(it, state) }

                Row(verticalAlignment = Alignment.Top) {
                    Text("Pigment retenu", Modifier.width(130.dp).padding(top = 6.dp),
                        style = MaterialTheme.typography.bodySmall)
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        DryingClass.entries.forEach { drying ->
                            Chip(drying.label, drying == state.dryingClass) {
                                onIntent(RecipesIntent.SetDryingClass(drying))
                            }
                        }
                    }
                }

                WorkshopForm(state.workshop, onChange = { onIntent(RecipesIntent.SetWorkshop(it)) })

                state.timeline?.let { timeline ->
                    LazyColumn(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        items(timeline.entries) { entry ->
                            HorizontalDivider()
                            Step(entry)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun Summary(timeline: RecipeTimelineService.Timeline, state: RecipesUiState) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(timeline.recipe.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            Text(
                "${timeline.entries.size} etapes  -  ${formatDuration(timeline.totalActiveSpan)} d'attente cumulee",
                style = MaterialTheme.typography.bodyMedium,
            )
            Text(
                "Vernis final envisageable apres ${formatDuration(timeline.untilVarnish)}.",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

@Composable
private fun Step(entry: RecipeTimelineService.TimelineEntry) {
    Column(Modifier.padding(vertical = 6.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("${entry.position}.", Modifier.width(28.dp), style = MaterialTheme.typography.bodyMedium)
            Column(Modifier.weight(1f)) {
                Text(entry.step.technique.label, style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.Medium)
                Text(
                    listOf(
                        entry.step.paintMix.takeIf { it.isNotBlank() },
                        "${entry.step.medium.label} a %.0f %%".format(entry.step.mediumRatio * 100),
                        entry.step.thickness.label,
                    ).filterNotNull().joinToString("  -  "),
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            Column(horizontalAlignment = Alignment.End) {
                Text(
                    if (entry.startOffset == kotlin.time.Duration.ZERO) "immediat"
                    else "T + ${formatDuration(entry.startOffset)}",
                    style = MaterialTheme.typography.bodySmall,
                )
                if (entry.waitAfter != kotlin.time.Duration.ZERO) {
                    Text("puis ${formatDuration(entry.waitAfter)} d'attente",
                        style = MaterialTheme.typography.bodySmall)
                }
            }
        }
    }
}
