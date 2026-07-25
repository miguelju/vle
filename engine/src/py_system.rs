//! The persistent `System` pyclass — Milestone 10 (Phases 16–17).
//!
//! Every M5–M9 binding in [`crate::py_bindings`] is a *free function* that
//! rebuilds its `Component` vector on each call. That is fine for a demo,
//! but the per-call rebuild (allocate strings/vecs, re-validate shapes)
//! dominates once Python starts calling in a loop. This module is the
//! Track-D answer (PERFORMANCE_PROPOSAL.md):
//!
//! - [`System`] is a `#[pyclass]` that owns the component list and every
//!   model selection **once**, at construction. Each method borrows that
//!   state into a [`SystemSpec`] (a cheap `Copy` of slices) — no per-call
//!   `Component` reconstruction ever happens again.
//! - The `*_batch` methods take **numpy arrays in and return numpy arrays
//!   out** (rust-numpy). Input arrays are read as zero-copy slices; there
//!   is exactly one FFI crossing per *array*, not per state point.
//! - Batch kernels run inside [`Python::allow_threads`] (the GIL is
//!   released) and fan out across cores with **rayon**. Isothermal-flash
//!   batches are **warm-started**: within each contiguous chunk, point
//!   `i+1` is seeded with point `i`'s converged K-values (§J, §M), which
//!   typically halves the iteration count on smooth property sweeps.
//!
//! ## Educational notes (Rust idioms used here)
//!
//! - `#[pyclass]` turns a plain Rust struct into a Python class; the
//!   `#[pymethods]` block below it defines what Python sees as methods.
//! - `Option<&[f64]>` ("maybe a slice") models Python's `None`-able
//!   arguments at zero cost — no allocation when the argument is absent.
//! - The `PointOut` struct is a plain "struct of results" used to move a
//!   batch point's numbers out of the parallel region; collecting a
//!   `Vec<PointOut>` and *then* building numpy arrays keeps all Python
//!   API calls outside the GIL-released section (a PyO3 requirement).
//!
//! ## Units
//!
//! Canonical engine units throughout: T in **K**, P in **kPa absolute**,
//! molar enthalpy in **kJ/kmol**, molar entropy in **kJ/(kmol·K)**,
//! molar volume in **cm³/mol** (except the critical point's Vc, reported
//! in **m³/kmol** to match the thesis tables).

use numpy::{
    IntoPyArray, PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2, PyUntypedArrayMethods,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use rayon::prelude::*;

use crate::activity::ActivityModel;
use crate::eos::{CubicEos, LiquidModel, PhaseId, VaporModel};
use crate::flash::adiabatic::flash_adiabatic;
use crate::flash::bubble::{SaturationResult, bubble_pressure, bubble_temperature};
use crate::flash::critical::critical_point;
use crate::flash::dew::{dew_pressure, dew_temperature};
use crate::flash::envelope::trace_envelope;
use crate::flash::isothermal::flash_isothermal_warm;
use crate::flash::stability::{Stability, stability_analysis};
use crate::flash::{FlashError, SystemSpec, k_values};
use crate::mixing::MixingRule;
use crate::mixture::{MixtureSpec, ln_phi_mix, z_mix};
use crate::types::Component;

/// Map a [`FlashError`] to a Python exception (same policy as
/// `py_bindings::map_flash_err`, duplicated here to keep the modules
/// independently readable).
fn flash_err(e: FlashError) -> PyErr {
    match e {
        // Bad-argument errors surface as Python `ValueError`; genuine runtime
        // failures (non-convergence, thermodynamic breakdown) as `RuntimeError`.
        FlashError::Dimension(_)
        | FlashError::InvalidInput(_)
        | FlashError::Unsupported(_)
        | FlashError::NoRachfordRiceRoot { .. } => PyValueError::new_err(e.to_string()),
        _ => PyRuntimeError::new_err(e.to_string()),
    }
}

fn mix_err(e: crate::mixture::MixError) -> PyErr {
    match e {
        crate::mixture::MixError::Dimension(_) | crate::mixture::MixError::Unsupported(_) => {
            PyValueError::new_err(e.to_string())
        }
        _ => PyRuntimeError::new_err(e.to_string()),
    }
}

fn parse_phase(s: &str) -> PyResult<PhaseId> {
    match s.to_ascii_lowercase().as_str() {
        "vapor" | "v" | "gas" => Ok(PhaseId::Vapor),
        "liquid" | "l" => Ok(PhaseId::Liquid),
        other => Err(PyValueError::new_err(format!(
            "phase must be 'vapor' or 'liquid' (got {other:?})"
        ))),
    }
}

/// Persistent VLE system handle: components + model selections, built once.
///
/// Python-visible as `vle._engine.System`. The high-level `vle.System`
/// Python class wraps this with name-based component lookup, result
/// dataclasses, and plotting helpers; this pyclass is the FFI workhorse.
#[pyclass(name = "System", module = "vle._engine")]
pub struct System {
    components: Vec<Component>,
    vapor: VaporModel,
    liquid: LiquidModel,
    mixing_rule: MixingRule,
    kij: Vec<Vec<f64>>,
    aij: Vec<Vec<f64>>,
    /// NRTL non-randomness matrix αᵢⱼ (N×N, symmetric); empty ⇒ ignored.
    alpha: Vec<Vec<f64>>,
    vl: Vec<f64>,
    delta: Vec<f64>,
    ge_model: Option<ActivityModel>,
    /// Ideal-gas reference temperature in **K** (enthalpy/entropy zero).
    t_ref: f64,
    /// Ideal-gas reference pressure in **kPa absolute**.
    p_ref: f64,
}

/// One batch point's outputs, moved out of the GIL-released region.
struct PointOut {
    value: f64,
    beta: f64,
    comp_a: Vec<f64>,
    comp_b: Vec<f64>,
    k: Vec<f64>,
    iterations: u64,
    two_phase: bool,
    converged: bool,
}

impl PointOut {
    /// A NaN-filled "this point failed" marker of width `n`.
    fn failed(n: usize) -> Self {
        Self {
            value: f64::NAN,
            beta: f64::NAN,
            comp_a: vec![f64::NAN; n],
            comp_b: vec![f64::NAN; n],
            k: vec![f64::NAN; n],
            iterations: 0,
            two_phase: false,
            converged: false,
        }
    }
}

impl System {
    /// Borrow the owned state into the slice-based [`SystemSpec`] the flash
    /// drivers consume. Cheap (`Copy` of pointers) — called per method.
    fn spec(&self) -> SystemSpec<'_> {
        SystemSpec {
            components: &self.components,
            vapor: self.vapor,
            liquid: self.liquid,
            mixing_rule: self.mixing_rule,
            kij: &self.kij,
            aij: &self.aij,
            alpha: &self.alpha,
            vl: &self.vl,
            delta: &self.delta,
            sat_models: &[],
            ge_model: self.ge_model,
        }
    }

    /// The cubic EOS modeling `phase`, or a `ValueError` if that phase is
    /// not EOS-modeled (property methods need a PVT model to evaluate).
    fn phase_eos(&self, phase: PhaseId) -> PyResult<CubicEos> {
        let model_eos = match phase {
            PhaseId::Vapor => match self.vapor {
                VaporModel::Cubic(eos) => Some(eos),
                _ => None,
            },
            PhaseId::Liquid => match self.liquid {
                LiquidModel::Cubic(eos) => Some(eos),
                _ => None,
            },
        };
        model_eos.ok_or_else(|| {
            PyValueError::new_err(
                "this property needs a cubic EOS on the requested phase \
                 (construct the System with vapor_kind/liquid_kind='cubic')",
            )
        })
    }

    fn mixture_spec(&self, eos: CubicEos) -> MixtureSpec<'_> {
        // `SystemSpec::mixture_spec` is pub(crate), so we can reuse the
        // GE-spec assembly logic it encapsulates.
        self.spec().mixture_spec(eos)
    }

    fn check_width(&self, what: &str, len: usize) -> PyResult<()> {
        if len != self.components.len() {
            return Err(PyValueError::new_err(format!(
                "{what} has {len} entries but the system has {} components",
                self.components.len()
            )));
        }
        Ok(())
    }
}

