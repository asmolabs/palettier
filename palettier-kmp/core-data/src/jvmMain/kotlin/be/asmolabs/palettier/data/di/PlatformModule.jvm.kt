package be.asmolabs.palettier.data.di

import app.cash.sqldelight.db.SqlDriver
import be.asmolabs.palettier.data.desktopDriver
import kotlinx.coroutines.Dispatchers
import org.koin.core.module.Module
import org.koin.dsl.module

actual val platformModule: Module = module {
    single<SqlDriver> { desktopDriver() }
    single(IO) { Dispatchers.IO }
}
