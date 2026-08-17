//! Interconversion between distillation-curve types.
//!
//! # Why there is more than one kind of distillation curve
//!
//! "How much of this oil boils below 200 °C" sounds like one question. It is
//! four, because the answer depends on the apparatus:
//!
//! | basis | what it is | separation |
//! |---|---|---|
//! | **ASTM D86** | the cheap, fast, universal lab test, run at atmospheric pressure in a single-stage flask | ~1 theoretical plate — poor |
//! | **TBP** (D2892) | a real 15-plate column at 5:1 reflux; the closest thing to "the actual boiling points of the molecules" | ~15 plates — good |
//! | **D2887 / SimDist** | gas chromatography; elution time calibrated against n-alkanes. Reports **weight** percent | effectively perfect |
//! | **EFV** | equilibrium flash vaporization — one equilibrium stage, no fractionation at all. The flattest curve of the four | 0 plates |
//!
//! Every property correlation downstream of here is written against **TBP**,
//! because TBP is the one that approximates a list of component boiling points.
//! But almost every assay you will actually be handed is **D86** (for light
//! products) or **D2887** (for anything modern). So the first thing any
//! characterization does is convert.
//!
//! ```text
//!          D2887                        EFV
//!            │ API 3A3.1                 │ Edmister-Okamoto
//!            ▼                           ▼
//!          ═══════════ TBP ══════════ D86
//!                          API 3A1.1
//! ```
//!
//! [`convert_curve`] routes any basis to any other through TBP as the hub, which
//! is why TBP is the only node with two edges into it.
//!
//! # Two families of method
//!
//! **Point-wise power laws** — `T* = a·Tᵇ` with `(a, b)` chosen by which decile
//! the point falls in. Riazi–Daubert (34) for D86↔TBP, Edmister–Okamoto (39) for
//! D86↔EFV. Each point converts independently, so they need no particular grid.
//!
//! > **Caveat, measured.** Because the rows of a point-wise table are fitted
//! > independently, nothing forces the converted curve to stay monotone. The
//! > Edmister–Okamoto **0–10 % row really does cross its 10–30 % neighbour**:
//! > for any D86 curve whose 0–90 % span is under roughly 250 K — which is most
//! > real feeds — the converted EFV initial point lands *above* the 10 % point.
//! > [`DistillationCurve::new`] rejects the result rather than returning a
//! > decreasing curve, so this surfaces as an error rather than as nonsense
//! > downstream. **The fix is to convert a 10–90 % curve**, dropping the initial
//! > point: it is the least reliable point of a D86 anyway, and the remaining
//! > rows are well behaved. Pinned by
//! > `efv_initial_point_row_crosses_its_neighbour_on_narrow_feeds`.
//!
//! **Difference (delta) methods** — the API procedures in (35). These convert
//! the **50 % point** with one power law, then convert each *temperature
//! difference* between adjacent cut points with its own power law, and
//! rebuild the curve by accumulating outward from 50 %. They are the API's
//! recommended procedures and are generally more accurate, but they are defined
//! on a fixed cut-point grid and need a 50 % point to anchor on.
//!
//! # References
//! - (34) Riazi & Daubert, *Oil Gas J.* **1986**, *84*, 50–57.
//! - (35) Daubert, *Hydrocarbon Process.* **1994**, *73* (9), 75–78 —
//!   API Procedures 3A1.1 (D86↔TBP), 3A3.1 (SD→TBP), 3A3.2 (SD→D86).
//! - (39) Edmister & Okamoto, *Pet. Refiner* **1959**, *38* (9), 271–288.
//! - (31) Riazi 2005, ch. 3 — the worked examples used as test oracles below.

use super::PetroleumError;
use super::gravity::{f_to_k, k_to_f};

/// Which kind of distillation curve a set of temperatures represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
pub enum DistillationBasis {
    /// ASTM D86 — atmospheric single-stage flask distillation, volume percent.
    D86,
    /// True boiling point (ASTM D2892) — 15-plate column, volume percent.
    /// The hub basis: every property correlation downstream wants this.
    Tbp,
    /// ASTM D2887 simulated distillation by gas chromatography, **weight**
    /// percent. Also written "SD" or "SimDist".
    D2887,
    /// Equilibrium flash vaporization, volume percent.
    Efv,
}

impl DistillationBasis {
    /// Human-readable name, for error messages and Python `repr`.
    pub fn name(&self) -> &'static str {
        match self {
            DistillationBasis::D86 => "ASTM D86",
            DistillationBasis::Tbp => "TBP",
            DistillationBasis::D2887 => "ASTM D2887 (SimDist)",
            DistillationBasis::Efv => "EFV",
        }
    }

    /// Whether the curve's abscissa is a **weight** fraction rather than a
    /// volume fraction. Only D2887 is on a weight basis.
    ///
    /// This matters when cutting: a weight-basis curve must be converted before
    /// its fractions can be treated as volumes.
    pub fn is_weight_basis(&self) -> bool {
        matches!(self, DistillationBasis::D2887)
    }
}

/// A distillation curve: a monotone map from fraction distilled to temperature.
///
/// Both vectors have the same length. `fractions` are **strictly increasing**
/// and lie in `[0, 1]`; `temperatures` are **non-decreasing** and in **K**.
#[derive(Debug, Clone, PartialEq)]
pub struct DistillationCurve {
    /// Which apparatus these temperatures came from.
    pub basis: DistillationBasis,
    /// Fraction distilled, **dimensionless** in `[0, 1]`. Volume fraction for
    /// every basis except [`DistillationBasis::D2887`], which is weight.
    pub fractions: Vec<f64>,
    /// Temperature at each fraction, **K**.
    pub temperatures: Vec<f64>,
}

/// The cut-point grid the API interconversion procedures are defined on.
pub const STANDARD_GRID: [f64; 7] = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95];

