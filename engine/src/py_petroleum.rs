//! PyO3 bindings for the petroleum-characterization layer (Milestone 19).
//!
//! Exposes [`crate::petroleum`] as `vle._engine`: an [`Assay`] class that runs
//! the whole assay → pseudocomponents pipeline, plus `petro_*` free functions
//! for each individual correlation so the pieces can be used (and taught)
//! separately.
//!
//! # Conventions
//!
//! - **Model selection is by string**, not by enum class — `"api"`,
//!   `"kesler-lee"`, `"twu"`, `"d86"`, `"tbp"`. Matches [`crate::py_system`],
//!   and keeps a Jupyter session from needing an import per model. Every parser
//!   accepts a few spellings and lists the valid ones when it rejects.
//! - **Cuts come back as dictionaries**, one per pseudocomponent, carrying
//!   everything the characterization produced. A dict rather than a class
//!   because this is data to tabulate — it drops straight into
//!   `pandas.DataFrame(...)`, which is what a notebook wants.
//! - **Units are the crate's canonical ones**: K, kPa absolute, g/mol,
//!   cm³/mol. The Python wrapper in `vle.petroleum` is where `pint` quantities
//!   are handled.
//!
//! Errors map the way the rest of the bindings do: bad input becomes
//! `ValueError`, a failed solve becomes `RuntimeError`.

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::petroleum::cp::{ideal_gas_cp_mass as cp_mass_rs, ideal_gas_cp_molar as cp_molar_rs};
use crate::petroleum::gravity::{
    api_from_sg as api_from_sg_rs, sg_from_api as sg_from_api_rs, watson_k as watson_k_rs,
};
use crate::petroleum::properties::acentric_lee_kesler as acentric_rs;
use crate::petroleum::vapor_pressure::{
    boiling_point_at_pressure as bp_at_p_rs, normal_boiling_point as nbp_rs,
    vapor_pressure as vp_rs,
};
use crate::petroleum::{
    Assay, CutSpec, DistillationBasis, DistillationCurve, GravitySpec, PetroleumError,
    PropertyMethod, ZcMethod, average_boiling_points as average_boiling_points_rs, convert_curve,
    cut_curve, estimate as estimate_rs, ideal_gas_cp_coeffs as ideal_gas_cp_coeffs_rs,
    petroleum_error_is_input,
};

/// Map a [`PetroleumError`] onto the Python exception the rest of the bindings
/// use: input problems are `ValueError`, everything else is `RuntimeError`.
fn petro_err(e: PetroleumError) -> PyErr {
    if petroleum_error_is_input(&e) {
        PyValueError::new_err(e.to_string())
    } else {
        PyRuntimeError::new_err(e.to_string())
    }
}

/// Parse a distillation-basis name.
fn parse_basis(s: &str) -> PyResult<DistillationBasis> {
    match s.to_ascii_lowercase().replace(['-', '_', ' '], "").as_str() {
        "d86" | "astmd86" => Ok(DistillationBasis::D86),
        "tbp" | "d2892" | "astmd2892" => Ok(DistillationBasis::Tbp),
        "d2887" | "astmd2887" | "sd" | "simdist" => Ok(DistillationBasis::D2887),
        "efv" => Ok(DistillationBasis::Efv),
        other => Err(PyValueError::new_err(format!(
            "unknown distillation basis {other:?}; expected one of \
             'd86', 'tbp', 'd2887' (aliases 'sd', 'simdist'), 'efv'"
        ))),
    }
}

/// Parse a critical-property correlation name.
fn parse_method(s: &str) -> PyResult<PropertyMethod> {
    match s.to_ascii_lowercase().replace(['-', '_', ' '], "").as_str() {
        "riazidaubert1980" | "rd1980" | "riazidaubert80" => Ok(PropertyMethod::RiaziDaubert1980),
        "api" | "api1987" | "riazidaubert" | "riazidaubert1987" | "rd1987" => {
            Ok(PropertyMethod::ApiRiaziDaubert1987)
        }
        "keslerlee" | "leekesler" | "kl" => Ok(PropertyMethod::KeslerLee),
        "twu" => Ok(PropertyMethod::Twu),
        other => Err(PyValueError::new_err(format!(
            "unknown property method {other:?}; expected one of \
             'api' (Riazi-Daubert 1987, the default), 'riazi-daubert-1980', \
             'kesler-lee', 'twu'"
        ))),
    }
}

