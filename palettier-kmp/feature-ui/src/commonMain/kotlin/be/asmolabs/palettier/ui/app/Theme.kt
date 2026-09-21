package be.asmolabs.palettier.ui.app

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.ui.graphics.Color

/**
 * Les couleurs de l'atelier, partagees par les deux clients.
 *
 * <p>Sombres et chaudes : on peint sous une lampe, et un ecran blanc a cote d'une piece
 * fausse le jugement des valeurs autant qu'une lumiere mal choisie.</p>
 */
val atelier: ColorScheme = darkColorScheme(
    primary = Color(0xFFE2A05F),
    onPrimary = Color(0xFF1B1410),
    surface = Color(0xFF1B1713),
    onSurface = Color(0xFFDED6CC),
    surfaceVariant = Color(0xFF252017),
    onSurfaceVariant = Color(0xFFBDB4A8),
    background = Color(0xFF14110E),
    onBackground = Color(0xFFDED6CC),
)
