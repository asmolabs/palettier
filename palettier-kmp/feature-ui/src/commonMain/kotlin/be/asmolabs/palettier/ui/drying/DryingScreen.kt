package be.asmolabs.palettier.ui.drying

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.drying.DryingEstimate
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.ui.component.Chip
import be.asmolabs.palettier.ui.component.WorkshopForm
import be.asmolabs.palettier.ui.component.formatDuration

/** Combien de temps pour une couche a venir : les cinq jalons du sechage. */
@Composable
fun DryingScreen(state: DryingUiState, onIntent: (DryingIntent) -> Unit, modifier: Modifier = Modifier) {
    Surface(modifier.fillMaxSize()) {
        Column(
            Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            state.estimate?.let { Milestones(it) }

            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Choices("Technique", Technique.entries.map { it.label }, state.technique.label) { label ->
                        onIntent(DryingIntent.SetTechnique(Technique.byLabel(label, Technique.GLAZE)))
                    }
                    Choices("Pigment", DryingClass.entries.map { it.label }, state.dryingClass.label) { label ->
                        onIntent(DryingIntent.SetDryingClass(DryingClass.entries.first { it.label == label }))
                    }
                    Choices("Medium", Medium.entries.map { it.label }, state.medium.label) { label ->
                        onIntent(DryingIntent.SetMedium(Medium.entries.first { it.label == label }))
                    }
                    Choices("Epaisseur", LayerThickness.entries.map { it.label }, state.thickness.label) { label ->
                        onIntent(DryingIntent.SetThickness(LayerThickness.entries.first { it.label == label }))
                    }

                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text("Dilution", Modifier.width(130.dp), style = MaterialTheme.typography.bodySmall)
                        Slider(
                            value = state.mediumRatio.toFloat(),
                            onValueChange = { onIntent(DryingIntent.SetRatio(it.toDouble())) },
                            valueRange = 0f..1f,
                            modifier = Modifier.width(220.dp),
                        )
                        Text("%.0f %%".format(state.mediumRatio * 100), style = MaterialTheme.typography.bodySmall)
                    }
                }
            }

            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    WorkshopForm(state.workshop, onChange = { onIntent(DryingIntent.SetWorkshop(it)) })
                }
            }

            state.estimate?.advice?.forEach {
                Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
            }
        }
    }
}

@Composable
private fun Milestones(estimate: DryingEstimate) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            listOf(
                "Temps ouvert" to estimate.openTime,
                "Sec au toucher" to estimate.touchDry,
                "Recouvrable" to estimate.recoat,
                "Sec a coeur" to estimate.throughDry,
                "Polymerise" to estimate.fullCure,
            ).forEach { (label, duration) ->
                Row {
                    Text(label, Modifier.width(160.dp), style = MaterialTheme.typography.bodyMedium)
                    Text(
                        formatDuration(duration),
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }
        }
    }
}

@Composable
private fun Choices(label: String, options: List<String>, selected: String, onSelect: (String) -> Unit) {
    Row(verticalAlignment = Alignment.Top) {
        Text(label, Modifier.width(130.dp).padding(top = 6.dp), style = MaterialTheme.typography.bodySmall)
        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            options.forEach { Chip(it, it == selected) { onSelect(it) } }
        }
    }
}