/// Parse a critical-compressibility correlation name.
fn parse_zc_method(s: &str) -> PyResult<ZcMethod> {
    match s.to_ascii_lowercase().replace(['-', '_', ' '], "").as_str() {
        "leekesler" | "keslerlee" | "lk" => Ok(ZcMethod::LeeKesler),
        "reid" => Ok(ZcMethod::Reid),
        "salerno" => Ok(ZcMethod::Salerno),
        "nath" => Ok(ZcMethod::Nath),
        other => Err(PyValueError::new_err(format!(
            "unknown Zc method {other:?}; expected one of \
             'lee-kesler' (the default), 'reid', 'salerno', 'nath'"
        ))),
    }
}

/// Turn `n` / `boundaries` / `equal_temperature` keyword arguments into a
/// [`CutSpec`], rejecting the combinations that do not mean anything.
fn parse_cut_spec(
    n: Option<usize>,
    boundaries: Option<Vec<f64>>,
    equal_temperature: bool,
) -> PyResult<CutSpec> {
    match (n, boundaries) {
        (Some(_), Some(_)) => Err(PyValueError::new_err(
            "pass either `n` (a cut count) or `boundaries` (cut temperatures), not both",
        )),
        (None, None) => Err(PyValueError::new_err(
            "pass `n` to cut into a fixed number of pseudocomponents, or \
             `boundaries` to cut at given temperatures in K",
        )),
        (Some(n), None) => Ok(if equal_temperature {
            CutSpec::EqualTemperature { n }
        } else {
            CutSpec::EqualVolume { n }
        }),
        (None, Some(boundaries)) => {
            if equal_temperature {
                return Err(PyValueError::new_err(
                    "`equal_temperature` selects how to space `n` cuts; it means \
                     nothing alongside explicit `boundaries`",
                ));
            }
            Ok(CutSpec::Boundaries { boundaries })
        }
    }
}

/// A crude assay: a distillation curve plus a gravity, and the correlations
/// used to turn them into pseudocomponents.
///
/// See [`crate::petroleum::assay`] for the full pipeline.
#[pyclass(name = "Assay", module = "vle._engine")]
pub struct PyAssay {
    inner: Assay,
}

