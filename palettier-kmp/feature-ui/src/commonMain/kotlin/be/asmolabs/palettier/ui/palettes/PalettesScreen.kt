package be.asmolabs.palettier.ui.palettes

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.ui.component.Swatch

/** Les palettes : six a douze tubes choisis ensemble pour un sujet. */
@Composable
fun PalettesScreen(state: PalettesUiState, onIntent: (PalettesIntent) -> Unit, modifier: Modifier = Modifier) {
    Surface(modifier.fillMaxSize()) {
        Row(Modifier.fillMaxSize()) {
            Sidebar(state, onIntent, Modifier.width(240.dp).fillMaxHeight())
            Detail(state, onIntent, Modifier.weight(1f).fillMaxHeight())
        }
    }
}

@Composable
private fun Sidebar(state: PalettesUiState, onIntent: (PalettesIntent) -> Unit, modifier: Modifier) {
    var name by remember { mutableStateOf("") }

    Surface(modifier, color = MaterialTheme.colorScheme.surfaceVariant) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            LazyColumn(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                items(state.palettes, key = { it.id ?: it.name.hashCode().toLong() }) { palette ->
                    Surface(
                        Modifier.fillMaxWidth().clickable { onIntent(PalettesIntent.Select(palette)) },
                        color = if (palette.id == state.selected?.id)
                            MaterialTheme.colorScheme.primary.copy(alpha = 0.18f) else Color.Transparent,
                        shape = RoundedCornerShape(6.dp),
                    ) {
                        Column(Modifier.padding(horizontal = 10.dp, vertical = 8.dp)) {
                            Text(palette.name, style = MaterialTheme.typography.bodyMedium)
                            Text("${palette.paints.size} tubes", style = MaterialTheme.typography.bodySmall)
                        }
                    }
                }
            }

            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("Nouvelle palette") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            TextButton(
                onClick = { onIntent(PalettesIntent.Create(name)); name = "" },
                enabled = name.isNotBlank(),
            ) { Text("Creer") }
        }
    }
}

@Composable
private fun Detail(state: PalettesUiState, onIntent: (PalettesIntent) -> Unit, modifier: Modifier) {
    val palette = state.selected ?: return

    Column(modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text(palette.name, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Text(summary(state), style = MaterialTheme.typography.bodySmall)
            }
            TextButton({ onIntent(PalettesIntent.Delete(palette)) }) { Text("Supprimer") }
        }

        if (state.tooManyPigments) {
            Card(Modifier.fillMaxWidth()) {
                Text(
                    "${state.pigments.size} pigments differents : au-dela de six, les melanges " +
                        "tendent vers le gris quoi qu'on fasse.",
                    Modifier.padding(12.dp),
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }

        Text("Tubes de la palette", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
        LazyColumn(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            items(palette.paints, key = { it.id ?: it.displayName.hashCode().toLong() }) { paint ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(paint.hexColor, size = 24)
                    Spacer(Modifier.width(10.dp))
                    Text(paint.displayName, Modifier.weight(1f), style = MaterialTheme.typography.bodyMedium)
                    Text(paint.dryingClass.label, style = MaterialTheme.typography.bodySmall)
                    TextButton({ onIntent(PalettesIntent.RemovePaint(paint)) }) { Text("Retirer") }
                }
            }
        }

        OutlinedTextField(
            value = state.query,
            onValueChange = { onIntent(PalettesIntent.Search(it)) },
            label = { Text("Ajouter un tube : nom, marque ou pigment") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        LazyColumn(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            items(state.candidates, key = { it.id ?: it.displayName.hashCode().toLong() }) { paint ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(paint.hexColor, size = 22, onClick = { onIntent(PalettesIntent.AddPaint(paint)) })
                    Spacer(Modifier.width(10.dp))
                    Text(paint.displayName, style = MaterialTheme.typography.bodySmall)
                }
            }
        }
    }
}

private fun summary(state: PalettesUiState): String {
    val palette = state.selected ?: return ""
    val drying = state.slowest?.let { "sechage ${it.label.lowercase()} impose par le plus lent" }
    return listOfNotNull(
        "${palette.paints.size} tubes",
        "${state.pigments.size} pigments",
        drying,
    ).joinToString("  -  ")
}
