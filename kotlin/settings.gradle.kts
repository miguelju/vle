// Standalone Gradle build for the Kotlin/Android side of vle (Milestone 16).
// Open THIS directory (vle/kotlin) in Android Studio. The Rust side is built
// separately by scripts/build-android.sh — Gradle never invokes cargo.
//
// No gradle-wrapper is committed (repo policy: no binaries in git). On first
// open Android Studio offers to use its bundled Gradle / create a wrapper;
// from a plain terminal, run `gradle wrapper` once (wrapper files are
// gitignored). See docs/en/android/README.md.

pluginManagement {
    repositories {
        google()             // the Android Gradle Plugin is published here
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    // One repository list for every module; a module declaring its own
    // repositories becomes a build error instead of a silent divergence.
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "vle-kotlin"
include(":VleThermo")
