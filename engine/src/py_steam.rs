//! PyO3 bindings for the IAPWS-IF97 steam tables (Milestone 13).
//!
//! Surfaces the `vle-steam` crate to Python as `vle.steam`:
//!
//! - [`SteamState`] — a read-only pyclass mirroring `vle_steam::SteamState`
//!   (`T, P, region, phase, x, ρ, v, u, h, s, cp, cv, w`), built by the
//!   `steam_tp` / `steam_tx` / `steam_px` / `steam_ph` / `steam_ps` free
//!   functions.
//! - [`SatState`] — the classic saturation-table row (`SatProps`), from
//!   `steam_sat_t` / `steam_sat_p`.
//! - Scalar helpers `steam_psat` / `steam_tsat` / `steam_psat_derivative` /
//!   `steam_latent_heat`.
//! - **Batch numpy kernels** `steam_tp_batch` / `steam_ph_batch` /
//!   `steam_sat_t_batch` (rust-numpy + rayon, GIL released) — steam property
//!   evaluation is exactly the "numpy for thermo" use case, mirroring the M10
//!   `System._batch` design.
//!
//! The high-level `vle.steam` Python wrapper adds the keyword-argument
//! `Water(T=..., P=...)` dispatch and pint/gauge-unit parsing on top of these.
//!
//! Units at this boundary: `T` in **K**, `P` in **kPa absolute**, mass-basis
//! properties (kJ/kg, kJ/(kg·K), m³/kg, m/s) — the `vle-steam` public canon.

use numpy::{IntoPyArray, PyReadonlyArray1};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rayon::prelude::*;

use vle_steam::{Phase, Region, SatProps, SteamError, SteamState as CoreState};

/// Map a steam-crate error to a Python `ValueError`.
fn steam_err(e: SteamError) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// IF97 region as a short string (`"1".."5"`, `"4"` = saturation line).
fn region_str(r: Region) -> &'static str {
    match r {
        Region::One => "1",
        Region::Two => "2",
        Region::Three => "3",
        Region::Saturated => "4",
        Region::Five => "5",
    }
}

/// Phase label as a lowercase string.
fn phase_str(p: Phase) -> &'static str {
    match p {
        Phase::Liquid => "liquid",
        Phase::Vapor => "vapor",
        Phase::TwoPhase => "two-phase",
        Phase::Supercritical => "supercritical",
    }
}

/// A resolved water/steam state (read-only), Python-visible as
/// `vle._engine.SteamState`.
#[pyclass(name = "SteamState", module = "vle._engine", frozen)]
pub struct SteamState {
    inner: CoreState,
}

#[pymethods]
impl SteamState {
    /// Temperature, **K**.
    #[getter]
    fn t(&self) -> f64 {
        self.inner.t
    }
    /// Pressure, **kPa absolute**.
    #[getter]
    fn p(&self) -> f64 {
        self.inner.p
    }
    /// IF97 region as a string (`"1".."5"`).
    #[getter]
    fn region(&self) -> &'static str {
        region_str(self.inner.region)
    }
    /// Phase label (`"liquid"`, `"vapor"`, `"two-phase"`, `"supercritical"`).
    #[getter]
    fn phase(&self) -> &'static str {
        phase_str(self.inner.phase)
    }
    /// Vapor quality (mass fraction) if two-phase, else `None`.
    #[getter]
    fn x(&self) -> Option<f64> {
        self.inner.x
    }
    /// Specific volume, **m³/kg**.
    #[getter]
    fn v(&self) -> f64 {
        self.inner.v
    }
    /// Density, **kg/m³**.
    #[getter]
    fn rho(&self) -> f64 {
        self.inner.rho
    }
    /// Specific internal energy, **kJ/kg**.
    #[getter]
    fn u(&self) -> f64 {
        self.inner.u
    }
    /// Specific enthalpy, **kJ/kg**.
    #[getter]
    fn h(&self) -> f64 {
        self.inner.h
    }
    /// Specific entropy, **kJ/(kg·K)**.
    #[getter]
    fn s(&self) -> f64 {
        self.inner.s
    }
    /// Isobaric heat capacity, **kJ/(kg·K)** (`NaN` if two-phase).
    #[getter]
    fn cp(&self) -> f64 {
        self.inner.cp
    }
    /// Isochoric heat capacity, **kJ/(kg·K)** (`NaN` if two-phase).
    #[getter]
    fn cv(&self) -> f64 {
        self.inner.cv
    }
    /// Speed of sound, **m/s** (`NaN` if two-phase).
    #[getter]
    fn w(&self) -> f64 {
        self.inner.w
    }
    /// Dynamic viscosity, **Pa·s** (IAPWS R12-08). `NaN` if two-phase —
    /// transport properties are per-phase; use the `SatState` row's
    /// `mu_f`/`mu_g`.
    #[getter]
    fn mu(&self) -> f64 {
        self.inner.viscosity().unwrap_or(f64::NAN)
    }
    /// Thermal conductivity, **W/(m·K)** (IAPWS R15-11). `NaN` if two-phase.
    #[getter]
    fn k(&self) -> f64 {
        self.inner.thermal_conductivity().unwrap_or(f64::NAN)
    }
    /// Prandtl number `cp·μ/λ`, **dimensionless**. `NaN` if two-phase.
    #[getter]
    fn pr(&self) -> f64 {
        self.inner.prandtl().unwrap_or(f64::NAN)
    }
    /// Kinematic viscosity `ν = μ/ρ`, **m²/s**. `NaN` if two-phase.
    #[getter]
    fn nu(&self) -> f64 {
        self.inner.kinematic_viscosity().unwrap_or(f64::NAN)
    }
    /// Thermal diffusivity `α = λ/(ρ·cp)`, **m²/s**. `NaN` if two-phase.
    #[getter]
    fn alpha(&self) -> f64 {
        self.inner.thermal_diffusivity().unwrap_or(f64::NAN)
    }

    fn __repr__(&self) -> String {
        format!(
            "SteamState(T={:.4} K, P={:.4} kPa, region={}, phase={}, h={:.6} kJ/kg)",
            self.inner.t,
            self.inner.p,
            region_str(self.inner.region),
            phase_str(self.inner.phase),
            self.inner.h,
        )
    }
}

