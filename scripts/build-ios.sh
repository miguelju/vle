#!/usr/bin/env bash
# build-ios.sh — clean checkout → VleFFI.xcframework + generated Swift.
#
# The single entry point for the Apple build (IOS_FFI_PLAN.md §4). Builds
# the vle-ffi static library for three Apple targets, generates the Swift
# bindings from the compiled artifact, assembles the XCFramework inside
# swift/VleThermo/, and runs the Swift package tests against the macOS
# slice. Idempotent — rerun after any Rust change.
#
# Everything this script produces is generated, never committed
# (.gitignore covers build/, *.xcframework, *.generated.swift).
#
# Prerequisites (checked below, not auto-installed):
#   - Xcode + command-line tools (xcodebuild, swift)
#   - rustup targets: aarch64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin
#
# Usage:  scripts/build-ios.sh [--skip-tests]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Minimum OS versions the static libs are built against. Keep in sync with
# the platforms in swift/VleThermo/Package.swift.
export IPHONEOS_DEPLOYMENT_TARGET="16.0"
export MACOSX_DEPLOYMENT_TARGET="13.0"

# One library slice per Apple platform variant. Device-arm64 and
# simulator-arm64 are the same CPU but DIFFERENT targets (different
# platform ABI) — an XCFramework keeps one library per variant. The macOS
# slice exists so `swift test` runs natively on this Mac, and so native
# macOS (SwiftUI multiplatform) apps work.
TARGETS=(
  aarch64-apple-ios       # physical iPhones/iPads
  aarch64-apple-ios-sim   # iOS simulator on Apple Silicon
  aarch64-apple-darwin    # native macOS (tests + Mac apps)
)

BUILD_DIR="$REPO_ROOT/build/ios"
SWIFT_PKG="$REPO_ROOT/swift/VleThermo"
XCFRAMEWORK="$SWIFT_PKG/VleFFI.xcframework"
MODULE_NAME="VleFFI"

# ── 0. Preflight ─────────────────────────────────────────────────────────
command -v xcodebuild >/dev/null || { echo "ERROR: xcodebuild not found — install Xcode"; exit 1; }
for t in "${TARGETS[@]}"; do
  if ! rustup target list --installed | grep -qx "$t"; then
    echo "ERROR: Rust target $t not installed. Run:"
    echo "  rustup target add ${TARGETS[*]}"
    exit 1
  fi
done

# ── 1. Static libraries (release, per target) ────────────────────────────
for t in "${TARGETS[@]}"; do
  echo "==> cargo build -p vle-ffi --release --target $t"
  cargo build -p vle-ffi --release --target "$t"
done

# ── 2. Swift bindings, generated FROM the compiled artifact ──────────────
# Library mode: the generator reads the UniFFI metadata embedded in the .a,
# so bindings can never drift from the code. Any slice works as input —
# they all embed identical metadata; we use the device one.
LIB_FOR_BINDGEN="$REPO_ROOT/target/aarch64-apple-ios/release/libvle_ffi.a"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/bindings" "$BUILD_DIR/Headers"

# The C-module name has to agree in three places: the generated Swift's
# `import` (set by ffi_module_name in ffi/uniffi.toml), the modulemap's
# `module …` declaration (set by --module-name here), and the binaryTarget
# name in Package.swift. All three say VleFFI.
echo "==> generating Swift sources"
cargo run -p vle-uniffi-bindgen --release --bin uniffi-bindgen-swift -- \
  "$LIB_FOR_BINDGEN" "$BUILD_DIR/bindings" --swift-sources

echo "==> generating C headers + modulemap"
# Naming the file exactly module.modulemap is what lets Xcode/SwiftPM find
# the module inside each slice's Headers directory (the classic gotcha —
# see docs/en/ios/README.md). No --xcframework flag: that emits a
# `framework module`, which is for .framework-bundle slices; ours are bare
# static libraries + Headers, which need a plain `module`.
cargo run -p vle-uniffi-bindgen --release --bin uniffi-bindgen-swift -- \
  "$LIB_FOR_BINDGEN" "$BUILD_DIR/Headers" \
  --headers --modulemap \
  --module-name "$MODULE_NAME" --modulemap-filename module.modulemap

# ── 3. XCFramework (one -library/-headers pair per slice) ────────────────
echo "==> assembling $XCFRAMEWORK"
rm -rf "$XCFRAMEWORK"   # xcodebuild refuses to overwrite
xcodebuild -create-xcframework \
  -library "$REPO_ROOT/target/aarch64-apple-ios/release/libvle_ffi.a"     -headers "$BUILD_DIR/Headers" \
  -library "$REPO_ROOT/target/aarch64-apple-ios-sim/release/libvle_ffi.a" -headers "$BUILD_DIR/Headers" \
  -library "$REPO_ROOT/target/aarch64-apple-darwin/release/libvle_ffi.a"  -headers "$BUILD_DIR/Headers" \
  -output "$XCFRAMEWORK"

# ── 4. Generated Swift wrapper into the package ──────────────────────────
# The .generated.swift suffix is gitignored; the hand-written package files
# around it are committed.
mkdir -p "$SWIFT_PKG/Sources/VleThermo"
cp "$BUILD_DIR/bindings/vle_ffi.swift" \
   "$SWIFT_PKG/Sources/VleThermo/VleFFI.generated.swift"

# ── 5. XCTest through the real FFI boundary (macOS slice) ────────────────
if [[ "${1:-}" != "--skip-tests" ]]; then
  echo "==> swift test (macOS slice)"
  (cd "$SWIFT_PKG" && swift test)
fi

echo "DONE: $XCFRAMEWORK"
echo "Consume from Xcode via File > Add Package Dependencies > Add Local… > swift/VleThermo"
