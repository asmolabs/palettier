package be.asmolabs.palettier.data.di

import android.content.Context
import app.cash.sqldelight.db.SqlDriver
import app.cash.sqldelight.driver.android.AndroidSqliteDriver
import be.asmolabs.palettier.db.PalettierDatabase
import kotlinx.coroutines.Dispatchers
import org.koin.core.module.Module
import org.koin.dsl.module

/**
 * Le pilote Android.
 *
 * <p>Le Context n'est pas cree ici mais attendu du graphe : c'est le module applicatif
 * qui le declare, parce que lui seul le possede. Cela evite a core-data de dependre de
 * koin-android pour une seule resolution.</p>
 */
actual val platformModule: Module = module {
    single<SqlDriver> { AndroidSqliteDriver(PalettierDatabase.Schema, get<Context>(), "palettier.db") }
    single(IO) { Dispatchers.IO }
}
