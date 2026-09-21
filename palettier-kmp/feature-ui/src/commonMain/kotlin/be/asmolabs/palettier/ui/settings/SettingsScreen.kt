package be.asmolabs.palettier.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

/**
 * Ce que l'ecran de reglages montre : l'etat de la sauvegarde, et ce qu'on y a fait.
 *
 * <p>L'acces au systeme de fichiers appartient a la plateforme : l'ecran ne connait que
 * deux actions et un message.</p>
 */
data class SettingsUiState(
    val busy: Boolean = false,
    val message: String? = null,
    val counts: String? = null,
)

/**
 * Sauvegarde et restauration.
 *
 * <p>L'export n'est pas une commodite : sans lui, la migration serait un aller simple.
 * Tant que les deux applications parlent la meme langue, revenir en arriere reste
 * possible -- et le format est celui de l'application Java, a la lettre.</p>
 */
@Composable
fun SettingsScreen(
    state: SettingsUiState,
    onExport: () -> Unit,
    onImport: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text("Sauvegarde", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                    Text(
                        "Une archive contient tout : le catalogue et vos corrections, les palettes, " +
                            "les projets avec leurs couches posees et leurs photos, les recettes. " +
                            "Elle se relit aussi bien ici que dans l'application d'origine.",
                        style = MaterialTheme.typography.bodySmall,
                    )
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Button(onExport, enabled = !state.busy) { Text("Exporter") }
                        Button(onImport, enabled = !state.busy) { Text("Importer") }
                        if (state.busy) CircularProgressIndicator(Modifier.size(18.dp))
                    }
                    state.message?.let { Text(it, style = MaterialTheme.typography.bodySmall) }
                }
            }

            state.counts?.let {
                Card(Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("Contenu", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        Text(it, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }

            Card(Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text("Assistant", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                    Text(
                        "Pas encore porte. Dans l'application d'origine il est desactive par defaut, " +
                            "et rien de ce qui precede n'en depend : le catalogue, les melanges, le " +
                            "sechage et les plans fonctionnent sans lui.",
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
            }

            Text(
                "Catalogue, palettes et projets sont ranges sur ce poste, sous ~/.palettier. " +
                    "Aucune donnee ne part ailleurs.",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}
