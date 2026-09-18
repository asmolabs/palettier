package be.asmolabs.palettier.data.di

import app.cash.sqldelight.db.SqlDriver
import be.asmolabs.palettier.data.SqlDelightPaintCatalogRepository
import be.asmolabs.palettier.data.SqlDelightPaletteMixRepository
import be.asmolabs.palettier.data.SqlDelightPaletteRepository
import be.asmolabs.palettier.data.SqlDelightProjectRepository
import be.asmolabs.palettier.data.SqlDelightRecipeRepository
import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.db.PalettierDatabase
import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.PaintMatcher
import be.asmolabs.palettier.domain.palette.PaletteMixService
import be.asmolabs.palettier.domain.plan.PlanDryingService
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteMixRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.port.RecipeRepository
import be.asmolabs.palettier.domain.project.FatOverLeanService
import be.asmolabs.palettier.domain.project.ProgressCheckService
import be.asmolabs.palettier.domain.project.SubstituteService
import be.asmolabs.palettier.domain.workbench.ReadinessWatch
import be.asmolabs.palettier.domain.workbench.WorkbenchService
import org.koin.core.module.Module
import org.koin.core.qualifier.named
import org.koin.dsl.module

/** Le repartiteur des acces disque et base, nomme parce qu'il change selon la plateforme. */
val IO = named("io")

/**
 * Ce que la plateforme seule peut fournir : le pilote SQLite et le repartiteur
 * d'entrees-sorties.
 *
 * <p>Ce n'est pas une precaution theorique. Dispatchers.IO n'existe ni sur Native ni sur
 * Wasm, et chaque plateforme a son propre pilote. C'est precisement ce que le reste du
 * code ne doit pas avoir a savoir.</p>
 */
expect val platformModule: Module

/**
 * Les services du domaine.
 *
 * <p>Declares ici et non dans core-domain, volontairement. Le domaine ne depend d'aucun
 * framework, Koin compris : ce sont des classes ordinaires qu'on peut construire a la
 * main, et tous leurs essais le font. Savoir les assembler est le travail de la couche
 * qui les assemble.</p>
 */
val domainModule = module {
    single { DryingTimeService() }
    single { ColorMixService() }
    single { PlanDryingService(get()) }
    single { WorkbenchService(get()) }
    single { PaletteMixService(get()) }
    single { FatOverLeanService() }
    single { SubstituteService() }
    single { ProgressCheckService() }
    single { PaintMatcher() }
    // Une seule instance, et c'est tout l'interet : elle se souvient de ce qui a deja
    // ete annonce. Deux surveillances repeteraient chacune la meme nouvelle.
    single { ReadinessWatch() }
}

/** La persistance. */
val dataModule = module {
    single { PalettierDatabase(get<SqlDriver>()) }

    single<PaintCatalogRepository> { SqlDelightPaintCatalogRepository(get(), get(IO)) }
    single<PaletteRepository> { SqlDelightPaletteRepository(get(), get(), get(IO)) }
    single<ProjectRepository> { SqlDelightProjectRepository(get(), get(), get(), get(IO)) }
    single<RecipeRepository> { SqlDelightRecipeRepository(get(), get(IO)) }
    single<PaletteMixRepository> { SqlDelightPaletteMixRepository(get(), get(IO)) }

    single { BackupImporter(get(), get(), get(), get()) }
}

/** Tout ce qu'il faut pour faire tourner Palettier, hors interface. */
val palettierModules: List<Module> = listOf(platformModule, dataModule, domainModule)
