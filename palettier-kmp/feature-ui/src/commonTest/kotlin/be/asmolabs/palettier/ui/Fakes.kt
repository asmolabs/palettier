package be.asmolabs.palettier.ui

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow

internal fun testPaint(
    id: Long,
    name: String,
    hex: String,
    brand: String = "W&N",
    inStock: Boolean = true,
    drying: DryingClass = DryingClass.MEDIUM,
    opacity: Opacity = Opacity.SEMI_OPAQUE,
    pigments: Set<String> = emptySet(),
    tinting: Double = 0.8,
) = Paint(
    id = id, brand = brand, name = name, hexColor = hex, inStock = inStock,
    opacity = opacity, dryingClass = drying, tintingStrength = tinting, pigments = pigments,
)

internal class FakeCatalogRepository(paints: List<Paint>) : PaintCatalogRepository {
    val paints = MutableStateFlow(paints)
    override fun observeAll(): Flow<List<Paint>> = this.paints
    override suspend fun all() = paints.value
    override suspend fun inStock() = paints.value.filter { it.inStock }
    override suspend fun findById(id: Long) = paints.value.firstOrNull { it.id == id }
    override suspend fun findByNaturalKey(brand: String, name: String) =
        paints.value.firstOrNull { it.brand == brand && it.name == name }
    override suspend fun save(paint: Paint) = paint
    override suspend fun saveAll(paints: List<Paint>) {
        this.paints.value = this.paints.value + paints
    }
    override suspend fun setOwned(paint: Paint, owned: Boolean) {
        paints.value = paints.value.map { if (it.id == paint.id) it.copy(inStock = owned) else it }
    }
    override suspend fun recordTint(paint: Paint, tint: Rgb) {}
    override suspend fun countOwned() = paints.value.count { it.inStock }.toLong()
    override suspend fun count() = paints.value.size.toLong()
    override suspend fun refreshIdentity(paint: Paint) {
        paints.value = paints.value.map { if (it.id == paint.id) paint else it }
    }
    override suspend fun idsInUse(): Set<Long> = emptySet()
    override suspend fun delete(paint: Paint) {
        paints.value = paints.value.filterNot { it.id == paint.id }
    }
}

internal class FakePaletteRepository(initial: List<Palette> = emptyList()) : PaletteRepository {
    val palettes = MutableStateFlow(initial)
    private var nextId = (initial.mapNotNull { it.id }.maxOrNull() ?: 0L) + 1

    override fun observeAll(): Flow<List<Palette>> = palettes
    override suspend fun all() = palettes.value
    override suspend fun findById(id: Long) = palettes.value.firstOrNull { it.id == id }
    override suspend fun save(palette: Palette): Palette {
        val saved = palette.id?.let { palette } ?: palette.copy(id = nextId++)
        palettes.value = if (palettes.value.any { it.id == saved.id }) {
            palettes.value.map { if (it.id == saved.id) saved else it }
        } else {
            palettes.value + saved
        }
        return saved
    }
    override suspend fun delete(palette: Palette) {
        palettes.value = palettes.value.filterNot { it.id == palette.id }
    }
}

/** Un depot de projets en memoire : on veut voir ce qui s'y enregistre. */
internal class FakeProjectRepository(
    initial: List<be.asmolabs.palettier.domain.project.Project> = emptyList(),
) : be.asmolabs.palettier.domain.port.ProjectRepository {

    val saved = MutableStateFlow(initial)
    private var nextId = (initial.mapNotNull { it.id }.maxOrNull() ?: 0L) + 1

    override fun observeAll(): Flow<List<be.asmolabs.palettier.domain.project.Project>> = saved
    override suspend fun all() = saved.value
    override suspend fun findWithPhotos(id: Long) = saved.value.firstOrNull { it.id == id }

    override suspend fun save(
        project: be.asmolabs.palettier.domain.project.Project,
    ): be.asmolabs.palettier.domain.project.Project {
        val stored = project.copy(id = project.id ?: nextId++)
        saved.value = saved.value.filterNot { it.id == stored.id } + stored
        return stored
    }

    override suspend fun delete(project: be.asmolabs.palettier.domain.project.Project) {
        saved.value = saved.value.filterNot { it.id == project.id }
    }

    override suspend fun addPhoto(
        project: be.asmolabs.palettier.domain.project.Project,
        photo: be.asmolabs.palettier.domain.project.ProjectPhoto,
    ): be.asmolabs.palettier.domain.project.Project {
        val grown = project.copy(photos = project.photos + photo.copy(id = nextId++))
        saved.value = saved.value.map { if (it.id == grown.id) grown else it }
        return grown
    }

    override suspend fun removePhoto(
        project: be.asmolabs.palettier.domain.project.Project,
        photoId: Long,
    ): be.asmolabs.palettier.domain.project.Project {
        val shrunk = project.copy(photos = project.photos.filterNot { it.id == photoId })
        saved.value = saved.value.map { if (it.id == shrunk.id) shrunk else it }
        return shrunk
    }
}
