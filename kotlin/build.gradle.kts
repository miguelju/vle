// Root build file: declares plugin VERSIONS once. `apply false` means
// "download this version and make it available to submodules, but don't
// apply it to the root project itself" — the standard multi-module pattern.
//
// Android Studio may suggest newer AGP/Kotlin versions; accepting is fine.
// Nothing in this build is version-sensitive beyond AGP ≥ 8.x / Kotlin ≥ 2.x.
plugins {
    id("com.android.library") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.1.0" apply false
}
