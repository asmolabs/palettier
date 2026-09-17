package be.asmolabs.palettier.data

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
    val driver = JdbcSqliteDriver("jdbc:sqlite:${directory.resolve("palettier.db")}")
    PalettierDatabase.Schema.create(driver)
    return driver
}

/** Base en memoire, pour les essais. */
fun inMemoryDriver(): SqlDriver =
    JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY).also { PalettierDatabase.Schema.create(it) }
