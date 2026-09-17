package be.asmolabs.palettier.domain.workbench

import be.asmolabs.palettier.domain.drying.DryingContext
import be.asmolabs.palettier.domain.drying.DryingEstimate
import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectZone
import kotlin.time.Clock
import kotlin.time.Duration
import kotlin.time.Instant

/**
 * Ou en est une couche posee.
 *
 * <p>Les etats se suivent dans l'ordre du temps : une couche les traverse tous, sans
 * jamais revenir en arriere. L'interet n'est pas de nommer un etat mais de dire ce qu'il
 * autorise -- c'est la question du peintre.</p>
 */
enum class Stage(val label: String, val meaning: String) {
    OPEN("Encore ouverte", "Fondus et reprises dans le frais sont encore possibles."),
    SETTING("En prise", "Elle ne se travaille plus sans l'arracher : ne la touchez pas."),
    TOUCH_DRY("Seche au toucher", "Elle ne marque plus, mais une couche posee dessus la releverait encore."),
    RECOATABLE("Recouvrable", "La couche suivante peut etre posee sans relever celle-ci."),
    THROUGH_DRY("Seche a coeur", "Manipulation, masquage et montage possibles."),
    CURED("Polymerisee", "Vernis final possible.");

    override fun toString() = label
}

/**
 * Une couche posee, replacee dans le present.
 *
 * @param untilNext attente restante avant de pouvoir poser la couche suivante, nulle si
 *                  l'on peut y aller -- ou s'il n'y a plus rien a poser
 * @param nextRole  role de la couche qui attend, ou null si la zone est finie
 */
data class CoatState(
    val zone: String,
    val role: String,
    val targetHex: String,
    val appliedAt: Instant,
    val dryingClass: DryingClass,
    val estimate: DryingEstimate,
    val stage: Stage,
    val nextRole: String?,
    val untilNext: Duration,
) {
    val readyForNext: Boolean get() = untilNext == Duration.ZERO
}

/**
 * Une zone et son avancement.
 *
 * @param last derniere couche posee, ou null si la zone n'est pas commencee
 */
data class ZoneState(
    val name: String,
    val material: String,
    val applied: Int,
    val total: Int,
    val last: CoatState?,
) {
    /** Toutes les couches prevues sont posees : il n'y a plus rien a y faire. */
    val done: Boolean get() = total > 0 && applied == total
    val notStarted: Boolean get() = applied == 0

    /** Du travail est possible tout de suite sur cette zone. */
    val readyNow: Boolean get() = !done && (last == null || last.readyForNext)

    /** Attente avant que cette zone redevienne disponible, nulle si elle l'est deja. */
    val remaining: Duration get() = if (readyNow || done) Duration.ZERO else last!!.untilNext
}

/** Une piece en cours et l'etat de chacune de ses zones. */
data class PieceState(val project: Project, val zones: List<ZoneState>) {

    val readyZones: List<ZoneState> get() = zones.filter { it.readyNow }
    val readyNow: Boolean get() = readyZones.isNotEmpty()

    /** La piece est peinte : toutes ses zones le sont. */
    val done: Boolean get() = zones.isNotEmpty() && zones.all { it.done }

    /**
     * Delai avant que la piece redevienne disponible, nul si elle l'est deja ou si elle
     * est finie. C'est la plus courte des attentes : il suffit qu'une zone se libere.
     */
    val nextAvailability: Duration?
        get() = if (readyNow || done) null else zones.filterNot { it.done }.minOfOrNull { it.remaining }

    val appliedCoats: Int get() = zones.sumOf { it.applied }
    val totalCoats: Int get() = zones.sumOf { it.total }
}

/** L'etabli entier, les pieces reprenables en tete. */
data class Bench(val pieces: List<PieceState>) {
    fun ready() = pieces.filter { it.readyNow }
    fun waiting() = pieces.filter { !it.readyNow && !it.done }
    fun done() = pieces.filter { it.done }

    /** Delai avant que quoi que ce soit se libere, nul si du travail attend deja. */
    fun nextAvailability(): Duration? = waiting().mapNotNull { it.nextAvailability }.minOrNull()
}

/**
 * L'etabli : ou en sont les pieces en cours, et laquelle peut etre reprise maintenant.
 *
 * <p>C'est la question que pose le peintre en entrant dans son atelier. Le sechage etait
 * calcule dans l'abstrait -- "cette couche demandera trois jours" -- faute de savoir quand
 * elle avait ete posee. Une fois la pose consignee, la meme formule repond au present.</p>
 *
 * <p>Les conditions employees sont celles figees au moment de la pose, et non celles de
 * l'atelier aujourd'hui : l'huile a seche dans l'atelier qu'elle a connu.</p>
 *
 * <p>Une piece est prete des qu'une de ses zones l'est, et non quand toutes le sont :
 * pendant qu'une joue seche, la cape avance.</p>
 */
