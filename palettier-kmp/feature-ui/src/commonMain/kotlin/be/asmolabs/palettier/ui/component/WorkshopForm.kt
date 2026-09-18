package be.asmolabs.palettier.ui.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.Ventilation

/**
 * Les conditions de l'atelier, partagees par les ecrans qui calculent un sechage.
 *
 * <p>Le peintre ne les saisit qu'une fois par ecran : ce sont les siennes, elles ne
 * changent pas d'un calcul a l'autre.</p>
 */
@Composable
fun WorkshopForm(workshop: Workshop, onChange: (Workshop) -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("Conditions de l'atelier", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Temperature", Modifier.width(130.dp), style = MaterialTheme.typography.bodySmall)
            Slider(
                value = workshop.temperatureCelsius.toFloat(),
                onValueChange = { onChange(workshop.copy(temperatureCelsius = it.toDouble())) },
                valueRange = 5f..40f,
                modifier = Modifier.width(220.dp),
            )
            Text("%.0f °C".format(workshop.temperatureCelsius), style = MaterialTheme.typography.bodySmall)
        }

        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Humidite", Modifier.width(130.dp), style = MaterialTheme.typography.bodySmall)
            Slider(
                value = workshop.relativeHumidity.toFloat(),
                onValueChange = { onChange(workshop.copy(relativeHumidity = it.toDouble())) },
                valueRange = 10f..95f,
                modifier = Modifier.width(220.dp),
            )
            Text("%.0f %%".format(workshop.relativeHumidity), style = MaterialTheme.typography.bodySmall)
        }

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Ventilation", Modifier.width(130.dp), style = MaterialTheme.typography.bodySmall)
            Ventilation.entries.forEach { ventilation ->
                Chip(
                    label = ventilation.label,
                    selected = ventilation == workshop.ventilation,
                    onClick = { onChange(workshop.copy(ventilation = ventilation)) },
                )
            }
        }
    }
}
