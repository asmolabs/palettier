package be.asmolabs.palettier.data

import app.cash.sqldelight.db.QueryResult
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.jdbc.sqlite.JdbcSqliteDriver
import be.asmolabs.palettier.db.PalettierDatabase
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.deleteIfExists
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Migrer la base d'hier doit donner la base d'aujourd'hui.
 *
 * <p>C'est la garantie que Liquibase ne donnait pas : il jouait les changements sans
 * jamais verifier qu'ils menaient au schema decrit. Ici, le schema de chaque version est
 * garde en fichier, et l'on compare ce que les migrations en font avec ce qu'une creation
 * neuve produit.</p>
 *
 * <p>La tache Gradle prevue pour cela epuise la memoire ; ce test la remplace et rend le
 * meme service, en une seconde.</p>
 */
class MigrationTest {

    /** Le schema tel qu'il etait, copie depuis les ressources : on ne travaille pas dessus. */
    private fun schemaOfVersion(version: Int): Path {
        val file = Files.createTempFile("palettier-v$version-", ".db")
        MigrationTest::class.java.getResourceAsStream("/schema-v$version.db")!!
            .use { source -> Files.newOutputStream(file).use { source.copyTo(it) } }
        return file
    }

    private fun SqlDriver.tables(): Map<String, Set<String>> {
        val names = mutableListOf<String>()
        executeQuery(
            null,
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name;",
            { cursor ->
                while (cursor.next().value) names += cursor.getString(0)!!
                QueryResult.Unit
            },
            0,
        )
        return names.associateWith { table ->
            val columns = mutableSetOf<String>()
            executeQuery(
                null,
                "SELECT name, type, \"notnull\" FROM pragma_table_info('$table') ORDER BY name;",
                { cursor ->
                    while (cursor.next().value) {
                        columns += "${cursor.getString(0)} ${cursor.getString(1)} ${cursor.getLong(2)}"
                    }
                    QueryResult.Unit
                },
                0,
            )
            columns
        }
    }

    @Test
    fun `le schema migre depuis la version 1 est celui d'une creation neuve`() {
        val ancien = schemaOfVersion(1)
        try {
            val migre = JdbcSqliteDriver("jdbc:sqlite:$ancien")
            PalettierDatabase.Schema.migrate(migre, 1, PalettierDatabase.Schema.version)

            val neuf = JdbcSqliteDriver(JdbcSqliteDriver.IN_MEMORY)
            PalettierDatabase.Schema.create(neuf)

            assertEquals(neuf.tables(), migre.tables())
        } finally {
            ancien.deleteIfExists()
        }
    }

    @Test
    fun `la migration conserve les donnees deja saisies`() {
        val ancien = schemaOfVersion(1)
        try {
            val driver = JdbcSqliteDriver("jdbc:sqlite:$ancien")
            driver.execute(
                null,
                "INSERT INTO paint(brand, name, code, legacy_code, hex_color, tint_hex, opacity, " +
                    "drying_class, tinting_strength, colour_derived, pigments_verified, user_added, " +
                    "in_stock, notes) VALUES ('Gamblin', 'Ivory Black', '', '', '#221F1C', NULL, " +
                    "'SEMI_OPAQUE', 'VERY_SLOW', 0.5, 0, 1, 0, 1, '');",
                0,
            )

            PalettierDatabase.Schema.migrate(driver, 1, PalettierDatabase.Schema.version)

            var found = 0L
            driver.executeQuery(null, "SELECT count(*) FROM paint;", { cursor ->
                cursor.next()
                found = cursor.getLong(0)!!
                QueryResult.Unit
            }, 0)
            assertEquals(1L, found)
        } finally {
            ancien.deleteIfExists()
        }
    }

    @Test
    fun `la version du schema a bien avance`() {
        assertTrue(
            PalettierDatabase.Schema.version > 1,
            "un .sqm a ete ajoute : la version doit suivre",
        )
    }
}
