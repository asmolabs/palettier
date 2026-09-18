package be.asmolabs.palettier.ui.projects

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.plan.ProjectPlanner
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectPhoto
import be.asmolabs.palettier.domain.project.ProjectZone
import be.asmolabs.palettier.domain.project.SubstituteService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.time.Clock
import kotlin.time.Instant

@OptIn(ExperimentalCoroutinesApi::class)
class ProjectsViewModelTest {

    private val now = Instant.parse("2026-03-01T10:00:00Z")
    private val dispatcher = StandardTestDispatcher()

    @BeforeTest fun setUp() = Dispatchers.setMain(dispatcher)
    @AfterTest fun tearDown() = Dispatchers.resetMain()

    private class FixedClock(val instant: Instant) : Clock {
        override fun now() = instant
    }

    private fun paint(id: Long, name: String, hex: String, inStock: Boolean = true) = Paint(
        id = id, brand = "W&N", name = name, hexColor = hex, inStock = inStock,
        opacity = Opacity.SEMI_OPAQUE, dryingClass = DryingClass.SLOW, tintingStrength = 0.8,
    )

    private val white = paint(1, "Titanium White", "#F4F2EC")
    private val umber = paint(2, "Burnt Umber", "#4A3427")
    private val absent = paint(3, "Cadmium Red", "#B22222", inStock = false)

    private class FakeProjects(initial: List<Project>) : ProjectRepository {
        val projects = MutableStateFlow(initial)
        override fun observeAll(): Flow<List<Project>> = projects
        override suspend fun all() = projects.value
        override suspend fun findWithPhotos(id: Long) = projects.value.firstOrNull { it.id == id }
        override suspend fun save(project: Project) = project.also { saved ->
            projects.value = projects.value.map { if (it.id == saved.id) saved else it }
        }
        override suspend fun addPhoto(project: Project, photo: ProjectPhoto) = project
        override suspend fun removePhoto(project: Project, photoId: Long) = project
        override suspend fun delete(project: Project) { projects.value = projects.value - project }
    }

    private class FakeCatalog(private val paints: List<Paint>) : PaintCatalogRepository {
        override fun observeAll(): Flow<List<Paint>> = MutableStateFlow(paints)
        override suspend fun all() = paints
        override suspend fun inStock() = paints.filter { it.inStock }
        override suspend fun findById(id: Long) = paints.firstOrNull { it.id == id }
        override suspend fun findByNaturalKey(brand: String, name: String) = null
        override suspend fun save(paint: Paint) = paint
        override suspend fun setOwned(paint: Paint, owned: Boolean) {}
        override suspend fun recordTint(paint: Paint, tint: Rgb) {}
        override suspend fun countOwned() = paints.count { it.inStock }.toLong()
    }

    private fun piece() = Project(
        id = 1, name = "Grognard", subject = "Buste",
        paints = listOf(white, umber, absent),
        zones = listOf(
            ProjectZone(
                name = "Visage", material = "Peau",
                layers = listOf(
                    ProjectLayer(role = "Ombre 1", targetHex = "#8A5F4A", technique = "Glacis"),
                    ProjectLayer(role = "Base", targetHex = "#C98F72", technique = "Glacis"),
                    ProjectLayer(
                        role = "Rougeur des pommettes", targetHex = "#C97A62",
                        technique = "Glacis", kind = ProjectLayer.Kind.ACCENT,
                    ),
                ),
            )
        ),
    )

    private fun viewModel(projects: FakeProjects) = ProjectsViewModel(
        projects = projects,
        catalog = FakeCatalog(listOf(white, umber, absent)),
        planner = ProjectPlanner(ColorMixService()),
        fatOverLean = FatOverLeanService(),
        substitutes = SubstituteService(),
        clock = FixedClock(now),
        workshop = { Workshop(24.0, 40.0, be.asmolabs.palettier.domain.paint.Ventilation.GOOD) },
    )

