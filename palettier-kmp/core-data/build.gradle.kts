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

/**
 * La tache de verification des migrations reste eteinte, et ce n'est pas un renoncement.
 *
 * Elle epuise la memoire de la JVM Gradle sur un schema de cent kilo-octets -- huit
 * minutes puis "Java heap space", a 4 Go, avec ou sans verifyMigrations. Ce n'est pas une
 * question de volume : c'est la tache qui est en cause.
 *
 * La garantie, elle, est gardee : MigrationTest ouvre le schema d'hier, lui applique les
 * migrations, et compare table par table avec le schema d'aujourd'hui. C'est exactement
 * ce que la tache promettait, en une seconde et sous notre controle.
 */
tasks.matching { it.name.startsWith("verify") && it.name.contains("Migration") }
    .configureEach { enabled = false }

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

android {
    namespace = "be.asmolabs.palettier.data"
    compileSdk = 36
    defaultConfig { minSdk = 26 }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}
