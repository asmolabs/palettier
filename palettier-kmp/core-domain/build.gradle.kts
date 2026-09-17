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
            // Aucune. C'est la regle du module : le domaine ne depend de rien.
            // kotlin.time.Duration remplace java.time.Duration sans dependance.
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
        }
    }
}