    @Test
    fun `choisir une piece calcule son plan, variation locale comprise`() = runTest(dispatcher) {
        val projects = FakeProjects(listOf(piece()))
        val model = viewModel(projects)
        // Un ecran ouvert collecte en permanence. Sans cela WhileSubscribed coupe le
        // flux entre deux assertions, et l'etat cesse d'avancer -- ce qui ressemble a un
        // bug du ViewModel et n'en est pas un.
        backgroundScope.launch { model.state.collect { } }
        model.state.first { !it.loading }

        model.onIntent(ProjectsIntent.Select(piece()))
        advanceUntilIdle()

        val zone = model.state.value.zones.single()
        assertEquals(listOf("Ombre 1", "Base", "Rougeur des pommettes"), zone.layers.map { it.role })
        // Chaque couche rangee retrouve son melange, la variation locale comprise --
        // c'est elle que le rang d'affichage faisait rater cote JavaFX.
        zone.layers.forEach { assertNotNull(it.mix, "melange pour ${it.role}") }
    }

    @Test
    fun `cocher une couche fige la pose, avec l'atelier et la vitesse du melange`() = runTest(dispatcher) {
        val projects = FakeProjects(listOf(piece()))
        val model = viewModel(projects)
        // Un ecran ouvert collecte en permanence. Sans cela WhileSubscribed coupe le
        // flux entre deux assertions, et l'etat cesse d'avancer -- ce qui ressemble a un
        // bug du ViewModel et n'en est pas un.
        backgroundScope.launch { model.state.collect { } }
        model.state.first { !it.loading }
        model.onIntent(ProjectsIntent.Select(piece()))
        advanceUntilIdle()

        model.onIntent(ProjectsIntent.ToggleApplied(zoneIndex = 0, layerIndex = 1))
        advanceUntilIdle()

        val applied = projects.projects.value.single().zones.single().layers[1].applied
        assertNotNull(applied)
        assertEquals(now, applied.at)
        assertEquals(24.0, applied.workshop.temperatureCelsius)
        assertEquals(be.asmolabs.palettier.domain.paint.Ventilation.GOOD, applied.workshop.ventilation)
        // La vitesse vient des tubes du melange, pas d'une saisie : ce sont des lents.
        assertEquals(DryingClass.SLOW, applied.dryingClass)
    }

    @Test
    fun `decocher remet la couche a peindre`() = runTest(dispatcher) {
        val projects = FakeProjects(listOf(piece()))
        val model = viewModel(projects)
        // Un ecran ouvert collecte en permanence. Sans cela WhileSubscribed coupe le
        // flux entre deux assertions, et l'etat cesse d'avancer -- ce qui ressemble a un
        // bug du ViewModel et n'en est pas un.
        backgroundScope.launch { model.state.collect { } }
        model.state.first { !it.loading }
        model.onIntent(ProjectsIntent.Select(piece()))
        advanceUntilIdle()

        model.onIntent(ProjectsIntent.ToggleApplied(0, 1))
        advanceUntilIdle()
        assertTrue(projects.projects.value.single().zones.single().layers[1].isApplied)

        model.onIntent(ProjectsIntent.ToggleApplied(0, 1))
        advanceUntilIdle()
        assertTrue(!projects.projects.value.single().zones.single().layers[1].isApplied)
    }

    @Test
    fun `ce qui manque a l'etagere est signale avec son remplacant`() = runTest(dispatcher) {
        val projects = FakeProjects(listOf(piece()))
        val model = viewModel(projects)
        // Un ecran ouvert collecte en permanence. Sans cela WhileSubscribed coupe le
        // flux entre deux assertions, et l'etat cesse d'avancer -- ce qui ressemble a un
        // bug du ViewModel et n'en est pas un.
        backgroundScope.launch { model.state.collect { } }
        model.state.first { !it.loading }

        model.onIntent(ProjectsIntent.Select(piece()))
        advanceUntilIdle()

        val missing = model.state.value.missing
        assertEquals(listOf("Cadmium Red"), missing.map { it.paint.name })
        assertNotNull(missing.single().nearest, "un remplacant est propose")
    }
}
