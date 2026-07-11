// Hand-written ergonomic sugar over the UniFFI-generated API.
//
// This file IS committed (unlike VleFFI.generated.swift, which
// scripts/build-ios.sh regenerates). Keep it thin: anything substantial
// belongs in the Rust wrapper (ffi/src/), where both this package and any
// future Kotlin bindings inherit it.
//
// The engine speaks canonical units only (K, kPa absolute — see the units
// rules in CLAUDE.md); presentation-side conveniences like °C live here,
// on the Swift side of the boundary.

import Foundation

public extension SteamStateData {
    /// Temperature in **°C** (the engine's canonical `t` is in K).
    var tCelsius: Double { t - 273.15 }

    /// Pressure in **bar absolute** (the engine's canonical `p` is in kPa).
    var pBar: Double { p / 100.0 }
}

public extension SatPropsData {
    /// Saturation temperature in **°C**.
    var tCelsius: Double { t - 273.15 }

    /// Saturation pressure in **bar absolute**.
    var pBar: Double { p / 100.0 }
}
