rootProject.name = "palettier-kmp"

pluginManagement {
    repositories {
        gradlePluginPortal()
        mavenCentral()
        google()
    }
}

dependencyResolutionManagement {
    repositories {
        mavenCentral()
        google()
    }
}

include(":core-domain")
include(":core-data")
include(":feature-ui")
include(":desktopApp")