impl DistillationCurve {
    /// Build and validate a curve.
    ///
    /// # Arguments
    /// * `basis` — which apparatus produced it.
    /// * `fractions` — fraction distilled, **dimensionless**, strictly
    ///   increasing, each in `[0, 1]`.
    /// * `temperatures` — **K**, non-decreasing, all positive.
    ///
    /// # Errors
    /// [`PetroleumError::Curve`] if the two vectors differ in length or hold
    /// fewer than two points; [`PetroleumError::CutPoints`] if the fractions are
    /// out of range or not increasing; [`PetroleumError::InvalidInput`] if a
    /// temperature is non-positive or the curve decreases.
    pub fn new(
        basis: DistillationBasis,
        fractions: Vec<f64>,
        temperatures: Vec<f64>,
    ) -> Result<Self, PetroleumError> {
        if fractions.len() != temperatures.len() {
            return Err(PetroleumError::Curve(format!(
                "{} fractions but {} temperatures",
                fractions.len(),
                temperatures.len()
            )));
        }
        if fractions.len() < 2 {
            return Err(PetroleumError::Curve(format!(
                "a distillation curve needs at least 2 points, got {}",
                fractions.len()
            )));
        }
        for (i, &x) in fractions.iter().enumerate() {
            if !(0.0..=1.0).contains(&x) || !x.is_finite() {
                return Err(PetroleumError::CutPoints(format!(
                    "fraction[{i}] = {x} is outside [0, 1]"
                )));
            }
            if i > 0 && x <= fractions[i - 1] {
                return Err(PetroleumError::CutPoints(format!(
                    "fractions must strictly increase: fraction[{}] = {} is not above fraction[{}] = {}",
                    i,
                    x,
                    i - 1,
                    fractions[i - 1]
                )));
            }
        }
        for (i, &t) in temperatures.iter().enumerate() {
            if t <= 0.0 || !t.is_finite() {
                return Err(PetroleumError::InvalidInput(format!(
                    "temperature[{i}] = {t} K is not a positive finite temperature"
                )));
            }
            if i > 0 && t < temperatures[i - 1] {
                return Err(PetroleumError::InvalidInput(format!(
                    "a distillation curve cannot decrease: temperature[{}] = {} K is below temperature[{}] = {} K",
                    i,
                    t,
                    i - 1,
                    temperatures[i - 1]
                )));
            }
        }
        Ok(Self {
            basis,
            fractions,
            temperatures,
        })
    }

    /// Number of points on the curve.
    pub fn len(&self) -> usize {
        self.fractions.len()
    }

    /// Always `false` — [`DistillationCurve::new`] rejects curves with fewer
    /// than two points, so a constructed curve is never empty. Present because
    /// clippy asks for it wherever `len` exists.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Temperature at an arbitrary fraction, by linear interpolation in **K**.
    ///
    /// Outside the curve's span the two end segments are extrapolated linearly.
    /// That is deliberate: an assay that stops at 90 % still has to be cut past
    /// 90 %, and a linear extrapolation of the last segment is the standard
    /// (if crude) engineering answer. A caller that wants the residue treated
    /// differently should add an explicit end point.
    ///
    /// # Arguments
    /// * `fraction` — fraction distilled, **dimensionless**.
    ///
    /// # Returns
    /// Interpolated temperature in **K**.
    pub fn temperature_at(&self, fraction: f64) -> f64 {
        let n = self.len();
        let xs = &self.fractions;
        let ts = &self.temperatures;
        // Below the first point: extrapolate along the first segment.
        if fraction <= xs[0] {
            let slope = (ts[1] - ts[0]) / (xs[1] - xs[0]);
            return ts[0] + slope * (fraction - xs[0]);
        }
        // Above the last: extrapolate along the last segment.
        if fraction >= xs[n - 1] {
            let slope = (ts[n - 1] - ts[n - 2]) / (xs[n - 1] - xs[n - 2]);
            return ts[n - 1] + slope * (fraction - xs[n - 1]);
        }
        // Inside: find the bracketing segment and interpolate.
        let hi = xs.partition_point(|&x| x < fraction).max(1);
        let lo = hi - 1;
        let w = (fraction - xs[lo]) / (xs[hi] - xs[lo]);
        ts[lo] + w * (ts[hi] - ts[lo])
    }

    /// The inverse of [`temperature_at`]: what fraction has distilled by the
    /// time the curve reaches `temperature`.
    ///
    /// Linear inverse interpolation, with the same end-segment extrapolation.
    /// The result is **not** clamped to `[0, 1]` — a caller cutting an assay
    /// at a temperature the curve never reaches wants to see that it fell off
    /// the end rather than get a silently saturated 0 or 1.
    ///
    /// [`temperature_at`]: DistillationCurve::temperature_at
    ///
    /// # Arguments
    /// * `temperature` — **K**.
    ///
    /// # Returns
    /// Fraction distilled, **dimensionless**, possibly outside `[0, 1]`.
    ///
    /// A curve with a flat segment (two grid points at the same temperature)
    /// has no unique inverse there; the lower edge of the flat segment is
    /// returned, which is the conventional choice.
    pub fn fraction_at(&self, temperature: f64) -> f64 {
        let n = self.len();
        let xs = &self.fractions;
        let ts = &self.temperatures;
        let invert = |lo: usize, hi: usize| -> f64 {
            let dt = ts[hi] - ts[lo];
            if dt.abs() < f64::EPSILON {
                // Flat segment: no unique inverse. Return its lower edge.
                return xs[lo];
            }
            xs[lo] + (temperature - ts[lo]) / dt * (xs[hi] - xs[lo])
        };
        if temperature <= ts[0] {
            return invert(0, 1);
        }
        if temperature >= ts[n - 1] {
            return invert(n - 2, n - 1);
        }
        let hi = ts.partition_point(|&t| t < temperature).max(1);
        invert(hi - 1, hi)
    }

    /// Re-express the curve on a different set of fractions, interpolating.
    ///
    /// The usual use is `curve.resample(&STANDARD_GRID)` before handing a curve
    /// to one of the API delta procedures, which are defined on that grid.
    ///
    /// # Arguments
    /// * `fractions` — the target fractions, **dimensionless**, strictly
    ///   increasing in `[0, 1]`.
    ///
    /// # Returns
    /// A new curve on the same basis at the requested fractions.
    pub fn resample(&self, fractions: &[f64]) -> Result<Self, PetroleumError> {
        let temperatures = fractions.iter().map(|&x| self.temperature_at(x)).collect();
        Self::new(self.basis, fractions.to_vec(), temperatures)
    }

    /// Index of the 50 % point, or an error naming what to do about it.
    ///
    /// The API delta procedures anchor on the 50 % point and accumulate
    /// temperature differences outward from it, so it must be an actual grid
    /// point rather than something interpolated — interpolating it would make
    /// the deltas on either side inconsistent with the anchor.
    fn index_of_half(&self) -> Result<usize, PetroleumError> {
        self.fractions
            .iter()
            .position(|&x| (x - 0.5).abs() < 1e-9)
            .ok_or_else(|| {
                PetroleumError::CutPoints(
                    "the API difference procedures need the 50% point on the curve; \
                     call `resample(&STANDARD_GRID)` first"
                        .into(),
                )
            })
    }
}

// ---------------------------------------------------------------------------
// Interval coefficient lookup
// ---------------------------------------------------------------------------

