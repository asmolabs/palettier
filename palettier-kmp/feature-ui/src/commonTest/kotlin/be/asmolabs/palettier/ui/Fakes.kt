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
    override suspend fun setOwned(paint: Paint, owned: Boolean) {
        paints.value = paints.value.map { if (it.id == paint.id) it.copy(inStock = owned) else it }
    }
    override suspend fun recordTint(paint: Paint, tint: Rgb) {}
    override suspend fun countOwned() = paints.value.count { it.inStock }.toLong()
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
