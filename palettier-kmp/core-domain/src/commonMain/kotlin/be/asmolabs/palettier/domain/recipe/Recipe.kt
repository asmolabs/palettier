package be.asmolabs.palettier.domain.recipe

import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique

/** Une etape de recette : une technique, un melange et un reglage de dilution. */
data class RecipeStep(
    val id: Long? = null,
    val technique: Technique,
    /** Description libre du melange, par exemple "Terre d'ombre brulee + une pointe de noir". */
    val paintMix: String = "",
    val medium: Medium = Medium.ODORLESS_THINNER,
    val mediumRatio: Double = 0.0,
    val thickness: LayerThickness = LayerThickness.THIN,
    val notes: String = "",
) {
    companion object {
        /** Etape pre-remplie avec les reglages usuels de la technique. */
        fun of(technique: Technique, paintMix: String) = RecipeStep(
            technique = technique,
            paintMix = paintMix,
            medium = technique.defaultMedium,
            mediumRatio = technique.defaultRatio,
            thickness = technique.typicalThickness,
        )
    }
}

/** Suite ordonnee d'etapes a l'huile pour un sujet donne. */
data class Recipe(
    val id: Long? = null,
    val name: String,
    /** Ce sur quoi la recette s'applique : "casque allemand", "cape rouge", "peau 1/10"... */
    val subject: String = "",
    val notes: String = "",
    val steps: List<RecipeStep> = emptyList(),
) {
    fun withStep(step: RecipeStep): Recipe = copy(steps = steps + step)
}
