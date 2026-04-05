# Dimensional Analysis for Units of Measurement

**Design document for the Units of Measurement Add-On (Milestone 1.5)**

This document explains the theoretical foundation of dimensional analysis, how it is used to implement type-safe unit conversion, and how it will be applied to build the `units/` Rust crate and `python/src/vle/units.py` wrapper for the VLE project.

---

## 1. Overview

A **physical quantity** has both a numeric value and a unit (e.g., `300 K`, `101.325 kPa`). Because thermodynamic calculations combine quantities through multiplication, division, and differentiation, the resulting units can be complex (e.g., `kJ/(kmol·K)` for molar entropy). Keeping units consistent by hand is error-prone — the NASA *Mars Climate Orbiter* was lost in 1999 due to a unit mismatch (pound-force vs. newton).

**Dimensional analysis** provides a rigorous mathematical framework for:
1. Checking that equations are **dimensionally homogeneous** (both sides have the same units)
2. Converting between unit systems (e.g., °C ↔ K, psi ↔ kPa)
3. Detecting unit errors at compile time (Rust) or runtime (Python)

This document describes the technique, then shows how to apply it to the VLE project's Units of Measurement Add-On.

---

## 2. Theoretical Foundation

### 2.1 The 7 SI Base Dimensions

The International System of Units (SI) defines seven **base dimensions**, each with one base unit. Every physical quantity can be expressed as a product of powers of these dimensions.

| # | Dimension | Symbol | SI Base Unit | Unit Symbol |
|---|-----------|--------|--------------|-------------|
| 1 | Length | L | meter | m |
| 2 | Mass | M | kilogram | kg |
| 3 | Time | T | second | s |
| 4 | Electric current | I | ampere | A |
| 5 | Thermodynamic temperature | Θ | kelvin | K |
| 6 | Amount of substance | N | mole | mol |
| 7 | Luminous intensity | J | candela | cd |

Source: BIPM (Bureau International des Poids et Mesures), *The International System of Units (SI)*, 9th ed., 2019.

### 2.2 Representing Quantities as Dimension Vectors

Every physical quantity is represented as a **7-tuple of integer exponents** `(a, b, c, d, e, f, g)`, meaning:

```
Quantity = L^a · M^b · T^c · I^d · Θ^e · N^f · J^g
```

**Examples (quantities relevant to VLE thermodynamics):**

| Quantity | Formula | Dimension Vector (L, M, T, I, Θ, N, J) |
|----------|---------|----------------------------------------|
| Length | m | (1, 0, 0, 0, 0, 0, 0) |
| Temperature (absolute) | K | (0, 0, 0, 0, 1, 0, 0) |
| Temperature difference / gradient | K (= ΔK) | (0, 0, 0, 0, 1, 0, 0) |
| Amount | mol | (0, 0, 0, 0, 0, 1, 0) |
| Volume | m³ | (3, 0, 0, 0, 0, 0, 0) |
| Molar volume | m³/mol | (3, 0, 0, 0, 0, −1, 0) |
| Force | N = kg·m/s² | (1, 1, −2, 0, 0, 0, 0) |
| Pressure | Pa = N/m² = kg/(m·s²) | (−1, 1, −2, 0, 0, 0, 0) |
| Energy | J = N·m = kg·m²/s² | (2, 1, −2, 0, 0, 0, 0) |
| Molar energy | J/mol | (2, 1, −2, 0, 0, −1, 0) |
| Molar entropy | J/(mol·K) | (2, 1, −2, 0, −1, −1, 0) |

### 2.3 Dimensional Arithmetic

The algebra of dimensions follows three rules:

**Rule 1 — Multiplication adds exponents:**
```
(a₁, b₁, …) × (a₂, b₂, …) = (a₁+a₂, b₁+b₂, …)
```
Example: Force × distance = energy
```
(1, 1, −2, 0, 0, 0, 0) × (1, 0, 0, 0, 0, 0, 0) = (2, 1, −2, 0, 0, 0, 0)  ✓
```

