package be.asmolabs.palettier.data

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import be.asmolabs.palettier.db.Paint as PaintRow

/**
 * Le catalogue, range dans SQLite.
 *
 * <p>Les pigments vivent dans une table a part : un tube en porte de zero a quatre, et
 * les coller dans une colonne separee par des virgules interdirait toute recherche par
 * pigment -- qui est precisement ce qui donne la vitesse de sechage.</p>
 *
 * <p>Ils sont relus en une requete pour tout le catalogue, puis regroupes en memoire.
 * Une requete par tube ferait sept cents allers-retours pour afficher une liste.</p>
 */
class SqlDelightPaintCatalogRepository(
    private val database: PalettierDatabase,
    private val io: CoroutineDispatcher,
) : PaintCatalogRepository {

    private val queries get() = database.paintQueries

    override fun observeAll(): Flow<List<Paint>> =
        queries.selectAll().asFlow().mapToList(io).map { rows -> withPigments(rows) }

    override suspend fun all(): List<Paint> = withContext(io) {
        withPigments(queries.selectAll().executeAsList())
    }

    override suspend fun inStock(): List<Paint> = withContext(io) {
        withPigments(queries.selectInStock().executeAsList())
    }

    override suspend fun findById(id: Long): Paint? = withContext(io) {
        queries.selectById(id).executeAsOneOrNull()?.let { row ->
            row.toDomain(queries.pigmentsOf(id).executeAsList().toSet())
        }
    }

    override suspend fun save(paint: Paint): Paint = withContext(io) {
        database.transactionWithResult {
            queries.insert(
                brand = paint.brand,
                name = paint.name,
                code = paint.code,
                legacy_code = paint.legacyCode,
                hex_color = paint.hexColor,
                tint_hex = paint.tintHex,
                opacity = paint.opacity.name,
                drying_class = paint.dryingClass.name,
                tinting_strength = paint.tintingStrength,
                colour_derived = paint.colorDerived.toLong(),
                pigments_verified = paint.pigmentsVerified.toLong(),
                user_added = paint.userAdded.toLong(),
                in_stock = paint.inStock.toLong(),
                notes = paint.notes,
            )
            val id = queries.lastInsertedId().executeAsOne()
            paint.pigments.forEachIndexed { position, pigment ->
                queries.addPigment(id, pigment, position.toLong())
            }
            paint.copy(id = id)
        }
    }

    override suspend fun setOwned(paint: Paint, owned: Boolean): Unit = withContext(io) {
        queries.setInStock(owned.toLong(), requireNotNull(paint.id) { "Tube sans identifiant" })
    }

    override suspend fun recordTint(paint: Paint, tint: Rgb): Unit = withContext(io) {
        queries.setTint(tint.toHex(), requireNotNull(paint.id) { "Tube sans identifiant" })
    }

    override suspend fun countOwned(): Long = withContext(io) {
        queries.selectInStock().executeAsList().size.toLong()
    }

    /** Les pigments de tout le catalogue en une requete, regroupes par tube. */
    private fun withPigments(rows: List<PaintRow>): List<Paint> {
        val byPaint = queries.allPigments().executeAsList()
            .groupBy({ it.paint_id }, { it.pigment })
        return rows.map { row -> row.toDomain(byPaint[row.id].orEmpty().toSet()) }
    }

    private fun PaintRow.toDomain(pigments: Set<String>) = Paint(
        id = id,
        brand = brand,
        name = name,
        code = code,
        legacyCode = legacy_code,
        pigments = pigments,
        hexColor = hex_color,
        tintHex = tint_hex,
        opacity = Opacity.valueOf(opacity),
        dryingClass = DryingClass.valueOf(drying_class),
        tintingStrength = tinting_strength,
        colorDerived = colour_derived.toBoolean(),
        pigmentsVerified = pigments_verified.toBoolean(),
        userAdded = user_added.toBoolean(),
        inStock = in_stock.toBoolean(),
        notes = notes,
    )
}

// SQLite ne connait pas le booleen : il stocke 0 ou 1. La conversion est faite ici, une
// fois, plutot que dispersee dans chaque requete.
internal fun Boolean.toLong(): Long = if (this) 1L else 0L
internal fun Long.toBoolean(): Boolean = this != 0L
