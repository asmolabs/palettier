package be.asmolabs.palettier.data

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import be.asmolabs.palettier.db.Palette as PaletteRow

/** Les palettes, et les tubes qu'elles rassemblent. */
class SqlDelightPaletteRepository(
    private val database: PalettierDatabase,
    private val catalog: PaintCatalogRepository,
    private val io: CoroutineDispatcher,
) : PaletteRepository {

    private val queries get() = database.paletteQueries

    override fun observeAll(): Flow<List<Palette>> =
        queries.selectAll().asFlow().mapToList(io).map { rows -> assemble(rows, catalog.all()) }

    override suspend fun all(): List<Palette> = withContext(io) {
        assemble(queries.selectAll().executeAsList(), catalog.all())
    }

    override suspend fun findById(id: Long): Palette? = withContext(io) {
        queries.selectById(id).executeAsOneOrNull()
            ?.let { assemble(listOf(it), catalog.all()).firstOrNull() }
    }

    override suspend fun save(palette: Palette): Palette = withContext(io) {
        database.transactionWithResult {
            val id = palette.id?.also { queries.update(palette.name, palette.purpose, palette.notes, it) }
                ?: run {
                    queries.insert(palette.name, palette.purpose, palette.notes)
                    queries.lastInsertedId().executeAsOne()
                }
            queries.clearPaints(id)
            palette.paints.forEachIndexed { position, paint ->
                paint.id?.let { queries.addPaint(id, it, position.toLong()) }
            }
            palette.copy(id = id)
        }
    }

    override suspend fun delete(palette: Palette): Unit = withContext(io) {
        palette.id?.let { queries.delete(it) }
    }

    /**
     * Assemble les palettes et leurs tubes en trois requetes, et non une par palette.
     * Le catalogue est lu une fois puis indexe : c'est la meme donnee pour toutes.
     */
    private fun assemble(rows: List<PaletteRow>, catalogue: List<Paint>): List<Palette> {
        val byId = catalogue.associateBy { it.id }
        val links = queries.allPaints().executeAsList().groupBy({ it.palette_id }, { it.paint_id })
        return rows.map { row ->
            Palette(
                id = row.id,
                name = row.name,
                purpose = row.purpose,
                notes = row.notes,
                paints = links[row.id].orEmpty().mapNotNull { byId[it] },
            )
        }
    }
}
