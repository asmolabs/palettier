package be.asmolabs.palettier.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import be.asmolabs.palettier.domain.color.Rgb

/** Pastille de couleur, partagee par tous les ecrans. */
@Composable
fun Swatch(
    hex: String,
    size: Int = 28,
    selected: Boolean = false,
    onClick: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    val rgb = runCatching { Rgb.ofHex(hex) }.getOrNull() ?: Rgb.ofHex("#221C18")
    Box(
        modifier
            .size(size.dp)
            .background(Color(rgb.r.toFloat(), rgb.g.toFloat(), rgb.b.toFloat()), RoundedCornerShape(5.dp))
            .then(
                if (selected) Modifier.border(2.dp, MaterialTheme.colorScheme.primary, RoundedCornerShape(5.dp))
                else Modifier
            )
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
    )
}
