package be.asmolabs.palettier.ui.component

import kotlin.time.Duration

/** Duree en langage courant : "45 min", "6 h 30", "2 j 4 h", "3 semaines". */
fun formatDuration(duration: Duration): String {
    val minutes = duration.inWholeMinutes
    if (minutes < 60) return "$minutes min"

    val hours = duration.inWholeHours
    if (hours < 48) {
        val rest = minutes % 60
        return if (rest == 0L) "$hours h" else "$hours h ${rest.toString().padStart(2, '0')}"
    }

    val days = duration.inWholeDays
    if (days < 14) {
        val rest = hours % 24
        return if (rest == 0L) "$days jours" else "$days j $rest h"
    }
    return "${(days + 3) / 7} semaines"
}
