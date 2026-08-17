//! Slicing a TBP curve into pseudocomponents.
//!
//! A distillation curve is a continuum. A flash calculation needs a *list*. This
//! module is the bridge: it chops the curve into narrow boiling slices and
//! hands back, for each one, the two numbers everything downstream needs — how
//! much of the barrel it is, and what temperature it boils at.
//!
//! # Three ways to cut, and when each is right
//!
//! | [`CutSpec`] | slices are equal in | use it when |
//! |---|---|---|
//! | [`EqualVolume`] | volume fraction | you want N pseudocomponents and do not care where they land. **The default for feeding a column model.** |
//! | [`EqualTemperature`] | boiling range | you want even resolution in temperature, so the flat middle of the curve does not get all the detail |
//! | [`Boundaries`] | nothing — you say where | you are modelling *real products*: naphtha, kerosene, diesel, AGO. The boundaries are the tower's draw specifications. |
//!
//! [`EqualVolume`]: CutSpec::EqualVolume
//! [`EqualTemperature`]: CutSpec::EqualTemperature
//! [`Boundaries`]: CutSpec::Boundaries
//!
//! The three answer genuinely different questions, and mixing them up is a
//! common way to get a plausible-looking but wrong column. Equal-volume cutting
//! of a crude puts most of its resolution where most of the *material* is;
//! equal-temperature cutting puts it where the *curve is steep*. If you are
//! trying to match a published product yield you want [`Boundaries`], because
//! that is what the refinery actually controls.
//!
//! # The cut's boiling point
//!
//! Each slice's `tb` is the **volume-average** of the TBP curve across the
//! slice, obtained by integrating the interpolant rather than by reading off
//! the midpoint. For narrow slices the two agree to many digits — but for a
//! wide slice that straddles a knee in the curve they do not, and the average
//! is the defensible one. See `wide_cuts_average_rather_than_sample_the_midpoint`
//! in the tests for the size of the difference.

use super::PetroleumError;
use super::distillation::{DistillationBasis, DistillationCurve};

/// How to slice a distillation curve.
#[derive(Debug, Clone, PartialEq)]
pub enum CutSpec {
    /// `n` slices of equal volume fraction, spanning the whole curve.
    EqualVolume {
        /// Number of pseudocomponents to produce. Must be at least 1.
        n: usize,
    },
    /// `n` slices of equal boiling range, spanning the curve's initial and
    /// final boiling points.
    EqualTemperature {
        /// Number of pseudocomponents to produce. Must be at least 1.
        n: usize,
    },
    /// Slices delimited by explicit cut-point temperatures.
    ///
    /// `boundaries` holds the **internal** boundaries only, strictly
    /// increasing, in **K**. The curve's own initial and final boiling points
    /// close the ends, so `k` boundaries produce `k + 1` cuts. Boundaries
    /// outside the curve's span are an error rather than a silent clamp.
    Boundaries {
        /// Internal cut-point temperatures, **K**, strictly increasing.
        boundaries: Vec<f64>,
    },
}

/// One pseudocomponent's share of the assay and where it boils.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    /// Position in the cut list, counting from 0 at the light end.
    pub index: usize,
    /// Fraction of the whole assay this cut represents, **dimensionless**.
    /// Volume fraction for a volume-basis curve. The cuts sum to 1.
    pub fraction: f64,
    /// Cumulative fraction distilled at the cut's lower edge, **dimensionless**.
    pub x_lower: f64,
    /// Cumulative fraction distilled at the cut's upper edge, **dimensionless**.
    pub x_upper: f64,
    /// Temperature at the lower edge, **K**.
    pub t_lower: f64,
    /// Temperature at the upper edge, **K**.
    pub t_upper: f64,
    /// The cut's characteristic normal boiling point, **K** — the
    /// volume-average of the curve across the slice. This is what
    /// [`super::properties::estimate`] wants.
    pub tb: f64,
}