impl From<CoreState> for SteamState {
    fn from(inner: CoreState) -> Self {
        SteamState { inner }
    }
}

/// A saturation-table row, Python-visible as `vle._engine.SatState`.
#[pyclass(name = "SatState", module = "vle._engine", frozen)]
pub struct SatState {
    inner: SatProps,
}

#[pymethods]
impl SatState {
    /// Saturation temperature, **K**.
    #[getter]
    fn t(&self) -> f64 {
        self.inner.t
    }
    /// Saturation pressure, **kPa absolute**.
    #[getter]
    fn p(&self) -> f64 {
        self.inner.p
    }
    /// Saturated-liquid specific volume, **m³/kg**.
    #[getter]
    fn v_f(&self) -> f64 {
        self.inner.v_f
    }
    /// Saturated-vapor specific volume, **m³/kg**.
    #[getter]
    fn v_g(&self) -> f64 {
        self.inner.v_g
    }
    /// Saturated-liquid enthalpy, **kJ/kg**.
    #[getter]
    fn h_f(&self) -> f64 {
        self.inner.h_f
    }
    /// Saturated-vapor enthalpy, **kJ/kg**.
    #[getter]
    fn h_g(&self) -> f64 {
        self.inner.h_g
    }
    /// Latent heat of vaporization, **kJ/kg**.
    #[getter]
    fn h_fg(&self) -> f64 {
        self.inner.h_fg
    }
    /// Saturated-liquid entropy, **kJ/(kg·K)**.
    #[getter]
    fn s_f(&self) -> f64 {
        self.inner.s_f
    }
    /// Saturated-vapor entropy, **kJ/(kg·K)**.
    #[getter]
    fn s_g(&self) -> f64 {
        self.inner.s_g
    }
    /// Entropy of vaporization, **kJ/(kg·K)**.
    #[getter]
    fn s_fg(&self) -> f64 {
        self.inner.s_fg
    }
    /// Saturated-liquid internal energy, **kJ/kg**.
    #[getter]
    fn u_f(&self) -> f64 {
        self.inner.u_f
    }
    /// Saturated-vapor internal energy, **kJ/kg**.
    #[getter]
    fn u_g(&self) -> f64 {
        self.inner.u_g
    }
    /// Saturated-liquid dynamic viscosity, **Pa·s** (IAPWS R12-08).
    #[getter]
    fn mu_f(&self) -> f64 {
        self.inner.mu_f()
    }
    /// Saturated-vapor dynamic viscosity, **Pa·s** (IAPWS R12-08).
    #[getter]
    fn mu_g(&self) -> f64 {
        self.inner.mu_g()
    }
    /// Saturated-liquid thermal conductivity, **W/(m·K)** (IAPWS R15-11).
    #[getter]
    fn k_f(&self) -> f64 {
        self.inner.k_f().unwrap_or(f64::NAN)
    }
    /// Saturated-vapor thermal conductivity, **W/(m·K)** (IAPWS R15-11).
    #[getter]
    fn k_g(&self) -> f64 {
        self.inner.k_g().unwrap_or(f64::NAN)
    }
    /// Liquid–vapor surface tension, **N/m** (IAPWS R1-76(2014)).
    #[getter]
    fn sigma(&self) -> f64 {
        self.inner.sigma().unwrap_or(f64::NAN)
    }

