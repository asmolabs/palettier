package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.paint.Paint

/**
 * Que faire des tubes qu'un projet reclame et qu'on n'a pas.
 *
 * <p>L'inventaire savait dire qu'un tube manquait ; il ne servait a rien d'autre. Or la
 * question du peintre n'est pas "que me manque-t-il" -- il le sait -- mais "avec quoi je
 * m'en sors ce soir".</p>
 *
 * <p>L'etagere est passee en parametre plutot que lue par un depot injecte. Le service
 * reste alors une fonction pure, testable sans faux depot, et c'est l'appelant qui sait
 * quand relire la base.</p>
 */
class SubstituteService {

    /**
     * Un tube absent de l'etagere, et le plus proche de ceux qu'on possede.
     *
     * @param nearest le tube de rechange, ou null si l'etagere est vide
     * @param deltaE  ecart percu entre le tube manquant et son remplacant
     */
    data class Missing(val paint: Paint, val nearest: Paint?, val deltaE: Double) {

        /** Vrai quand le remplacement ne se verra pas sur la piece. */
        val isComfortable: Boolean get() = nearest != null && deltaE < VISIBLE_GAP

        fun verdict(): String = when {
            nearest == null -> "Rien sur l'etagere pour le remplacer."
            deltaE < 2 -> "${nearest.displayName} le remplace sans que cela se voie."
            deltaE < VISIBLE_GAP -> "${nearest.displayName} en approche : l'ecart ne se verra pas sur la piece."
            deltaE < 12 ->
                "${nearest.displayName} est ce qui s'en rapproche le plus, mais l'ecart se voit : " +
                    "a rattraper au glacis."
            else -> "Rien d'approchant sur l'etagere : celui-la, il faut l'acheter."
        }
    }

    /** Les tubes du projet qui ne sont pas sur l'etagere, avec leur meilleur remplacant. */
    fun missingFrom(project: Project, owned: List<Paint>): List<Missing> =
        missingAmong(project.effectivePaints(), owned)

    /** Idem pour une selection quelconque de tubes. */
    fun missingAmong(wanted: List<Paint>, owned: List<Paint>): List<Missing> =
        wanted.filterNot { it.inStock }.map { paint ->
            // Le tube manquant ne doit evidemment pas se proposer lui-meme, et un tube
            // identique d'une autre gamme reste un remplacant legitime.
            val nearest = owned
                .filterNot { it.id != null && it.id == paint.id }
                .minByOrNull { gap(paint, it) }
            Missing(paint, nearest, nearest?.let { gap(paint, it) } ?: Double.MAX_VALUE)
        }

    private fun gap(wanted: Paint, candidate: Paint): Double =
        deltaE2000(wanted.color, candidate.color)

    companion object {
        /** Au-dela, l'ecart se voit cote a cote et le remplacement ne va plus de soi. */
        private const val VISIBLE_GAP = 5.0
    }
}
