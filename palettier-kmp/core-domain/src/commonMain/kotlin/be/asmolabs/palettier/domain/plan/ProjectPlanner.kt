package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectZone

/**
 * Reconstitue le plan d'un projet, dosages recalcules.
 *
 * <p>Le projet garde les decisions -- quelles zones, quelle couleur viser, quelle
 * technique -- et la liste des tubes avec lesquels elles ont ete prises. Les dosages, eux,
 * sont refaits a chaque lecture : un ecart qui se resserre signale que la palette s'est
 * enrichie depuis, un ecart qui s'ouvre qu'un tube en a disparu.</p>
 *
 * <p>Le calcul se fait avec les tubes du projet et non ceux de la palette : c'est ce qui
 * rend un plan reproductible des mois plus tard, quoi qu'il soit arrive a la palette
 * entre-temps.</p>
 */
class ProjectPlanner(private val mixer: ColorMixService) {

    suspend fun plan(project: Project): PaintingPlan {
        val paints = project.effectivePaints()
        val zones = project.zones.map { zone -> rebuild(zone, paints) }

        val paletteName = project.palette?.name
            ?: "${paints.size} tubes conserves avec le projet"

        return PaintingPlan(project.subject, paletteName, project.approach, zones)
    }

    /**
     * Remet les couches dans la forme attendue par le plan.
     *
     * <p>Les roles sont ce qui dit ou va chaque couche : "Ombre 1" descend, "Lumiere 2"
     * monte. Une couche dont le role ne se reconnait pas devient la base, faute de mieux
     * -- un plan incomplet vaut mieux qu'un plan refuse.</p>
     */
    private suspend fun rebuild(zone: ProjectZone, paints: List<Paint>): PaintingPlan.Zone {
        val ladder = zone.layers.filter { it.kind == ProjectLayer.Kind.LADDER }.map { recompute(it, paints) }
        val accents = zone.layers.filter { it.kind == ProjectLayer.Kind.ACCENT }.map { recompute(it, paints) }

        val shadows = ladder.filter { it.role.lowercase().startsWith("ombre") }
        val highlights = ladder.filter { it.role.lowercase().startsWith("lumiere") }
        val base = ladder.firstOrNull { it.role.equals("base", ignoreCase = true) }
            ?: ladder.getOrNull(ladder.size / 2)

        // Les ombres ont ete enregistrees de la plus sombre a la plus claire : on rend
        // l'ordre attendu, de la plus legere a la plus profonde.
        return PaintingPlan.Zone(zone.name, zone.material, zone.note, base, shadows.reversed(), highlights, accents)
    }

    private suspend fun recompute(layer: ProjectLayer, paints: List<Paint>): PaintingPlan.Layer {
        val target = layer.target
        val found = if (paints.isEmpty()) emptyList()
        else mixer.suggestMixes(target, paints, maxResults = 1, maxPaints = MAX_PAINTS)

        val best = found.firstOrNull()
            ?: return PaintingPlan.Layer(layer.role, target, layer.technique, layer.note, null, target, 0.0)

        return PaintingPlan.Layer(
            layer.role, target, layer.technique, layer.note,
            best, best.color, deltaE2000(target, best.color),
        )
    }

    companion object {
        /** Nombre maximal de tubes par melange lors de la restitution. */
        private const val MAX_PAINTS = 3
    }
}