/// Pick the coefficient row whose `[lo, hi)` fraction interval contains `x`.
///
/// The published tables are step functions of the cut point, so every method
/// below is "look up the row, apply the formula". The last interval is closed
/// on the right so that `x = 1.0` lands in it rather than falling off the end.
fn row_for<const N: usize>(bounds: &[f64; N], x: f64) -> usize {
    for i in (0..N).rev() {
        if x >= bounds[i] {
            return i;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// D86 <-> TBP, Riazi-Daubert (34) point-wise power law, temperatures in K
// ---------------------------------------------------------------------------

/// Lower edges of the seven Riazi–Daubert cut-point intervals.
const RIAZI_BOUNDS: [f64; 7] = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95];
/// `a` in `T_TBP = a · T_D86^b`, T in **K**. Ref (34).
const RIAZI_A: [f64; 7] = [0.9177, 0.5564, 0.765_17, 0.9013, 0.8821, 0.9552, 0.8177];
/// `b` in `T_TBP = a · T_D86^b`, T in **K**. Ref (34).
const RIAZI_B: [f64; 7] = [1.0019, 1.09, 1.0425, 1.0176, 1.0226, 1.011, 1.0355];

/// Convert an ASTM D86 curve to TBP with the Riazi–Daubert power law.
///
/// `T_TBP = a · T_D86^b`, applied point by point with `(a, b)` chosen by cut
/// point. Temperatures are in **K** throughout — this correlation, unlike the
/// API delta procedure, really was published in Kelvin.
///
/// Ref (34): Riazi & Daubert, *Oil Gas J.* **1986**, *84*, 50–57.
///
/// # Arguments
/// * `curve` — a curve on basis [`DistillationBasis::D86`], temperatures in **K**.
///
/// # Returns
/// The same fractions on basis [`DistillationBasis::Tbp`], temperatures in **K**.
pub fn d86_to_tbp_riazi(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::D86)?;
    let t = curve
        .fractions
        .iter()
        .zip(&curve.temperatures)
        .map(|(&x, &t)| {
            let i = row_for(&RIAZI_BOUNDS, x);
            RIAZI_A[i] * t.powf(RIAZI_B[i])
        })
        .collect();
    DistillationCurve::new(DistillationBasis::Tbp, curve.fractions.clone(), t)
}

/// Convert a TBP curve to ASTM D86 — the exact algebraic inverse of
/// [`d86_to_tbp_riazi`], `T_D86 = (T_TBP / a)^(1/b)`.
///
/// # Arguments
/// * `curve` — a curve on basis [`DistillationBasis::Tbp`], temperatures in **K**.
///
/// # Returns
/// The same fractions on basis [`DistillationBasis::D86`], temperatures in **K**.
pub fn tbp_to_d86_riazi(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::Tbp)?;
    let t = curve
        .fractions
        .iter()
        .zip(&curve.temperatures)
        .map(|(&x, &t)| {
            let i = row_for(&RIAZI_BOUNDS, x);
            (t / RIAZI_A[i]).powf(1.0 / RIAZI_B[i])
        })
        .collect();
    DistillationCurve::new(DistillationBasis::D86, curve.fractions.clone(), t)
}

// ---------------------------------------------------------------------------
// D86 <-> TBP, API Procedure 3A1.1 (Daubert 1994) difference method, in °F
// ---------------------------------------------------------------------------

/// Lower edges of the six API difference intervals, `[0-10, 10-30, …, 90-100]`.
const DELTA_BOUNDS: [f64; 6] = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90];
/// `A` in `ΔT_TBP = A · (ΔT_D86)^B`, ΔT in **°F**. API 3A1.1, ref (35).
const D86_TBP_A: [f64; 6] = [7.4012, 4.9004, 3.0305, 2.5282, 3.0419, 0.117_98];
/// `B` in `ΔT_TBP = A · (ΔT_D86)^B`, ΔT in **°F**. API 3A1.1, ref (35).
const D86_TBP_B: [f64; 6] = [0.602_44, 0.716_44, 0.800_76, 0.820_02, 0.754_97, 1.6606];

/// Rebuild a curve from a converted 50 % anchor plus converted differences.
///
/// This is the shape shared by all three API difference procedures. Every point
/// starts at the converted 50 % temperature; points below 50 % subtract the
/// converted differences between themselves and the 50 % point, points above add
/// them. Working outward from 50 % rather than from the initial point is what
/// keeps an error in one difference from propagating across the whole curve.
///
/// `deltas[j]` is the converted difference across the interval from grid point
/// `j` to `j + 1`, in the same units as `anchor`.
fn accumulate_from_half(anchor: f64, deltas: &[f64], half: usize) -> Vec<f64> {
    let n = deltas.len() + 1;
    let mut out = vec![anchor; n];
    // Below the anchor: walk down, subtracting each interval as we pass it.
    for i in (0..half).rev() {
        out[i] = out[i + 1] - deltas[i];
    }
    // Above the anchor: walk up, adding each interval as we pass it.
    for i in half + 1..n {
        out[i] = out[i - 1] + deltas[i - 1];
    }
    out
}

/// Convert an ASTM D86 curve to TBP with API Procedure 3A1.1.
///
/// The 50 % point converts as `T_TBP,50 = 0.87180 · T_D86,50^1.0258`, and each
/// temperature difference as `ΔT_TBP = A · ΔT_D86^B`. **All of this is in °F** —
/// the near-identity of the 50 % relation only holds on the Fahrenheit scale
/// (its fixed point is 204 °F, a real mid-distillate temperature), and running
/// it in K or °R silently biases every result. The conversion in and out
/// happens here.
///
/// Ref (35): Daubert, *Hydrocarbon Process.* **1994**, *73* (9), 75–78.
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::D86`], temperatures in **K**, and it
///   must carry a 50 % point (see [`DistillationCurve::resample`]).
///
/// # Returns
/// Basis [`DistillationBasis::Tbp`], temperatures in **K**.
pub fn d86_to_tbp_daubert(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::D86)?;
    let half = curve.index_of_half()?;
    let f: Vec<f64> = curve.temperatures.iter().map(|&t| k_to_f(t)).collect();

    let anchor = 0.871_80 * f[half].powf(1.0258);
    let deltas: Vec<f64> = (0..f.len() - 1)
        .map(|j| {
            let i = row_for(&DELTA_BOUNDS, curve.fractions[j]);
            D86_TBP_A[i] * (f[j + 1] - f[j]).max(0.0).powf(D86_TBP_B[i])
        })
        .collect();

    let tbp = accumulate_from_half(anchor, &deltas, half);
    DistillationCurve::new(
        DistillationBasis::Tbp,
        curve.fractions.clone(),
        tbp.into_iter().map(f_to_k).collect(),
    )
}

/// Convert a TBP curve to ASTM D86 — the inverse of [`d86_to_tbp_daubert`].
///
/// Both halves invert exactly: the anchor as `(T/0.87180)^(1/1.0258)` and each
/// difference as `(ΔT_TBP/A)^(1/B)`.
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::Tbp`], **K**, carrying a 50 % point.
///
/// # Returns
/// Basis [`DistillationBasis::D86`], temperatures in **K**.
pub fn tbp_to_d86_daubert(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::Tbp)?;
    let half = curve.index_of_half()?;
    let f: Vec<f64> = curve.temperatures.iter().map(|&t| k_to_f(t)).collect();

    let anchor = (f[half] / 0.871_80).powf(1.0 / 1.0258);
    let deltas: Vec<f64> = (0..f.len() - 1)
        .map(|j| {
            let i = row_for(&DELTA_BOUNDS, curve.fractions[j]);
            ((f[j + 1] - f[j]).max(0.0) / D86_TBP_A[i]).powf(1.0 / D86_TBP_B[i])
        })
        .collect();

    let d86 = accumulate_from_half(anchor, &deltas, half);
    DistillationCurve::new(
        DistillationBasis::D86,
        curve.fractions.clone(),
        d86.into_iter().map(f_to_k).collect(),
    )
}