impl Cut {
    /// The cut's boiling range, **K**. Narrow is good: every correlation
    /// downstream assumes the slice behaves like a single compound.
    pub fn width(&self) -> f64 {
        self.t_upper - self.t_lower
    }
}

/// Volume-average of the curve's interpolant over `[x_lo, x_hi]`, in **K**.
///
/// The interpolant is piecewise linear, so its integral over a slice is exact
/// by the trapezoidal rule *provided the panel boundaries include every knot
/// inside the slice* — which is what the loop below arranges. No quadrature
/// error is involved; this is an exact evaluation of the interpolant's mean.
fn average_temperature(curve: &DistillationCurve, x_lo: f64, x_hi: f64) -> f64 {
    if (x_hi - x_lo).abs() < 1e-15 {
        return curve.temperature_at(x_lo);
    }
    // Panel edges: the slice ends plus every grid point strictly inside it.
    let mut edges = vec![x_lo];
    edges.extend(
        curve
            .fractions
            .iter()
            .copied()
            .filter(|&x| x > x_lo && x < x_hi),
    );
    edges.push(x_hi);

    let mut integral = 0.0;
    for w in edges.windows(2) {
        let (a, b) = (w[0], w[1]);
        integral += 0.5 * (curve.temperature_at(a) + curve.temperature_at(b)) * (b - a);
    }
    integral / (x_hi - x_lo)
}

/// Turn the cumulative fractions at every cut boundary into a list of [`Cut`]s.
fn cuts_from_edges(curve: &DistillationCurve, edges: &[f64]) -> Result<Vec<Cut>, PetroleumError> {
    let total = edges[edges.len() - 1] - edges[0];
    if total <= 0.0 {
        return Err(PetroleumError::CutPoints(format!(
            "the cut range has no width: {} to {}",
            edges[0],
            edges[edges.len() - 1]
        )));
    }
    Ok(edges
        .windows(2)
        .enumerate()
        .map(|(index, w)| {
            let (x_lower, x_upper) = (w[0], w[1]);
            Cut {
                index,
                // Normalized by the span actually cut, so the cuts always sum
                // to exactly 1 even when the curve does not run 0 to 1.
                fraction: (x_upper - x_lower) / total,
                x_lower,
                x_upper,
                t_lower: curve.temperature_at(x_lower),
                t_upper: curve.temperature_at(x_upper),
                tb: average_temperature(curve, x_lower, x_upper),
            }
        })
        .collect())
}

