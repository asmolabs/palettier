plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    androidTarget {
        compilations.all {
            compileTaskProvider.configure {
                compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) }
            }
        }
    }

    jvmToolchain(25)

    sourceSets {
        androidMain.dependencies {
            implementation(project(":core-data"))
            implementation(project(":core-image"))
            implementation(project(":feature-ui"))
            implementation(project(":ai"))

            implementation(compose.runtime)
            implementation(compose.foundation)
            implementation(compose.material3)

            implementation(libs.androidx.activity.compose)
            implementation(libs.androidx.lifecycle.viewmodel)
            implementation(libs.androidx.lifecycle.runtime.compose)
            implementation(libs.koin.core)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.kotlinx.coroutines.core)
        }
    }
}

android {
    namespace = "be.asmolabs.palettier.android"
    compileSdk = 36

    defaultConfig {
        applicationId = "be.asmolabs.palettier"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildTypes {
        // Pas de minification : l'application ne sort pas encore d'ici, et R8 sur des
        // reflexions de Koin se regle quand on en a besoin, pas avant.
        getByName("release") { isMinifyEnabled = false }
    }
}
