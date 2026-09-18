package be.asmolabs.palettier.domain.workbench

import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectZone
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class ReadinessWatchTest {

    private val now = Instant.parse("2026-03-01T10:00:00Z")
    private val service = WorkbenchService()
    private val watch = ReadinessWatch()

    private fun piece(name: String, applied: Boolean) = Project(
        id = 1, name = name,
        zones = listOf(
            ProjectZone(
                name = "Visage",
                layers = listOf(
                    ProjectLayer(role = "Ombre 1", targetHex = "#8A5F4A", technique = "Glacis")
                        .let { if (applied) it.markApplied(now, Workshop.standard(), DryingClass.FAST) else it },
                    ProjectLayer(role = "Base", targetHex = "#C98F72", technique = "Glacis"),
                ),
            )
        ),
    )

    private fun benchAt(project: Project, at: Instant) = service.bench(listOf(project), at)

    @Test
    fun `le premier regard n'annonce rien, tout est deja a l'ecran`() {
        assertTrue(watch.newlyReady(benchAt(piece("Rien de pose", applied = false), now)).isEmpty())
    }

    @Test
    fun `une zone qui se libere est annoncee une fois, puis se tait`() {
        val project = piece("Visage frais", applied = true)

        // La couche est fraiche : rien de disponible, et c'est le passage qui amorce.
        assertTrue(watch.newlyReady(benchAt(project, now + 20.minutes)).isEmpty())
        assertTrue(watch.newlyReady(benchAt(project, now + 20.minutes)).isEmpty())

        // Le temps passe : la zone bascule, et on l'apprend.
        assertEquals(
            listOf("Visage frais - Visage"),
            watch.newlyReady(benchAt(project, now + 30.days)).map { it.label() },
        )

        // Elle reste disponible, mais on ne le repete pas : une alerte qui se repete
        // cesse d'etre lue.
        assertTrue(watch.newlyReady(benchAt(project, now + 30.days)).isEmpty())
    }

    @Test
    fun `oublier ce qui a ete vu remet la surveillance a zero`() {
        val project = piece("A oublier", applied = true)
        watch.newlyReady(benchAt(project, now + 20.minutes))
        assertTrue(watch.newlyReady(benchAt(project, now + 30.days)).isNotEmpty())

        watch.reset()
        assertTrue(watch.newlyReady(benchAt(project, now + 30.days)).isEmpty(), "le regard qui reamorce")
        assertTrue(watch.newlyReady(benchAt(project, now + 30.days)).isEmpty())
    }
}
