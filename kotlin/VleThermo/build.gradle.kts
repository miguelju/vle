// VleThermo — Android library module wrapping the Rust vle engine.
//
// The Kotlin analog of swift/VleThermo: this module contains NO
// thermodynamics and almost no hand-written code. Its two ingredients are
// produced by scripts/build-android.sh and are gitignored, never committed:
//
//   src/main/kotlin/dev/migueljackson/vle/ffi/vle_ffi.kt   (UniFFI wrapper)
//   src/main/jniLibs/<abi>/libvle_ffi.so                   (Rust engine)
//
// AGP packages everything under jniLibs/ into the AAR/APK automatically; at
// runtime the generated wrapper loads the library through JNA. Consume this
// module from an app repo via settings.gradle.kts:
//
//   include(":vlethermo")
//   project(":vlethermo").projectDir = file("/path/to/vle/kotlin/VleThermo")
//
// Full guide: docs/en/android/README.md.

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

android {
    // Kotlin package for this module's own (non-generated) identifiers; the
    // generated bindings use dev.migueljackson.vle.ffi (ffi/uniffi.toml).
    namespace = "dev.migueljackson.vle"
    compileSdk = 35

    defaultConfig {
        // Android 7.0+. JNA's own floor is lower; 24 is a sane modern
        // minimum that still covers ~99% of active devices.
        minSdk = 24
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

dependencies {
    // JNA is how the generated Kotlin calls the Rust C ABI. `@aar` selects
    // JNA's Android packaging (bundles its libjnidispatch.so per ABI).
    implementation("net.java.dev.jna:jna:5.17.0@aar")

    // Host-JVM unit tests run on this machine's JVM, not on Android, so
    // they need the plain desktop JNA jar instead of the AAR.
    testImplementation("net.java.dev.jna:jna:5.17.0")
    testImplementation("junit:junit:4.13.2")
}

// Unit tests load the HOST build of the Rust library (target/release/
// libvle_ffi.dylib on macOS, libvle_ffi.so on Linux, vle_ffi.dll on
// Windows) — the same trick as `swift test` using the XCFramework's macOS
// slice. scripts/build-android.sh produces it; jna.library.path tells JNA
// where to look. projectDir-relative so the path also resolves when this
// module is include()d from an app repo by absolute projectDir.
tasks.withType<Test>().configureEach {
    systemProperty(
        "jna.library.path",
        projectDir.resolve("../../target/release").absolutePath
    )
}