    fn __repr__(&self) -> String {
        format!(
            "SatState(T={:.4} K, P={:.4} kPa, h_fg={:.4} kJ/kg)",
            self.inner.t, self.inner.p, self.inner.h_fg,
        )
    }
}

// ── Scalar constructors ──────────────────────────────────────────────────

/// State from `(T, P)`. `t` in K, `p` in kPa absolute.
#[pyfunction]
pub fn steam_tp(t: f64, p: f64) -> PyResult<SteamState> {
    CoreState::tp(t, p).map(Into::into).map_err(steam_err)
}

/// State from `(T, x)` (quality). `t` in K, `x` in [0, 1].
#[pyfunction]
pub fn steam_tx(t: f64, x: f64) -> PyResult<SteamState> {
    CoreState::tx(t, x).map(Into::into).map_err(steam_err)
}

/// State from `(P, x)` (quality). `p` in kPa absolute, `x` in [0, 1].
#[pyfunction]
pub fn steam_px(p: f64, x: f64) -> PyResult<SteamState> {
    CoreState::px(p, x).map(Into::into).map_err(steam_err)
}

/// State from `(P, h)` (PH flash). `p` in kPa, `h` in kJ/kg.
#[pyfunction]
pub fn steam_ph(p: f64, h: f64) -> PyResult<SteamState> {
    CoreState::ph(p, h).map(Into::into).map_err(steam_err)
}

/// State from `(P, s)` (PS flash). `p` in kPa, `s` in kJ/(kg·K).
#[pyfunction]
pub fn steam_ps(p: f64, s: f64) -> PyResult<SteamState> {
    CoreState::ps(p, s).map(Into::into).map_err(steam_err)
}

/// Saturation-table row at temperature `t` (**K**).
#[pyfunction]
pub fn steam_sat_t(t: f64) -> PyResult<SatState> {
    vle_steam::sat_t(t)
        .map(|inner| SatState { inner })
        .map_err(steam_err)
}

/// Saturation-table row at pressure `p` (**kPa absolute**).
#[pyfunction]
pub fn steam_sat_p(p: f64) -> PyResult<SatState> {
    vle_steam::sat_p(p)
        .map(|inner| SatState { inner })
        .map_err(steam_err)
}

/// Saturation pressure `Psat(T)` in **kPa**. `t` in K.
#[pyfunction]
pub fn steam_psat(t: f64) -> PyResult<f64> {
    vle_steam::psat(t).map_err(steam_err)
}

/// Saturation temperature `Tsat(P)` in **K**. `p` in kPa.
#[pyfunction]
pub fn steam_tsat(p: f64) -> PyResult<f64> {
    vle_steam::tsat(p).map_err(steam_err)
}

/// Analytic `dPsat/dT` in **kPa/K**. `t` in K.
#[pyfunction]
pub fn steam_psat_derivative(t: f64) -> PyResult<f64> {
    vle_steam::psat_derivative(t).map_err(steam_err)
}

/// Latent heat of vaporization `h_fg(T)` in **kJ/kg**. `t` in K.
#[pyfunction]
pub fn steam_latent_heat(t: f64) -> PyResult<f64> {
    vle_steam::latent_heat(t).map_err(steam_err)
}

// ── Batch numpy kernels (GIL released, rayon) ────────────────────────────

/// Dynamic viscosity at `(T, P)`. `t` in K, `p` in kPa absolute; **Pa·s**.
#[pyfunction]
pub fn steam_viscosity(t: f64, p: f64) -> PyResult<f64> {
    vle_steam::viscosity(t, p).map_err(steam_err)
}

/// Thermal conductivity at `(T, P)`. `t` in K, `p` in kPa absolute;
/// **W/(m·K)**.
#[pyfunction]
pub fn steam_thermal_conductivity(t: f64, p: f64) -> PyResult<f64> {
    vle_steam::thermal_conductivity(t, p).map_err(steam_err)
}

