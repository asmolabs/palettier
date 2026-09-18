package be.asmolabs.palettier.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.color.Rgb

/**
 * Le catalogue : ce qui existe, et ce qui approche une teinte.
 *
 * <p>Sept cents lignes dans une LazyColumn, qui n'en compose que ce qui est visible. La
 * version JavaFX employait une TableView pour la meme raison.</p>
 */
@Composable
fun CatalogScreen(
    state: CatalogUiState,
    onIntent: (CatalogIntent) -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            Filters(state, onIntent)

            when {
                state.loading -> Box(Modifier.fillMaxSize(), Alignment.Center) { CircularProgressIndicator() }

                state.rows.isEmpty() -> Box(Modifier.fillMaxSize().padding(24.dp), Alignment.Center) {
                    Text("Aucun tube ne correspond.", style = MaterialTheme.typography.bodyLarge)
                }

                else -> LazyColumn(Modifier.fillMaxSize().padding(horizontal = 20.dp)) {
                    items(state.rows, key = { it.paint.id ?: it.paint.displayName.hashCode().toLong() }) { row ->
                        PaintLine(row, onIntent)
                    }
                }
            }
        }
    }
}

@Composable
private fun Filters(state: CatalogUiState, onIntent: (CatalogIntent) -> Unit) {
    Surface(color = MaterialTheme.colorScheme.surfaceVariant) {
        Column(Modifier.fillMaxWidth().padding(20.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = state.query,
                    onValueChange = { onIntent(CatalogIntent.Search(it)) },
                    label = { Text("Nom, marque, reference ou pigment") },
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                )
                OutlinedTextField(
                    value = state.targetHex.orEmpty(),
                    onValueChange = { onIntent(CatalogIntent.AimAt(it)) },
                    label = { Text("Teinte visee") },
                    placeholder = { Text("#C98F72") },
                    singleLine = true,
                    modifier = Modifier.width(180.dp),
                )
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(state.onlyInStock, { onIntent(CatalogIntent.OnlyInStock(it)) })
                Text("Seulement ce que je possede", style = MaterialTheme.typography.bodySmall)
                Spacer(Modifier.width(16.dp))
                Text(
                    if (state.sortedByDistance) "${state.rows.size} tubes, classes par ecart a la teinte visee"
                    else "${state.rows.size} tubes sur ${state.total}",
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}

@Composable
private fun PaintLine(row: CatalogRow, onIntent: (CatalogIntent) -> Unit) {
    val paint = row.paint
    Row(Modifier.fillMaxWidth().padding(vertical = 6.dp), verticalAlignment = Alignment.CenterVertically) {
        Swatch(paint.hexColor)
        Spacer(Modifier.width(12.dp))

        Column(Modifier.weight(1f)) {
            Text(paint.displayName, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
            Text(details(row), style = MaterialTheme.typography.bodySmall)
        }

        Checkbox(paint.inStock, { onIntent(CatalogIntent.SetOwned(paint, it)) })
    }
}

private fun details(row: CatalogRow): String {
    val paint = row.paint
    val reference = when {
        paint.code.isBlank() -> ""
        paint.legacyCode.isBlank() -> paint.code
        // Un tube renumerote garde son ancienne etiquette visible : le peintre a souvent
        // les deux sur son etagere.
        else -> "${paint.code} (${paint.legacyCode})"
    }
    val pigments = paint.pigments.joinToString(", ")
    val gap = row.deltaE?.let { "ecart %.1f".format(it) }

    return listOf(reference, pigments, paint.opacity.label, paint.dryingClass.label, gap)
        .filter { !it.isNullOrBlank() }
        .joinToString("  -  ")
}

@Composable
private fun Swatch(hex: String) {
    val rgb = Rgb.ofHex(hex)
    Box(
        Modifier.size(30.dp).background(
            Color(rgb.r.toFloat(), rgb.g.toFloat(), rgb.b.toFloat()),
            RoundedCornerShape(5.dp),
        )
    )
}
