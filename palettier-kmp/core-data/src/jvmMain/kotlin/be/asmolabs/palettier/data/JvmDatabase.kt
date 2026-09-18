package be.asmolabs.palettier.data

import app.cash.sqldelight.db.QueryResult
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import be.asmolabs.palettier.db.PalettierDatabase
import java.nio.file.Files
import java.nio.file.Path

/**
 * La base du poste, sous ~/.palettier comme du temps de H2.
 *
 * <p>Le chemin est garde a l'identique : le peintre n'a pas a savoir que le moteur a
 * change, et ses sauvegardes sont deja rangees a cote.</p>
 */
fun desktopDriver(directory: Path = Path.of(System.getProperty("user.home"), ".palettier")): SqlDriver {
    Files.createDirectories(directory)
    return JdbcSqliteDriver("jdbc:sqlite:${directory.resolve("palettier.db")}").prepared()
}

/** Base en memoire, pour les essais. */
fun inMemoryDriver(): SqlDriver = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY).prepared()

/**
 * Cree le schema, ou le fait avancer, selon ce que la base contient deja.
 *
 * <p>Creer sans regarder marche une fois et echoue au deuxieme demarrage sur "table paint
 * already exists" -- l'application ne s'ouvre alors plus du tout. La version vit dans le
 * pragma user_version de SQLite, qui est fait pour ca, et c'est ce qui prend la suite de
 * Liquibase : un schema neuf est cree, un schema ancien est migre, un schema a jour est
 * laisse tranquille.</p>
 */
private fun SqlDriver.prepared(): SqlDriver {
    val target = PalettierDatabase.Schema.version
    val current = executeQuery(
        identifier = null,
        sql = "PRAGMA user_version;",
        mapper = { cursor ->
            cursor.next()
            QueryResult.Value(cursor.getLong(0) ?: 0L)
        },
        parameters = 0,
    ).value

    when {
        current == 0L -> PalettierDatabase.Schema.create(this)
        current < target -> PalettierDatabase.Schema.migrate(this, current, target)
        else -> return this
    }
    execute(null, "PRAGMA user_version = $target;", 0)
    return this
}