#[pymethods]
impl PyAssay {
    /// Build an assay.
    ///
    /// Supply gravity **either** as `bulk_sg` (one number for the barrel;
    /// per-cut gravities follow from holding Watson K constant) **or** as
    /// `sg_fractions` + `sg_values` (a measured gravity curve). Exactly one.
    #[new]
    #[pyo3(signature = (
        fractions,
        temperatures,
        basis = "tbp",
        bulk_sg = None,
        sg_fractions = None,
        sg_values = None,
        method = "api",
        zc_method = "lee-kesler",
        name_prefix = "PC",
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        fractions: Vec<f64>,
        temperatures: Vec<f64>,
        basis: &str,
        bulk_sg: Option<f64>,
        sg_fractions: Option<Vec<f64>>,
        sg_values: Option<Vec<f64>>,
        method: &str,
        zc_method: &str,
        name_prefix: &str,
    ) -> PyResult<Self> {
        let curve = DistillationCurve::new(parse_basis(basis)?, fractions, temperatures)
            .map_err(petro_err)?;

        let gravity = match (bulk_sg, sg_fractions, sg_values) {
            (Some(bulk_sg), None, None) => GravitySpec::ConstantWatsonK { bulk_sg },
            (None, Some(fractions), Some(sg)) => GravitySpec::Curve { fractions, sg },
            (None, Some(_), None) | (None, None, Some(_)) => {
                return Err(PyValueError::new_err(
                    "a gravity curve needs both `sg_fractions` and `sg_values`",
                ));
            }
            (Some(_), _, _) => {
                return Err(PyValueError::new_err(
                    "pass either `bulk_sg` or a gravity curve \
                     (`sg_fractions` + `sg_values`), not both",
                ));
            }
            (None, None, None) => {
                return Err(PyValueError::new_err(
                    "an assay needs a gravity: pass `bulk_sg`, or a gravity curve \
                     as `sg_fractions` + `sg_values`",
                ));
            }
        };

        let inner = Assay::new(curve, gravity)
            .map_err(petro_err)?
            .with_property_method(parse_method(method)?)
            .with_zc_method(parse_zc_method(zc_method)?)
            .with_name_prefix(name_prefix);
        Ok(Self { inner })
    }

    /// The assay's distillation curve converted to TBP, as
    /// `(fractions, temperatures_in_K)`.
    fn tbp_curve(&self) -> PyResult<(Vec<f64>, Vec<f64>)> {
        let bulk = match &self.inner.gravity {
            GravitySpec::ConstantWatsonK { bulk_sg } => Some(*bulk_sg),
            GravitySpec::Curve { sg, .. } => sg.first().copied(),
        };
        let tbp =
            convert_curve(&self.inner.curve, DistillationBasis::Tbp, bulk).map_err(petro_err)?;
        Ok((tbp.fractions, tbp.temperatures))
    }

    /// The assay's Watson characterization factor, textbook definition.
    #[pyo3(signature = (n = None, boundaries = None, equal_temperature = false))]
    fn watson_k(
        &self,
        n: Option<usize>,
        boundaries: Option<Vec<f64>>,
        equal_temperature: bool,
    ) -> PyResult<f64> {
        let spec = parse_cut_spec(n.or(Some(39)), boundaries, equal_temperature)?;
        self.inner.conventional_watson_k(&spec).map_err(petro_err)
    }

    /// Characterize the assay into pseudocomponents.
    ///
    /// Returns one dict per cut with every characterized property. Pass `n` for
    /// a fixed number of cuts (add `equal_temperature=True` to space them by
    /// boiling range instead of by volume), or `boundaries` to cut at explicit
    /// temperatures in K.
    #[pyo3(signature = (n = None, boundaries = None, equal_temperature = false))]
    fn characterize<'py>(
        &self,
        py: Python<'py>,
        n: Option<usize>,
        boundaries: Option<Vec<f64>>,
        equal_temperature: bool,
    ) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let spec = parse_cut_spec(n, boundaries, equal_temperature)?;
        let pcs = self.inner.characterize(&spec).map_err(petro_err)?;
        pcs.iter()
            .map(|p| {
                let d = PyDict::new_bound(py);
                d.set_item("index", p.cut.index)?;
                d.set_item("name", &p.component.name)?;
                // Where in the barrel it came from.
                d.set_item("fraction", p.cut.fraction)?;
                d.set_item("mole_fraction", p.mole_fraction)?;
                d.set_item("x_lower", p.cut.x_lower)?;
                d.set_item("x_upper", p.cut.x_upper)?;
                d.set_item("t_lower", p.cut.t_lower)?;
                d.set_item("t_upper", p.cut.t_upper)?;
                // What it is.
                d.set_item("tb", p.properties.tb)?;
                d.set_item("sg", p.properties.sg)?;
                d.set_item(
                    "api_gravity",
                    api_from_sg_rs(p.properties.sg).map_err(petro_err)?,
                )?;
                d.set_item("watson_k", p.properties.watson_k)?;
                d.set_item("mw", p.properties.mw)?;
                d.set_item("tc", p.properties.tc)?;
                d.set_item("pc", p.properties.pc)?;
                d.set_item("vc", p.properties.vc)?;
                d.set_item("zc", p.properties.zc)?;
                d.set_item("omega", p.properties.omega)?;
                // What the engine will use.
                d.set_item("cp_coeffs", p.component.cp_coeffs.to_vec())?;
                d.set_item("psat_coeffs", p.component.psat_coeffs.clone())?;
                d.set_item("zra", p.component.zra)?;
                d.set_item("liquid_volume", p.component.liquid_volume)?;
                // M20: regular-solution δ for Grayson-Streed, (cal/cm³)^½.
                d.set_item("solubility_param", p.component.solubility_param)?;
                Ok(d)
            })
            .collect()
    }

    /// Characterize and return just what a flash needs, as
    /// `(names, tc, pc, omega, mw, tb, psat_coeffs, cp_coeffs, mole_fractions)`.
    ///
    /// The tuple is ordered to line up with `vle.System`'s constructor, which
    /// is what the `vle.petroleum` wrapper feeds it into.
    #[pyo3(signature = (n = None, boundaries = None, equal_temperature = false))]
    #[allow(clippy::type_complexity)]
    fn mixture(
        &self,
        n: Option<usize>,
        boundaries: Option<Vec<f64>>,
        equal_temperature: bool,
    ) -> PyResult<(
        Vec<String>,
        Vec<f64>,
        Vec<f64>,
        Vec<f64>,
        Vec<f64>,
        Vec<f64>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<f64>,
    )> {
        let spec = parse_cut_spec(n, boundaries, equal_temperature)?;
        let pcs = self.inner.characterize(&spec).map_err(petro_err)?;
        Ok((
            pcs.iter().map(|p| p.component.name.clone()).collect(),
            pcs.iter().map(|p| p.component.tc).collect(),
            pcs.iter().map(|p| p.component.pc).collect(),
            pcs.iter().map(|p| p.component.omega).collect(),
            pcs.iter().map(|p| p.component.mw).collect(),
            pcs.iter().map(|p| p.component.tb).collect(),
            pcs.iter()
                .map(|p| p.component.psat_coeffs.clone())
                .collect(),
            pcs.iter().map(|p| p.component.cp_coeffs.to_vec()).collect(),
            pcs.iter().map(|p| p.mole_fraction).collect(),
        ))
    }

    fn __repr__(&self) -> String {
        let n = self.inner.curve.len();
        let gravity = match &self.inner.gravity {
            GravitySpec::ConstantWatsonK { bulk_sg } => format!("bulk_sg={bulk_sg}"),
            GravitySpec::Curve { fractions, .. } => {
                format!("gravity curve with {} points", fractions.len())
            }
        };
        format!(
            "Assay({} curve, {n} points, {gravity}, method={:?})",
            self.inner.curve.basis.name(),
            self.inner.property_method
        )
    }
}

