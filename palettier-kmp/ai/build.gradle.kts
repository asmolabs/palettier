plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
    alias(libs.plugins.kotlinSerialization)
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
            api(project(":core-domain"))
            api(project(":core-image"))
            implementation(libs.ktor.client.core)
            implementation(libs.ktor.client.content.negotiation)
            implementation(libs.ktor.serialization.json)
            implementation(libs.kotlinx.serialization.json)
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.koin.core)
        }
        jvmMain.dependencies { implementation(libs.ktor.client.cio) }
        androidMain.dependencies { implementation(libs.ktor.client.okhttp) }
        jvmTest.dependencies {
            // L'essai en direct lit une vraie image : il lui faut le decodeur du poste.
            implementation(project(":core-image"))
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
            implementation(libs.ktor.client.mock)
        }
    }
}

android {
    namespace = "be.asmolabs.palettier.ai"
    compileSdk = 36
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
