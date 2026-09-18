package be.asmolabs.palettier.ui.workbench

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.palette.PaletteMix
import be.asmolabs.palettier.domain.palette.PaletteMixService
import be.asmolabs.palettier.domain.port.PaletteMixRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.project.ProjectZone
import be.asmolabs.palettier.domain.workbench.WorkbenchService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Clock
import kotlin.time.Duration.Companion.days
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

/**
 * Le comportement reactif de l'ecran, sans ecran.
 *
 * <p>C'est la partie de l'interface qui se prouve : ce qui doit apparaitre quand la base
 * change, et quand le temps passe. Le reste -- la mise en page, la lisibilite au fond d'un
 * atelier -- ne se verifie pas ici.</p>
 */
@OptIn(ExperimentalCoroutinesApi::class)
class WorkbenchViewModelTest {

    private val posed = Instant.parse("2026-03-01T10:00:00Z")

    /**
     * viewModelScope tourne sur le repartiteur principal, qui n'existe pas dans un essai
     * ordinaire. Sans ce branchement, le flux partage ne demarre jamais et l'ecran reste
     * en chargement -- ce qui ressemble a un bug du ViewModel et n'en est pas un.
     */
    private val dispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setUp() = Dispatchers.setMain(dispatcher)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    /** Une horloge qu'on avance a la main : le temps est un parametre, pas une fatalite. */
    private class FixedClock(var instant: Instant) : Clock {
        override fun now() = instant
    }

    private class FakeProjects(initial: List<Project> = emptyList()) : ProjectRepository {
        val projects = MutableStateFlow(initial)
        override fun observeAll(): Flow<List<Project>> = projects
        override suspend fun all() = projects.value
        override suspend fun findWithPhotos(id: Long) = projects.value.firstOrNull { it.id == id }
        override suspend fun save(project: Project) = project.also { saved ->
            projects.value = projects.value.filterNot { it.id == saved.id } + saved
        }
        override suspend fun addPhoto(project: Project, photo: ProjectPhoto) = project
        override suspend fun removePhoto(project: Project, photoId: Long) = project
        override suspend fun delete(project: Project) {
            projects.value = projects.value - project
        }
    }

    private class FakeMixes : PaletteMixRepository {
        val mixes = MutableStateFlow<List<PaletteMix>>(emptyList())
        private var nextId = 1L
        override fun observeAll(): Flow<List<PaletteMix>> = mixes
        override suspend fun all() = mixes.value
        override suspend fun save(mix: PaletteMix) = mix.copy(id = nextId++).also { mixes.value = mixes.value + it }
        override suspend fun delete(id: Long) {
            mixes.value = mixes.value.filterNot { it.id == id }
        }
    }

    private fun piece(name: String, applied: Boolean) = Project(
        id = 1, name = name,
        zones = listOf(
            ProjectZone(
                name = "Visage",
                layers = listOf(
                    ProjectLayer(role = "Ombre 1", targetHex = "#8A5F4A", technique = "Glacis")
                        .let { if (applied) it.markApplied(posed, Workshop.standard(), DryingClass.FAST) else it },
                    ProjectLayer(role = "Base", targetHex = "#C98F72", technique = "Glacis"),
                ),
            )
        ),
    )

    private fun viewModel(
        projects: FakeProjects,
        mixes: FakeMixes,
        clock: FixedClock,
    ) = WorkbenchViewModel(
        projects = projects,
        mixesRepository = mixes,
        workbench = WorkbenchService(),
        mixes = PaletteMixService(),
        clock = clock,
    )

    @Test
    fun `l'etabli arrive sans qu'on le demande`() = runTest(dispatcher) {
        val projects = FakeProjects(listOf(piece("Grognard", applied = false)))
        val model = viewModel(projects, FakeMixes(), FixedClock(posed))

        val state = model.state.first { !it.loading }

        assertEquals(1, state.bench.pieces.size)
        assertTrue(state.bench.pieces.single().readyNow)
    }

    @Test
    fun `une ecriture en base rafraichit l'ecran, sans que personne ne relie les deux`() = runTest(dispatcher) {
        val projects = FakeProjects()
        val model = viewModel(projects, FakeMixes(), FixedClock(posed))

        assertTrue(model.state.first { !it.loading }.bench.pieces.isEmpty())

        // C'est tout l'interet du Flow : l'ecran Projets ecrit, celui-ci se met a jour.
        projects.save(piece("Arrivee en cours de route", applied = false))

        assertEquals(1, model.state.first { it.bench.pieces.isNotEmpty() }.bench.pieces.size)
    }

    @Test
    fun `le temps qui passe suffit a changer l'etat, sans aucune ecriture`() = runTest(dispatcher) {
        val clock = FixedClock(posed)
        val projects = FakeProjects(listOf(piece("Visage frais", applied = true)))
        val model = viewModel(projects, FakeMixes(), clock)

        // Vingt minutes apres la pose : la zone est encore prise.
        clock.instant = posed + 20.minutes
        assertTrue(!model.state.first { !it.loading }.bench.pieces.single().readyNow)

        // Un mois plus tard, sans la moindre ecriture en base, elle est reprenable.
        clock.instant = posed + 30.days
        advanceTimeBy(2.minutes)

        assertTrue(model.state.first { it.bench.pieces.single().readyNow }.bench.pieces.single().readyNow)
    }

    @Test
    fun `un melange se pose avec la teinte relevee, et pas sans elle`() = runTest(dispatcher) {
        val mixes = FakeMixes()
        val model = viewModel(FakeProjects(), mixes, FixedClock(posed))
        model.state.first { !it.loading }

        // Sans teinte relevee, la demande n'aboutit pas : on ne devine pas une couleur.
        model.onIntent(WorkbenchIntent.RecordMix("Gris rompu", "", DryingClass.MEDIUM))
        advanceUntilIdle()
        assertTrue(mixes.mixes.value.isEmpty())

        model.sample(Rgb.ofHex("#6B6259"))
        // Une demande lance une coroutine : l'essai doit la laisser finir, comme l'ecran
        // laisse le temps a l'action de s'executer.
        model.onIntent(WorkbenchIntent.RecordMix("Gris rompu", "2 parts de terre + 1 de blanc", DryingClass.MEDIUM))
        advanceUntilIdle()

        val posedMix = mixes.mixes.value.single()
        assertEquals("Gris rompu", posedMix.name)
        assertEquals("#6B6259", posedMix.hexColor)
        assertEquals(posed, posedMix.mixedAt)
    }

    @Test
    fun `le menage ne retire que ce qui a pris`() = runTest(dispatcher) {
        val clock = FixedClock(posed)
        val mixes = FakeMixes()
        val model = viewModel(FakeProjects(), mixes, clock)
        model.state.first { !it.loading }

        model.sample(Rgb.ofHex("#6B6259"))
        model.onIntent(WorkbenchIntent.RecordMix("Frais", "", DryingClass.MEDIUM))
        advanceUntilIdle()
        assertEquals(1, mixes.mixes.value.size)

        model.onIntent(WorkbenchIntent.CleanSpentMixes)
        advanceUntilIdle()
        assertEquals(1, mixes.mixes.value.size, "rien n'a pris a l'instant meme")

        clock.instant = posed + 21.days
        model.onIntent(WorkbenchIntent.CleanSpentMixes)
        advanceUntilIdle()
        assertTrue(mixes.mixes.value.isEmpty(), "trois semaines plus tard, il n'a plus rien a faire la")
    }
}