// ---------------------------------------------------------------------------
// Free functions — one per correlation, so the pieces are usable on their own
// ---------------------------------------------------------------------------

/// Watson (UOP) characterization factor from `tb` (**K**) and `sg`.
#[pyfunction]
pub fn petro_watson_k(tb: f64, sg: f64) -> PyResult<f64> {
    watson_k_rs(tb, sg).map_err(petro_err)
}

/// API gravity (**°API**) from specific gravity.
#[pyfunction]
pub fn petro_api_from_sg(sg: f64) -> PyResult<f64> {
    api_from_sg_rs(sg).map_err(petro_err)
}

/// Specific gravity from API gravity (**°API**).
#[pyfunction]
pub fn petro_sg_from_api(api: f64) -> PyResult<f64> {
    sg_from_api_rs(api).map_err(petro_err)
}

/// The five average boiling points of a fraction from its ASTM D86 curve.
///
/// Arguments are the 10/30/50/70/90 % D86 temperatures in **K**; returns a dict
/// with keys `vabp`, `wabp`, `mabp`, `cabp`, `meabp`, all in **K**.
#[pyfunction]
pub fn petro_average_boiling_points(
    py: Python<'_>,
    d86_10: f64,
    d86_30: f64,
    d86_50: f64,
    d86_70: f64,
    d86_90: f64,
) -> PyResult<Bound<'_, PyDict>> {
    let a = average_boiling_points_rs(d86_10, d86_30, d86_50, d86_70, d86_90).map_err(petro_err)?;
    let d = PyDict::new_bound(py);
    d.set_item("vabp", a.vabp)?;
    d.set_item("wabp", a.wabp)?;
    d.set_item("mabp", a.mabp)?;
    d.set_item("cabp", a.cabp)?;
    d.set_item("meabp", a.meabp)?;
    Ok(d)
}