**Rule 2 — Division subtracts exponents:**
```
(a₁, b₁, …) ÷ (a₂, b₂, …) = (a₁−a₂, b₁−b₂, …)
```
Example: Energy ÷ amount = molar energy
```
(2, 1, −2, 0, 0, 0, 0) − (0, 0, 0, 0, 0, 1, 0) = (2, 1, −2, 0, 0, −1, 0)  ✓
```

**Rule 3 — Addition requires equal dimensions:**
```
(a₁, b₁, …) + (a₂, b₂, …)  is defined only if  (a₁, b₁, …) = (a₂, b₂, …)
```
Example: `300 K + 5 kPa` is a **dimensional error** (temperature ≠ pressure).

### 2.4 Dimensional Homogeneity Principle

**Bridgman's principle** (Bridgman, 1922): A physical equation is dimensionally homogeneous if and only if both sides of the equation have the same dimension vector.

This principle underpins the technique: if an equation has mismatched dimensions, it is **provably wrong**, regardless of the numeric result. For example, consider the ideal gas law `PV = nRT`:

| Term | Dimension Vector |
|------|------------------|
| P (pressure) | (−1, 1, −2, 0, 0, 0, 0) |
| V (volume) | (3, 0, 0, 0, 0, 0, 0) |
| PV | (2, 1, −2, 0, 0, 0, 0) |
| n (amount) | (0, 0, 0, 0, 0, 1, 0) |
| R (J/(mol·K)) | (2, 1, −2, 0, −1, −1, 0) |
| T (temperature) | (0, 0, 0, 0, 1, 0, 0) |
| nRT | (2, 1, −2, 0, 0, 0, 0) |

Both sides reduce to `(2, 1, −2, 0, 0, 0, 0)` = energy. The equation is dimensionally homogeneous. ✓

---

## 3. Unit Conversion via Dimensional Analysis

Within a given dimension, multiple units may exist (e.g., K, °C, °F, °R for temperature). Conversion between them requires a **scale factor** and optionally an **offset**.

### 3.1 Linear (scale-only) conversions

Most unit conversions are pure scaling:
```
value_in_unit_B = value_in_unit_A × (factor_A / factor_B)
```

Example: `1 kPa = 1000 Pa`, so `factor(kPa) = 1000`, `factor(Pa) = 1`.

### 3.2 Affine (scale + offset) conversions

Temperature is the notable exception — absolute scales (K, °R) and relative scales (°C, °F) require an offset:
```
T(K) = T(°C) + 273.15
T(°R) = T(°F) + 459.67
T(K) = T(°R) × 5/9
```

This affects **absolute temperatures** (state variables) but not **temperature differences** (gradients), where the offset cancels.

### 3.3 Absolute Temperature vs. Temperature Difference — A Critical Distinction

The single most common unit-handling bug in thermodynamic code (and in the broader engineering literature) is conflating an **absolute temperature** with a **temperature difference**. Because both have the dimension vector `(0, 0, 0, 0, 1, 0, 0)`, a dimensional-analysis system that only tracks base-dimension exponents cannot distinguish them — but they convert between units under *different rules* and obey *different algebraic laws*. A robust units library must therefore treat them as **distinct quantity types**.

#### 3.3.1 What they mean physically

| Concept | Symbol | Physical meaning | Example |
|---------|--------|------------------|---------|
| Absolute temperature | *T* | A point on the thermodynamic temperature axis. A state variable. | "The reactor is at 350 K." |
| Temperature difference (interval / gradient) | Δ*T* | A signed displacement between two points on the axis. Never a location. | "Heat the reactor by 50 K." |

A *difference* appears whenever you see Δ, a subtraction of temperatures, a derivative with respect to *T*, or a coefficient like Cp, κ (thermal conductivity), or a heat transfer coefficient:

- `Cp` has units of `J/(mol·K)` — the **K** here is a temperature *difference*, not an absolute temperature. Doubling the reference zero does not change Cp.
- `∂H/∂T` at constant P — the denominator is a temperature difference.
- `ΔT_lm` (log-mean temperature difference) is unambiguously a difference.
- Thermal expansion coefficient `α = (1/V)(∂V/∂T)` uses a difference.

#### 3.3.2 Conversion rules differ

For an **absolute temperature** the offset matters:

```
T(°F) → T(K):    T_K  = (T_F − 32) × 5/9 + 273.15     (offset present)
T(°C) → T(K):    T_K  = T_C + 273.15                   (offset present)
```

