package be.asmolabs.palettier.domain.workbench

import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectZone
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class WorkbenchServiceTest {

    private val now = Instant.parse("2026-03-01T10:00:00Z")
    private val service = WorkbenchService()

    private fun layer(role: String) = ProjectLayer(role = role, targetHex = "#C98F72", technique = "Glacis")

    /** Deux zones de trois couches, rangees [Ombre 1, Base, Lumiere 1]. */
    private fun piece(name: String, id: Long = 1) = Project(
        id = id, name = name, subject = "Buste",
        zones = listOf(
            ProjectZone(name = "Visage", material = "Peau",
                layers = listOf(layer("Ombre 1"), layer("Base"), layer("Lumiere 1"))),
            ProjectZone(name = "Cape", material = "Tissu",
                layers = listOf(layer("Ombre 1"), layer("Base"), layer("Lumiere 1"))),
        ),
    )

    private fun Project.applied(
        zoneIndex: Int,
        layerIndex: Int,
        at: Instant = now,
        workshop: Workshop = Workshop.standard(),
        drying: DryingClass = DryingClass.FAST,
    ): Project = copy(
        zones = zones.mapIndexed { z, zone ->
            if (z != zoneIndex) zone else zone.copy(
                layers = zone.layers.mapIndexed { l, layer ->
                    if (l != layerIndex) layer else layer.markApplied(at, workshop, drying)
                }
            )
        }
    )

    private fun stateOf(project: Project, at: Instant) = service.bench(listOf(project), at).pieces.first()

    @Test
    fun `une piece dont rien n'est peint est disponible tout de suite`() {
        val piece = stateOf(piece("Rien de pose"), now)

        assertTrue(piece.zones.all { it.notStarted })
        assertTrue(piece.readyNow)
        assertNull(piece.nextAvailability)
        assertEquals(0, piece.appliedCoats)
        assertEquals(6, piece.totalCoats)
    }

    @Test
    fun `une couche fraiche met sa zone en attente, sans bloquer les autres`() {
        val project = piece("Visage frais").applied(0, 0, drying = DryingClass.SLOW)

        val piece = stateOf(project, now + 30.minutes)
        val painted = piece.zones.first()
        val untouched = piece.zones[1]

        assertTrue(!painted.readyNow)
        assertEquals(Stage.OPEN, painted.last!!.stage)
        assertTrue(painted.remaining > kotlin.time.Duration.ZERO)
        assertTrue(untouched.readyNow)

        // Il suffit qu'une zone soit libre pour que la piece le soit : pendant que le
        // visage seche, la cape avance.
        assertTrue(piece.readyNow)
    }

    @Test
    fun `passe le delai, la zone redevient disponible et l'annonce`() {
        val project = piece("Visage sec").applied(0, 0).applied(1, 0)

        val wet = stateOf(project, now + 2.hours)
        assertTrue(!wet.readyNow)
        assertTrue(wet.nextAvailability != null)

        val dry = stateOf(project, now + 30.days)
        assertTrue(dry.zones.all { it.readyNow })
        assertEquals(Stage.CURED, dry.zones.first().last!!.stage)
        assertNull(dry.nextAvailability)
    }

    @Test
    fun `le sechage court avec les conditions de la pose, pas avec celles d'aujourd'hui`() {
        val cold = piece("Atelier froid")
            .applied(0, 0, workshop = Workshop(10.0, 80.0, Ventilation.CONFINED), drying = DryingClass.MEDIUM)
        val warm = piece("Atelier chaud")
            .applied(0, 0, workshop = Workshop(28.0, 35.0, Ventilation.GOOD), drying = DryingClass.MEDIUM)

        val later = now + 2.days
        // Meme couche, meme age : seul l'atelier du jour de la pose les separe.
        assertTrue(stateOf(cold, later).zones.first().remaining > stateOf(warm, later).zones.first().remaining)
    }

    @Test
    fun `une zone entierement peinte ne reclame plus rien`() {
        var project = piece("Visage fini")
        repeat(3) { project = project.applied(0, it) }

        val finished = stateOf(project, now + 10.minutes).zones.first()
        assertTrue(finished.done)
        assertTrue(!finished.readyNow)
        assertEquals(kotlin.time.Duration.ZERO, finished.remaining)
        assertNull(finished.last!!.nextRole)
    }

    @Test
    fun `peindre hors de l'ordre du plan n'annonce pas une couche deja recouverte`() {
        val later = now + 30.days

        // On commence par la base : c'est la lumiere qui attend, pas l'ombre restee en
        // arriere. La prendre par simple rang aurait annonce "Ombre 1".
        val afterBase = stateOf(piece("Base d'abord").applied(0, 1), later).zones.first().last!!
        assertEquals("Base", afterBase.role)
        assertEquals("Lumiere 1", afterBase.nextRole)

        // Rien apres la derniere posee : on revient a ce qui manque, hors sequence.
        val afterHighlight = stateOf(piece("Lumiere d'abord").applied(0, 2), later).zones.first().last!!
        assertEquals("Lumiere 1", afterHighlight.role)
        assertEquals("Ombre 1", afterHighlight.nextRole)
    }

    @Test
    fun `l'etabli met en tete ce qui peut etre repris`() {
        val waiting = piece("Tout frais", id = 1)
            .applied(0, 0, drying = DryingClass.VERY_SLOW)
            .applied(1, 0, drying = DryingClass.VERY_SLOW)
        val untouched = piece("Pas commence", id = 2)

        val bench = service.bench(listOf(waiting, untouched), now + 1.hours)

        assertEquals(listOf("Pas commence"), bench.ready().map { it.project.name })
        assertEquals(listOf("Tout frais"), bench.waiting().map { it.project.name })
        assertEquals("Pas commence", bench.pieces.first().project.name)
        assertTrue(bench.nextAvailability() != null)
    }

    @Test
    fun `annuler une pose remet la couche a peindre`() {
        val project = piece("Fausse manoeuvre").applied(0, 0, drying = DryingClass.SLOW)
        assertTrue(!stateOf(project, now).zones.first().notStarted)

        val undone = project.copy(
            zones = project.zones.mapIndexed { z, zone ->
                if (z != 0) zone else zone.copy(layers = zone.layers.mapIndexed { l, layer ->
                    if (l != 0) layer else layer.clearApplied()
                })
            }
        )
        assertTrue(stateOf(undone, now).zones.first().notStarted)
    }
}