/// Broadcast helper: `m`-point batches accept per-point arrays of length
/// `m` or a length-1 array reused for every point (numpy-style broadcast).
fn bcast(arr: &[f64], i: usize) -> f64 {
    if arr.len() == 1 { arr[0] } else { arr[i] }
}

/// Validate two broadcastable input arrays and return the batch size `m`.
fn batch_len(a: usize, b: usize) -> PyResult<usize> {
    match (a, b) {
        (x, y) if x == y => Ok(x),
        (1, y) => Ok(y),
        (x, 1) => Ok(x),
        (x, y) => Err(PyValueError::new_err(format!(
            "array lengths {x} and {y} are not broadcastable (equal or 1)"
        ))),
    }
}

/// Run `f` over `0..m` with rayon, preserving order, sequential within
/// chunks so `f` may warm-start from its chunk-local predecessor via the
/// mutable carry each chunk owns.
///
/// `Carry` is the warm-start state (e.g. the previous converged K vector);
/// each chunk starts with `Carry::default()` (cold) and threads it through
/// its own points in order. With `parallel=false` there is exactly one
/// chunk, i.e. one fully warm-started sequential chain.
fn chunked_run<Carry: Default + Send>(
    m: usize,
    parallel: bool,
    f: impl (Fn(usize, &mut Carry) -> PointOut) + Sync,
) -> Vec<PointOut> {
    let n_chunks = if parallel {
        rayon::current_num_threads().min(m).max(1)
    } else {
        1
    };
    let chunk_size = m.div_ceil(n_chunks);
    // Each chunk is an independent warm-start chain; rayon's collect
    // preserves chunk order, so flattening restores point order.
    (0..n_chunks)
        .into_par_iter()
        .map(|c| {
            let lo = c * chunk_size;
            let hi = ((c + 1) * chunk_size).min(m);
            let mut carry = Carry::default();
            (lo..hi).map(|i| f(i, &mut carry)).collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>()
        .into_iter()
        .flatten()
        .collect()
}

/// Assemble the `(values, incipient, k, converged)` numpy tuple shared by
/// the four bubble/dew batch methods.
type SatBatch<'py> = (
    Bound<'py, PyArray1<f64>>,
    Bound<'py, PyArray2<f64>>,
    Bound<'py, PyArray2<f64>>,
    Bound<'py, PyArray1<bool>>,
);

fn sat_batch_out<'py>(py: Python<'py>, n: usize, pts: Vec<PointOut>) -> PyResult<SatBatch<'py>> {
    let values: Vec<f64> = pts.iter().map(|p| p.value).collect();
    let converged: Vec<bool> = pts.iter().map(|p| p.converged).collect();
    let incipient: Vec<Vec<f64>> = pts.iter().map(|p| p.comp_a.clone()).collect();
    let k: Vec<Vec<f64>> = pts.iter().map(|p| p.k.clone()).collect();
    let _ = n;
    Ok((
        values.into_pyarray_bound(py),
        PyArray2::from_vec2_bound(py, &incipient)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        PyArray2::from_vec2_bound(py, &k).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        converged.into_pyarray_bound(py),
    ))
}