For a **temperature difference** the offset cancels and only the scale survives:

```
ΔT(°F) → ΔT(K):   ΔT_K = ΔT_F × 5/9                    (no offset)
ΔT(°C) → ΔT(K):   ΔT_K = ΔT_C                          (no offset — same size)
```

**Concrete consequences:**

| Value | Interpreted as absolute | Interpreted as difference |
|-------|------------------------|--------------------------|
| `10 °C` | = 283.15 K | = 10 K |
| `10 °F` | = 260.93 K | ≈ 5.556 K |
| `0 °C`  | = 273.15 K | = 0 K |
| `1 °R`  | = 0.556 K | ≈ 0.556 K (same; °R and K differ only in scale) |

So `10 °C` becomes either `283.15 K` or `10 K` depending on what it represents. **No library can guess correctly from the number alone — the user (or the API) must say which one it is.**

#### 3.3.3 Algebraic laws differ

Because absolute temperature is affine and differences are linear, they obey distinct algebraic rules:

| Operation | Absolute (T) | Difference (ΔT) | Result |
|-----------|--------------|-----------------|--------|
| T₁ + T₂ | ❌ meaningless | — | error |
| T₁ − T₂ | ✓ | — | **ΔT** |
| T + ΔT | ✓ | ✓ | **T** (new absolute) |
| ΔT₁ + ΔT₂ | — | ✓ | ΔT |
| ΔT₁ − ΔT₂ | — | ✓ | ΔT |
| k · T (k scalar) | ⚠ only if k is dimensionless and T is in K (absolute zero preserved) | ✓ | depends |
| k · ΔT | ✓ | ✓ | ΔT |
| T / ΔT | ✓ | — | dimensionless ratio |

The canonical summary: **you cannot add two absolute temperatures** (what would `300 K + 400 K` mean — 700 K of what?). But you can *always* add two temperature differences, and you can add a difference to an absolute temperature to get a new absolute temperature.

#### 3.3.4 Worked example — why the distinction matters

Suppose a reactor is held at **T₁ = 25 °C** and must be heated to **T₂ = 85 °C**. The required rise is `ΔT = 60 °C` of difference. A naive user converts everything "to Fahrenheit" by applying the absolute-temperature formula `°F = °C × 9/5 + 32` to all three numbers:

| Quantity | Correct (°F) | Naive (°C × 9/5 + 32) | Error |
|----------|--------------|----------------------|-------|
| T₁ = 25 °C | 77 °F (absolute) | 77 °F | ✓ |
| T₂ = 85 °C | 185 °F (absolute) | 185 °F | ✓ |
| ΔT = 60 °C | **108 °F** (difference, × 9/5 only) | 140 °F (wrong) | +32 °F spurious |

The naive answer `140 °F` is wrong by exactly the offset (32). The correct difference conversion `ΔT_F = ΔT_C × 9/5 = 108 °F` contains no offset.

Check: `T₂ − T₁ = 185 °F − 77 °F = 108 °F` ✓

In a VLE context, the same error appears in expressions like `H = Cp · ΔT`: if the user supplies Cp in `J/(mol·°C)` and ΔT in `°C`, the result is correct; if they instead pass the *absolute* temperatures and the code computes `Cp · (T₂ − T₁)` internally, conversion must be done with difference semantics. Mixing the two produces bugs that look "almost right" (a few percent off) and are hard to track down.

#### 3.3.5 How the library enforces the distinction

The Units Add-On treats **absolute temperature** and **temperature difference** as two separate quantity types with two separate unit symbols, exactly like `pint` and `uom` do:

| Library | Absolute type | Difference type | Difference unit prefix |
|---------|---------------|-----------------|-----------------------|
| Rust (`uom`) | `ThermodynamicTemperature` | `TemperatureInterval` | — (separate type) |
| Python (`pint`) | `ureg.kelvin`, `ureg.degC` | `ureg.delta_degC`, `ureg.delta_degF` | `delta_` |
| VLE canonical | **K** | **K** (ΔK) | — (shared SI unit; type differs) |