/// Slice a distillation curve into pseudocomponents.
///
/// # Arguments
/// * `curve` — the curve to cut. It **must** be on
///   [`DistillationBasis::Tbp`]: every property correlation downstream is
///   written against true boiling points, and cutting a D86 curve directly
///   would quietly bias every cut. Convert first with
///   [`super::distillation::convert_curve`].
/// * `spec` — how to slice it.
///
/// # Returns
/// The cuts, light end first, with `fraction` summing to 1.
///
/// # Errors
/// [`PetroleumError::InvalidInput`] if the curve is not TBP or `n` is zero;
/// [`PetroleumError::CutPoints`] if explicit boundaries are unsorted, or fall
/// outside the curve's temperature span.
pub fn cut_curve(curve: &DistillationCurve, spec: &CutSpec) -> Result<Vec<Cut>, PetroleumError> {
    if curve.basis != DistillationBasis::Tbp {
        return Err(PetroleumError::InvalidInput(format!(
            "cutting needs a TBP curve, got {} — convert it with `convert_curve` first",
            curve.basis.name()
        )));
    }
    let x_first = curve.fractions[0];
    let x_last = curve.fractions[curve.len() - 1];

    let edges: Vec<f64> = match spec {
        CutSpec::EqualVolume { n } => {
            if *n == 0 {
                return Err(PetroleumError::InvalidInput(
                    "a cut count of zero produces no pseudocomponents".into(),
                ));
            }
            (0..=*n)
                .map(|i| x_first + (x_last - x_first) * i as f64 / *n as f64)
                .collect()
        }
        CutSpec::EqualTemperature { n } => {
            if *n == 0 {
                return Err(PetroleumError::InvalidInput(
                    "a cut count of zero produces no pseudocomponents".into(),
                ));
            }
            let (t_first, t_last) = (curve.temperatures[0], curve.temperatures[curve.len() - 1]);
            if t_last <= t_first {
                return Err(PetroleumError::CutPoints(
                    "the curve has no boiling range to divide into equal temperature cuts".into(),
                ));
            }
            // Equal steps in temperature, mapped back onto the volume axis.
            // The first and last edges are pinned to the curve's own end
            // fractions rather than round-tripped through `fraction_at`, so
            // floating-point drift cannot make the cuts fail to span the curve.
            let mut edges: Vec<f64> = Vec::with_capacity(n + 1);
            edges.push(x_first);
            for i in 1..*n {
                let t = t_first + (t_last - t_first) * i as f64 / *n as f64;
                edges.push(curve.fraction_at(t));
            }
            edges.push(x_last);
            edges
        }
        CutSpec::Boundaries { boundaries } => {
            let (t_first, t_last) = (curve.temperatures[0], curve.temperatures[curve.len() - 1]);
            for (i, &t) in boundaries.iter().enumerate() {
                if !t.is_finite() {
                    return Err(PetroleumError::CutPoints(format!(
                        "cut boundary[{i}] = {t} is not a finite temperature"
                    )));
                }
                if i > 0 && t <= boundaries[i - 1] {
                    return Err(PetroleumError::CutPoints(format!(
                        "cut boundaries must strictly increase: boundary[{i}] = {t} K \
                         is not above boundary[{}] = {} K",
                        i - 1,
                        boundaries[i - 1]
                    )));
                }
                if t <= t_first || t >= t_last {
                    return Err(PetroleumError::CutPoints(format!(
                        "cut boundary {t} K is outside the curve's {t_first}-{t_last} K span"
                    )));
                }
            }
            let mut edges = Vec::with_capacity(boundaries.len() + 2);
            edges.push(x_first);
            edges.extend(boundaries.iter().map(|&t| curve.fraction_at(t)));
            edges.push(x_last);
            edges
        }
    };

    // Inverse interpolation can, on a curve with a flat segment, return the
    // same fraction for two different boundary temperatures. That would make a
    // zero-width cut, which is not a pseudocomponent — catch it here rather
    // than letting a zero-mole-fraction component reach the flash.
    for w in edges.windows(2) {
        if w[1] <= w[0] {
            return Err(PetroleumError::CutPoints(format!(
                "cutting produced a zero- or negative-width slice between \
                 fractions {} and {} — the curve is probably flat there",
                w[0], w[1]
            )));
        }
    }

    cuts_from_edges(curve, &edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A wide crude-like TBP curve, 0-95 % over roughly 300-800 K.
    fn crude() -> DistillationCurve {
        DistillationCurve::new(
            DistillationBasis::Tbp,
            vec![0.0, 0.10, 0.30, 0.50, 0.70, 0.90, 0.95],
            vec![310.0, 380.0, 460.0, 540.0, 620.0, 730.0, 790.0],
        )
        .unwrap()
    }

    fn sum_fractions(cuts: &[Cut]) -> f64 {
        cuts.iter().map(|c| c.fraction).sum()
    }

    // === Equal-volume cutting ============================================

    #[test]
    fn equal_volume_produces_the_requested_number_of_equal_slices() {
        for n in [1, 2, 5, 20, 300] {
            let cuts = cut_curve(&crude(), &CutSpec::EqualVolume { n }).unwrap();
            assert_eq!(cuts.len(), n);
            assert!(
                (sum_fractions(&cuts) - 1.0).abs() < 1e-12,
                "n = {n}: fractions sum to {}",
                sum_fractions(&cuts)
            );
            for c in &cuts {
                assert!(
                    (c.fraction - 1.0 / n as f64).abs() < 1e-12,
                    "n = {n}: cut {} has fraction {}",
                    c.index,
                    c.fraction
                );
            }
        }
    }

    #[test]
    fn three_hundred_cuts_is_the_scale_this_exists_for() {
        // The stated target of the whole petroleum track: a crude column with
        // hundreds of pseudocomponents. Check the cuts are sane at that scale
        // and genuinely narrow — a 300-cut assay should have cuts a couple of
        // kelvin wide, which is what justifies treating each as one compound.
        let cuts = cut_curve(&crude(), &CutSpec::EqualVolume { n: 300 }).unwrap();
        assert_eq!(cuts.len(), 300);
        for c in &cuts {
            assert!(c.width() > 0.0, "cut {} has zero width", c.index);
            assert!(
                c.width() < 15.0,
                "cut {} is {:.1} K wide — too coarse to treat as one compound",
                c.index,
                c.width()
            );
            assert!(c.tb > c.t_lower - 1e-9 && c.tb < c.t_upper + 1e-9);
        }
    }

    // === Equal-temperature cutting ========================================

    #[test]
    fn equal_temperature_produces_equal_boiling_ranges() {
        let curve = crude();
        let n = 10;
        let cuts = cut_curve(&curve, &CutSpec::EqualTemperature { n }).unwrap();
        assert_eq!(cuts.len(), n);
        let span = curve.temperatures[curve.len() - 1] - curve.temperatures[0];
        let want = span / n as f64;
        for c in &cuts {
            assert!(
                (c.width() - want).abs() < 1e-6,
                "cut {} spans {:.4} K, expected {want:.4} K",
                c.index,
                c.width()
            );
        }
        assert!((sum_fractions(&cuts) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn equal_temperature_and_equal_volume_disagree_on_a_curved_assay() {
        // The point of having both. On a curve whose slope changes, equal
        // volume slices are not equal temperature slices — if these agreed,
        // one of the two implementations would be wrong.
        let curve = crude();
        let vol = cut_curve(&curve, &CutSpec::EqualVolume { n: 5 }).unwrap();
        let temp = cut_curve(&curve, &CutSpec::EqualTemperature { n: 5 }).unwrap();
        let differs = vol
            .iter()
            .zip(&temp)
            .any(|(a, b)| (a.tb - b.tb).abs() > 1.0);
        assert!(differs, "the two cut specs produced the same slices");
    }

    // === Explicit boundaries =============================================

    #[test]
    fn explicit_boundaries_reproduce_refinery_product_cuts() {
        // A real atmospheric tower: naphtha / kerosene / diesel / AGO / residue,
        // cut at 175, 235, 340 and 370 °C. Four boundaries -> five products.
        let curve = crude();
        let boundaries: Vec<f64> = [175.0, 235.0, 340.0, 370.0]
            .iter()
            .map(|c| c + 273.15)
            .collect();
        let cuts = cut_curve(
            &curve,
            &CutSpec::Boundaries {
                boundaries: boundaries.clone(),
            },
        )
        .unwrap();
        assert_eq!(cuts.len(), 5);
        assert!((sum_fractions(&cuts) - 1.0).abs() < 1e-12);
        // Each internal boundary must land on the temperature that was asked
        // for — that is the whole contract of this cut spec.
        for (i, &want) in boundaries.iter().enumerate() {
            assert!(
                (cuts[i].t_upper - want).abs() < 1e-6,
                "boundary {i}: cut ends at {} K, asked for {want} K",
                cuts[i].t_upper
            );
        }
    }

    #[test]
    fn boundaries_are_validated_rather_than_clamped() {
        let curve = crude();
        // Below the initial boiling point.
        assert!(
            cut_curve(
                &curve,
                &CutSpec::Boundaries {
                    boundaries: vec![250.0]
                }
            )
            .is_err()
        );
        // Above the final boiling point.
        assert!(
            cut_curve(
                &curve,
                &CutSpec::Boundaries {
                    boundaries: vec![900.0]
                }
            )
            .is_err()
        );
        // Not increasing.
        assert!(
            cut_curve(
                &curve,
                &CutSpec::Boundaries {
                    boundaries: vec![500.0, 450.0]
                }
            )
            .is_err()
        );
        // Duplicated.
        assert!(
            cut_curve(
                &curve,
                &CutSpec::Boundaries {
                    boundaries: vec![500.0, 500.0]
                }
            )
            .is_err()
        );
        // Not finite.
        assert!(
            cut_curve(
                &curve,
                &CutSpec::Boundaries {
                    boundaries: vec![f64::NAN]
                }
            )
            .is_err()
        );
    }

    #[test]
    fn no_boundaries_gives_one_cut_spanning_the_whole_assay() {
        let curve = crude();
        let cuts = cut_curve(&curve, &CutSpec::Boundaries { boundaries: vec![] }).unwrap();
        assert_eq!(cuts.len(), 1);
        assert!((cuts[0].fraction - 1.0).abs() < 1e-12);
        assert!((cuts[0].t_lower - 310.0).abs() < 1e-9);
        assert!((cuts[0].t_upper - 790.0).abs() < 1e-9);
    }

    // === The cut's boiling point ==========================================

    #[test]
    fn cuts_inside_one_segment_average_exactly_to_their_midpoint() {
        // The interpolant is piecewise linear, so over a slice that lies wholly
        // within one segment the mean of a linear function *is* its midpoint
        // value — to machine precision, not approximately. Splitting the
        // assertion this way says something exact about the majority of cuts
        // instead of something vague about all of them.
        let curve = crude();
        let cuts = cut_curve(&curve, &CutSpec::EqualVolume { n: 200 }).unwrap();
        let straddles_knot = |c: &Cut| {
            curve
                .fractions
                .iter()
                .any(|&k| k > c.x_lower + 1e-12 && k < c.x_upper - 1e-12)
        };

        let mut interior = 0;
        for c in cuts.iter().filter(|c| !straddles_knot(c)) {
            let midpoint = curve.temperature_at(0.5 * (c.x_lower + c.x_upper));
            assert!(
                (c.tb - midpoint).abs() < 1e-9,
                "cut {}: average {} vs midpoint {midpoint}",
                c.index,
                c.tb
            );
            interior += 1;
        }
        assert!(interior > 190, "only {interior} of 200 cuts were interior");

        // The handful that do straddle a knot differ, but only slightly — the
        // curve's slope change is what they are averaging across.
        for c in cuts.iter().filter(|c| straddles_knot(c)) {
            let midpoint = curve.temperature_at(0.5 * (c.x_lower + c.x_upper));
            let d = (c.tb - midpoint).abs();
            assert!(
                d < 1.0,
                "knot-straddling cut {} differs by {d:.3} K, more than a narrow \
                 cut should",
                c.index
            );
        }
    }

    #[test]
    fn wide_cuts_average_rather_than_sample_the_midpoint() {
        // The justification for integrating instead of sampling. A cut wide
        // enough to straddle a knee in the curve has an average that differs
        // from its midpoint value, and the average is the honest one.
        let curve = crude();
        let cuts = cut_curve(&curve, &CutSpec::EqualVolume { n: 2 }).unwrap();
        let biggest = cuts
            .iter()
            .map(|c| {
                let mid = curve.temperature_at(0.5 * (c.x_lower + c.x_upper));
                (c.tb - mid).abs()
            })
            .fold(0.0, f64::max);
        assert!(
            biggest > 0.5,
            "midpoint and average agree to {biggest:.3} K even on half-the-barrel \
             cuts — if the curve here became straight, this test lost its point"
        );
    }

    #[test]
    fn every_cut_boiling_point_lies_inside_its_own_range() {
        for spec in [
            CutSpec::EqualVolume { n: 7 },
            CutSpec::EqualTemperature { n: 7 },
            CutSpec::Boundaries {
                boundaries: vec![400.0, 500.0, 600.0, 700.0],
            },
        ] {
            for c in cut_curve(&crude(), &spec).unwrap() {
                assert!(
                    c.t_lower <= c.tb && c.tb <= c.t_upper,
                    "{spec:?} cut {}: Tb {} outside [{}, {}]",
                    c.index,
                    c.tb,
                    c.t_lower,
                    c.t_upper
                );
            }
        }
    }

    #[test]
    fn cuts_are_contiguous_and_ordered() {
        for spec in [
            CutSpec::EqualVolume { n: 13 },
            CutSpec::EqualTemperature { n: 13 },
        ] {
            let cuts = cut_curve(&crude(), &spec).unwrap();
            for w in cuts.windows(2) {
                assert!(
                    (w[1].x_lower - w[0].x_upper).abs() < 1e-12,
                    "{spec:?}: gap between cuts {} and {}",
                    w[0].index,
                    w[1].index
                );
                assert!(w[1].tb > w[0].tb, "{spec:?}: boiling points not increasing");
            }
            assert!((cuts[0].x_lower - 0.0).abs() < 1e-12);
            assert!((cuts[cuts.len() - 1].x_upper - 0.95).abs() < 1e-12);
        }
    }

    // === Guard rails ======================================================

    #[test]
    fn cutting_refuses_a_curve_that_is_not_tbp() {
        // Cutting a D86 curve would bias every pseudocomponent, so this is an
        // error naming the fix rather than a silently wrong answer.
        let d86 = DistillationCurve::new(
            DistillationBasis::D86,
            vec![0.0, 0.5, 1.0],
            vec![350.0, 450.0, 550.0],
        )
        .unwrap();
        let err = cut_curve(&d86, &CutSpec::EqualVolume { n: 4 }).unwrap_err();
        assert!(
            matches!(err, PetroleumError::InvalidInput(ref m) if m.contains("convert_curve")),
            "got {err:?}"
        );
    }

    #[test]
    fn a_zero_cut_count_is_an_error() {
        assert!(cut_curve(&crude(), &CutSpec::EqualVolume { n: 0 }).is_err());
        assert!(cut_curve(&crude(), &CutSpec::EqualTemperature { n: 0 }).is_err());
    }

    #[test]
    fn a_flat_curve_cannot_be_cut_by_temperature() {
        // An isothermal "curve" is a pure compound, not an assay. Equal-volume
        // cutting still works on it; equal-temperature cutting cannot, and says
        // so instead of dividing by zero.
        let flat = DistillationCurve::new(
            DistillationBasis::Tbp,
            vec![0.0, 0.5, 1.0],
            vec![400.0, 400.0, 400.0],
        )
        .unwrap();
        assert!(cut_curve(&flat, &CutSpec::EqualTemperature { n: 3 }).is_err());
        let cuts = cut_curve(&flat, &CutSpec::EqualVolume { n: 3 }).unwrap();
        assert_eq!(cuts.len(), 3);
        for c in &cuts {
            assert!((c.tb - 400.0).abs() < 1e-9);
        }
    }

    #[test]
    fn cuts_span_a_curve_that_does_not_start_at_zero() {
        // Assays routinely start at 5 % and stop at 95 %. The fractions must
        // still sum to 1 — they are fractions *of the cut material*.
        let partial = DistillationCurve::new(
            DistillationBasis::Tbp,
            vec![0.05, 0.5, 0.95],
            vec![350.0, 500.0, 700.0],
        )
        .unwrap();
        let cuts = cut_curve(&partial, &CutSpec::EqualVolume { n: 6 }).unwrap();
        assert!((sum_fractions(&cuts) - 1.0).abs() < 1e-12);
        assert!((cuts[0].x_lower - 0.05).abs() < 1e-12);
        assert!((cuts[5].x_upper - 0.95).abs() < 1e-12);
    }
}