// ---------------------------------------------------------------------------
// D2887 (SimDist) <-> TBP, API Procedure 3A3.1, in °F
// ---------------------------------------------------------------------------

/// Lower edges of the seven SD→TBP intervals. Note this grid starts at **5 %**,
/// not 0 % — a chromatogram has no meaningful initial point.
const SD_BOUNDS: [f64; 7] = [0.05, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95];
/// `C` in `ΔT_TBP = C · (ΔT_SD)^D`, ΔT in **°F**. API 3A3.1, ref (35).
const SD_TBP_C: [f64; 7] = [
    0.157_79, 0.011_903, 0.053_42, 0.198_61, 0.315_31, 0.974_76, 0.021_72,
];
/// `D` in `ΔT_TBP = C · (ΔT_SD)^D`, ΔT in **°F**. API 3A3.1, ref (35).
const SD_TBP_D: [f64; 7] = [1.4296, 2.0253, 1.6988, 1.3975, 1.2938, 0.8723, 1.9733];

/// Convert an ASTM D2887 (SimDist) curve to TBP with API Procedure 3A3.1.
///
/// The 50 % points of the two curves are taken to be **equal** — gas
/// chromatography separates so well that its median already *is* the true
/// boiling median — so only the differences need converting. Ref (35).
///
/// Note the abscissa: D2887 reports **weight** percent while TBP is volume
/// percent. This procedure converts the temperatures only and leaves the
/// fractions alone, which is what the API procedure specifies; converting a
/// weight basis to a volume basis needs a gravity distribution and is done in
/// [`super::cuts`].
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::D2887`], **K**, carrying a 50 % point.
///
/// # Returns
/// Basis [`DistillationBasis::Tbp`], temperatures in **K**.
pub fn d2887_to_tbp(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::D2887)?;
    let half = curve.index_of_half()?;
    let f: Vec<f64> = curve.temperatures.iter().map(|&t| k_to_f(t)).collect();

    let deltas: Vec<f64> = (0..f.len() - 1)
        .map(|j| {
            let i = row_for(&SD_BOUNDS, curve.fractions[j]);
            SD_TBP_C[i] * (f[j + 1] - f[j]).max(0.0).powf(SD_TBP_D[i])
        })
        .collect();

    let tbp = accumulate_from_half(f[half], &deltas, half);
    DistillationCurve::new(
        DistillationBasis::Tbp,
        curve.fractions.clone(),
        tbp.into_iter().map(f_to_k).collect(),
    )
}

/// Convert a TBP curve to ASTM D2887 — the inverse of [`d2887_to_tbp`].
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::Tbp`], **K**, carrying a 50 % point.
///
/// # Returns
/// Basis [`DistillationBasis::D2887`], temperatures in **K**.
pub fn tbp_to_d2887(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::Tbp)?;
    let half = curve.index_of_half()?;
    let f: Vec<f64> = curve.temperatures.iter().map(|&t| k_to_f(t)).collect();

    let deltas: Vec<f64> = (0..f.len() - 1)
        .map(|j| {
            let i = row_for(&SD_BOUNDS, curve.fractions[j]);
            ((f[j + 1] - f[j]).max(0.0) / SD_TBP_C[i]).powf(1.0 / SD_TBP_D[i])
        })
        .collect();

    let sd = accumulate_from_half(f[half], &deltas, half);
    DistillationCurve::new(
        DistillationBasis::D2887,
        curve.fractions.clone(),
        sd.into_iter().map(f_to_k).collect(),
    )
}

// ---------------------------------------------------------------------------
// D2887 -> D86, API Procedure 3A3.2, in °F
// ---------------------------------------------------------------------------

/// `E` in `ΔT_D86 = E · (ΔT_SD)^F`, ΔT in **°F**. API 3A3.2, ref (35).
const SD_D86_E: [f64; 6] = [0.3047, 0.060_69, 0.079_78, 0.148_62, 0.307_85, 2.6029];
/// `F` in `ΔT_D86 = E · (ΔT_SD)^F`, ΔT in **°F**. API 3A3.2, ref (35).
const SD_D86_F: [f64; 6] = [1.1259, 1.5176, 1.5386, 1.4287, 1.2341, 0.659_62];