Both canonical units happen to be `K`, because 1 K of absolute change equals 1 K of interval. But the *types* are distinct so the compiler/runtime can reject `T + T` and also apply the correct conversion formula on each side of the API.

**Rust (compile-time enforcement):**

```rust
use uom::si::f64::{ThermodynamicTemperature, TemperatureInterval};
use uom::si::thermodynamic_temperature::degree_celsius;
use uom::si::temperature_interval::degree_celsius as delta_celsius;

let t1 = ThermodynamicTemperature::new::<degree_celsius>(25.0);  // absolute 25 °C
let t2 = ThermodynamicTemperature::new::<degree_celsius>(85.0);  // absolute 85 °C
let dt: TemperatureInterval = t2 - t1;                           // type inferred: interval
let t3 = t1 + dt;                                                // ✓ = 85 °C
let bad = t1 + t2;                                               // ✗ compile error
```

**Python (runtime enforcement):**

```python
from vle.units import ureg

t1 = ureg.Quantity(25, "degC")        # absolute
t2 = ureg.Quantity(85, "degC")        # absolute
dt = t2 - t1                          # → 60 delta_degC (pint promotes automatically)

dt.to("delta_degF")                   # → 108 delta_degF (× 9/5, no offset) ✓
dt.to("degF")                         # raises OffsetUnitCalculusError — pint refuses

t1 + t2                               # raises OffsetUnitCalculusError
t1 + dt                               # → 85 degC (absolute + interval = absolute) ✓
```

The VLE API follows the same rule: functions that accept a temperature *gradient*, a heat capacity, or a heat-transfer coefficient take the **difference** type; functions that accept a state temperature (bubble point, reactor T, etc.) take the **absolute** type. See §6 for the canonical type assignments.

### 3.4 Canonical unit strategy

To avoid `O(n²)` conversion pairs for `n` units in a dimension, define one **canonical unit** per dimension and convert through it:

```
any_unit_A → canonical → any_unit_B
```

This reduces the number of required conversions from `n²` to `2n`.

---

## 4. Implementation in Rust (Phantom Types)

Rust's type system allows dimensions to be encoded as **types**, so dimensional errors become **compile-time errors** with zero runtime overhead. The standard approach uses **phantom types**.

### 4.1 The Phantom Type Pattern

A phantom type is a generic type parameter that doesn't appear in the struct's runtime layout but participates in type checking:

```rust
use std::marker::PhantomData;

// A quantity with a dimension tag (phantom type)
pub struct Quantity<D> {
    value: f64,
    _dimension: PhantomData<D>,
}

// Dimension tags (zero-sized types)
pub struct Temperature;
pub struct Pressure;

// Typed constructors
impl Quantity<Temperature> {
    pub fn from_kelvin(k: f64) -> Self {
        Quantity { value: k, _dimension: PhantomData }
    }
}
```

Because `Quantity<Temperature>` and `Quantity<Pressure>` are **distinct types**, the compiler rejects:
```rust
let t: Quantity<Temperature> = Quantity::from_kelvin(300.0);
let p: Quantity<Pressure> = Quantity::from_kpa(101.325);
let sum = t + p;  // ← compile error: mismatched types
```

### 4.2 Encoding the 7-tuple with `typenum`

To represent arbitrary dimensions (including derived ones like molar entropy), the 7 exponents must be encoded as types. The `typenum` crate provides type-level integers:

```rust
use typenum::{N1, Z0, P1, P2};  // -1, 0, 1, 2

// MolarEnergy = L² · M · T⁻² · N⁻¹
type MolarEnergy = Dimension<P2, P1, N1, Z0, Z0, N1, Z0>;
//                            L   M   T   I   Θ   N   J
```

Multiplying two quantities becomes a type-level operation that adds the exponents, producing a new `Dimension<…>` type.

### 4.3 The `uom` crate

We'll use the `uom` (Units of Measurement) crate rather than writing this from scratch. `uom` provides:
- All 7 SI base dimensions as type-level tuples
- ~50 derived quantity types (Temperature, Pressure, Energy, MolarEnergy, etc.)
- ~500 unit definitions (kelvin, degree_celsius, pascal, kilopascal, atmosphere, psi, etc.)
- Compile-time dimension checking via typenum
- Zero runtime cost — quantities compile to plain `f64`