/// Convert a distillation curve from one basis to another.
///
/// `temperatures` in **K**; returns the converted temperatures in **K** at the
/// same fractions. `sg` is required only when the route touches EFV.
#[pyfunction]
#[pyo3(signature = (fractions, temperatures, from_basis, to_basis, sg = None))]
pub fn petro_convert_curve(
    fractions: Vec<f64>,
    temperatures: Vec<f64>,
    from_basis: &str,
    to_basis: &str,
    sg: Option<f64>,
) -> PyResult<Vec<f64>> {
    let curve = DistillationCurve::new(parse_basis(from_basis)?, fractions, temperatures)
        .map_err(petro_err)?;
    let out = convert_curve(&curve, parse_basis(to_basis)?, sg).map_err(petro_err)?;
    Ok(out.temperatures)
}

/// Slice a TBP curve into cuts, without characterizing them.
///
/// `temperatures` in **K**. Returns one dict per cut with `index`, `fraction`,
/// `x_lower`, `x_upper`, `t_lower`, `t_upper` and `tb`.
#[pyfunction]
#[pyo3(signature = (fractions, temperatures, n = None, boundaries = None, equal_temperature = false))]
pub fn petro_cut_curve<'py>(
    py: Python<'py>,
    fractions: Vec<f64>,
    temperatures: Vec<f64>,
    n: Option<usize>,
    boundaries: Option<Vec<f64>>,
    equal_temperature: bool,
) -> PyResult<Vec<Bound<'py, PyDict>>> {
    let curve = DistillationCurve::new(DistillationBasis::Tbp, fractions, temperatures)
        .map_err(petro_err)?;
    let spec = parse_cut_spec(n, boundaries, equal_temperature)?;
    cut_curve(&curve, &spec)
        .map_err(petro_err)?
        .into_iter()
        .map(|c| {
            let d = PyDict::new_bound(py);
            d.set_item("index", c.index)?;
            d.set_item("fraction", c.fraction)?;
            d.set_item("x_lower", c.x_lower)?;
            d.set_item("x_upper", c.x_upper)?;
            d.set_item("t_lower", c.t_lower)?;
            d.set_item("t_upper", c.t_upper)?;
            d.set_item("tb", c.tb)?;
            Ok(d)
        })
        .collect()
}

/// Estimate a pseudocomponent's properties from `tb` (**K**) and `sg`.
///
/// Returns a dict with `watson_k`, `mw` (g/mol), `tc` (K), `pc` (kPa),
/// `vc` (cm³/mol), `zc` and `omega`.
#[pyfunction]
#[pyo3(signature = (tb, sg, method = "api", zc_method = "lee-kesler"))]
pub fn petro_estimate<'py>(
    py: Python<'py>,
    tb: f64,
    sg: f64,
    method: &str,
    zc_method: &str,
) -> PyResult<Bound<'py, PyDict>> {
    let p = estimate_rs(parse_method(method)?, tb, sg, parse_zc_method(zc_method)?)
        .map_err(petro_err)?;
    let d = PyDict::new_bound(py);
    d.set_item("tb", p.tb)?;
    d.set_item("sg", p.sg)?;
    d.set_item("watson_k", p.watson_k)?;
    d.set_item("mw", p.mw)?;
    d.set_item("tc", p.tc)?;
    d.set_item("pc", p.pc)?;
    d.set_item("vc", p.vc)?;
    d.set_item("zc", p.zc)?;
    d.set_item("omega", p.omega)?;
    Ok(d)
}

