plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
    alias(libs.plugins.sqldelight)
    alias(libs.plugins.kotlinSerialization)
}

sqldelight {
    databases {
        create("PalettierDatabase") {
            packageName.set("be.asmolabs.palettier.db")
            // Les migrations vivent en .sqm numerotes : c'est le successeur direct de
            // db.changelog-master.yaml, meme discipline -- un fichier joue une fois,
            // jamais modifie ensuite.
            schemaOutputDirectory.set(file("src/commonMain/sqldelight/databases"))
            verifyMigrations.set(false)
        }
    }
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
            implementation(libs.sqldelight.runtime)
            implementation(libs.sqldelight.coroutines)
            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.kotlinx.serialization.json)
            api(libs.koin.core)
        }
        jvmMain.dependencies {
            implementation(libs.sqldelight.sqlite)
        }
        androidMain.dependencies {
            implementation(libs.sqldelight.android)
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
        }
        jvmTest.dependencies {
            implementation(libs.sqldelight.sqlite)
            implementation(libs.koin.test)
        }
    }
}

/**
 * La verification des migrations est mise de cote tant qu'il n'y en a aucune.
 *
 * Le schema est a sa version 1 et le repertoire ne contient aucun .sqm : la tache n'a rien
 * a comparer, et elle epuise la memoire de la JVM Gradle a essayer -- meme a 4 Go, meme
 * avec verifyMigrations a false, qui ne suffit pas a l'empecher de s'executer.
 *
 * A rallumer avec le premier .sqm, ou elle reprendra tout son sens : elle verifie alors
 * qu'appliquer les migrations a l'ancien schema redonne bien le nouveau. C'est la garantie
 * qui manquait a Liquibase et qu'on ne veut pas perdre.
 */
tasks.matching { it.name.startsWith("verify") && it.name.contains("Migration") }
    .configureEach { enabled = false }

android {
    namespace = "be.asmolabs.palettier.data"
    compileSdk = 36
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
