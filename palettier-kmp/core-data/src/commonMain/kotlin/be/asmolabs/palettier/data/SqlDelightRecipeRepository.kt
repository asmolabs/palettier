package be.asmolabs.palettier.data

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.recipe.Recipe
import be.asmolabs.palettier.domain.recipe.RecipeStep
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import be.asmolabs.palettier.db.Recipe as RecipeRow

/** Les recettes et leurs etapes. */
class SqlDelightRecipeRepository(
    private val database: PalettierDatabase,
    private val io: CoroutineDispatcher,
) : RecipeRepository {

    private val queries get() = database.recipeQueries

    override fun observeAll(): Flow<List<Recipe>> =
        queries.selectAll().asFlow().mapToList(io).map(::assemble)

    override suspend fun all(): List<Recipe> = withContext(io) {
        assemble(queries.selectAll().executeAsList())
    }

    override suspend fun exists(name: String): Boolean = withContext(io) {
        queries.existsByName(name).executeAsOne() > 0
    }

    override suspend fun save(recipe: Recipe): Recipe = withContext(io) {
        database.transactionWithResult {
            queries.insert(recipe.name, recipe.subject, recipe.notes)
            val id = queries.lastInsertedId().executeAsOne()
            recipe.steps.forEachIndexed { position, step ->
                queries.insertStep(
                    id, step.technique.name, step.paintMix, step.medium.name,
                    step.mediumRatio, step.thickness.name, step.notes, position.toLong(),
                )
            }
            recipe.copy(id = id)
        }
    }

    override suspend fun delete(recipe: Recipe): Unit = withContext(io) {
        recipe.id?.let { queries.delete(it) }
    }

    private fun assemble(rows: List<RecipeRow>): List<Recipe> {
        val stepsByRecipe = queries.allSteps().executeAsList().groupBy { it.recipe_id }
        return rows.map { row ->
            Recipe(
                id = row.id,
                name = row.name,
                subject = row.subject,
                notes = row.notes,
                steps = stepsByRecipe[row.id].orEmpty().map {
                    RecipeStep(
                        id = it.id,
                        technique = Technique.valueOf(it.technique),
                        paintMix = it.paint_mix,
                        medium = Medium.valueOf(it.medium),
                        mediumRatio = it.medium_ratio,
                        thickness = LayerThickness.valueOf(it.thickness),
                        notes = it.notes,
                    )
                },
            )
        }
    }
}