/// Liquid–vapor surface tension at `t` (K); **N/m**.
#[pyfunction]
pub fn steam_surface_tension(t: f64) -> PyResult<f64> {
    vle_steam::surface_tension(t).map_err(steam_err)
}

/// One batch point's single-phase outputs, moved out of the parallel region.
struct StatePoint {
    v: f64,
    rho: f64,
    u: f64,
    h: f64,
    s: f64,
    cp: f64,
    cv: f64,
    w: f64,
    x: f64,
}

impl StatePoint {
    fn nan() -> Self {
        StatePoint {
            v: f64::NAN,
            rho: f64::NAN,
            u: f64::NAN,
            h: f64::NAN,
            s: f64::NAN,
            cp: f64::NAN,
            cv: f64::NAN,
            w: f64::NAN,
            x: f64::NAN,
        }
    }
    fn from_state(s: &CoreState) -> Self {
        StatePoint {
            v: s.v,
            rho: s.rho,
            u: s.u,
            h: s.h,
            s: s.s,
            cp: s.cp,
            cv: s.cv,
            w: s.w,
            x: s.x.unwrap_or(f64::NAN),
        }
    }
}

/// Broadcast a length-1 array over `n`, else index directly.
fn bcast(a: &[f64], i: usize) -> f64 {
    if a.len() == 1 { a[0] } else { a[i] }
}

/// Common length for two broadcastable inputs.
fn batch_len(a: usize, b: usize) -> PyResult<usize> {
    let n = a.max(b);
    if (a == n || a == 1) && (b == n || b == 1) {
        Ok(n)
    } else {
        Err(PyValueError::new_err(format!(
            "batch length mismatch: {a} vs {b} (each must equal the max or be length-1)"
        )))
    }
}

/// Assemble the property-array dict shared by the state batch kernels.
fn state_batch_dict<'py>(
    py: Python<'py>,
    ts: Vec<f64>,
    ps: Vec<f64>,
    pts: Vec<StatePoint>,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new_bound(py);
    d.set_item("t", ts.into_pyarray_bound(py))?;
    d.set_item("p", ps.into_pyarray_bound(py))?;
    macro_rules! col {
        ($key:expr, $field:ident) => {{
            let v: Vec<f64> = pts.iter().map(|p| p.$field).collect();
            d.set_item($key, v.into_pyarray_bound(py))?;
        }};
    }
    col!("v", v);
    col!("rho", rho);
    col!("u", u);
    col!("h", h);
    col!("s", s);
    col!("cp", cp);
    col!("cv", cv);
    col!("w", w);
    col!("x", x);
    Ok(d)
}

/// Batch `(T, P) → properties`. Arrays broadcast against a length-1 partner.
/// Returns a dict of numpy arrays keyed `t, p, v, rho, u, h, s, cp, cv, w, x`.
#[pyfunction]
pub fn steam_tp_batch<'py>(
    py: Python<'py>,
    ts: PyReadonlyArray1<'py, f64>,
    ps: PyReadonlyArray1<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    let ta = ts.as_slice()?;
    let pa = ps.as_slice()?;
    let n = batch_len(ta.len(), pa.len())?;
    let (t_out, p_out): (Vec<f64>, Vec<f64>) = (0..n).map(|i| (bcast(ta, i), bcast(pa, i))).unzip();

    let pts = py.allow_threads(|| {
        (0..n)
            .into_par_iter()
            .map(|i| match CoreState::tp(bcast(ta, i), bcast(pa, i)) {
                Ok(s) => StatePoint::from_state(&s),
                Err(_) => StatePoint::nan(),
            })
            .collect::<Vec<_>>()
    });
    state_batch_dict(py, t_out, p_out, pts)
}

