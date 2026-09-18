package be.asmolabs.palettier.data

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Ventilation
import be.asmolabs.palettier.domain.palette.PaletteMix
import be.asmolabs.palettier.domain.port.PaletteMixRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import kotlin.time.Instant
import be.asmolabs.palettier.db.Palette_mix as MixRow

/** Les melanges poses sur la palette, avec les conditions figees a leur preparation. */
class SqlDelightPaletteMixRepository(
    private val database: PalettierDatabase,
    private val io: CoroutineDispatcher,
) : PaletteMixRepository {

    private val queries get() = database.paletteMixQueries

    override fun observeAll(): Flow<List<PaletteMix>> =
        queries.selectAll().asFlow().mapToList(io).map { rows -> rows.map { it.toDomain() } }

    override suspend fun all(): List<PaletteMix> = withContext(io) {
        queries.selectAll().executeAsList().map { it.toDomain() }
    }

    override suspend fun save(mix: PaletteMix): PaletteMix = withContext(io) {
        database.transactionWithResult {
            queries.insert(
                mix.name, mix.hexColor, mix.recipe, mix.dryingClass.name,
                mix.mixedAt.toEpochMilliseconds(),
                mix.workshop.temperatureCelsius, mix.workshop.relativeHumidity,
                mix.workshop.ventilation.name,
            )
            mix.copy(id = queries.lastInsertedId().executeAsOne())
        }
    }

    override suspend fun delete(id: Long): Unit = withContext(io) {
        queries.delete(id)
    }

    private fun MixRow.toDomain() = PaletteMix(
        id = id,
        name = name,
        hexColor = hex_color,
        recipe = recipe,
        dryingClass = DryingClass.valueOf(drying_class),
        mixedAt = Instant.fromEpochMilliseconds(mixed_at),
        workshop = Workshop(temperature, humidity, Ventilation.valueOf(ventilation)),
    )
}
