package be.asmolabs.palettier.ui.assistant

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.ai.AiProvider
import be.asmolabs.palettier.domain.plan.PaintingPlan
import be.asmolabs.palettier.ui.component.Chip
import be.asmolabs.palettier.ui.component.Swatch
import kotlin.math.roundToInt

/**
 * L'assistant : un sujet, une palette, et un plan par zones.
 *
 * <p>Le partage des roles se lit a l'ecran, et c'est voulu. Le modele a choisi une
 * couleur a viser ; le melange dessous et l'ecart affiche sont calcules par
 * l'application, avec les tubes du peintre. Un ecart important ne se cache pas : il dit
 * que la palette ne va pas jusque-la.</p>
 */
@Composable
fun AssistantScreen(
    state: AssistantUiState,
    onIntent: (AssistantIntent) -> Unit,
    onChooseFigurine: () -> Unit,
    onChooseReference: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize().padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item { EngineCard(state, onIntent) }

            if (state.settings.usable) {
                item { RequestCard(state, onIntent, onChooseFigurine, onChooseReference) }
            }

            state.error?.let { message ->
                item {
                    Card(Modifier.fillMaxWidth()) {
                        Text(message, Modifier.padding(16.dp), style = MaterialTheme.typography.bodySmall)
                    }
                }
            }

            state.plan?.let { plan ->
                item {
                    Card(Modifier.fillMaxWidth()) {
                        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text(
                                plan.subject.ifBlank { "D'apres les photos" },
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.Bold,
                            )
                            Text("Palette ${plan.paletteName}", style = MaterialTheme.typography.bodySmall)
                            if (plan.approach.isNotBlank()) {
                                Text(plan.approach, style = MaterialTheme.typography.bodySmall)
                            }
                        }
                    }
                }
                items(plan.zones) { zone -> ZoneCard(zone) }
            }
        }
    }
}

/** Le choix du moteur : sans lui, la page n'a rien d'autre a proposer. */
@Composable
private fun EngineCard(state: AssistantUiState, onIntent: (AssistantIntent) -> Unit) {
    val settings = state.settings
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text("Moteur", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            Text(
                "L'assistant est un supplement : le catalogue, les melanges, le sechage et les " +
                    "projets fonctionnent sans lui. Ollama tourne sur cette machine et rien n'en " +
                    "sort ; les deux autres envoient votre sujet et vos photos a un service distant.",
                style = MaterialTheme.typography.bodySmall,
            )

            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                AiProvider.entries.forEach { provider ->
                    Chip(provider.label, settings.provider == provider) {
                        onIntent(AssistantIntent.SaveSettings(settings.copy(provider = provider)))
                    }
                }
            }

            if (settings.provider != AiProvider.NONE) {
                OutlinedTextField(
                    value = settings.model,
                    onValueChange = { onIntent(AssistantIntent.SaveSettings(settings.copy(model = it))) },
                    label = { Text("Modele (vide pour celui par defaut)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = settings.baseUrl,
                    onValueChange = { onIntent(AssistantIntent.SaveSettings(settings.copy(baseUrl = it))) },
                    label = { Text("Adresse (vide pour celle par defaut)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            if (settings.provider.needsKey) {
                OutlinedTextField(
                    value = settings.apiKey,
                    onValueChange = { onIntent(AssistantIntent.SaveSettings(settings.copy(apiKey = it))) },
                    label = { Text("Cle d'acces") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                Text(
                    "La cle est rangee en clair dans la base de ce poste, et n'est pas reprise " +
                        "dans les sauvegardes.",
                    style = MaterialTheme.typography.bodySmall,
                )
            }
        }
    }
}

/** La demande : un sujet, une palette, des photos. */
@Composable
private fun RequestCard(
    state: AssistantUiState,
    onIntent: (AssistantIntent) -> Unit,
    onChooseFigurine: () -> Unit,
    onChooseReference: () -> Unit,
) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            OutlinedTextField(
                value = state.subject,
                onValueChange = { onIntent(AssistantIntent.SetSubject(it)) },
                label = { Text("Sujet : buste de grognard, manteau bleu et bonnet a poil") },
                modifier = Modifier.fillMaxWidth(),
            )

            Text("Palette", style = MaterialTheme.typography.bodySmall)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                state.palettes.forEach { palette ->
                    Chip(palette.name, palette.id == state.paletteId) {
                        palette.id?.let { onIntent(AssistantIntent.ChoosePalette(it)) }
                    }
                }
            }

            Text("Tubes par melange", style = MaterialTheme.typography.bodySmall)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                (1..5).forEach { count ->
                    Chip("$count", count == state.maxPaints) {
                        onIntent(AssistantIntent.SetMaxPaints(count))
                    }
                }
            }

            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                OutlinedButton(onChooseFigurine) { Text("Photo de la piece") }
                OutlinedButton(onChooseReference) { Text("Ajouter une reference") }
                if (state.figurine != null || state.references.isNotEmpty()) {
                    OutlinedButton({ onIntent(AssistantIntent.ClearPhotos) }) { Text("Retirer les photos") }
                }
            }
            val joined = listOfNotNull(state.figurine) + state.references
            if (joined.isNotEmpty()) {
                Text(
                    joined.joinToString(", ") { it.caption },
                    style = MaterialTheme.typography.bodySmall,
                )
            }

            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                Button({ onIntent(AssistantIntent.Ask) }, enabled = state.canAsk) { Text("Demander un plan") }
                if (state.working) {
                    CircularProgressIndicator(Modifier.size(18.dp))
                    Text(
                        "Un modele local qui lit une photo prend plusieurs minutes.",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }
        }
    }
}

@Composable
private fun ZoneCard(zone: PaintingPlan.Zone) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(
                if (zone.material.isBlank()) zone.name else "${zone.name} - ${zone.material}",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.Bold,
            )
            if (zone.note.isNotBlank()) {
                Text(zone.note, style = MaterialTheme.typography.bodySmall)
            }

            zone.layers().forEach { LayerRow(it) }

            if (zone.accents.isNotEmpty()) {
                Text(
                    "Variations locales",
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.Bold,
                )
                zone.accents.forEach { LayerRow(it) }
            }
        }
    }
}

/**
 * Une couche : ce qui est vise, ce que la palette atteint, et l'ecart entre les deux.
 *
 * <p>Les deux pastilles sont cote a cote a dessein. Quand elles se distinguent a l'oeil,
 * l'ecart n'a pas besoin d'etre explique.</p>
 */
@Composable
private fun LayerRow(layer: PaintingPlan.Layer) {
    Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
        Swatch(layer.target.toHex(), size = 24)
        Swatch(layer.achieved.toHex(), size = 24)
        Column(Modifier.padding(start = 2.dp)) {
            Text(
                "${layer.role}${if (layer.technique.isBlank()) "" else " - ${layer.technique}"}",
                style = MaterialTheme.typography.bodySmall,
                fontWeight = FontWeight.Bold,
            )
            Text(
                layer.recipe?.describe() ?: "Aucun melange : la palette ne va pas jusque-la.",
                style = MaterialTheme.typography.bodySmall,
            )
            Text(
                "ecart ${(layer.deltaE * 10).roundToInt() / 10.0} - ${layer.reachability()}",
                style = MaterialTheme.typography.bodySmall,
            )
            if (layer.note.isNotBlank()) {
                Text(layer.note, style = MaterialTheme.typography.bodySmall)
            }
        }
    }
}
