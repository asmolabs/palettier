plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
}

kotlin {
    jvm()
    androidTarget {
        compilations.all {
            compileTaskProvider.configure { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }
        }
    }

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

android {
    namespace = "be.asmolabs.palettier.domain"
    compileSdk = 36
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