**Example use in `units/src/vle_units.rs`:**

```rust
use uom::si::f64::*;                    // Quantity types over f64
use uom::si::thermodynamic_temperature::{kelvin, degree_celsius};
use uom::si::temperature_interval::{kelvin as delta_kelvin, degree_celsius as delta_celsius};
use uom::si::pressure::{kilopascal, bar, atmosphere, psi};
use uom::si::molar_energy::kilojoule_per_kilomole;

/// Absolute temperature quantity (canonical unit: K).
/// Use for state variables: reactor T, bubble point, dew point, etc.
/// Conversion from °C/°F includes an offset.
pub type VleTemperature = ThermodynamicTemperature;

/// Temperature difference / gradient quantity (canonical unit: K = ΔK).
/// Use for ΔT, dT/dx, Cp denominators, heat-transfer driving forces.
/// Conversion from Δ°C/Δ°F is scale-only (no offset).
pub type VleTemperatureDiff = TemperatureInterval;

/// Pressure quantity (canonical unit: kPa)
pub type VlePressure = Pressure;

/// Molar energy quantity (canonical unit: kJ/kmol)
pub type VleMolarEnergy = MolarEnergy;

/// Construct temperature from Celsius.
///
/// # Arguments
/// * `c` — temperature in **°C**
///
/// # Returns
/// A typed `VleTemperature` (stored internally in **K**)
pub fn from_celsius(c: f64) -> VleTemperature {
    VleTemperature::new::<degree_celsius>(c)
}

/// Extract temperature in Kelvin.
///
/// # Returns
/// Temperature value in **K**
pub fn to_kelvin(t: VleTemperature) -> f64 {
    t.get::<kelvin>()
}
```

### 4.4 Safety Guarantees

With `uom`, the following cases are detected at **compile time**:

| Error | Compile-time check |
|-------|-------------------|
| `temperature + pressure` | ✓ Rejected (dimension mismatch) |
| `temperature * 2.0` | ✓ Allowed (scaling) |
| `volume / amount` → new type `MolarVolume` | ✓ Inferred correctly |
| Passing `Pressure` where `Temperature` is expected | ✓ Rejected |
| Forgetting a unit conversion | ✓ Rejected (cannot assign `f64` to `Pressure`) |

---

## 5. Implementation in Python (Runtime Dimensional Analysis)

Python is dynamically typed, so dimension checking happens at **runtime** instead of compile time. The `pint` library provides this capability.

### 5.1 The `pint` library

`pint` wraps numeric values in `Quantity` objects that carry dimension information:

```python
from pint import UnitRegistry

ureg = UnitRegistry()

t = 300 * ureg.kelvin              # Quantity(300, 'kelvin')
p = 101.325 * ureg.kPa             # Quantity(101.325, 'kilopascal')
v = t + p                          # DimensionalityError at runtime
```

Internally, `pint` stores dimensions as a mapping from base dimension name to exponent (equivalent to our 7-tuple):
```python
t.dimensionality  # <UnitsContainer({'[temperature]': 1})>
p.dimensionality  # <UnitsContainer({'[mass]': 1, '[length]': -1, '[time]': -2})>
```

### 5.2 NumPy integration

`pint` plays well with NumPy arrays:
```python
temperatures = np.array([273.15, 298.15, 373.15]) * ureg.kelvin
t_celsius = temperatures.to(ureg.degC)  # array([0, 25, 100]) °C
```

This is important for VLE calculations that operate on composition arrays or experimental data.

### 5.3 Example use in `python/src/vle/units.py`

