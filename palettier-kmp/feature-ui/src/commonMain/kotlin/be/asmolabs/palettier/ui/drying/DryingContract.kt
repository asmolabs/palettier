package be.asmolabs.palettier.ui.drying

import be.asmolabs.palettier.domain.drying.DryingEstimate
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique

data class DryingUiState(
    val workshop: Workshop = Workshop.standard(),
    val technique: Technique = Technique.GLAZE,
    val dryingClass: DryingClass = DryingClass.MEDIUM,
    val medium: Medium = Technique.GLAZE.defaultMedium,
    val mediumRatio: Double = Technique.GLAZE.defaultRatio,
    val thickness: LayerThickness = Technique.GLAZE.typicalThickness,
    val estimate: DryingEstimate? = null,
)

sealed interface DryingIntent {
    data class SetWorkshop(val workshop: Workshop) : DryingIntent

    /** Choisir une technique repositionne medium, dilution et epaisseur sur ses usages. */
    data class SetTechnique(val technique: Technique) : DryingIntent

    data class SetDryingClass(val dryingClass: DryingClass) : DryingIntent
    data class SetMedium(val medium: Medium) : DryingIntent
    data class SetRatio(val ratio: Double) : DryingIntent
    data class SetThickness(val thickness: LayerThickness) : DryingIntent
}
