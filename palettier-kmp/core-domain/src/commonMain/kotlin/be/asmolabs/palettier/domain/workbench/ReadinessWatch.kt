package be.asmolabs.palettier.domain.workbench

/**
 * Ce qui vient de se liberer depuis le dernier regard.
 *
 * <p>Une zone recouvrable ne l'annonce qu'une fois. Sans cette memoire, une surveillance
 * qui repasse toutes les cinq minutes repeterait indefiniment que la cape est prete, et
 * l'on cesserait de la lire -- c'est le defaut qui rend une alerte inutile.</p>
 *
 * <p>Le premier passage ne signale rien. En ouvrant l'application, le peintre n'attend
 * pas la liste de tout ce qui a seche pendant son absence : il la voit a l'ecran. Une
 * alerte n'a de sens que pour ce qui bascule sous ses yeux.</p>
 */
class ReadinessWatch {

    /** Une zone qui vient de se liberer, telle qu'on l'annonce. */
    data class Freed(val piece: String, val zone: String) {
        fun label(): String = "$piece - $zone"
    }

    private var ready: Set<String> = emptySet()
    private var primed = false

    /**
     * Les zones devenues disponibles depuis l'appel precedent.
     *
     * @return la liste, vide au premier appel comme lorsque rien n'a bouge
     */
    fun newlyReady(bench: Bench): List<Freed> {
        val now = LinkedHashSet<String>()
        val freed = mutableListOf<Freed>()

        for (piece in bench.pieces) {
            for (zone in piece.readyZones) {
                val key = "${piece.project.id}/${zone.name}"
                now += key
                if (primed && key !in ready) freed += Freed(piece.project.name, zone.name)
            }
        }

        ready = now
        val first = !primed
        primed = true
        return if (first) emptyList() else freed.toList()
    }

    /** Oublie ce qui a ete vu. Le prochain regard repart comme au premier jour. */
    fun reset() {
        ready = emptySet()
        primed = false
    }
}