/// Convert an ASTM D2887 (SimDist) curve directly to ASTM D86, API 3A3.2.
///
/// Provided because it is the procedure a refinery lab actually runs when it
/// wants to report a chromatographic result on the D86 basis a product spec is
/// written against. The 50 % anchor is `T_D86,50 = 0.77601 · T_SD,50^1.0395`,
/// in **°F**. Ref (35).
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::D2887`], **K**, carrying a 50 % point.
///
/// # Returns
/// Basis [`DistillationBasis::D86`], temperatures in **K**.
pub fn d2887_to_d86(curve: &DistillationCurve) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::D2887)?;
    let half = curve.index_of_half()?;
    let f: Vec<f64> = curve.temperatures.iter().map(|&t| k_to_f(t)).collect();

    let anchor = 0.776_01 * f[half].powf(1.0395);
    let deltas: Vec<f64> = (0..f.len() - 1)
        .map(|j| {
            let i = row_for(&DELTA_BOUNDS, curve.fractions[j]);
            SD_D86_E[i] * (f[j + 1] - f[j]).max(0.0).powf(SD_D86_F[i])
        })
        .collect();

    let d86 = accumulate_from_half(anchor, &deltas, half);
    DistillationCurve::new(
        DistillationBasis::D86,
        curve.fractions.clone(),
        d86.into_iter().map(f_to_k).collect(),
    )
}

// ---------------------------------------------------------------------------
// D86 <-> EFV, Edmister-Okamoto (39), point-wise, temperatures in K
// ---------------------------------------------------------------------------

/// Lower edges of the seven EFV intervals.
const EFV_BOUNDS: [f64; 7] = [0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 1.0];
/// `a` in `T_EFV = a · T_D86^b · SG^c`, T in **K**. Ref (39).
const EFV_A: [f64; 7] = [2.9747, 1.4459, 0.8506, 3.268, 8.2873, 10.6266, 7.9952];
/// `b` in `T_EFV = a · T_D86^b · SG^c`, T in **K**. Ref (39).
const EFV_B: [f64; 7] = [0.8466, 0.9511, 1.0315, 0.8274, 0.6874, 0.6529, 0.6949];
/// `c` in `T_EFV = a · T_D86^b · SG^c`, T in **K**. Ref (39).
const EFV_C: [f64; 7] = [0.4209, 0.1287, 0.0817, 0.6214, 0.934, 1.1025, 1.0737];

/// Convert an ASTM D86 curve to EFV, Edmister–Okamoto.
///
/// `T_EFV = a · T_D86^b · SG^c`, point by point, T in **K**. Unlike the other
/// conversions this one needs the fraction's **specific gravity**: an EFV curve
/// is a real phase-equilibrium calculation in disguise, and how much a given
/// cut flashes depends on how dense it is.
///
/// Ref (39): Edmister & Okamoto, *Pet. Refiner* **1959**, *38* (9), 271–288.
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::D86`], temperatures in **K**. On a
///   curve that includes a **0 % point**, expect this to fail for anything but a
///   very wide-boiling feed — see the caveat in the module docs, and pass a
///   10–90 % curve instead.
/// * `sg` — bulk specific gravity at 60/60 °F, **dimensionless**.
///
/// # Returns
/// Basis [`DistillationBasis::Efv`], temperatures in **K**.
pub fn d86_to_efv(curve: &DistillationCurve, sg: f64) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::D86)?;
    check_sg(sg)?;
    let t = curve
        .fractions
        .iter()
        .zip(&curve.temperatures)
        .map(|(&x, &t)| {
            let i = row_for(&EFV_BOUNDS, x);
            EFV_A[i] * t.powf(EFV_B[i]) * sg.powf(EFV_C[i])
        })
        .collect();
    DistillationCurve::new(DistillationBasis::Efv, curve.fractions.clone(), t)
}

/// Convert an EFV curve to ASTM D86 — the exact inverse of [`d86_to_efv`].
///
/// # Arguments
/// * `curve` — basis [`DistillationBasis::Efv`], temperatures in **K**.
/// * `sg` — bulk specific gravity at 60/60 °F, **dimensionless**.
///
/// # Returns
/// Basis [`DistillationBasis::D86`], temperatures in **K**.
pub fn efv_to_d86(curve: &DistillationCurve, sg: f64) -> Result<DistillationCurve, PetroleumError> {
    expect_basis(curve, DistillationBasis::Efv)?;
    check_sg(sg)?;
    let t = curve
        .fractions
        .iter()
        .zip(&curve.temperatures)
        .map(|(&x, &t)| {
            let i = row_for(&EFV_BOUNDS, x);
            (t / (EFV_A[i] * sg.powf(EFV_C[i]))).powf(1.0 / EFV_B[i])
        })
        .collect();
    DistillationCurve::new(DistillationBasis::D86, curve.fractions.clone(), t)
}

// ---------------------------------------------------------------------------
// The router
// ---------------------------------------------------------------------------

/// Convert a distillation curve from any basis to any other.
///
/// Routes through TBP as the hub (see the module diagram), preferring the API
/// difference procedures where they exist because they are the more accurate
/// family. EFV is reached through D86, since that is the only leg
/// Edmister–Okamoto provides.
///
/// # Arguments
/// * `curve` — the source curve, temperatures in **K**.
/// * `target` — the basis to convert to.
/// * `sg` — bulk specific gravity, **dimensionless**. Required only when the
///   route touches EFV; pass `None` otherwise.
///
/// # Returns
/// A curve on `target` with the same fractions, temperatures in **K**.
/// Converting to the basis a curve is already on returns a clone.
///
/// # Errors
/// [`PetroleumError::InvalidInput`] if an EFV leg is requested without `sg`;
/// [`PetroleumError::CutPoints`] if a difference procedure is reached with no
/// 50 % point on the curve.
pub fn convert_curve(
    curve: &DistillationCurve,
    target: DistillationBasis,
    sg: Option<f64>,
) -> Result<DistillationCurve, PetroleumError> {
    if curve.basis == target {
        return Ok(curve.clone());
    }
    // Short-circuit the one pair that has a direct published procedure in each
    // direction and would otherwise take a lossy two-hop route.
    match (curve.basis, target) {
        (DistillationBasis::D86, DistillationBasis::Efv) => {
            return d86_to_efv(curve, need_sg(sg)?);
        }
        (DistillationBasis::Efv, DistillationBasis::D86) => {
            return efv_to_d86(curve, need_sg(sg)?);
        }
        (DistillationBasis::D2887, DistillationBasis::D86) => return d2887_to_d86(curve),
        _ => {}
    }

    // Everything else: hop to TBP, then out to the target.
    let tbp = match curve.basis {
        DistillationBasis::Tbp => curve.clone(),
        DistillationBasis::D86 => d86_to_tbp_daubert(curve)?,
        DistillationBasis::D2887 => d2887_to_tbp(curve)?,
        DistillationBasis::Efv => d86_to_tbp_daubert(&efv_to_d86(curve, need_sg(sg)?)?)?,
    };
    match target {
        DistillationBasis::Tbp => Ok(tbp),
        DistillationBasis::D86 => tbp_to_d86_daubert(&tbp),
        DistillationBasis::D2887 => tbp_to_d2887(&tbp),
        DistillationBasis::Efv => d86_to_efv(&tbp_to_d86_daubert(&tbp)?, need_sg(sg)?),
    }
}

fn need_sg(sg: Option<f64>) -> Result<f64, PetroleumError> {
    let sg = sg.ok_or_else(|| {
        PetroleumError::InvalidInput(
            "an EFV conversion needs the fraction's specific gravity".into(),
        )
    })?;
    check_sg(sg)?;
    Ok(sg)
}

fn check_sg(sg: f64) -> Result<(), PetroleumError> {
    if sg <= 0.0 || !sg.is_finite() {
        return Err(PetroleumError::InvalidInput(format!(
            "specific gravity must be positive and finite, got {sg}"
        )));
    }
    Ok(())
}

