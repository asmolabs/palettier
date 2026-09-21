package be.asmolabs.palettier.android

import android.app.Application
import android.content.Context
import be.asmolabs.palettier.ai.di.aiModule
import be.asmolabs.palettier.data.di.dataModule
import be.asmolabs.palettier.data.di.domainModule
import be.asmolabs.palettier.data.di.platformModule
import be.asmolabs.palettier.ui.app.uiModule
import org.koin.core.context.startKoin
import org.koin.dsl.module

/**
 * Le graphe de l'application, monte une fois pour toutes.
 *
 * <p>Sur le bureau, KoinApplication vit dans la composition. Ici il demarre avec le
 * processus : Android peut detruire et recreer l'Activity a la rotation de l'ecran, et
 * reconstruire la base a chaque fois serait absurde.</p>
 *
 * <p>Le Context est declare ici parce que l'application est seule a le posseder. C'est ce
 * que le module Android de core-data attend du graphe, plutot que de dependre de
 * koin-android pour une seule resolution.</p>
 */
class PalettierApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        startKoin {
            modules(
                module { single<Context> { this@PalettierApplication } },
                platformModule,
                dataModule,
                domainModule,
                aiModule,
                uiModule,
            )
        }
    }
}
