plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.sqldelight)
}

sqldelight {
    databases {
        create("PalettierDatabase") {
            packageName.set("be.asmolabs.palettier.db")
            // Les migrations vivent en .sqm numerotes : c'est le successeur direct de
            // db.changelog-master.yaml, meme discipline -- un fichier joue une fois,
            // jamais modifie ensuite.
            schemaOutputDirectory.set(file("src/commonMain/sqldelight/databases"))
            verifyMigrations.set(true)
        }
    }
}

kotlin {
    jvm()
    jvmToolchain(25)

    sourceSets {
        commonMain.dependencies {
            api(project(":core-domain"))
            implementation(libs.sqldelight.runtime)
            implementation(libs.sqldelight.coroutines)
            implementation(libs.kotlinx.coroutines.core)
        }
        jvmMain.dependencies {
            implementation(libs.sqldelight.sqlite)
        }
        commonTest.dependencies {
            implementation(kotlin("test"))
            implementation(libs.kotlinx.coroutines.test)
        }
        jvmTest.dependencies {
            implementation(libs.sqldelight.sqlite)
        }
    }
}