fn expect_basis(curve: &DistillationCurve, want: DistillationBasis) -> Result<(), PetroleumError> {
    if curve.basis != want {
        return Err(PetroleumError::InvalidInput(format!(
            "expected a {} curve, got {}",
            want.name(),
            curve.basis.name()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a curve from °C temperatures, which is how the source examples are
    /// printed. Keeps the test data visually identical to the published table.
    fn from_celsius(basis: DistillationBasis, x: &[f64], t_c: &[f64]) -> DistillationCurve {
        DistillationCurve::new(basis, x.to_vec(), t_c.iter().map(|t| t + 273.15).collect()).unwrap()
    }

    fn to_celsius(c: &DistillationCurve) -> Vec<f64> {
        c.temperatures.iter().map(|t| t - 273.15).collect()
    }

    fn from_fahrenheit(basis: DistillationBasis, x: &[f64], t_f: &[f64]) -> DistillationCurve {
        DistillationCurve::new(basis, x.to_vec(), t_f.iter().map(|&t| f_to_k(t)).collect()).unwrap()
    }

    // === Published worked examples ======================================
    //
    // These are the acceptance tests for the whole module. Each reproduces a
    // numbered example from Riazi 2005 (31) or the API Technical Data Book
    // (41); if a coefficient is mistyped, one of these fails.

    #[test]
    fn riazi_example_3_3_d86_to_tbp_power_law() {
        // Riazi 2005 (31), Example 3.3 — a gas oil, D86 -> TBP by the
        // Riazi-Daubert power law. Expected TBP in °C:
        //   134.2  157.4  190.3  209.0  230.2  254.7
        let x = [0.0, 0.1, 0.3, 0.5, 0.7, 0.9];
        let d86 = from_celsius(
            DistillationBasis::D86,
            &x,
            &[165.6, 173.7, 193.3, 206.7, 222.8, 242.8],
        );
        let tbp = to_celsius(&d86_to_tbp_riazi(&d86).unwrap());
        let want = [134.2, 157.4, 190.3, 209.0, 230.2, 254.7];
        for (got, want) in tbp.iter().zip(want) {
            assert!(
                (got - want).abs() < 0.15,
                "TBP {got:.2} °C vs published {want:.1} °C (curve {tbp:?})"
            );
        }
    }

    #[test]
    fn riazi_example_3_3_d86_to_tbp_api_difference_method() {
        // The same feed through API 3A1.1 instead. Riazi prints both so the
        // two methods can be compared; expected TBP in °C:
        //   133.5  154.2  189.2  210.7  232.9  258.2
        // This is the test that pins the °F convention: running the identical
        // coefficients in K instead moves the 0 % point by tens of degrees.
        let x = [0.0, 0.1, 0.3, 0.5, 0.7, 0.9];
        let d86 = from_celsius(
            DistillationBasis::D86,
            &x,
            &[165.6, 173.7, 193.3, 206.7, 222.8, 242.8],
        );
        let tbp = to_celsius(&d86_to_tbp_daubert(&d86).unwrap());
        let want = [133.5, 154.2, 189.2, 210.7, 232.9, 258.2];
        for (got, want) in tbp.iter().zip(want) {
            assert!(
                (got - want).abs() < 0.15,
                "TBP {got:.2} °C vs published {want:.1} °C (curve {tbp:?})"
            );
        }
    }

    #[test]
    fn api_data_book_example_d86_to_tbp_in_fahrenheit() {
        // API Technical Data Book (41) worked example, kept in °F so it can be
        // read straight off the page. D86 350/380/404/433/469 °F ->
        // TBP 316.5/372.6/411.2/451.2/496.7 °F.
        let x = [0.1, 0.3, 0.5, 0.7, 0.9];
        let d86 = from_fahrenheit(
            DistillationBasis::D86,
            &x,
            &[350.0, 380.0, 404.0, 433.0, 469.0],
        );
        let tbp = d86_to_tbp_daubert(&d86).unwrap();
        let got: Vec<f64> = tbp.temperatures.iter().map(|&t| k_to_f(t)).collect();
        let want = [316.5, 372.6, 411.2, 451.2, 496.7];
        for (g, w) in got.iter().zip(want) {
            assert!((g - w).abs() < 0.15, "TBP {g:.2} °F vs published {w:.1} °F");
        }
    }

    #[test]
    fn riazi_example_3_4_simdist_to_tbp() {
        // Riazi 2005 (31), Example 3.4 — API 3A3.1 on a narrow-boiling cut.
        // Expected TBP in °C: 164.3  166.9  168.9  170.9  176.8
        let x = [0.1, 0.3, 0.5, 0.7, 0.9];
        let sd = from_celsius(
            DistillationBasis::D2887,
            &x,
            &[151.7, 162.2, 168.9, 173.3, 181.7],
        );
        let tbp = to_celsius(&d2887_to_tbp(&sd).unwrap());
        let want = [164.3, 166.9, 168.9, 170.9, 176.8];
        for (got, want) in tbp.iter().zip(want) {
            assert!(
                (got - want).abs() < 0.15,
                "TBP {got:.2} °C vs published {want:.1} °C (curve {tbp:?})"
            );
        }
    }

    #[test]
    fn riazi_example_3_5_simdist_to_d86() {
        // Riazi 2005 (31), Example 3.5 — API 3A3.2.
        // Expected D86 in °C: 53.5  68.2  96.9  132.6  167.8
        let x = [0.1, 0.3, 0.5, 0.7, 0.9];
        let sd = from_celsius(
            DistillationBasis::D2887,
            &x,
            &[33.9, 64.4, 101.7, 140.6, 182.2],
        );
        let d86 = to_celsius(&d2887_to_d86(&sd).unwrap());
        let want = [53.5, 68.2, 96.9, 132.6, 167.8];
        for (got, want) in d86.iter().zip(want) {
            assert!(
                (got - want).abs() < 0.15,
                "D86 {got:.2} °C vs published {want:.1} °C (curve {d86:?})"
            );
        }
    }

    #[test]
    fn api_data_book_example_simdist_to_tbp_in_fahrenheit() {
        // API (41) worked example on the full 5-95 % grid.
        let x = [0.05, 0.1, 0.3, 0.5, 0.7, 0.9, 0.95];
        let sd = from_fahrenheit(
            DistillationBasis::D2887,
            &x,
            &[293.0, 305.0, 324.0, 336.0, 344.0, 359.0, 369.0],
        );
        let got: Vec<f64> = d2887_to_tbp(&sd)
            .unwrap()
            .temperatures
            .iter()
            .map(|&t| k_to_f(t))
            .collect();
        let want = [322.2, 327.7, 332.4, 336.0, 339.6, 350.1, 357.4];
        for (g, w) in got.iter().zip(want) {
            assert!((g - w).abs() < 0.15, "TBP {g:.2} °F vs published {w:.1} °F");
        }
    }

    #[test]
    fn riazi_example_3_2_tbp_to_d86_and_on_to_efv() {
        // Riazi 2005 (31), Example 3.2 — a light naphtha, SG 0.7862.
        // TBP 0 % = 10 °C converts back to D86 0 % ≈ 32 °C, and on to
        // EFV 0 % ≈ 68 °C. The EFV initial point being 58 K above the TBP
        // initial point is the physical content of the example: a flash has no
        // fractionation, so its curve is dramatically flatter at both ends.
        let x = [0.0, 0.1, 0.3, 0.5, 0.7, 0.9];
        let tbp = from_celsius(
            DistillationBasis::Tbp,
            &x,
            &[10.0, 71.1, 143.3, 204.4, 250.6, 291.7],
        );
        let d86 = tbp_to_d86_riazi(&tbp).unwrap();
        let d86_c = to_celsius(&d86);
        assert!(
            (d86_c[0] - 32.0).abs() < 0.5,
            "D86 initial point {:.1} °C vs published 32 °C",
            d86_c[0]
        );

        let efv_c = to_celsius(&d86_to_efv(&d86, 0.7862).unwrap());
        assert!(
            (efv_c[0] - 68.0).abs() < 0.5,
            "EFV initial point {:.1} °C vs published 68 °C",
            efv_c[0]
        );
    }

    // === Round trips ====================================================
    //
    // Every conversion here is algebraically invertible. A round trip that
    // does not return the input means an inverse is mis-transcribed, which is
    // the failure mode the published examples above cannot catch (they only
    // exercise the forward direction).

    fn standard_d86() -> DistillationCurve {
        from_celsius(
            DistillationBasis::D86,
            &STANDARD_GRID,
            &[150.0, 170.0, 200.0, 230.0, 262.0, 300.0, 320.0],
        )
    }

    #[test]
    fn riazi_power_law_round_trips() {
        let d86 = standard_d86();
        let back = tbp_to_d86_riazi(&d86_to_tbp_riazi(&d86).unwrap()).unwrap();
        for (a, b) in d86.temperatures.iter().zip(&back.temperatures) {
            assert!((a - b).abs() < 1e-9, "{a} K -> {b} K");
        }
    }

    #[test]
    fn api_difference_method_round_trips() {
        let d86 = standard_d86();
        let back = tbp_to_d86_daubert(&d86_to_tbp_daubert(&d86).unwrap()).unwrap();
        for (a, b) in d86.temperatures.iter().zip(&back.temperatures) {
            assert!((a - b).abs() < 1e-8, "{a} K -> {b} K");
        }
    }

    #[test]
    fn simdist_round_trips_through_tbp() {
        let sd = from_celsius(
            DistillationBasis::D2887,
            &STANDARD_GRID,
            &[140.0, 165.0, 198.0, 228.0, 258.0, 296.0, 318.0],
        );
        let back = tbp_to_d2887(&d2887_to_tbp(&sd).unwrap()).unwrap();
        for (a, b) in sd.temperatures.iter().zip(&back.temperatures) {
            assert!((a - b).abs() < 1e-8, "{a} K -> {b} K");
        }
    }

    #[test]
    fn efv_round_trips() {
        let d86 = standard_d86();
        let back = efv_to_d86(&d86_to_efv(&d86, 0.82).unwrap(), 0.82).unwrap();
        for (a, b) in d86.temperatures.iter().zip(&back.temperatures) {
            assert!((a - b).abs() < 1e-9, "{a} K -> {b} K");
        }
    }

    // === The router =====================================================

    #[test]
    fn router_reaches_every_basis_from_every_basis() {
        let bases = [
            DistillationBasis::D86,
            DistillationBasis::Tbp,
            DistillationBasis::D2887,
            DistillationBasis::Efv,
        ];
        let source = standard_d86();
        for &from in &bases {
            // Put the source curve on `from` by routing there first.
            let start = convert_curve(&source, from, Some(0.82)).unwrap();
            for &to in &bases {
                let out = convert_curve(&start, to, Some(0.82)).unwrap();
                assert_eq!(out.basis, to, "routing {from:?} -> {to:?} kept the basis");
                assert_eq!(out.fractions, start.fractions);
                for &t in &out.temperatures {
                    assert!(t.is_finite() && t > 0.0, "{from:?} -> {to:?} gave {t} K");
                }
            }
        }
    }

    #[test]
    fn router_is_identity_on_the_same_basis() {
        let d86 = standard_d86();
        let same = convert_curve(&d86, DistillationBasis::D86, None).unwrap();
        assert_eq!(d86, same);
    }

    #[test]
    fn router_demands_gravity_only_for_efv_legs() {
        let d86 = standard_d86();
        // TBP needs no gravity.
        assert!(convert_curve(&d86, DistillationBasis::Tbp, None).is_ok());
        // EFV does, and says so rather than silently defaulting.
        let err = convert_curve(&d86, DistillationBasis::Efv, None).unwrap_err();
        assert!(
            matches!(err, PetroleumError::InvalidInput(ref m) if m.contains("specific gravity")),
            "got {err:?}"
        );
    }

    // === Physical sanity =================================================

    #[test]
    fn tbp_is_wider_boiling_than_d86() {
        // The physical content of the D86 -> TBP conversion: a real 15-plate
        // column separates better than a single-stage flask, so the TBP curve
        // starts lower and ends higher than the D86 curve of the same material.
        let d86 = standard_d86();
        let tbp = d86_to_tbp_daubert(&d86).unwrap();
        let n = d86.len() - 1;
        assert!(
            tbp.temperatures[0] < d86.temperatures[0],
            "TBP initial {} K should be below D86 initial {} K",
            tbp.temperatures[0],
            d86.temperatures[0]
        );
        assert!(
            tbp.temperatures[n] > d86.temperatures[n],
            "TBP final {} K should be above D86 final {} K",
            tbp.temperatures[n],
            d86.temperatures[n]
        );
    }

    #[test]
    fn efv_initial_point_row_crosses_its_neighbour_on_narrow_feeds() {
        // A limitation of the published Edmister-Okamoto table, not of this
        // code: its 0-10 % row is fitted independently of the 10-30 % row and
        // the two cross. Below roughly a 250 K boiling span the converted EFV
        // initial point lands above the 10 % point, and the curve constructor
        // rejects the result. Documented in the module header; asserted here so
        // the documented workaround cannot go stale.
        let x = vec![0.0, 0.10, 0.30, 0.50, 0.70, 0.90];
        let narrow = DistillationCurve::new(
            DistillationBasis::D86,
            x.clone(),
            vec![400.0, 410.0, 430.0, 450.0, 470.0, 490.0], // 90 K span
        )
        .unwrap();
        let err = d86_to_efv(&narrow, 0.85).unwrap_err();
        assert!(
            matches!(err, PetroleumError::InvalidInput(ref m) if m.contains("cannot decrease")),
            "expected a monotonicity rejection, got {err:?}"
        );

        // The documented fix: drop the 0 % point.
        let from_ten = DistillationCurve::new(
            DistillationBasis::D86,
            vec![0.10, 0.30, 0.50, 0.70, 0.90],
            vec![410.0, 430.0, 450.0, 470.0, 490.0],
        )
        .unwrap();
        let efv = d86_to_efv(&from_ten, 0.85).unwrap();
        for w in efv.temperatures.windows(2) {
            assert!(w[1] >= w[0], "10-90 % conversion is not monotone: {w:?}");
        }

        // And a genuinely wide feed converts with the 0 % point intact.
        let wide = DistillationCurve::new(
            DistillationBasis::D86,
            x,
            vec![330.0, 370.0, 450.0, 530.0, 610.0, 700.0], // 370 K span
        )
        .unwrap();
        assert!(d86_to_efv(&wide, 0.85).is_ok());
    }

    #[test]
    fn efv_is_flatter_than_d86() {
        // No fractionation at all means the flattest curve of the four.
        let d86 = standard_d86();
        let efv = d86_to_efv(&d86, 0.82).unwrap();
        let n = d86.len() - 1;
        let span_d86 = d86.temperatures[n] - d86.temperatures[0];
        let span_efv = efv.temperatures[n] - efv.temperatures[0];
        assert!(
            span_efv < span_d86,
            "EFV span {span_efv:.1} K should be under the D86 span {span_d86:.1} K"
        );
    }

    #[test]
    fn every_conversion_preserves_monotonicity() {
        // A distillation curve that decreases is not physical, and
        // `DistillationCurve::new` rejects one — so any conversion that
        // produced a decreasing curve would surface as an Err here rather
        // than as a plausible-looking wrong answer downstream.
        let d86 = standard_d86();
        for target in [
            DistillationBasis::Tbp,
            DistillationBasis::D2887,
            DistillationBasis::Efv,
        ] {
            let out = convert_curve(&d86, target, Some(0.82)).unwrap();
            for w in out.temperatures.windows(2) {
                assert!(w[1] >= w[0], "{target:?} produced {w:?}");
            }
        }
    }

    // === Curve mechanics =================================================

    #[test]
    fn interpolation_hits_the_grid_points_exactly() {
        let c = standard_d86();
        for (&x, &t) in c.fractions.iter().zip(&c.temperatures) {
            assert!((c.temperature_at(x) - t).abs() < 1e-9, "at x = {x}");
        }
    }

    #[test]
    fn interpolation_is_monotone_between_grid_points() {
        let c = standard_d86();
        let mut prev = f64::NEG_INFINITY;
        for i in 0..=100 {
            let t = c.temperature_at(i as f64 / 100.0);
            assert!(
                t >= prev,
                "interpolant decreased at x = {}",
                i as f64 / 100.0
            );
            prev = t;
        }
    }

    #[test]
    fn extrapolation_past_the_last_point_keeps_rising() {
        // An assay that stops at 95 % still has to be cut past 95 %.
        let c = standard_d86();
        let last = *c.temperatures.last().unwrap();
        assert!(c.temperature_at(1.0) > last);
        assert!(c.temperature_at(0.0) < *c.temperatures.first().unwrap() + 1e-9);
    }

    #[test]
    fn resampling_onto_the_standard_grid_preserves_shared_points() {
        let sparse = from_celsius(
            DistillationBasis::D86,
            &[0.1, 0.5, 0.9],
            &[170.0, 230.0, 300.0],
        );
        let dense = sparse.resample(&STANDARD_GRID).unwrap();
        assert_eq!(dense.len(), STANDARD_GRID.len());
        // The three points that were already there must come back unchanged.
        for (x, t) in [(0.1, 170.0), (0.5, 230.0), (0.9, 300.0)] {
            let i = dense.fractions.iter().position(|&f| f == x).unwrap();
            assert!((dense.temperatures[i] - (t + 273.15)).abs() < 1e-9);
        }
    }

    #[test]
    fn difference_methods_explain_themselves_without_a_fifty_percent_point() {
        let odd = from_celsius(
            DistillationBasis::D86,
            &[0.1, 0.3, 0.7, 0.9],
            &[170.0, 200.0, 262.0, 300.0],
        );
        let err = d86_to_tbp_daubert(&odd).unwrap_err();
        assert!(
            matches!(err, PetroleumError::CutPoints(ref m) if m.contains("resample")),
            "the error should point at the fix, got {err:?}"
        );
    }

    // === Validation ======================================================

    #[test]
    fn curve_construction_rejects_malformed_input() {
        use DistillationBasis::D86;
        // mismatched lengths
        assert!(DistillationCurve::new(D86, vec![0.1, 0.5], vec![400.0]).is_err());
        // too few points
        assert!(DistillationCurve::new(D86, vec![0.5], vec![400.0]).is_err());
        // fraction out of range
        assert!(DistillationCurve::new(D86, vec![0.1, 1.5], vec![400.0, 450.0]).is_err());
        // fractions not increasing
        assert!(DistillationCurve::new(D86, vec![0.5, 0.1], vec![400.0, 450.0]).is_err());
        // duplicated fraction
        assert!(DistillationCurve::new(D86, vec![0.5, 0.5], vec![400.0, 450.0]).is_err());
        // decreasing temperature
        assert!(DistillationCurve::new(D86, vec![0.1, 0.5], vec![450.0, 400.0]).is_err());
        // non-physical temperature
        assert!(DistillationCurve::new(D86, vec![0.1, 0.5], vec![-10.0, 400.0]).is_err());
    }

    #[test]
    fn conversions_reject_a_curve_on_the_wrong_basis() {
        let tbp = from_celsius(DistillationBasis::Tbp, &[0.1, 0.5], &[170.0, 230.0]);
        let err = d86_to_tbp_riazi(&tbp).unwrap_err();
        assert!(
            matches!(err, PetroleumError::InvalidInput(ref m) if m.contains("TBP")),
            "the error should name the basis it got, got {err:?}"
        );
    }

    #[test]
    fn interval_lookup_covers_the_whole_unit_range() {
        // Every fraction in [0, 1] must select a row; a gap would silently
        // return row 0 and use the wrong coefficients.
        for i in 0..=1000 {
            let x = i as f64 / 1000.0;
            assert!(row_for(&RIAZI_BOUNDS, x) < RIAZI_BOUNDS.len());
            assert!(row_for(&DELTA_BOUNDS, x) < DELTA_BOUNDS.len());
            assert!(row_for(&EFV_BOUNDS, x) < EFV_BOUNDS.len());
        }
        // Spot-check the boundaries: an interval is closed on the left.
        assert_eq!(row_for(&RIAZI_BOUNDS, 0.0), 0);
        assert_eq!(row_for(&RIAZI_BOUNDS, 0.099), 0);
        assert_eq!(row_for(&RIAZI_BOUNDS, 0.10), 1);
        assert_eq!(row_for(&RIAZI_BOUNDS, 0.95), 6);
        assert_eq!(row_for(&RIAZI_BOUNDS, 1.0), 6);
    }
}