#[pymethods]
impl System {
    /// Build a persistent System.
    ///
    /// Parallel per-component arrays (`tcs` K, `pcs` kPa abs, `omegas`) plus
    /// the model selectors used by every M9 flash binding. Optional
    /// per-component data: `psat_coeffs` (reduced Antoine `[a1,a2,a3]`),
    /// `vl` (liquid molar volume, cm³/mol), `cp_coeffs` (N×5 ideal-Cp/R
    /// polynomial rows), `tbs` (normal boiling T, K), `names`.
    /// `t_ref`/`p_ref` set the ideal-gas enthalpy/entropy reference state
    /// (K, kPa abs).
    #[new]
    #[pyo3(signature = (tcs, pcs, omegas, vapor_kind="cubic", liquid_kind="cubic",
        vapor_eos=None, liquid_eos=None, liquid_activity=None,
        mixing_rule=MixingRule::Classical, kij=vec![], aij=vec![], alpha=vec![],
        vl=vec![], delta=vec![], psat_coeffs=vec![], cp_coeffs=vec![], tbs=vec![],
        names=vec![], ge_model=None, t_ref=298.15, p_ref=101.325))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        tcs: Vec<f64>,
        pcs: Vec<f64>,
        omegas: Vec<f64>,
        vapor_kind: &str,
        liquid_kind: &str,
        vapor_eos: Option<CubicEos>,
        liquid_eos: Option<CubicEos>,
        liquid_activity: Option<ActivityModel>,
        mixing_rule: MixingRule,
        kij: Vec<Vec<f64>>,
        aij: Vec<Vec<f64>>,
        alpha: Vec<Vec<f64>>,
        vl: Vec<f64>,
        delta: Vec<f64>,
        psat_coeffs: Vec<Vec<f64>>,
        cp_coeffs: Vec<Vec<f64>>,
        tbs: Vec<f64>,
        names: Vec<String>,
        ge_model: Option<ActivityModel>,
        t_ref: f64,
        p_ref: f64,
    ) -> PyResult<Self> {
        let n = tcs.len();
        if pcs.len() != n || omegas.len() != n {
            return Err(PyValueError::new_err(
                "tcs, pcs, omegas must have the same length",
            ));
        }
        for (label, len) in [
            ("vl", vl.len()),
            ("delta", delta.len()),
            ("psat_coeffs", psat_coeffs.len()),
            ("cp_coeffs", cp_coeffs.len()),
            ("tbs", tbs.len()),
            ("names", names.len()),
        ] {
            if len != 0 && len != n {
                return Err(PyValueError::new_err(format!(
                    "{label} must be empty or have one entry per component ({n})"
                )));
            }
        }
        let components: Vec<Component> = (0..n)
            .map(|i| {
                let mut c = Component {
                    name: names.get(i).cloned().unwrap_or_default(),
                    tc: tcs[i],
                    pc: pcs[i],
                    omega: omegas[i],
                    tb: tbs.get(i).copied().unwrap_or(0.0),
                    psat_coeffs: psat_coeffs.get(i).cloned().unwrap_or_default(),
                    liquid_volume: vl.get(i).copied().unwrap_or(0.0),
                    solubility_param: delta.get(i).copied().unwrap_or(0.0),
                    ..Component::default()
                };
                if let Some(row) = cp_coeffs.get(i) {
                    if row.len() == 5 {
                        c.cp_coeffs = [row[0], row[1], row[2], row[3], row[4]];
                    } else if !row.is_empty() {
                        return Err(PyValueError::new_err(
                            "each cp_coeffs row must have exactly 5 entries",
                        ));
                    }
                }
                Ok(c)
            })
            .collect::<PyResult<_>>()?;

        // Resolve the model tags exactly like the free-function bindings.
        let vapor = match vapor_kind.to_ascii_lowercase().as_str() {
            "ideal" | "idealgas" | "ideal_gas" => VaporModel::IdealGas,
            "virial" => VaporModel::Virial,
            "cubic" | "eos" => VaporModel::Cubic(
                vapor_eos
                    .ok_or_else(|| PyValueError::new_err("vapor_kind='cubic' needs vapor_eos"))?,
            ),
            other => {
                return Err(PyValueError::new_err(format!(
                    "vapor_kind must be 'ideal', 'virial', or 'cubic' (got {other:?})"
                )));
            }
        };
        let liquid = match liquid_kind.to_ascii_lowercase().as_str() {
            "ideal" | "idealsolution" | "ideal_solution" => LiquidModel::IdealSolution,
            "cubic" | "eos" => LiquidModel::Cubic(
                liquid_eos
                    .ok_or_else(|| PyValueError::new_err("liquid_kind='cubic' needs liquid_eos"))?,
            ),
            "activity" | "gamma" => LiquidModel::Activity(liquid_activity.ok_or_else(|| {
                PyValueError::new_err("liquid_kind='activity' needs liquid_activity")
            })?),
            "chao_seader" | "chaoseader" => LiquidModel::ChaoSeader,
            other => {
                return Err(PyValueError::new_err(format!(
                    "liquid_kind must be 'ideal', 'cubic', 'activity', or 'chao_seader' \
                     (got {other:?})"
                )));
            }
        };

        Ok(Self {
            components,
            vapor,
            liquid,
            mixing_rule,
            kij,
            aij,
            alpha,
            vl,
            delta,
            ge_model,
            t_ref,
            p_ref,
        })
    }

    /// Number of components in the system.
    #[getter]
    fn n_components(&self) -> usize {
        self.components.len()
    }

    /// Component names (empty strings where not provided).
    #[getter]
    fn names(&self) -> Vec<String> {
        self.components.iter().map(|c| c.name.clone()).collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "System(n={}, vapor={:?}, liquid={:?}, rule={:?})",
            self.components.len(),
            self.vapor,
            self.liquid,
            self.mixing_rule
        )
    }

    // ── Scalar methods ────────────────────────────────────────────────

    /// Equilibrium ratios `Kᵢ = yᵢ/xᵢ` at trial `(t [K], p [kPa], x, y)`.
    fn k_values(&self, t: f64, p: f64, x: Vec<f64>, y: Vec<f64>) -> PyResult<Vec<f64>> {
        k_values(&self.spec(), t, p, &x, &y).map_err(flash_err)
    }

    /// Isothermal (PT) flash. `t` in **K**, `p` in **kPa abs**. Optional
    /// `k_init` warm-starts the K-loop (e.g. a neighboring point's K).
    /// Returns `(beta, x, y, k, iterations, two_phase)`.
    #[pyo3(signature = (t, p, z, k_init=None, tol=1e-10, max_iter=200))]
    fn flash_pt(
        &self,
        t: f64,
        p: f64,
        z: Vec<f64>,
        k_init: Option<Vec<f64>>,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(f64, Vec<f64>, Vec<f64>, Vec<f64>, usize, bool)> {
        self.check_width("z", z.len())?;
        let r = flash_isothermal_warm(&self.spec(), t, p, &z, k_init.as_deref(), tol, max_iter)
            .map_err(flash_err)?;
        Ok((r.beta, r.x, r.y, r.k, r.iterations, r.two_phase))
    }

    /// Bubble pressure at fixed `t` [K] and liquid `x`.
    /// Returns `(p [kPa], y, k)`.
    #[pyo3(signature = (x, t, tol=1e-9, max_iter=200))]
    fn bubble_pressure(
        &self,
        x: Vec<f64>,
        t: f64,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(f64, Vec<f64>, Vec<f64>)> {
        let r = bubble_pressure(&self.spec(), t, &x, tol, max_iter).map_err(flash_err)?;
        Ok((r.value, r.incipient, r.k))
    }

    /// Bubble temperature at fixed `p` [kPa abs] and liquid `x`.
    /// Returns `(t [K], y, k)`.
    #[pyo3(signature = (x, p, tol=1e-9, max_iter=200))]
    fn bubble_temperature(
        &self,
        x: Vec<f64>,
        p: f64,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(f64, Vec<f64>, Vec<f64>)> {
        let r = bubble_temperature(&self.spec(), p, &x, tol, max_iter).map_err(flash_err)?;
        Ok((r.value, r.incipient, r.k))
    }

    /// Dew pressure at fixed `t` [K] and vapor `y`. Returns `(p [kPa], x, k)`.
    #[pyo3(signature = (y, t, tol=1e-9, max_iter=200))]
    fn dew_pressure(
        &self,
        y: Vec<f64>,
        t: f64,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(f64, Vec<f64>, Vec<f64>)> {
        let r = dew_pressure(&self.spec(), t, &y, tol, max_iter).map_err(flash_err)?;
        Ok((r.value, r.incipient, r.k))
    }

    /// Dew temperature at fixed `p` [kPa abs] and vapor `y`.
    /// Returns `(t [K], x, k)`.
    #[pyo3(signature = (y, p, tol=1e-9, max_iter=200))]
    fn dew_temperature(
        &self,
        y: Vec<f64>,
        p: f64,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(f64, Vec<f64>, Vec<f64>)> {
        let r = dew_temperature(&self.spec(), p, &y, tol, max_iter).map_err(flash_err)?;
        Ok((r.value, r.incipient, r.k))
    }

    /// Adiabatic (PH) flash — φ-φ cubic systems only. `p` in kPa abs,
    /// `h_feed` in kJ/kmol, `[t_lo, t_hi]` the temperature bracket in K.
    /// Returns `(t, beta, x, y, enthalpy)`.
    #[pyo3(signature = (p, z, h_feed, t_lo, t_hi, tol=1e-4, max_iter=200))]
    #[allow(clippy::too_many_arguments)]
    fn flash_ph(
        &self,
        p: f64,
        z: Vec<f64>,
        h_feed: f64,
        t_lo: f64,
        t_hi: f64,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(f64, f64, Vec<f64>, Vec<f64>, f64)> {
        self.check_width("z", z.len())?;
        let r = flash_adiabatic(
            &self.spec(),
            p,
            &z,
            h_feed,
            self.t_ref,
            self.p_ref,
            t_lo,
            t_hi,
            tol,
            max_iter,
        )
        .map_err(flash_err)?;
        Ok((r.t, r.flash.beta, r.flash.x, r.flash.y, r.enthalpy))
    }

    /// Mixture critical point (Heidemann §G) — cubic + classical mixing.
    /// Returns `(Tc [K], Pc [kPa], Vc [m³/kmol])`. `t_init=0` uses the
    /// mole-fraction-average Tc as the initial guess.
    #[pyo3(signature = (z, t_init=0.0, max_iter=200))]
    fn critical_point(
        &self,
        z: Vec<f64>,
        t_init: f64,
        max_iter: usize,
    ) -> PyResult<(f64, f64, f64)> {
        self.check_width("z", z.len())?;
        let cp = critical_point(&self.spec(), &z, t_init, max_iter).map_err(flash_err)?;
        Ok((cp.tc, cp.pc, cp.vc))
    }

    /// Trace the (T, P) phase envelope at composition `z` (§K continuation).
    /// Returns a list of `(T [K], P [kPa])` tuples.
    #[pyo3(signature = (z, p_start=100.0, max_points=60))]
    fn trace_envelope(
        &self,
        z: Vec<f64>,
        p_start: f64,
        max_points: usize,
    ) -> PyResult<Vec<(f64, f64)>> {
        self.check_width("z", z.len())?;
        let pts = trace_envelope(&self.spec(), &z, p_start, max_points).map_err(flash_err)?;
        Ok(pts.into_iter().map(|p| (p.t, p.p)).collect())
    }

    /// Tangent-plane-distance stability test at `(t [K], p [kPa])` —
    /// cubic systems only. Returns `(is_stable, trial_k, tpd)`.
    #[pyo3(signature = (z, t, p, max_iter=100))]
    fn stability(
        &self,
        z: Vec<f64>,
        t: f64,
        p: f64,
        max_iter: usize,
    ) -> PyResult<(bool, Vec<f64>, f64)> {
        self.check_width("z", z.len())?;
        match stability_analysis(&self.spec(), t, p, &z, max_iter).map_err(flash_err)? {
            Stability::Stable => Ok((true, vec![], 0.0)),
            Stability::Unstable { trial_k, tpd } => Ok((false, trial_k, tpd)),
        }
    }

    /// Mixture compressibility factor Z of `phase` ("vapor"/"liquid") at
    /// `(t [K], p [kPa])`, composition `x`. Needs a cubic model on that phase.
    fn z_factor(&self, t: f64, p: f64, x: Vec<f64>, phase: &str) -> PyResult<f64> {
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        z_mix(&self.mixture_spec(eos), t, p, &x, ph).map_err(mix_err)
    }

    /// Partial fugacity coefficients ln φ̂ᵢ of `phase` at `(t, p, x)`.
    fn ln_phi(&self, t: f64, p: f64, x: Vec<f64>, phase: &str) -> PyResult<Vec<f64>> {
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        ln_phi_mix(&self.mixture_spec(eos), t, p, &x, ph).map_err(mix_err)
    }

    /// ∂ln φ̂ᵢ/∂T of `phase` at `(t [K], p [kPa], x)`, in **1/K** (M12.3).
    /// Exact (dual AD). Needs a cubic model on that phase.
    fn d_ln_phi_d_t(&self, t: f64, p: f64, x: Vec<f64>, phase: &str) -> PyResult<Vec<f64>> {
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        crate::mixture::d_ln_phi_d_t(&self.mixture_spec(eos), t, p, &x, ph).map_err(mix_err)
    }

    /// ∂ln φ̂ᵢ/∂P of `phase` at `(t [K], p [kPa], x)`, in **1/kPa** (M12.3).
    /// Exact (dual AD). Needs a cubic model on that phase.
    fn d_ln_phi_d_p(&self, t: f64, p: f64, x: Vec<f64>, phase: &str) -> PyResult<Vec<f64>> {
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        crate::mixture::d_ln_phi_d_p(&self.mixture_spec(eos), t, p, &x, ph).map_err(mix_err)
    }

    /// K-values and their exact T/P derivatives at `(t [K], p [kPa])` given
    /// trial phase compositions `x` (liquid) and `y` (vapor) (M12.3).
    ///
    /// Returns `(k, d_ln_k_d_t [1/K], d_ln_k_d_p [1/kPa])`. Dispatches on the
    /// System's liquid/vapor model exactly like [`Self::k_values`].
    fn k_values_with_derivs(
        &self,
        t: f64,
        p: f64,
        x: Vec<f64>,
        y: Vec<f64>,
    ) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let kv =
            crate::flash::k_values_with_derivs(&self.spec(), t, p, &x, &y).map_err(flash_err)?;
        Ok((kv.k, kv.d_ln_k_d_t, kv.d_ln_k_d_p))
    }

    /// Total molar enthalpy and entropy of one phase relative to the System's
    /// `(t_ref, p_ref)` ideal-gas reference. Returns `(H [kJ/kmol],
    /// S [kJ/(kmol·K)])`.
    ///
    /// Routed through the SystemSpec-level dispatch (M12.4): a **γ-φ** liquid
    /// now returns the ideal − condensation + excess assembly instead of
    /// erroring for lack of a cubic liquid EOS. φ-φ / vapor behavior is
    /// unchanged.
    fn enthalpy_entropy(&self, t: f64, p: f64, x: Vec<f64>, phase: &str) -> PyResult<(f64, f64)> {
        let ph = parse_phase(phase)?;
        crate::flash::phase_enthalpy_entropy(
            &self.spec(),
            t,
            p,
            &x,
            ph,
            self.t_ref,
            self.p_ref,
            &[],
            &[],
        )
        .map_err(flash_err)
    }

    /// Partial molar enthalpies H̄ᵢ of `phase` at `(t [K], p [kPa], x)`, in
    /// **kJ/kmol** (M12.4). `H̄ᵢ = h°ᵢ(T) − R·T²·∂ln φ̂ᵢ/∂T`; Σxᵢ·H̄ᵢ = H.
    /// Needs a cubic model on that phase.
    fn partial_molar_enthalpy(
        &self,
        t: f64,
        p: f64,
        x: Vec<f64>,
        phase: &str,
    ) -> PyResult<Vec<f64>> {
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        crate::energy::partial_molar_enthalpy(
            &self.mixture_spec(eos),
            t,
            p,
            &x,
            ph,
            self.t_ref,
            &[],
        )
        .map_err(mix_err)
    }

    /// Real-mixture isobaric heat capacity Cp of `phase` at `(t [K], p [kPa],
    /// x)`, in **kJ/(kmol·K)** (M12.4). `Cp = Σxᵢ·Cpᵢ°(T) + Cp^R`, the residual
    /// via a second-order dual. Needs a cubic model on that phase.
    fn phase_cp(&self, t: f64, p: f64, x: Vec<f64>, phase: &str) -> PyResult<f64> {
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        crate::energy::phase_cp(&self.mixture_spec(eos), t, p, &x, ph).map_err(mix_err)
    }

    // ── Batch numpy methods (Track D) ─────────────────────────────────
    //
    // All take numpy float64 arrays, broadcast length-1 inputs, release
    // the GIL, and fan out with rayon. Failed points come back as NaN
    // with `converged=False` instead of raising, so one pathological
    // state point can't kill a 10⁵-point sweep.

    /// Batch isothermal flash over paired `(ts [K], ps [kPa])` arrays at
    /// fixed feed `z` (length-1 arrays broadcast).
    ///
    /// `warm_start=True` seeds each point's K-loop with its chunk
    /// predecessor's converged K (§J/§M); `parallel=True` fans the chunks
    /// out with rayon under a released GIL.
    ///
    /// Returns `(beta, x, y, k, iterations, two_phase, converged)` —
    /// 1-D arrays of length m except `x`/`y`/`k` which are m×n.
    #[pyo3(signature = (ts, ps, z, warm_start=true, parallel=true, tol=1e-10, max_iter=200))]
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn flash_pt_batch<'py>(
        &self,
        py: Python<'py>,
        ts: PyReadonlyArray1<'py, f64>,
        ps: PyReadonlyArray1<'py, f64>,
        z: Vec<f64>,
        warm_start: bool,
        parallel: bool,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<(
        Bound<'py, PyArray1<f64>>,
        Bound<'py, PyArray2<f64>>,
        Bound<'py, PyArray2<f64>>,
        Bound<'py, PyArray2<f64>>,
        Bound<'py, PyArray1<u64>>,
        Bound<'py, PyArray1<bool>>,
        Bound<'py, PyArray1<bool>>,
    )> {
        self.check_width("z", z.len())?;
        let n = z.len();
        // Zero-copy views of the numpy buffers (must be C-contiguous).
        let ts = ts
            .as_slice()
            .map_err(|_| PyValueError::new_err("ts must be a contiguous float64 array"))?;
        let ps = ps
            .as_slice()
            .map_err(|_| PyValueError::new_err("ps must be a contiguous float64 array"))?;
        let m = batch_len(ts.len(), ps.len())?;

        // Release the GIL for the whole compute region: pure Rust below.
        let pts = py.allow_threads(|| {
            chunked_run::<Option<Vec<f64>>>(m, parallel, |i, carry| {
                let seed = if warm_start { carry.as_deref() } else { None };
                match flash_isothermal_warm(
                    &self.spec(),
                    bcast(ts, i),
                    bcast(ps, i),
                    &z,
                    seed,
                    tol,
                    max_iter,
                ) {
                    Ok(r) => {
                        // Only reuse K from a genuinely two-phase solve;
                        // a trivial K≈1 carry would poison the neighbor.
                        *carry = if r.two_phase { Some(r.k.clone()) } else { None };
                        PointOut {
                            value: f64::NAN,
                            beta: r.beta,
                            comp_a: r.x,
                            comp_b: r.y,
                            k: r.k,
                            iterations: r.iterations as u64,
                            two_phase: r.two_phase,
                            converged: true,
                        }
                    }
                    Err(_) => {
                        *carry = None;
                        PointOut::failed(n)
                    }
                }
            })
        });

        let beta: Vec<f64> = pts.iter().map(|p| p.beta).collect();
        let iters: Vec<u64> = pts.iter().map(|p| p.iterations).collect();
        let two_phase: Vec<bool> = pts.iter().map(|p| p.two_phase).collect();
        let converged: Vec<bool> = pts.iter().map(|p| p.converged).collect();
        let x: Vec<Vec<f64>> = pts.iter().map(|p| p.comp_a.clone()).collect();
        let y: Vec<Vec<f64>> = pts.iter().map(|p| p.comp_b.clone()).collect();
        let k: Vec<Vec<f64>> = pts.iter().map(|p| p.k.clone()).collect();
        Ok((
            beta.into_pyarray_bound(py),
            PyArray2::from_vec2_bound(py, &x)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            PyArray2::from_vec2_bound(py, &y)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            PyArray2::from_vec2_bound(py, &k)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
            iters.into_pyarray_bound(py),
            two_phase.into_pyarray_bound(py),
            converged.into_pyarray_bound(py),
        ))
    }

    /// Batch bubble pressure: liquid compositions `xs` (m×n) at
    /// temperatures `ts` [K] (length m or 1). Returns
    /// `(p [kPa], y, k, converged)`.
    #[pyo3(signature = (xs, ts, parallel=true, tol=1e-9, max_iter=200))]
    fn bubble_pressure_batch<'py>(
        &self,
        py: Python<'py>,
        xs: PyReadonlyArray2<'py, f64>,
        ts: PyReadonlyArray1<'py, f64>,
        parallel: bool,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<SatBatch<'py>> {
        self.sat_batch(py, xs, ts, parallel, tol, max_iter, |spec, fixed, comp| {
            bubble_pressure(spec, fixed, comp, tol, max_iter)
        })
    }

    /// Batch bubble temperature: liquid compositions `xs` (m×n) at
    /// pressures `ps` [kPa] (length m or 1). Returns `(t [K], y, k, converged)`.
    #[pyo3(signature = (xs, ps, parallel=true, tol=1e-9, max_iter=200))]
    fn bubble_temperature_batch<'py>(
        &self,
        py: Python<'py>,
        xs: PyReadonlyArray2<'py, f64>,
        ps: PyReadonlyArray1<'py, f64>,
        parallel: bool,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<SatBatch<'py>> {
        self.sat_batch(py, xs, ps, parallel, tol, max_iter, |spec, fixed, comp| {
            bubble_temperature(spec, fixed, comp, tol, max_iter)
        })
    }

    /// Batch dew pressure: vapor compositions `ys` (m×n) at temperatures
    /// `ts` [K] (length m or 1). Returns `(p [kPa], x, k, converged)`.
    #[pyo3(signature = (ys, ts, parallel=true, tol=1e-9, max_iter=200))]
    fn dew_pressure_batch<'py>(
        &self,
        py: Python<'py>,
        ys: PyReadonlyArray2<'py, f64>,
        ts: PyReadonlyArray1<'py, f64>,
        parallel: bool,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<SatBatch<'py>> {
        self.sat_batch(py, ys, ts, parallel, tol, max_iter, |spec, fixed, comp| {
            dew_pressure(spec, fixed, comp, tol, max_iter)
        })
    }

    /// Batch dew temperature: vapor compositions `ys` (m×n) at pressures
    /// `ps` [kPa] (length m or 1). Returns `(t [K], x, k, converged)`.
    #[pyo3(signature = (ys, ps, parallel=true, tol=1e-9, max_iter=200))]
    fn dew_temperature_batch<'py>(
        &self,
        py: Python<'py>,
        ys: PyReadonlyArray2<'py, f64>,
        ps: PyReadonlyArray1<'py, f64>,
        parallel: bool,
        tol: f64,
        max_iter: usize,
    ) -> PyResult<SatBatch<'py>> {
        self.sat_batch(py, ys, ps, parallel, tol, max_iter, |spec, fixed, comp| {
            dew_temperature(spec, fixed, comp, tol, max_iter)
        })
    }

    /// Batch compressibility factor Z of `phase` over paired `(ts, ps)`
    /// at fixed composition `x`. Returns a 1-D array of length m.
    #[pyo3(signature = (ts, ps, x, phase, parallel=true))]
    fn z_factor_batch<'py>(
        &self,
        py: Python<'py>,
        ts: PyReadonlyArray1<'py, f64>,
        ps: PyReadonlyArray1<'py, f64>,
        x: Vec<f64>,
        phase: &str,
        parallel: bool,
    ) -> PyResult<Bound<'py, PyArray1<f64>>> {
        self.check_width("x", x.len())?;
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        let ts = ts
            .as_slice()
            .map_err(|_| PyValueError::new_err("ts must be a contiguous float64 array"))?;
        let ps = ps
            .as_slice()
            .map_err(|_| PyValueError::new_err("ps must be a contiguous float64 array"))?;
        let m = batch_len(ts.len(), ps.len())?;
        let out = py.allow_threads(|| {
            let ms = self.mixture_spec(eos);
            let eval =
                |i: usize| z_mix(&ms, bcast(ts, i), bcast(ps, i), &x, ph).unwrap_or(f64::NAN);
            if parallel {
                (0..m).into_par_iter().map(eval).collect::<Vec<f64>>()
            } else {
                (0..m).map(eval).collect()
            }
        });
        Ok(out.into_pyarray_bound(py))
    }

    /// Batch ln φ̂ᵢ of `phase` over paired `(ts, ps)` at fixed composition
    /// `x`. Returns an m×n array.
    #[pyo3(signature = (ts, ps, x, phase, parallel=true))]
    fn ln_phi_batch<'py>(
        &self,
        py: Python<'py>,
        ts: PyReadonlyArray1<'py, f64>,
        ps: PyReadonlyArray1<'py, f64>,
        x: Vec<f64>,
        phase: &str,
        parallel: bool,
    ) -> PyResult<Bound<'py, PyArray2<f64>>> {
        self.check_width("x", x.len())?;
        let n = x.len();
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        let ts = ts
            .as_slice()
            .map_err(|_| PyValueError::new_err("ts must be a contiguous float64 array"))?;
        let ps = ps
            .as_slice()
            .map_err(|_| PyValueError::new_err("ps must be a contiguous float64 array"))?;
        let m = batch_len(ts.len(), ps.len())?;
        let out = py.allow_threads(|| {
            let ms = self.mixture_spec(eos);
            let eval = |i: usize| {
                ln_phi_mix(&ms, bcast(ts, i), bcast(ps, i), &x, ph)
                    .unwrap_or_else(|_| vec![f64::NAN; n])
            };
            if parallel {
                (0..m).into_par_iter().map(eval).collect::<Vec<Vec<f64>>>()
            } else {
                (0..m).map(eval).collect()
            }
        });
        PyArray2::from_vec2_bound(py, &out).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Batch phase enthalpy + entropy over paired `(ts, ps)` at fixed
    /// composition `x`. Returns `(H [kJ/kmol], S [kJ/(kmol·K)])` 1-D arrays.
    #[pyo3(signature = (ts, ps, x, phase, parallel=true))]
    #[allow(clippy::type_complexity)]
    fn enthalpy_entropy_batch<'py>(
        &self,
        py: Python<'py>,
        ts: PyReadonlyArray1<'py, f64>,
        ps: PyReadonlyArray1<'py, f64>,
        x: Vec<f64>,
        phase: &str,
        parallel: bool,
    ) -> PyResult<(Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>)> {
        self.check_width("x", x.len())?;
        let ph = parse_phase(phase)?;
        let eos = self.phase_eos(ph)?;
        let ts = ts
            .as_slice()
            .map_err(|_| PyValueError::new_err("ts must be a contiguous float64 array"))?;
        let ps = ps
            .as_slice()
            .map_err(|_| PyValueError::new_err("ps must be a contiguous float64 array"))?;
        let m = batch_len(ts.len(), ps.len())?;
        let out = py.allow_threads(|| {
            let ms = self.mixture_spec(eos);
            let eval = |i: usize| {
                crate::energy::phase_enthalpy_entropy(
                    &ms,
                    bcast(ts, i),
                    bcast(ps, i),
                    &x,
                    ph,
                    self.t_ref,
                    self.p_ref,
                    &[],
                    &[],
                )
                .unwrap_or((f64::NAN, f64::NAN))
            };
            if parallel {
                (0..m)
                    .into_par_iter()
                    .map(eval)
                    .collect::<Vec<(f64, f64)>>()
            } else {
                (0..m).map(eval).collect()
            }
        });
        let h: Vec<f64> = out.iter().map(|&(h, _)| h).collect();
        let s: Vec<f64> = out.iter().map(|&(_, s)| s).collect();
        Ok((h.into_pyarray_bound(py), s.into_pyarray_bound(py)))
    }
}

