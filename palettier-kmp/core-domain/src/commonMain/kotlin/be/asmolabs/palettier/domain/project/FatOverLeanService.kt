package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.recipe.Recipe

/**
 * La regle du gras sur maigre, verifiee sur l'ordre des couches.
 *
 * <p>Une couche maigre posee sur une couche grasse seche plus vite qu'elle. Le dessous
 * continue de bouger quand le dessus a pris : la couche superieure tire, et craquele --
 * des mois plus tard, sur une piece finie. C'est la seule faute de cet atelier qui ne se
 * rattrape pas.</p>
 *
 * <p>La verification est volontairement silencieuse sur ce qui se pratique tous les
 * jours. Un glacis tres dilue sur un aplat de base est plus maigre que lui, et pourtant
 * personne n'a jamais fait craqueler une figurine ainsi : le glacis est un voile, il n'a
 * pas de quoi tirer. L'alerte ne se declenche que lorsque les deux conditions sont
 * reunies -- un ecart de gras franc, et une couche qui n'est pas plus fine que celle
 * qu'elle recouvre.</p>
 */
class FatOverLeanService {

    /**
     * Un empilement a risque.
     *
     * @param where zone du projet, ou nom de la recette
     * @param under couche du dessous, la plus grasse
     * @param over  couche posee dessus, plus maigre
     */
    data class Risk(
        val where: String,
        val under: String,
        val over: String,
        val drop: Double,
        val explanation: String,
    )

    /** Une couche reduite a ce qui decide du risque : son gras et son epaisseur. */
    private data class Coat(
        val label: String,
        val role: String,
        val fatness: Double,
        val thickness: LayerThickness,
    )

    /** Les empilements a risque du projet, zone par zone, dans l'ordre des couches. */
    fun inspect(project: Project): List<Risk> = project.zones.flatMap { zone ->
        inspect(zone.name, zone.layers.map(::coatOf))
    }

    /**
     * Les empilements a risque d'une recette.
     *
     * <p>Une recette enonce son medium, sa dilution et son epaisseur pour chaque etape :
     * la verification s'appuie dessus, sans rien deduire. C'est le controle le plus sur
     * des deux, et c'est logique -- une recette est ecrite pour etre suivie telle quelle,
     * et la faute s'y reproduirait sur chaque piece.</p>
     */
    fun inspect(recipe: Recipe): List<Risk> = inspect(
        recipe.name,
        recipe.steps.mapIndexed { index, step ->
            Coat(
                step.technique.label,
                "Etape ${index + 1}",
                step.medium.fatness(step.mediumRatio),
                step.thickness,
            )
        },
    )

    /** Comparaison brute de deux techniques, pour l'ecran qui veut l'expliquer. */
    fun fatnessOf(technique: String?): Double = fatness(Technique.byLabel(technique, Technique.GLAZE))

    /** Vrai si l'epaisseur usuelle de la technique en fait un simple voile. */
    fun isVeil(technique: String?): Boolean =
        Technique.byLabel(technique, Technique.GLAZE).typicalThickness == LayerThickness.GLAZE

    private fun inspect(where: String, coats: List<Coat>): List<Risk> = buildList {
        for (i in 1 until coats.size) {
            compare(where, coats[i - 1], coats[i])?.let(::add)
        }
    }

    private fun compare(where: String, under: Coat, over: Coat): Risk? {
        val drop = under.fatness - over.fatness
        if (drop < SIGNIFICANT_DROP) return null
        // Un voile ne tire pas sur ce qu'il recouvre : seule une couche au moins aussi
        // chargee que celle du dessous pose un probleme.
        if (over.thickness < under.thickness) return null
        return Risk(where, under.role, over.role, drop, explain(under.label, over.label))
    }

    private fun coatOf(layer: ProjectLayer): Coat {
        val technique = Technique.byLabel(layer.technique, Technique.GLAZE)
        return Coat(technique.label, layer.role, fatness(technique), technique.typicalThickness)
    }

    private fun fatness(technique: Technique): Double =
        technique.defaultMedium.fatness(technique.defaultRatio)

    private fun explain(below: String, above: String): String =
        "$above est plus maigre que $below, et pas plus fine. Posee dessus, elle sechera " +
            "avant elle et tirera dessus en vieillissant. Faites l'inverse, ou attendez le " +
            "sechage a coeur de la couche du dessous et allegez la main."

    companion object {
        /**
         * Ecart de gras a partir duquel l'empilement merite d'etre signale. Sous ce seuil,
         * on est dans la variation normale entre deux jus plus ou moins dilues.
         */
        private const val SIGNIFICANT_DROP = 0.40
    }
}
