plugins {
    alias(libs.plugins.kotlinMultiplatform)
}

kotlin {
    jvm()
    // Android et iOS s'ajoutent ici. Laisses de cote pour l'instant : le SDK Android
    // et Xcode ne sont pas installes sur cette machine, et declarer une cible qu'on ne
    // peut pas compiler donnerait une illusion de verification.
    //
    //   androidTarget()
    //   iosArm64(); iosSimulatorArm64()

    jvmToolchain(25)

    sourceSets {
        commonMain.dependencies {
            // La seule dependance autorisee ici, et la regle est celle-ci : rien qui ne
            // soit du Kotlin multiplateforme pur. Coroutines passe -- elle porte Flow,
            // dont les ports du domaine ont besoin, et le parallelisme de la recherche
            // de melange. SQLDelight, Compose et tout framework ne passent pas.
            implementation(libs.kotlinx.coroutines.core)
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
        }
    }
}
