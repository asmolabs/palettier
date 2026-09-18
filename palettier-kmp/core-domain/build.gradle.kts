plugins {
    alias(libs.plugins.kotlinMultiplatform)
}

kotlin {
    jvm()
    // Android s'ajoute ici. Laisse de cote pour l'instant : le SDK n'est pas installe
    // sur cette machine, et declarer une cible qu'on ne peut pas compiler donnerait une
    // illusion de verification.
    //
    //   androidTarget()

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