```python
from pint import UnitRegistry

# Shared unit registry for the VLE package
ureg = UnitRegistry()
Q_ = ureg.Quantity

# Canonical units (match the Rust engine and legacy code)
CANONICAL = {
    "temperature": ureg.kelvin,              # absolute — offset conversion
    "temperature_diff": ureg.delta_degC,     # difference — scale-only (1 ΔK == 1 Δ°C)
    "pressure": ureg.kilopascal,
    "molar_energy": ureg.kJ / ureg.kmol,
    "molar_entropy": ureg.kJ / (ureg.kmol * ureg.kelvin),   # the K here is a difference
    "molar_volume": ureg.cm**3 / ureg.mol,
    "amount": ureg.kmol,
}

def to_canonical(value: float, unit_str: str, quantity: str) -> float:
    """Convert a value from the given unit to the canonical internal unit.

    Args:
        value: Numeric value
        unit_str: Source unit string (e.g., "degC", "psi", "cal/mol")
        quantity: Quantity name (e.g., "temperature", "pressure")

    Returns:
        Value in canonical units (K for temperature, kPa for pressure, etc.)
    """
    q = Q_(value, unit_str)
    return q.to(CANONICAL[quantity]).magnitude
```

---

## 6. Application to the VLE Units Add-On

The VLE project's canonical internal units match the legacy VB6/Pascal code (see `legacy/vb6/McommonFunctions.bas:3`, `legacy/pascal/TERMOI.PAS:13`, and the explicit comment at `legacy/pascal/TERMOII.PAS:62-63`).

### 6.1 Canonical Internal Units

| Quantity | Canonical Unit | Dimension Vector (L, M, T, I, Θ, N, J) |
|----------|---------------|----------------------------------------|
| Temperature (absolute) | K | (0, 0, 0, 0, 1, 0, 0) |
| Temperature difference / gradient (ΔT) | K | (0, 0, 0, 0, 1, 0, 0) |
| Pressure | kPa | (−1, 1, −2, 0, 0, 0, 0) |
| Molar energy (H, G) | kJ/kmol | (2, 1, −2, 0, 0, −1, 0) |
| Molar entropy (S) | kJ/(kmol·K) | (2, 1, −2, 0, −1, −1, 0) |
| Molar volume (V) | cm³/mol | (3, 0, 0, 0, 0, −1, 0) |
| Amount (n) | kmol | (0, 0, 0, 0, 0, 1, 0) |

Temperature (absolute) and temperature difference share the same dimension vector but are **separate quantity types** in the library (see §3.3). The `K` in `kJ/(kmol·K)` is a *difference* — so molar entropy scales linearly with °R without an offset. The `K` in "reactor temperature" is *absolute* and requires an offset when converting from °C or °F.

### 6.2 User-Facing Units (Supported Conversions)

| Quantity | User-Facing Alternatives |
|----------|-------------------------|
| Temperature (absolute) | °C, °F, °R |
| Temperature difference (ΔT) | Δ°C (= delta_degC), Δ°F (= delta_degF), Δ°R (= delta_degR) |
| Pressure | Pa, bar, atm, psi, mmHg, torr |
| Molar energy | J/mol, cal/mol, kcal/kmol, BTU/lbmol |
| Molar entropy | J/(mol·K), cal/(mol·K) |
| Molar volume | m³/kmol, L/mol, ft³/lbmol |
| Amount | mol, lbmol |

### 6.3 API Design

**User input flow** (Python wrapper → Rust engine):
```
User: system.bubble_point_T(P="1 atm")
  ↓ Python: pint parses "1 atm" → Quantity(1, atm)
  ↓ Python: convert to canonical kPa → 101.325
  ↓ PyO3: pass f64 to Rust engine
  ↓ Rust: engine computes in canonical units (kPa throughout)
```

**User output flow** (Rust engine → Python wrapper → user):
```
Rust: returns temperature as f64 in K (e.g., 394.263)
  ↓ PyO3: convert to Python float
  ↓ Python: wrap as Quantity(394.263, K)
  ↓ User: result.temperature.to("degC") → 121.113 °C
```

### 6.4 Verification Strategy

The units library will be verified by:

1. **Round-trip tests**: `from_canonical(to_canonical(x, "degC"), "degC") == x` for representative values
2. **Cross-library parity**: Rust-side conversion of `1 atm` → kPa must equal Python-side `pint` conversion to 6 decimal places
3. **Compile-time dimension check** (Rust): a unit test that attempts `temperature + pressure` must fail to compile (verified via `trybuild` crate)
4. **Runtime dimension check** (Python): a `pytest` test that attempts `t + p` must raise `pint.DimensionalityError`
5. **Validation against legacy values**: R = 8.31451 kJ/(kmol·K) (VB6) must round-trip through the units library unchanged

---

