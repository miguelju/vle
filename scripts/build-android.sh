#!/usr/bin/env bash
# build-android.sh — clean checkout → Android .so libraries + generated Kotlin.
#
# The single entry point for the Android/Kotlin build (ANDROID_FFI_PLAN.md),
# the sibling of scripts/build-ios.sh. Cross-compiles the vle-ffi shared
# library for the Android ABIs, builds the HOST shared library (used by the
# Gradle unit tests and by Compose for Desktop), generates the Kotlin
# bindings from the compiled artifact, and drops everything into
# kotlin/VleThermo/. Idempotent — rerun after any Rust change.
#
# Everything this script produces is generated, never committed (.gitignore
# covers jniLibs/ and the generated bindings package).
#
# Prerequisites (checked below, not auto-installed):
#   - Android NDK — install via Android Studio's SDK Manager, then export
#     ANDROID_NDK_HOME (or let cargo-ndk find it via ANDROID_HOME)
#   - cargo-ndk:       cargo install cargo-ndk
#   - rustup targets:  rustup target add aarch64-linux-android x86_64-linux-android
#
# Usage:  scripts/build-android.sh [--skip-tests]
#         ABIS="arm64-v8a x86_64 armeabi-v7a" scripts/build-android.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Android ABIs to build. arm64-v8a covers every modern device AND the
# emulator on Apple Silicon; x86_64 covers the emulator on Intel/Windows
# machines. Add armeabi-v7a (32-bit) only for very old devices (it also
# needs: rustup target add armv7-linux-androideabi).
ABIS="${ABIS:-arm64-v8a x86_64}"

KOTLIN_MODULE="$REPO_ROOT/kotlin/VleThermo"
JNILIBS_DIR="$KOTLIN_MODULE/src/main/jniLibs"
KOTLIN_OUT="$KOTLIN_MODULE/src/main/kotlin"

# Map each ABI to the rustup target cargo-ndk compiles with, so the
# preflight can name exactly what's missing.
rust_target_for_abi() {
  case "$1" in
    arm64-v8a)   echo aarch64-linux-android ;;
    x86_64)      echo x86_64-linux-android ;;
    armeabi-v7a) echo armv7-linux-androideabi ;;
    *) echo "ERROR: unknown ABI '$1'" >&2; exit 1 ;;
  esac
}

# ── 0. Preflight ─────────────────────────────────────────────────────────
command -v cargo-ndk >/dev/null || {
  echo "ERROR: cargo-ndk not found. Run:  cargo install cargo-ndk"; exit 1; }
for abi in $ABIS; do
  t="$(rust_target_for_abi "$abi")"
  if ! rustup target list --installed | grep -qx "$t"; then
    echo "ERROR: Rust target $t not installed (needed for ABI $abi). Run:"
    echo "  rustup target add $t"
    exit 1
  fi
done

# ── 1. Android shared libraries (release, one .so per ABI) ───────────────
# cargo-ndk wires the NDK's clang as the cross-linker and lays the output
# out exactly the way AGP expects: jniLibs/<abi>/libvle_ffi.so.
ABI_FLAGS=()
for abi in $ABIS; do ABI_FLAGS+=(-t "$abi"); done
echo "==> cargo ndk (${ABIS}) build -p vle-ffi --release"
cargo ndk "${ABI_FLAGS[@]}" -o "$JNILIBS_DIR" build -p vle-ffi --release

# ── 2. Host shared library (bindgen input + unit tests + Compose Desktop) ─
echo "==> cargo build -p vle-ffi --release (host)"
cargo build -p vle-ffi --release
case "$(uname -s)" in
  Darwin) HOST_LIB="$REPO_ROOT/target/release/libvle_ffi.dylib" ;;
  Linux)  HOST_LIB="$REPO_ROOT/target/release/libvle_ffi.so" ;;
  *)      echo "ERROR: unsupported host $(uname -s) — on Windows run the"
          echo "commands from docs/en/android/README.md §Windows instead."
          exit 1 ;;
esac

# ── 3. Kotlin bindings, generated FROM the compiled artifact ─────────────
# Library mode: the generator reads the UniFFI metadata embedded in the
# compiled library, so bindings can never drift from the code. Any artifact
# works as input — every target embeds identical metadata; the host one is
# simply always present. Package + cdylib name come from ffi/uniffi.toml
# ([bindings.kotlin]); the file lands in
#   $KOTLIN_OUT/dev/migueljackson/vle/ffi/vle_ffi.kt
echo "==> generating Kotlin bindings"
cargo run -p vle-uniffi-bindgen --release --bin uniffi-bindgen -- \
  generate --library "$HOST_LIB" --language kotlin \
  --out-dir "$KOTLIN_OUT" --no-format

# ── 4. Host-JVM smoke tests through the real FFI boundary ────────────────
# The Kotlin analog of build-ios.sh running `swift test` on the macOS
# slice. Needs a Gradle on PATH or a locally generated wrapper (neither is
# committed — no binaries in the repo); Android Studio runs the same tests
# from the IDE if you skip this.
if [[ "${1:-}" != "--skip-tests" ]]; then
  if [[ -x "$REPO_ROOT/kotlin/gradlew" ]]; then
    (cd "$REPO_ROOT/kotlin" && ./gradlew --console=plain :VleThermo:test)
  elif command -v gradle >/dev/null; then
    (cd "$REPO_ROOT/kotlin" && gradle --console=plain :VleThermo:test)
  else
    echo "NOTE: no gradle found — skipping host-JVM tests."
    echo "      Run them from Android Studio, or: cd kotlin && gradle :VleThermo:test"
  fi
fi

echo "DONE: $JNILIBS_DIR (ABIs: $ABIS) + generated Kotlin in $KOTLIN_OUT"
echo "Consume from Android Studio — see docs/en/android/README.md"