impl System {
    /// Shared engine for the four bubble/dew batch methods: `comps` is the
    /// m×n composition matrix, `fixed` the m-or-1 array of the fixed
    /// variable (T for `*_pressure`, P for `*_temperature`), and `solve`
    /// the scalar saturation solver.
    #[allow(clippy::too_many_arguments)]
    fn sat_batch<'py>(
        &self,
        py: Python<'py>,
        comps: PyReadonlyArray2<'py, f64>,
        fixed: PyReadonlyArray1<'py, f64>,
        parallel: bool,
        _tol: f64,
        _max_iter: usize,
        solve: impl (Fn(&SystemSpec, f64, &[f64]) -> Result<SaturationResult, FlashError>) + Sync,
    ) -> PyResult<SatBatch<'py>> {
        let shape = comps.shape();
        let (m, n) = (shape[0], shape[1]);
        self.check_width("composition matrix columns", n)?;
        let comps = comps.as_slice().map_err(|_| {
            PyValueError::new_err("compositions must be a contiguous float64 array")
        })?;
        let fixed = fixed.as_slice().map_err(|_| {
            PyValueError::new_err("fixed variable must be a contiguous float64 array")
        })?;
        if fixed.len() != m && fixed.len() != 1 {
            return Err(PyValueError::new_err(format!(
                "fixed-variable array has length {} but the composition matrix has {m} rows",
                fixed.len()
            )));
        }
        let pts = py.allow_threads(|| {
            chunked_run::<()>(m, parallel, |i, _| {
                // Row i of the C-contiguous m×n matrix.
                let row = &comps[i * n..(i + 1) * n];
                match solve(&self.spec(), bcast(fixed, i), row) {
                    Ok(r) => PointOut {
                        value: r.value,
                        beta: f64::NAN,
                        comp_a: r.incipient,
                        comp_b: vec![],
                        k: r.k,
                        iterations: 0,
                        two_phase: true,
                        converged: true,
                    },
                    Err(_) => PointOut::failed(n),
                }
            })
        });
        sat_batch_out(py, n, pts)
    }
}