class WorkbenchService(private val dryingTimeService: DryingTimeService = DryingTimeService()) {

    /**
     * L'etat de l'etabli a un instant donne.
     *
     * <p>Les projets sont passes en parametre plutot que lus par un depot injecte : le
     * calcul reste une fonction pure, et l'instant de reference en fait un comportement
     * verifiable.</p>
     */
    fun bench(projects: List<Project>, now: Instant = Clock.System.now()): Bench =
        Bench(projects.map { state(it, now) }.sortedWith(byUrgency))

    private fun state(project: Project, now: Instant): PieceState =
        PieceState(project, project.zones.map { state(it, now) })

    private fun state(zone: ProjectZone, now: Instant): ZoneState {
        val applied = zone.layers.filter { it.isApplied }

        // La derniere posee au sens du temps, et non du rang : le peintre ne suit pas
        // forcement l'ordre du plan, et c'est la couche la plus fraiche qui commande.
        val last = applied.maxByOrNull { it.applied!!.at }
        val next = nextAfter(zone.layers, last)

        return ZoneState(
            zone.name, zone.material, applied.size, zone.layers.size,
            last?.let { coat(zone, it, next, now) },
        )
    }

    /**
     * La couche qui attend son tour : celle qui suit la derniere posee.
     *
     * <p>Prendre simplement la premiere non posee serait faux des que le peintre sort de
     * l'ordre du plan -- annoncer "au tour de la base" alors que la lumiere est deja
     * dessus n'a aucun sens. Quand il ne reste rien apres la derniere posee, on revient a
     * la premiere couche manquante : il reste du travail, hors sequence.</p>
     */
    private fun nextAfter(layers: List<ProjectLayer>, last: ProjectLayer?): ProjectLayer? {
        val from = if (last == null) 0 else layers.indexOf(last) + 1
        for (i in from until layers.size) {
            if (!layers[i].isApplied) return layers[i]
        }
        return layers.firstOrNull { !it.isApplied }
    }

    private fun coat(zone: ProjectZone, layer: ProjectLayer, next: ProjectLayer?, now: Instant): CoatState {
        val applied = layer.applied!!
        val technique = Technique.byLabel(layer.technique, Technique.GLAZE)
        val estimate = dryingTimeService.estimate(
            DryingContext(
                applied.dryingClass, technique.defaultMedium, technique.defaultRatio,
                technique.typicalThickness, applied.workshop,
            )
        )

        val elapsed = (now - applied.at).coerceAtLeast(Duration.ZERO)

        // Une technique qui demande un support ferme attend le sechage a coeur ; les
        // autres se contentent du delai de recouvrement. Meme regle que le planning
        // previsionnel, appliquee a la couche qui attend reellement son tour.
        val gate = when {
            next == null -> Duration.ZERO
            Technique.byLabel(next.technique, Technique.GLAZE).requiresCuredBase -> estimate.throughDry
            else -> estimate.recoat
        }

        return CoatState(
            zone.name, layer.role, layer.targetHex, applied.at, applied.dryingClass,
            estimate, stageOf(elapsed, estimate), next?.role,
            (gate - elapsed).coerceAtLeast(Duration.ZERO),
        )
    }

    companion object {
        /**
         * Les pieces reprenables d'abord, puis celles dont l'attente est la plus courte,
         * et les pieces finies en dernier.
         */
        private val byUrgency = compareBy<PieceState>(
            { if (it.readyNow) 0 else if (it.done) 2 else 1 },
            { it.nextAvailability ?: Duration.ZERO },
        )

        /**
         * Un etat et le jalon avant lequel il vaut, nomme explicitement plutot que deduit
         * du rang dans l'enumeration : ecrite par positions, la correspondance devenait
         * fausse en silence des que l'on reordonnait l'une des deux listes.
         */
        private val steps: List<Pair<Stage, (DryingEstimate) -> Duration>> = listOf(
            Stage.OPEN to { it.openTime },
            Stage.SETTING to { it.touchDry },
            Stage.TOUCH_DRY to { it.recoat },
            Stage.RECOATABLE to { it.throughDry },
            Stage.THROUGH_DRY to { it.fullCure },
        )

        private fun stageOf(elapsed: Duration, estimate: DryingEstimate): Stage {
            var previous = Duration.ZERO
            for ((stage, milestone) in steps) {
                // Les jalons sont croissants avec les reglages livres, mais les facteurs
                // se configurent : un reglage qui les croise ferait sauter un etat.
                val bound = milestone(estimate).coerceAtLeast(previous)
                if (elapsed < bound) return stage
                previous = bound
            }
            return Stage.CURED
        }
    }
}
