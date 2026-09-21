plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    jvm()
    androidTarget {
        compilations.all {
            compileTaskProvider.configure {
                compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) }
            }
        }
    }

    jvmToolchain(25)

    sourceSets {
        commonMain.dependencies {
            // L'interface connait le domaine, jamais la persistance : c'est le module
            // applicatif qui les met en presence.
            implementation(project(":core-domain"))
            implementation(project(":core-image"))
            implementation(project(":ai"))

            implementation(compose.runtime)
            implementation(compose.foundation)
            implementation(compose.material3)
            implementation(compose.components.resources)

            implementation(libs.androidx.lifecycle.viewmodel)
            implementation(libs.androidx.lifecycle.runtime.compose)
            implementation(libs.koin.core)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.kotlinx.coroutines.core)
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.serialization.json)
            // Le catalogue de modeles parle a un serveur : ici, un serveur qui n'est pas la.
            implementation(libs.ktor.client.mock)
            implementation(libs.kotlinx.coroutines.test)
        }
        jvmTest.dependencies {
            // La pipette s'essaie la ou l'on sait fabriquer une image : ImageIO.
            implementation(project(":core-image"))
            implementation(project(":ai"))
        }
    }
}

android {
    namespace = "be.asmolabs.palettier.ui"
    compileSdk = 36
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
