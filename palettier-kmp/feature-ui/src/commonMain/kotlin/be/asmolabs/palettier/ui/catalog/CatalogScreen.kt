package be.asmolabs.palettier.ui.catalog

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.getValue
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

                else -> Row(Modifier.fillMaxSize()) {
                    LazyColumn(Modifier.weight(1f).fillMaxHeight().padding(horizontal = 20.dp)) {
                        items(state.rows, key = { it.paint.id ?: it.paint.displayName.hashCode().toLong() }) { row ->
                            PaintLine(row, row.paint.id == state.selected?.id, onIntent)
                        }
                    }
                    CalibrationCard(state, onIntent)
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
                    trailingIcon = {
                        // Le pont avec la pipette : elle mesure, le catalogue attribue.
                        state.sampledHex?.let { measured ->
                            TextButton({ onIntent(CatalogIntent.AimAt(measured)) }) {
                                Text("relevee")
                            }
                        }
                    },
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
private fun PaintLine(row: CatalogRow, selected: Boolean, onIntent: (CatalogIntent) -> Unit) {
    val paint = row.paint
    Row(
        Modifier
            .fillMaxWidth()
            .background(
                if (selected) MaterialTheme.colorScheme.primary.copy(alpha = 0.16f) else Color.Transparent,
                RoundedCornerShape(6.dp),
            )
            .clickable { onIntent(CatalogIntent.Select(paint)) }
            .padding(vertical = 6.dp, horizontal = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Swatch(paint.hexColor)
        Spacer(Modifier.width(12.dp))

        Column(Modifier.weight(1f)) {
            Text(paint.displayName, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
            Text(details(row), style = MaterialTheme.typography.bodySmall)
        }

        Checkbox(paint.inStock, { onIntent(CatalogIntent.SetOwned(paint, it)) })
    }
}

/**
 * Calibrer le tube choisi d'apres ses propres ecouvillons.
 *
 * <p>Le partage est deliberé : la pipette mesure, le catalogue attribue. C'est dans le
 * tableau qu'on voit ce qui manque, et c'est la qu'on dispose deja de la recherche, des
 * filtres et de la colonne coupee de blanc.</p>
 */
@Composable
private fun CalibrationCard(state: CatalogUiState, onIntent: (CatalogIntent) -> Unit) {
    val paint = state.selected
    var measured by remember(paint?.id) { mutableStateOf(paint?.hexColor.orEmpty()) }
    val valid = runCatching { Rgb.ofHex(measured) }.isSuccess

    Surface(
        Modifier.width(320.dp).fillMaxHeight().verticalScroll(rememberScrollState()),
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text("Calibrer le tube choisi", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)

            if (paint == null) {
                Text(
                    "Choisissez une ligne du tableau pour la calibrer.",
                    style = MaterialTheme.typography.bodySmall,
                )
                return@Column
            }

            Text(paint.displayName, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)

            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Actuel", style = MaterialTheme.typography.bodySmall)
                Swatch(paint.hexColor)
                Text("coupe de blanc", style = MaterialTheme.typography.bodySmall)
                if (paint.tintHex == null) {
                    Text("?", style = MaterialTheme.typography.bodyMedium)
                } else {
                    Swatch(paint.tintHex!!)
                }
            }

            if (paint.tintHex == null) {
                Text(
                    "Teinte diluee inconnue : ce tube se melange encore avec le modele a une " +
                        "seule constante.",
                    style = MaterialTheme.typography.bodySmall,
                )
            }

            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (valid) Swatch(measured)
                OutlinedTextField(
                    value = measured,
                    onValueChange = { measured = it },
                    label = { Text("Couleur mesuree") },
                    isError = !valid,
                    singleLine = true,
                    modifier = Modifier.weight(1f),
                )
            }

            // Le pont avec la pipette, dans l'autre sens que la teinte visee : ici la
            // mesure ne sert pas a chercher, elle s'attribue.
            state.sampledHex?.let { relieved ->
                TextButton({ measured = relieved }) { Text("Utiliser la teinte relevee $relieved") }
            }

            Text("Enregistrer comme", style = MaterialTheme.typography.bodySmall)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                TextButton(
                    { onIntent(CatalogIntent.Calibrate(measured, asTint = false)) },
                    enabled = valid,
                ) { Text("Ton de masse") }
                TextButton(
                    { onIntent(CatalogIntent.Calibrate(measured, asTint = true)) },
                    enabled = valid,
                ) { Text("Teinte diluee 1:9") }
            }

            state.calibrationMessage?.let { Text(it, style = MaterialTheme.typography.bodySmall) }

            Text(
                "Peignez un ecouvillon de couleur pure et un autre d'une part de couleur pour " +
                    "neuf de blanc de titane, photographiez-les sous lumiere neutre, relevez-les " +
                    "dans la Pipette puis revenez ici les attribuer. C'est la seconde mesure qui " +
                    "fait passer le tube au modele a deux constantes, et elle vaut mieux que " +
                    "toute valeur de catalogue : elle vient de votre peinture, sous votre lumiere.",
                style = MaterialTheme.typography.labelSmall,
            )
        }
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