## 7. Extending the Library with Custom Units

The VLE units library ships with the units most commonly needed for thermodynamic calculations (see §6.2). However, users can **add their own units** without modifying the library source code. This is a **first-class feature** of the add-on.

There are two extension scenarios:

### Scenario A — Adding an Alternative Unit Within an Existing Dimension

Example: "I want to use `mmH2O` (millimeters of water) for pressure alongside the built-in kPa, bar, atm, psi, mmHg, torr."

### Scenario B — Adding a New Derived Quantity

Example: "I want to track heat transfer coefficient `W/(m²·K)`, which is not in the VLE canonical set."

Both scenarios are supported through a **runtime unit registry**. The registry is pre-populated with the VLE defaults but can be extended by the user.

### 7.1 Python — Extending via `pint`

The Python wrapper exposes the `UnitRegistry` instance directly, so users can call any `pint` extension API. This is the easiest path and covers both scenarios.

**Add a new unit (Scenario A):**

```python
from vle.units import ureg  # shared pint registry

# Define mmH2O using the existing pascal unit
ureg.define("mmH2O = 9.80665 * pascal = mm_H2O")

# Now usable anywhere the library accepts unit strings
from vle import System
system = System(components=["water", "ethanol"])
system.bubble_point_T(P="500 mmH2O")   # works transparently
```

**Add units from a file (bulk import):**

```python
# my_custom_units.txt
@defaults
    group = vle
@end

inH2O = 2.49088908333333 * hectopascal = inch_H2O
barg = 1 * bar = gauge_bar    # gauge pressure
```

```python
from vle.units import ureg
ureg.load_definitions("my_custom_units.txt")
```

**Add a new derived quantity (Scenario B):**

`pint` automatically composes derived units from base dimensions. No registration is needed — just write the unit string:

```python
from vle.units import ureg

h = 150 * ureg.watt / (ureg.meter**2 * ureg.kelvin)
h.to("BTU / (hour * foot**2 * degR)")    # works without defining anything
```

See the full `pint` extension guide: https://pint.readthedocs.io/en/stable/advanced/defining.html

### 7.2 Rust — Extending via the `UnitRegistry`

The Rust side offers two APIs in parallel:

1. **Compile-time typed API** (via `uom`) — used by the engine internals for maximum safety. Not extensible at runtime.
2. **Runtime registry API** (custom, built on top of `uom`) — used by the public FFI boundary for parsing user-provided unit strings. Fully extensible.

Users who want to add custom units interact with the **runtime registry**:

```rust
use vle_units::{UnitRegistry, Dimension};

let mut registry = UnitRegistry::with_vle_defaults();

// Scenario A: add a new unit within the Pressure dimension
// 1 mmH2O = 0.00980665 kPa (conversion factor to canonical kPa)
registry.define(
    "mmH2O",
    Dimension::Pressure,
    0.00980665,   // scale factor to canonical unit
    0.0,          // offset (0 for linear conversions)
)?;

// Now parseable
let p = registry.parse("100 mmH2O")?;   // → Pressure { value: 0.980665, unit: kPa }
```

**Adding a new derived dimension (Scenario B):**

```rust
use vle_units::{UnitRegistry, DimensionVector};

// Heat transfer coefficient: W/(m²·K) = kg·s⁻³·K⁻¹
// Dimension vector (L, M, T, I, Θ, N, J) = (0, 1, -3, 0, -1, 0, 0)
let htc = DimensionVector::new([0, 1, -3, 0, -1, 0, 0]);

registry.define_dimension("heat_transfer_coefficient", htc)?;
registry.define_with_dimension(
    "W_per_m2K",
    "heat_transfer_coefficient",
    1.0,   // canonical scale
    0.0,
)?;
registry.define_with_dimension(
    "BTU_per_hr_ft2_R",
    "heat_transfer_coefficient",
    5.67826,   // W/(m²·K) per BTU/(hr·ft²·°R)
    0.0,
)?;

let h = registry.parse("26.4 BTU_per_hr_ft2_R")?;
```

**Load from a TOML file (bulk import):**

