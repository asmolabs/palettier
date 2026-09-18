package be.asmolabs.palettier.ui.workbench

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
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
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
import be.asmolabs.palettier.domain.workbench.Bench
import be.asmolabs.palettier.domain.workbench.PieceState
import be.asmolabs.palettier.domain.workbench.ZoneState
import be.asmolabs.palettier.ui.component.formatDuration

/**
 * Aujourd'hui : ou en sont les pieces, et laquelle peut etre reprise maintenant.
 *
 * <p>L'ecran ne decide de rien. Il recoit un etat et le dessine ; tout ce qui se calcule
 * -- le classement des pieces, l'etat d'une zone, le temps restant -- vient du domaine, et
 * est deja verifie par ses propres essais.</p>
 */
@Composable
fun WorkbenchScreen(state: WorkbenchUiState, modifier: Modifier = Modifier) {
    Surface(modifier = modifier.fillMaxSize()) {
        when {
            state.loading -> Box(Modifier.fillMaxSize(), Alignment.Center) { CircularProgressIndicator() }

            state.error != null -> Box(Modifier.fillMaxSize().padding(24.dp), Alignment.Center) {
                Text(state.error, style = MaterialTheme.typography.bodyLarge)
            }

            else -> LazyColumn(
                modifier = Modifier.fillMaxSize().padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                item { Headline(state.bench) }
                items(state.bench.pieces, key = { it.project.id ?: it.project.name.hashCode().toLong() }) {
                    PieceCard(it)
                }
                if (state.isEmpty) {
                    item {
                        Text(
                            "Aucun projet enregistre. Cochez ensuite chaque couche posee : cet ecran " +
                                "saura alors quand la piece redevient reprenable.",
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun Headline(bench: Bench) {
    val ready = bench.ready().size
    val text = when {
        ready == 1 -> "Une piece peut etre reprise maintenant."
        ready > 1 -> "$ready pieces peuvent etre reprises maintenant."
        else -> bench.nextAvailability()
            ?.let { "Rien a reprendre pour l'instant. La premiere zone se libere dans ${formatDuration(it)}." }
            ?: "Toutes les pieces sont terminees."
    }
    Text(text, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
}

@Composable
private fun PieceCard(piece: PieceState) {
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(piece.project.name, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            Text(pieceSummary(piece), style = MaterialTheme.typography.bodySmall)
            piece.zones.forEach { ZoneRow(it) }
        }
    }
}

private fun pieceSummary(piece: PieceState): String {
    val progress = "${piece.appliedCoats} couches posees sur ${piece.totalCoats}"
    if (piece.done) return "$progress  -  piece terminee."

    val ready = piece.readyZones
    if (ready.isNotEmpty()) {
        val which = ready.joinToString(", ") { it.name }
        return "$progress  -  ${if (ready.size == 1) "une zone disponible" else "${ready.size} zones disponibles"} : $which."
    }
    return piece.nextAvailability
        ?.let { "$progress  -  rien avant ${formatDuration(it)}." }
        ?: "$progress."
}

@Composable
private fun ZoneRow(zone: ZoneState) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Swatch(zone.last?.targetHex)
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text(zoneTitle(zone), style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
            Text(zoneState(zone), style = MaterialTheme.typography.bodySmall)
        }
        StagePill(zone)
    }
}

private fun zoneTitle(zone: ZoneState): String =
    if (zone.material.isBlank()) zone.name else "${zone.name} - ${zone.material}"

private fun zoneState(zone: ZoneState): String {
    if (zone.done) return "Les ${zone.total} couches prevues sont posees."
    if (zone.notStarted) return "Aucune couche posee, ${zone.total} prevues. Rien n'empeche de commencer."

    val last = zone.last!!
    val posed = "${last.role} posee."
    val next = last.nextRole?.let { " Au tour de : $it." }.orEmpty()

    return if (zone.readyNow) "$posed  ${last.stage.meaning}$next"
    else "$posed  ${last.stage.meaning} Disponible dans ${formatDuration(zone.remaining)}."
}

/** Vert quand on peut y aller, chaud quand il faut attendre, eteint quand c'est fini. */
@Composable
private fun StagePill(zone: ZoneState) {
    val (label, colour) = when {
        zone.done -> "Terminee" to Color(0xFF6B6259)
        zone.notStarted -> "A commencer" to Color(0xFF3F8F79)
        zone.readyNow -> zone.last!!.stage.label to Color(0xFF3F8F79)
        else -> zone.last!!.stage.label to Color(0xFFB0603F)
    }
    Surface(color = colour, shape = RoundedCornerShape(20.dp)) {
        Text(
            label,
            color = Color.White,
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 3.dp),
        )
    }
}

/** La pastille de couleur. Une zone jamais commencee n'en a pas : on montre un vide. */
@Composable
private fun Swatch(hex: String?) {
    val rgb = hex?.let(Rgb::ofHex)
    Box(
        Modifier
            .size(34.dp)
            .background(
                color = rgb?.let { Color(it.r.toFloat(), it.g.toFloat(), it.b.toFloat()) }
                    ?: Color(0xFF221C18),
                shape = RoundedCornerShape(6.dp),
            )
    )
}
