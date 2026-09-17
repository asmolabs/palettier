package be.asmolabs.palettier.domain.project

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000

/**
 * Une teinte relevee sur une image et la place qu'elle y occupe.
 *
 * <p>Le releve lui-meme est affaire de plateforme : decoder un JPEG n'a rien de commun
 * entre une machine de bureau, un telephone et un iPhone. Le domaine ne connait que le
 * resultat.</p>
 */
data class DominantColour(val color: Rgb, val share: Double)

/**
 * Ce qu'il y a sur la piece, compare a ce qui etait vise.
 *
 * <p>Le sens de lecture va de la piece vers le plan, et non l'inverse. Partir des
 * couleurs visees pour chercher la plus proche sur la photo trouverait toujours quelque
 * chose : une photo contient des milliers de teintes. Partir de ce qui occupe reellement
 * la piece, et demander a quoi cela devait correspondre, se trompe moins -- et signale au
 * passage ce qui n'etait prevu nulle part.</p>
 *
 * <p>La mesure se fait sur le fichier d'origine du peintre, jamais sur la photo rangee
 * avec le projet : celle-ci est reduite et reencodee, et ses couleurs ont bouge. Meme
 * ainsi, une photo reste prise sous une lumiere quelconque -- l'ecart chiffre ici se lit
 * comme une tendance, pas comme un verdict.</p>
 */
class ProgressCheckService {

    /**
     * Une teinte relevee sur la piece, et la couche a laquelle elle repond.
     *
     * @param share part de l'image occupee par cette teinte
     * @param zone  zone du plan visee, ou null si rien n'y ressemble
     */
    data class Observed(
        val measured: Rgb,
        val share: Double,
        val zone: String?,
        val role: String?,
        val target: Rgb?,
        val deltaE: Double,
    ) {
        val matchesPlan: Boolean get() = zone != null && deltaE < UNPLANNED

        fun verdict(): String = when {
            !matchesPlan -> "Rien de prevu ne ressemble a cette teinte : appret, socle, ou une couche improvisee."
            deltaE < 2 -> "$zone / $role : vous y etes."
            deltaE < 5 -> "$zone / $role : l'ecart ne se verra pas sur la piece."
            else -> "$zone / $role : l'ecart se voit, un glacis le rattraperait."
        }
    }

    /** Confronte les teintes relevees aux couleurs visees par le projet. */
    fun compare(project: Project, measured: List<DominantColour>): List<Observed> =
        measured.map { dominant ->
            var closest: Observed? = null
            for (zone in project.zones) {
                for (layer in zone.layers) {
                    val gap = deltaE2000(dominant.color, layer.target)
                    if (closest == null || gap < closest.deltaE) {
                        closest = Observed(
                            dominant.color, dominant.share,
                            zone.name, layer.role, layer.target, gap,
                        )
                    }
                }
            }
            closest ?: Observed(dominant.color, dominant.share, null, null, null, Double.MAX_VALUE)
        }

    companion object {
        /** Au-dela de cet ecart, la teinte relevee ne correspond a rien du plan. */
        private const val UNPLANNED = 12.0
    }
}
