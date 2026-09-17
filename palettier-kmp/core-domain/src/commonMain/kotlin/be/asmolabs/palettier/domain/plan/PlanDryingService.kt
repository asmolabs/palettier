package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.drying.DryingContext
import be.asmolabs.palettier.domain.drying.DryingEstimate
import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Technique
import kotlin.time.Duration

/**
 * Deroule un plan de peinture dans le temps.
 *
 * <p>La difference avec le calcul d'une couche isolee tient a une chose : ici, on ne
 * demande pas au peintre quelle est la vitesse de sechage. Elle se deduit des pigments
 * reellement presents dans chaque melange. Un cadmium glisse dans un point lumineux
 * impose plusieurs jours a lui seul, et cela se voit sans avoir a y penser.</p>
 *
 * <p>Deux totaux sont produits, et leur ecart est l'information utile : le peintre qui
 * finit une zone avant d'attaquer la suivante attend la somme des zones, celui qui les
 * mene de front n'attend que la plus lente. Sur une figurine, la seconde facon est la
 * bonne, et c'est souvent plusieurs semaines de difference.</p>
 */
class PlanDryingService(private val dryingTimeService: DryingTimeService = DryingTimeService()) {

    /**
     * Une couche replacee dans le temps.
     *
     * @param startOffset delai entre le debut du travail sur la zone et cette couche
     * @param waitAfter   attente imposee avant la couche suivante de la meme zone
     */
    data class LayerSchedule(
        val role: String,
        val mix: String,
        val technique: Technique,
        val dryingClass: DryingClass,
        val estimate: DryingEstimate,
        val startOffset: Duration,
        val waitAfter: Duration,
    )

    /** Une zone et son enchainement. */
    data class ZoneSchedule(
        val zone: String,
        val material: String,
        val layers: List<LayerSchedule>,
        val span: Duration,
    ) {
        /** Le pigment le plus lent rencontre dans la zone : c'est lui qui commande. */
        fun slowest(): DryingClass =
            layers.fold(DryingClass.FAST) { slowest, layer -> slowest.slowest(layer.dryingClass) }
    }

    /**
     * @param sequential duree si l'on termine chaque zone avant d'attaquer la suivante
     * @param parallel   duree si l'on mene toutes les zones de front
     */
    data class Schedule(
        val zones: List<ZoneSchedule>,
        val sequential: Duration,
        val parallel: Duration,
        val advice: List<String>,
    )

    fun schedule(plan: PaintingPlan, workshop: Workshop): Schedule {
        val zones = plan.zones.map { schedule(it, workshop) }
        val sequential = zones.fold(Duration.ZERO) { total, zone -> total + zone.span }
        val parallel = zones.maxOfOrNull { it.span } ?: Duration.ZERO
        return Schedule(zones, sequential, parallel, advice(zones, sequential, parallel))
    }

    private fun schedule(zone: PaintingPlan.Zone, workshop: Workshop): ZoneSchedule {
        // Les variations locales se posent apres l'echelle, et comptent dans le planning :
        // elles seront recouvertes comme le reste.
        val layers = zone.layers() + zone.accents
        val scheduled = ArrayList<LayerSchedule>(layers.size)
        var offset = Duration.ZERO

        for (i in layers.indices) {
            val layer = layers[i]
            val technique = Technique.byLabel(layer.technique, Technique.GLAZE)
            val drying = dryingClassOf(layer)

            val estimate = dryingTimeService.estimate(
                DryingContext(
                    drying, technique.defaultMedium, technique.defaultRatio,
                    technique.typicalThickness, workshop,
                )
            )

            // Une technique qui demande un support ferme attend le sechage a coeur ;
            // les autres se contentent du delai de recouvrement.
            val wait = when {
                i == layers.lastIndex -> Duration.ZERO
                Technique.byLabel(layers[i + 1].technique, Technique.GLAZE).requiresCuredBase -> estimate.throughDry
                else -> estimate.recoat
            }

            scheduled += LayerSchedule(
                layer.role, layer.recipe?.describe() ?: "",
                technique, drying, estimate, offset, wait,
            )
            offset += wait
        }
        return ZoneSchedule(zone.name, zone.material, scheduled, offset)
    }

    companion object {
        /**
         * La vitesse de sechage d'une couche, lue dans les tubes qui la composent.
         *
         * <p>C'est tout l'interet de partir d'un projet plutot que d'un reglage : le
         * peintre n'a pas a savoir que son blanc de titane est lent, l'application le
         * sait.</p>
         */
        fun dryingClassOf(layer: PaintingPlan.Layer): DryingClass =
            layer.recipe?.dryingClass() ?: DryingClass.MEDIUM

        private fun advice(zones: List<ZoneSchedule>, sequential: Duration, parallel: Duration): List<String> {
            if (zones.isEmpty()) return emptyList()
            return buildList {
                if (sequential > parallel) {
                    add(
                        "Menez les zones de front plutot que l'une apres l'autre : pendant qu'une " +
                            "couche seche, les autres zones avancent. Il n'y a rien a gagner a attendre."
                    )
                }
                if (zones.size > 1) {
                    zones.maxByOrNull { it.span }?.let {
                        add("La zone ${it.zone} commande le planning a elle seule : commencez par elle.")
                    }
                }
                zones.filter { it.slowest() >= DryingClass.SLOW }.forEach {
                    add(
                        "${it.zone} contient un pigment a sechage ${it.slowest().label.lowercase()} : " +
                            "c'est lui qui impose les delais de cette zone."
                    )
                }
            }
        }
    }
}
