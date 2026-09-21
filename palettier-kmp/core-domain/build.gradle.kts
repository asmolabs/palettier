plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidLibrary)
}

/**
 * Le catalogue, compile dans l'application plutot que lu a l'execution.
 *
 * Ce module n'admet que du Kotlin multiplateforme pur, et il n'y a pas de classpath sur
 * un telephone : une ressource lue au demarrage aurait demande un expect/actual par
 * plateforme, plus un index des fichiers ecrit a la main. Une constante generee marche
 * partout et ne coute rien -- les gammes tiennent en 80 ko de texte.
 *
 * La liste des fichiers est relevee ici, a la compilation. La propriete que le
 * chargement Java tenait de son classpath est donc gardee : ajouter une gamme, c'est
 * deposer un fichier, pas modifier du code.
 *
 * La source est encore celle du module Java : une seule copie de la donnee, donc aucune
 * derive possible tant que les deux applications vivent. Le jour ou palettier-core
 * disparait, ces fichiers emmenagent dans core-domain/catalog et seule la ligne
 * ci-dessous change.
 */
val catalogSource: File = rootDir.resolve("../palettier-core/src/main/resources/catalog")

require(catalogSource.isDirectory) {
    "Catalogue introuvable en $catalogSource. Si palettier-core a ete retire, deplacez " +
        "ses fichiers de catalog/ vers core-domain/catalog et corrigez catalogSource."
}

val generateCatalogFiles by tasks.registering {
    description = "Compile catalog/pigments.csv et catalog/brands/*.csv en constantes Kotlin"
    val source = catalogSource
    val outputDir = layout.buildDirectory.dir("generated/catalog")
    inputs.dir(source).withPropertyName("catalogue")
    outputs.dir(outputDir).withPropertyName("sources")

    doLast {
        fun quote(text: String): String = text.lineSequence()
            .joinToString(",\n        ", prefix = "listOf(\n        ", postfix = ",\n    ).joinToString(\"\\n\")") { line ->
                '"' + line.replace("\\", "\\\\").replace("\"", "\\\"").replace("$", "\\$") + '"'
            }

        val pigments = source.resolve("pigments.csv")
        require(pigments.isFile) { "pigments.csv introuvable en $pigments" }
        val brands = source.resolve("brands").listFiles()
            .orEmpty()
            .filter { it.isFile && it.name.endsWith(".csv") }
            .sortedBy { it.name }
        require(brands.isNotEmpty()) { "Aucune gamme en ${source.resolve("brands")}" }

        val file = outputDir.get().asFile.resolve("CatalogFiles.kt")
        file.parentFile.mkdirs()
        file.writeText(buildString {
            appendLine("package be.asmolabs.palettier.domain.catalog")
            appendLine()
            appendLine("// Fichier genere par la tache generateCatalogFiles. Ne pas editer :")
            appendLine("// la source est ${source.name}/, cote ressources.")
            appendLine("object CatalogFiles {")
            appendLine()
            appendLine("    val PIGMENTS: String = ${quote(pigments.readText())}")
            appendLine()
            appendLine("    val BRANDS: List<BrandFile> = listOf(")
            brands.forEach { brand ->
                appendLine("        BrandFile(")
                appendLine("            \"${brand.name}\",")
                appendLine("            ${quote(brand.readText())},")
                appendLine("        ),")
            }
            appendLine("    )")
            appendLine("}")
        })
        logger.lifecycle("Catalogue compile : ${brands.size} gammes, ${file.length() / 1024} ko")
    }
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
        commonMain {
            kotlin.srcDir(generateCatalogFiles)
        }
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
