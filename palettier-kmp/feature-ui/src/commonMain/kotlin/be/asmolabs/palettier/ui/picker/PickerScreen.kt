package be.asmolabs.palettier.ui.picker

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.color.Rgb

/**
 * La pipette : relever une teinte sur une photo, et voir avec quoi la faire.
 *
 * <p>Deux precautions y sont visibles. La mesure porte sur le fichier d'origine, jamais
 * sur une image reduite -- reduire reencode, et reencoder deplace les couleurs. Et une
 * photo prise sous lampe chaude tire au jaune : designer un point cense etre gris annule
 * cette dominante.</p>
 */
@Composable
fun PickerScreen(
    state: PickerUiState,
    onIntent: (PickerIntent) -> Unit,
    onChooseImage: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                TextButton(onChooseImage) { Text("Choisir une photo") }
                if (state.loading) CircularProgressIndicator(Modifier.size(18.dp))
                state.error?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
            }

            Text(
                "La mesure se fait sur votre fichier d'origine. Une photo reste prise sous une " +
                    "lumiere quelconque : designez un point cense etre gris pour annuler la dominante.",
                style = MaterialTheme.typography.bodySmall,
            )

            if (state.dominant.isNotEmpty()) {
                Swatches(state, onIntent)
                HorizontalDivider()
                Working(state, onIntent)
                Closest(state, onIntent)
            }
        }
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

/** Les tubes les plus proches de la teinte retenue. */
@Composable
private fun Closest(state: PickerUiState, onIntent: (PickerIntent) -> Unit) {
    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Ce qui s'en approche", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            Spacer(Modifier.width(16.dp))
            Checkbox(state.onlyInStock, { onIntent(PickerIntent.OnlyInStock(it)) })
            Text("seulement ce que je possede", style = MaterialTheme.typography.bodySmall)
        }

        LazyColumn(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            items(state.closest, key = { it.paint.id ?: it.paint.displayName.hashCode().toLong() }) { match ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Swatch(match.paint.hexColor, size = 26)
                    Spacer(Modifier.width(10.dp))
                    Column {
                        Text(match.paint.displayName, style = MaterialTheme.typography.bodyMedium)
                        Text(
                            "ecart %.1f  -  %s".format(match.deltaE, verdict(match.deltaE)),
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                }
            }
        }
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