```toml
# custom_units.toml

[[unit]]
name = "mmH2O"
dimension = "pressure"
scale = 0.00980665    # kPa per mmH2O
offset = 0.0

[[unit]]
name = "barg"
dimension = "pressure"
scale = 100.0         # kPa per bar
offset = 101.325      # absolute offset (gauge → absolute)

[[dimension]]
name = "heat_transfer_coefficient"
exponents = [0, 1, -3, 0, -1, 0, 0]   # L M T I Θ N J
```

```rust
let mut registry = UnitRegistry::with_vle_defaults();
registry.load_from_toml("custom_units.toml")?;
```

### 7.3 Keeping Rust and Python Registries in Sync

When a user adds a unit, they usually want it available **both** in Rust (for direct engine calls) and in Python (for the wrapper API). The library solves this by:

1. **Single source of truth**: custom units defined in a TOML file (`custom_units.toml`) can be loaded by both sides.
2. **Symmetric API**: `registry.define(name, dim, scale, offset)` works identically in Rust and Python.
3. **Parity test**: the test suite includes a golden file comparing conversions computed by both implementations for every registered unit.

**Recommended workflow:**

```
1. Create custom_units.toml with your extensions
2. In Rust:   UnitRegistry::with_vle_defaults().load_from_toml("custom_units.toml")
3. In Python: ureg.load_definitions("custom_units.toml")  (auto-translated from TOML)
4. Use the same unit string on both sides: parse("100 mmH2O")
```

### 7.4 Rules for User-Added Units

When adding custom units:

- **Name**: must be a valid identifier — letters, digits, underscores; no spaces or operators (`*`, `/`, `^`, `-`, `+`)
- **Dimension**: must either match an existing dimension name (case-sensitive) or be registered first via `define_dimension()`
- **Scale factor**: the conversion factor **to the canonical unit** for that dimension (not from; direction is "how many canonical units in one of my units")
- **Offset**: zero for pure scale conversions; nonzero for affine conversions like °C or gauge pressure
- **Uniqueness**: if a unit name already exists, `define()` returns an error unless `overwrite=true` is passed
- **Reserved names**: the VLE canonical unit names (`kelvin`, `kilopascal`, `kilojoule_per_kilomole`, etc.) cannot be redefined

### 7.5 Verifying a Custom Unit

After defining a unit, verify it with a round-trip test:

```python
from vle.units import ureg, to_canonical

ureg.define("mmH2O = 9.80665 * pascal")

original = 1000.0  # mmH2O
canonical = to_canonical(original, "mmH2O", "pressure")   # → kPa
restored = canonical / 0.00980665                          # should equal original

assert abs(restored - original) < 1e-9, "Round-trip failed"
```

For an unfamiliar physical quantity, always cross-check the dimension vector against a reference table (e.g., NIST's guide to the SI, Appendix B).

---

## 8. References

### Academic

- **Bridgman, P. W.** *Dimensional Analysis*, Yale University Press, 1922. — Foundational text establishing dimensional homogeneity as a physical principle.
- **BIPM** (Bureau International des Poids et Mesures), *The International System of Units (SI)*, 9th ed., 2019. — Official definition of the 7 SI base units. Available at https://www.bipm.org/en/publications/si-brochure.
- **Buckingham, E.** "On physically similar systems; illustrations of the use of dimensional equations", *Physical Review*, 4(4), 345 (1914). — Buckingham π theorem.

### Software

- **`uom` crate** (Rust): https://crates.io/crates/uom — Compile-time dimensional analysis via typenum-encoded phantom types. Zero runtime cost.
- **`typenum` crate** (Rust): https://crates.io/crates/typenum — Type-level integers used by `uom` to encode dimension exponents.
- **`pint` library** (Python): https://pint.readthedocs.io/ — Runtime dimensional analysis with NumPy integration.

### Project

- [MODERNIZATION_PLAN.md](../../../MODERNIZATION_PLAN.md) — Overall project plan (references this document)
- [CLAUDE.md](../../../CLAUDE.md) — "Units Documentation Rules" (units must be stated in every function doc comment)
- [ROADMAP.md](../../../ROADMAP.md) — Milestone 1.5 tracks implementation of this add-on
- [TODO.md](../../../TODO.md) — Detailed task breakdown with time estimates

---

[Back to English docs](../)
