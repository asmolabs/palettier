plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
}

kotlin {
    jvm()
    jvmToolchain(25)

    sourceSets {
        jvmMain.dependencies {
            implementation(project(":core-data"))
            implementation(project(":feature-ui"))
            implementation(project(":core-image"))
            implementation(project(":ai"))
            implementation(compose.desktop.currentOs)
            // Dispatchers.Main n'existe pas sur le bureau sans cela : Compose et les
            // ViewModels s'en servent, et l'absence ne se voit qu'a l'execution.
            implementation(libs.kotlinx.coroutines.swing)
            implementation(compose.material3)
            implementation(libs.koin.core)
            implementation(libs.koin.compose.viewmodel)
            implementation(libs.androidx.lifecycle.viewmodel)
            implementation(libs.androidx.lifecycle.runtime.compose)
        }
    }
}

compose.desktop {
    application {
        mainClass = "be.asmolabs.palettier.MainKt"
    }
}