/// Batch `(T, P) → transport properties`. Arrays broadcast against a length-1
/// partner. Returns a dict of numpy arrays keyed `t, p, mu, k, pr, nu, alpha`.
///
/// Deliberately a **separate** kernel rather than extra columns on
/// `steam_tp_batch`: the R15-11 critical enhancement is several times the cost
/// of the thermodynamic surface it sits on, so folding it in would tax every
/// caller of the existing batch API to serve the ones who want it. Two-phase
/// points come back `NaN` — transport properties are per-phase.
#[pyfunction]
pub fn steam_transport_batch<'py>(
    py: Python<'py>,
    ts: PyReadonlyArray1<'py, f64>,
    ps: PyReadonlyArray1<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    let ta = ts.as_slice()?;
    let pa = ps.as_slice()?;
    let n = batch_len(ta.len(), pa.len())?;
    let (t_out, p_out): (Vec<f64>, Vec<f64>) = (0..n).map(|i| (bcast(ta, i), bcast(pa, i))).unzip();

    let pts = py.allow_threads(|| {
        (0..n)
            .into_par_iter()
            .map(|i| match CoreState::tp(bcast(ta, i), bcast(pa, i)) {
                Ok(s) => (
                    s.viscosity().unwrap_or(f64::NAN),
                    s.thermal_conductivity().unwrap_or(f64::NAN),
                    s.prandtl().unwrap_or(f64::NAN),
                    s.kinematic_viscosity().unwrap_or(f64::NAN),
                    s.thermal_diffusivity().unwrap_or(f64::NAN),
                ),
                Err(_) => (f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN),
            })
            .collect::<Vec<_>>()
    });

    let d = PyDict::new_bound(py);
    d.set_item("t", t_out.into_pyarray_bound(py))?;
    d.set_item("p", p_out.into_pyarray_bound(py))?;
    for (key, idx) in [("mu", 0), ("k", 1), ("pr", 2), ("nu", 3), ("alpha", 4)] {
        let v: Vec<f64> = pts
            .iter()
            .map(|t| match idx {
                0 => t.0,
                1 => t.1,
                2 => t.2,
                3 => t.3,
                _ => t.4,
            })
            .collect();
        d.set_item(key, v.into_pyarray_bound(py))?;
    }
    Ok(d)
}

/// Batch `(P, h) → properties` (PH flash). Returns the same dict shape as
/// [`steam_tp_batch`], with `t` the resolved temperature per point.
#[pyfunction]
pub fn steam_ph_batch<'py>(
    py: Python<'py>,
    ps: PyReadonlyArray1<'py, f64>,
    hs: PyReadonlyArray1<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    let pa = ps.as_slice()?;
    let ha = hs.as_slice()?;
    let n = batch_len(pa.len(), ha.len())?;

    let states = py.allow_threads(|| {
        (0..n)
            .into_par_iter()
            .map(|i| CoreState::ph(bcast(pa, i), bcast(ha, i)).ok())
            .collect::<Vec<_>>()
    });
    let p_out: Vec<f64> = (0..n).map(|i| bcast(pa, i)).collect();
    let t_out: Vec<f64> = states
        .iter()
        .map(|s| s.map(|s| s.t).unwrap_or(f64::NAN))
        .collect();
    let pts: Vec<StatePoint> = states
        .iter()
        .map(|s| {
            s.as_ref()
                .map(StatePoint::from_state)
                .unwrap_or_else(StatePoint::nan)
        })
        .collect();
    state_batch_dict(py, t_out, p_out, pts)
}

/// Batch saturation rows `sat_t(T)`. Returns a dict of numpy arrays keyed
/// `t, p, v_f, v_g, h_f, h_g, h_fg, s_f, s_g, s_fg, u_f, u_g`.
#[pyfunction]
pub fn steam_sat_t_batch<'py>(
    py: Python<'py>,
    ts: PyReadonlyArray1<'py, f64>,
) -> PyResult<Bound<'py, PyDict>> {
    let ta = ts.as_slice()?;
    let n = ta.len();
    let rows = py.allow_threads(|| {
        (0..n)
            .into_par_iter()
            .map(|i| vle_steam::sat_t(ta[i]).ok())
            .collect::<Vec<_>>()
    });
    let d = PyDict::new_bound(py);
    macro_rules! col {
        ($key:expr, $f:expr) => {{
            let v: Vec<f64> = rows
                .iter()
                .map(|r| r.as_ref().map($f).unwrap_or(f64::NAN))
                .collect();
            d.set_item($key, v.into_pyarray_bound(py))?;
        }};
    }
    col!("t", |r: &SatProps| r.t);
    col!("p", |r: &SatProps| r.p);
    col!("v_f", |r: &SatProps| r.v_f);
    col!("v_g", |r: &SatProps| r.v_g);
    col!("h_f", |r: &SatProps| r.h_f);
    col!("h_g", |r: &SatProps| r.h_g);
    col!("h_fg", |r: &SatProps| r.h_fg);
    col!("s_f", |r: &SatProps| r.s_f);
    col!("s_g", |r: &SatProps| r.s_g);
    col!("s_fg", |r: &SatProps| r.s_fg);
    col!("u_f", |r: &SatProps| r.u_f);
    col!("u_g", |r: &SatProps| r.u_g);
    Ok(d)
}
