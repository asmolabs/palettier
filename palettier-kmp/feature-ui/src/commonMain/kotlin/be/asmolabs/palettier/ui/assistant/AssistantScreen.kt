package be.asmolabs.palettier.ui.assistant

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.background
import androidx.compose.foundation.Image
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
    onChooseShelf: () -> Unit,
    onExport: () -> Unit,
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
                item { ShelfCard(state, onIntent, onChooseShelf) }
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

                            SaveRow(state, onIntent, onExport)
                        }
                    }
                }
                items(plan.zones) { zone -> ZoneCard(zone) }
            }
        }
    }
}

/**
 * Figer le plan en projet.
 *
 * <p>Sans cela, un plan propose disparait a la demande suivante : le modele a travaille
 * plusieurs minutes pour rien. Les photos qui ont servi a l'etablir partent avec lui --
 * sans elles, on relit dans six mois un plan dont on ne sait plus a quoi il
 * s'appliquait.</p>
 */
@Composable
private fun SaveRow(state: AssistantUiState, onIntent: (AssistantIntent) -> Unit, onExport: () -> Unit) {
    // Le sujet decrit fait un nom de depart : on tape rarement deux fois la meme chose.
    var name by remember(state.plan) { mutableStateOf(state.suggestedProjectName) }

    Column(verticalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.padding(top = 10.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("Nom du projet") },
                singleLine = true,
                modifier = Modifier.weight(1f),
            )
            Button(
                { onIntent(AssistantIntent.SaveProject(name)) },
                enabled = !state.saving,
            ) { Text("Enregistrer le projet") }
            TextButton(onExport) { Text("Exporter en PDF") }
            if (state.saving) CircularProgressIndicator(Modifier.size(18.dp))
        }
        state.savedMessage?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
    }
}

/**
 * Une photo jointe, telle qu'on la reconnait.
 *
 * <p>Une ligne de texte ne dit pas qu'on s'est trompe de fichier. Une vignette, si -- et
 * c'est la seule facon de le voir avant que le modele ne travaille plusieurs minutes sur
 * la mauvaise image.</p>
 *
 * <p>Ce qui est decode est la copie reduite, jamais la photo d'origine : un cliche de
 * telephone rendu dans un carre de quatre-vingts points ferait decoder plusieurs millions
 * de pixels pour rien.</p>
 */
@Composable
private fun Thumbnail(photo: AttachedPhoto, onRemove: () -> Unit) {
    val bitmap = remember(photo) { runCatching { photo.thumbnail.decodeToImageBitmap() }.getOrNull() }

    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            Modifier
                .size(80.dp)
                .background(MaterialTheme.colorScheme.surfaceVariant, RoundedCornerShape(6.dp)),
            contentAlignment = Alignment.Center,
        ) {
            if (bitmap == null) {
                // Lisible par le modele, mais pas par le decodeur local : cela arrive, et
                // ce n'est pas une raison pour refuser la photo.
                Text("?", style = MaterialTheme.typography.titleMedium)
            } else {
                Image(
                    bitmap = bitmap,
                    contentDescription = photo.caption,
                    modifier = Modifier.fillMaxSize().clip(RoundedCornerShape(6.dp)),
                    contentScale = ContentScale.Crop,
                )
            }
        }
        Text(
            photo.caption,
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.widthIn(max = 80.dp),
        )
        TextButton(onRemove) { Text("\u2715", style = MaterialTheme.typography.labelSmall) }
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
                if (state.installed.isNotEmpty()) {
                    Text(
                        "Installes sur ce poste. Un modele sans la mention Vision ne verra pas " +
                            "vos photos : il travaillera sur le seul texte du sujet.",
                        style = MaterialTheme.typography.bodySmall,
                    )
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        state.installed.forEach { installed ->
                            val label = buildString {
                                append(installed.name)
                                if (installed.supportsVision) append("  Vision")
                                if (installed.sizeLabel.isNotBlank()) append("  ${installed.sizeLabel}")
                            }
                            Chip(label, settings.model == installed.name) {
                                onIntent(AssistantIntent.SaveSettings(settings.copy(model = installed.name)))
                            }
                        }
                    }
                }
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
            if (state.figurine != null || state.references.isNotEmpty()) {
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    state.figurine?.let { photo ->
                        Thumbnail(photo) { onIntent(AssistantIntent.RemoveFigurine) }
                    }
                    state.references.forEachIndexed { index, photo ->
                        Thumbnail(photo) { onIntent(AssistantIntent.RemoveReference(index)) }
                    }
                }
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

/**
 * La lecture d'etiquettes.
 *
 * <p>Rien n'est coche d'avance : le modele lit, le peintre tranche. Se voir attribuer
 * des tubes qu'on ne possede pas serait pire que de tout saisir a la main.</p>
 */
@Composable
private fun ShelfCard(
    state: AssistantUiState,
    onIntent: (AssistantIntent) -> Unit,
    onChooseShelf: () -> Unit,
) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text("Mon etagere", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            Text(
                "Photographiez vos tubes plutot que de cocher quatre cents cases. Le modele lit " +
                    "les etiquettes ; c'est vous qui confirmez, un tube a la fois.",
                style = MaterialTheme.typography.bodySmall,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                OutlinedButton(onChooseShelf, enabled = !state.reading) { Text("Photo de mes tubes") }
                if (state.reading) CircularProgressIndicator(Modifier.size(18.dp))
                if (state.tubes.isNotEmpty()) {
                    OutlinedButton({ onIntent(AssistantIntent.ForgetTubes) }) { Text("Tout ecarter") }
                }
            }

            state.tubes.forEach { read ->
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    read.match?.paint?.let { Swatch(it.hexColor, size = 22) }
                    Column(Modifier.weight(1f)) {
                        Text(read.label, style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.Bold)
                        Text(
                            read.match?.let { match ->
                                val confiance = (match.confidence * 100).roundToInt()
                                "${match.paint.displayName} - confiance ${confiance} %" +
                                    if (match.isReliable) "" else ", a verifier"
                            } ?: "Aucune fiche du catalogue ne correspond : a saisir a la main.",
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                    if (read.match != null) {
                        Button({ onIntent(AssistantIntent.KeepTube(read)) }) { Text("Je l'ai") }
                    }
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