/// Acentric factor from `tb`, `tc` (**K**), `pc` (**kPa**) and `sg`.
#[pyfunction]
pub fn petro_acentric_factor(tb: f64, tc: f64, pc: f64, sg: f64) -> PyResult<f64> {
    acentric_rs(tb, tc, pc, sg).map_err(petro_err)
}

/// Ideal-gas `Cp°` of a fraction in **kJ/(kg·K)** at temperature `t` (**K**).
#[pyfunction]
pub fn petro_ideal_gas_cp_mass(watson_k: f64, t: f64) -> PyResult<f64> {
    cp_mass_rs(watson_k, t).map_err(petro_err)
}

/// Ideal-gas `Cp°` of a fraction in **kJ/(kmol·K)** at temperature `t` (**K**).
#[pyfunction]
pub fn petro_ideal_gas_cp_molar(watson_k: f64, mw: f64, t: f64) -> PyResult<f64> {
    cp_molar_rs(watson_k, mw, t).map_err(petro_err)
}

/// Ideal-gas `Cp°/R` polynomial coefficients (T in **K**) for a fraction.
#[pyfunction]
pub fn petro_ideal_gas_cp_coeffs(watson_k: f64, mw: f64) -> PyResult<Vec<f64>> {
    Ok(ideal_gas_cp_coeffs_rs(watson_k, mw)
        .map_err(petro_err)?
        .to_vec())
}

/// Maxwell–Bonnell atmospheric equivalent temperature, **K**.
///
/// `t` is the observed boiling temperature in **K** at pressure `p` in **kPa**.
#[pyfunction]
#[pyo3(signature = (t, p, watson_k = None))]
pub fn petro_normal_boiling_point(t: f64, p: f64, watson_k: Option<f64>) -> PyResult<f64> {
    nbp_rs(t, p, watson_k).map_err(petro_err)
}

/// Maxwell–Bonnell boiling temperature at pressure `p` (**kPa**) of a fraction
/// whose normal boiling point is `tb` (**K**). Returns **K**.
#[pyfunction]
#[pyo3(signature = (tb, p, watson_k = None))]
pub fn petro_boiling_point_at_pressure(tb: f64, p: f64, watson_k: Option<f64>) -> PyResult<f64> {
    bp_at_p_rs(tb, p, watson_k).map_err(petro_err)
}

/// Maxwell–Bonnell vapor pressure in **kPa** at temperature `t` (**K**) of a
/// fraction whose normal boiling point is `tb` (**K**).
#[pyfunction]
#[pyo3(signature = (t, tb, watson_k = None))]
pub fn petro_vapor_pressure(t: f64, tb: f64, watson_k: Option<f64>) -> PyResult<f64> {
    vp_rs(t, tb, watson_k).map_err(petro_err)
}

/// Register everything in this module into `vle._engine`.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAssay>()?;
    m.add_function(wrap_pyfunction!(petro_watson_k, m)?)?;
    m.add_function(wrap_pyfunction!(petro_api_from_sg, m)?)?;
    m.add_function(wrap_pyfunction!(petro_sg_from_api, m)?)?;
    m.add_function(wrap_pyfunction!(petro_average_boiling_points, m)?)?;
    m.add_function(wrap_pyfunction!(petro_convert_curve, m)?)?;
    m.add_function(wrap_pyfunction!(petro_cut_curve, m)?)?;
    m.add_function(wrap_pyfunction!(petro_estimate, m)?)?;
    m.add_function(wrap_pyfunction!(petro_acentric_factor, m)?)?;
    m.add_function(wrap_pyfunction!(petro_ideal_gas_cp_mass, m)?)?;
    m.add_function(wrap_pyfunction!(petro_ideal_gas_cp_molar, m)?)?;
    m.add_function(wrap_pyfunction!(petro_ideal_gas_cp_coeffs, m)?)?;
    m.add_function(wrap_pyfunction!(petro_normal_boiling_point, m)?)?;
    m.add_function(wrap_pyfunction!(petro_boiling_point_at_pressure, m)?)?;
    m.add_function(wrap_pyfunction!(petro_vapor_pressure, m)?)?;
    Ok(())
}
