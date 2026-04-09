//! # VLE Engine — Vapor-Liquid Equilibrium Thermodynamic Calculator
//!
//! This crate is the computational core of the VLE project, a modernization of two
//! legacy thermodynamic codebases (VB6 ~15,000 lines + Pascal ~2,500 lines) into a
//! high-performance Rust library with Python bindings via PyO3.
//!
//! ## What this library does
//!
//! VLE (Vapor-Liquid Equilibrium) calculations predict how chemical mixtures split
//! between liquid and vapor phases at given temperature and pressure conditions.
//! This is fundamental to chemical engineering — every distillation column, flash
//! drum, and separation process relies on VLE predictions.
//!
//! The library supports:
//! - **22+ cubic equations of state** (EOS) for modeling PVT behavior of gases and liquids
//! - **5 activity coefficient models** for liquid-phase non-ideality (van Laar, Wilson, etc.)
//! - **11 mixing rules** for combining pure-component parameters into mixture parameters
//! - **6 saturation pressure correlations** for estimating boiling points
//! - **6 flash calculation types**: bubble point (T/P), dew point (T/P), isothermal flash,
//!   adiabatic flash
//! - **Parameter regression**: kij (binary interaction) and Aij (activity model) fitting
//!
//! ## Why these modules exist
//!
//! The module structure mirrors the mathematical layers of a VLE calculation:
//!
//! 1. **`eos`** — Equation of state definitions. An EOS is a mathematical model
//!    (e.g., Peng-Robinson, Soave-Redlich-Kwong) that relates pressure, volume, and
//!    temperature for a substance. Each variant uses a different α(T) function to
//!    capture how molecular attractions change with temperature. The 22+ variants come
//!    from merging 19 VB6 models with 3 additional Pascal models (Schmidt-Wenzel,
//!    Patel-Teja) that handle polar and asymmetric molecules better via a third
//!    parameter.
//!
//! 2. **`activity`** — Activity coefficient models. These capture liquid-phase
//!    non-ideality (deviations from Raoult's law) using empirical correlations fit
//!    to experimental data. Essential for polar/associating mixtures (e.g., water +
//!    alcohol) where cubic EOS alone gives poor liquid-phase predictions.
//!
//! 3. **`mixing`** — Mixing rules. When applying a pure-component EOS to a mixture,
//!    you need rules for combining the individual a, b (and sometimes c) parameters
//!    into mixture-average values. Classical rules (quadratic in composition) work
//!    for simple mixtures; advanced rules like Wong-Sandler incorporate activity
//!    coefficient information for better accuracy with non-ideal systems.
//!
//! 4. **`saturation`** — Saturation pressure models. These correlations (Antoine,
//!    Riedel, etc.) estimate pure-component boiling pressure at a given temperature.
//!    Flash calculations use these for initial K-value estimates to seed iterative
//!    convergence.
//!
//! 5. **`types`** — Core data structures shared across all modules. `Component`
//!    holds pure-component properties (critical T/P, acentric factor, etc.),
//!    `Mixture` groups components with their compositions, `Flow` represents a
//!    process stream with phase-specific data, `Tolerances` controls convergence
//!    criteria, and `ReferenceState` defines the thermodynamic reference for
//!    enthalpy/entropy calculations.
//!
//! ## Origin and academic references
//!
//! Based on the thesis: *"Desarrollo de un Programa Computacional para el Cálculo
//! del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente Windows"*
//! (Jackson & Mendible, USB, 1999). Additional models from Reference (4): Da Silva &
//! Báez (1989) — Mac Pascal program contributing Schmidt-Wenzel, Patel-Teja, and
//! Chao-Seader models. All code derived from (4) is annotated with citation comments.
//!
//! ## Internal units
//!
//! All calculations use these canonical units (matching the legacy codebases):
//! - Temperature: **K** (Kelvin, absolute)
//! - Pressure: **kPa** (absolute — never gauge)
//! - Molar energy: **kJ/kmol**
//! - Molar entropy: **kJ/(kmol·K)**
//! - Molar volume: **cm³/mol**
//! - Amount: **kmol**
//! - Gas constant R: **8.31451 kJ/(kmol·K)**

// `pub mod eos` does two things:
// 1. `mod eos` tells Rust to look for a file named `eos.rs` (or `eos/mod.rs`)
//    and include it as a sub-module of this library.
// 2. `pub` makes the module public, so code outside this crate (e.g., the
//    Python bindings or other Rust crates) can access it. Without `pub`,
//    the module would only be usable within this crate.
pub mod eos;
pub mod activity;
pub mod mixing;
pub mod saturation;
pub mod types;

// `pub use` re-exports an item from a sub-module, making it available
// directly at this crate's top level. Without these lines, users would
// have to write the full path: `vle_engine::eos::CubicEos`. With them,
// they can write the shorter `vle_engine::CubicEos` instead. It's purely
// a convenience — the types still live in their original modules, but
// `pub use` creates a public shortcut at the crate root.
pub use eos::CubicEos;
pub use activity::ActivityModel;
pub use mixing::MixingRule;
pub use saturation::SatPressureModel;

// Re-export all core data structures from types module.
pub use types::*;
