package be.asmolabs.palettier.data

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToOneOrNull
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.port.SettingsRepository
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.withContext

/**
 * Les reglages, dans la base plutot que dans un fichier de preferences.
 *
 * <p>La base est deja la, et elle est la meme sur les deux plateformes. Un fichier de
 * preferences ne l'est pas : Android a le sien, le poste de travail un autre.</p>
 */
class SqlDelightSettingsRepository(
    private val database: PalettierDatabase,
    private val io: CoroutineDispatcher,
) : SettingsRepository {

    private val queries get() = database.settingQueries

    override fun observe(key: String): Flow<String?> =
        queries.find(key).asFlow().mapToOneOrNull(io)

    override suspend fun get(key: String): String? = withContext(io) {
        queries.find(key).executeAsOneOrNull()
    }

    override suspend fun put(key: String, value: String) {
        withContext(io) { queries.put(key, value) }
    }

    override suspend fun remove(key: String) {
        withContext(io) { queries.remove(key) }
    }
}
