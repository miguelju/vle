#!/usr/bin/env bash
# build-wasm.sh — clean checkout → wasm/pkg/, a ready-to-install npm package.
#
# The single entry point for the JavaScript/WebAssembly build
# (WEB_UI_PLAN.md), the third sibling of scripts/build-ios.sh and
# scripts/build-android.sh. Compiles the vle-wasm wrapper crate to
# wasm32-unknown-unknown, generates the JS glue + TypeScript declarations,
# and drops everything into wasm/pkg/. Idempotent — rerun after any Rust
# change.
#
# Everything this script produces is generated, never committed (.gitignore
# covers wasm/pkg/), and never published to npm — a JS project consumes it
# by local path:  npm install <path-to-vle>/wasm/pkg
#
# Prerequisites (checked below, not auto-installed):
#   - wasm-pack:      cargo install wasm-pack   (or: brew install wasm-pack)
#   - rustup target:  rustup target add wasm32-unknown-unknown
#
# Usage:  scripts/build-wasm.sh [--skip-tests]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# ── 0. Preflight ─────────────────────────────────────────────────────────
command -v wasm-pack >/dev/null || {
  echo "ERROR: wasm-pack not found. Run:  cargo install wasm-pack"; exit 1; }
if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  echo "ERROR: Rust target wasm32-unknown-unknown not installed. Run:"
  echo "  rustup target add wasm32-unknown-unknown"
  exit 1
fi

# ── 1. Smoke tests through the real JS↔wasm boundary ─────────────────────
# The analog of build-android.sh running the Gradle/JNA tests: wasm-pack
# compiles wasm/tests/smoke.rs to wasm and executes it inside Node.js.
# Skippable because a browser app build doesn't strictly need Node.
if [[ "${1:-}" != "--skip-tests" ]]; then
  if command -v node >/dev/null; then
    echo "==> wasm-pack test --node wasm"
    wasm-pack test --node wasm
  else
    echo "NOTE: no node found — skipping the boundary smoke tests."
    echo "      Install Node.js and rerun, or: wasm-pack test --node wasm"
  fi
fi

# ── 2. The npm package ───────────────────────────────────────────────────
# `--target web` emits a universal ES module: it works imported through a
# bundler (Vite/webpack in a React app) AND from a plain
# <script type="module"> — one artifact for the website, Tauri, Electron,
# and Capacitor. Output: wasm/pkg/{vle_wasm_bg.wasm, vle_wasm.js,
# vle_wasm.d.ts, package.json}. wasm-opt (bundled with wasm-pack) shrinks
# the binary as a post-pass.
echo "==> wasm-pack build wasm --target web --release"
wasm-pack build wasm --target web --release

echo
echo "DONE: wasm/pkg/ ($(du -h wasm/pkg/vle_wasm_bg.wasm | cut -f1) wasm module)"
echo "Consume from a JS project:   npm install \"$REPO_ROOT/wasm/pkg\""
echo "Then:                        import init, { VleSystem } from \"vle-wasm\";"
echo "                             await init();"
echo "Guide: docs/en/web/README.md"
