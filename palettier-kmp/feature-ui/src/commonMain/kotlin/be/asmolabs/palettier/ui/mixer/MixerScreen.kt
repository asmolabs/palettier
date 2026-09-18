package be.asmolabs.palettier.ui.mixer

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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Card
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Slider
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.ui.component.Swatch

/**
 * Le melangeur, dans les deux sens : quelle couleur donne ce melange, et quel melange
 * donne cette couleur.
 */
@Composable
fun MixerScreen(state: MixerUiState, onIntent: (MixerIntent) -> Unit, modifier: Modifier = Modifier) {
    Surface(modifier.fillMaxSize()) {
        Row(Modifier.fillMaxSize()) {
            Available(state, onIntent, Modifier.width(280.dp).fillMaxHeight())
            Column(
                Modifier.weight(1f).fillMaxHeight().padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                Forward(state, onIntent)
                Reverse(state, onIntent)
            }
        }
    }
}

@Composable
private fun Available(state: MixerUiState, onIntent: (MixerIntent) -> Unit, modifier: Modifier) {
    Surface(modifier, color = MaterialTheme.colorScheme.surfaceVariant) {
        Column(Modifier.padding(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(state.onlyInStock, { onIntent(MixerIntent.OnlyInStock(it)) })
                Text("ce que je possede", style = MaterialTheme.typography.bodySmall)
            }
            LazyColumn(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                items(state.available, key = { it.id ?: it.displayName.hashCode().toLong() }) { paint ->
                    Row(
                        Modifier.fillMaxWidth().padding(vertical = 3.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Swatch(paint.hexColor, size = 22, onClick = { onIntent(MixerIntent.Add(paint)) })
                        Spacer(Modifier.width(8.dp))
                        Text(paint.displayName, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        }
    }
}

/** Sens direct : le melange qu'on compose, et ce qu'il donne. */
@Composable
private fun Forward(state: MixerUiState, onIntent: (MixerIntent) -> Unit) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Ce melange donne", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                Spacer(Modifier.width(12.dp))
                state.result?.let { Swatch(it.hex, size = 36); Spacer(Modifier.width(8.dp)); Text(it.hex) }
                Spacer(Modifier.weight(1f))
                if (state.doses.isNotEmpty()) TextButton({ onIntent(MixerIntent.Clear) }) { Text("Vider") }
            }

            if (state.doses.isEmpty()) {
                Text("Cliquez une pastille a gauche pour commencer.", style = MaterialTheme.typography.bodySmall)
            }

            state.doses.forEach { dose ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(dose.paint.hexColor, size = 22)
                    Spacer(Modifier.width(8.dp))
                    Text(dose.paint.name, Modifier.width(150.dp), style = MaterialTheme.typography.bodySmall)
                    Slider(
                        value = dose.parts.toFloat(),
                        onValueChange = { onIntent(MixerIntent.SetParts(dose.paint, it.toInt())) },
                        valueRange = 1f..12f,
                        steps = 10,
                        modifier = Modifier.width(180.dp),
                    )
                    Text("${dose.parts} parts", style = MaterialTheme.typography.bodySmall)
                    Spacer(Modifier.weight(1f))
                    TextButton({ onIntent(MixerIntent.Remove(dose.paint)) }) { Text("Retirer") }
                }
            }

            state.result?.let { result ->
                Text(
                    "Sechage ${result.dryingClass.label.lowercase()}  -  ${result.pigments.size} pigments",
                    style = MaterialTheme.typography.bodySmall,
                )
                result.warnings.forEach {
                    Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
                }
            }
        }
    }
}

/** Sens inverse : la teinte visee, et les recettes qui y menent. */
@Composable
private fun Reverse(state: MixerUiState, onIntent: (MixerIntent) -> Unit) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("Pour obtenir", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                OutlinedTextField(
                    value = state.targetHex,
                    onValueChange = { onIntent(MixerIntent.AimAt(it)) },
                    placeholder = { Text("#C98F72") },
                    singleLine = true,
                    modifier = Modifier.width(180.dp),
                )
                if (state.searching) CircularProgressIndicator(Modifier.size(18.dp))
            }

            state.suggestions.forEach { suggestion ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(suggestion.color.toHex(), size = 24)
                    Spacer(Modifier.width(10.dp))
                    Column {
                        Text(suggestion.describe(), style = MaterialTheme.typography.bodyMedium)
                        Text("ecart %.1f".format(suggestion.deltaE), style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        }
    }
}
