# Ruthless Performance Audit: “NumPy for Thermo”

Scope was strictly limited to the Rust files listed in the request. The central conclusion is that the accepted plan targets the right algorithms but misses the dominant architectural cost:

> Rachford–Rice is not the bottleneck. Repeated state construction, redundant EOS evaluations, pointer-chasing matrices, transient vectors, scalar transcendental work, and finite-difference Jacobians dominate execution time.

The highest-value change is to make every calculation operate through a caller-owned, reusable workspace and a prepared immutable model. Algorithmic modernization should be layered on top of that foundation.

# Part 1 — Flash Calculation Deep-Dive

## 1. The current isothermal loop allocates on every iteration

Each outer iteration currently creates:

- `x` and `y` in `split`.
- `vap` and usually `liq` in `k_values`.
- `k_new`.
- Residual vector `r`.
- `ln_k`.
- Another `k` vector after GDEM.
- Error strings on thermodynamic failures.

See [isothermal.rs](/Users/migueljackson/dev/vle/engine/src/flash/isothermal.rs:248) and [system.rs](/Users/migueljackson/dev/vle/engine/src/flash/system.rs:139).

For an eight-component flash taking 20 iterations, this produces hundreds of allocator interactions before stability or derivative work is counted. `SmallVec` inside the mixture core does not fix allocations at the flash orchestration layer.

### Replace value-returning APIs with workspace-filling kernels

```rust
use smallvec::SmallVec;

const INLINE_COMPONENTS: usize = 16;
type WorkVec = SmallVec<[f64; INLINE_COMPONENTS]>;

pub struct FlashWorkspace {
    pub ln_k: WorkVec,
    pub k_new: WorkVec,
    pub x: WorkVec,
    pub y: WorkVec,
    pub residual: WorkVec,
    pub residual_prev: WorkVec,
    pub ln_phi_l: WorkVec,
    pub ln_phi_v: WorkVec,
    pub scratch0: WorkVec,
    pub scratch1: WorkVec,
}

impl FlashWorkspace {
    pub fn new(n: usize) -> Self {
        fn zeros(n: usize) -> WorkVec {
            let mut v = WorkVec::new();
            v.resize(n, 0.0);
            v
        }

        Self {
            ln_k: zeros(n),
            k_new: zeros(n),
            x: zeros(n),
            y: zeros(n),
            residual: zeros(n),
            residual_prev: zeros(n),
            ln_phi_l: zeros(n),
            ln_phi_v: zeros(n),
            scratch0: zeros(n),
            scratch1: zeros(n),
        }
    }
}

#[inline]
fn split_into(
    z: &[f64],
    ln_k: &[f64],
    beta: f64,
    x: &mut [f64],
    y: &mut [f64],
) {
    for (((&zi, &lki), xi), yi) in z
        .iter()
        .zip(ln_k)
        .zip(x.iter_mut())
        .zip(y.iter_mut())
    {
        let ki = lki.exp();
        let inv_d = (1.0 + beta * (ki - 1.0)).recip();
        *xi = zi * inv_d;
        *yi = ki * *xi;
    }
}
```

Public convenience methods can still return owned `FlashResult`; only the final successful result should allocate.

## 2. Keep K-values in logarithmic form throughout

The loop currently repeatedly performs:

```rust
(k_new[i] / k[i]).ln()
k[i].ln()
ln_k[i].exp()
```

This adds transcendental operations and creates overflow/underflow paths. Equilibrium models naturally produce `ln φ`, `ln γ`, and `ln K`, so converting to `K` before computing a logarithmic residual is counterproductive.

For φ–φ:

```text
ln Kᵢ = ln φᵢᴸ − ln φᵢⱽ
```

For γ–φ:

```text
ln Kᵢ = ln γᵢ + ln Psatᵢ + ln φᵢsat + ln POYᵢ − ln φᵢⱽ − ln P
```

The current γ–φ implementation exponentiates several terms independently and then divides them in [system.rs](/Users/migueljackson/dev/vle/engine/src/flash/system.rs:249), increasing both cost and numerical range.

```rust
#[inline]
fn ln_k_gamma_phi_into(
    ln_gamma: &[f64],
    ln_psat: &[f64],
    ln_phi_sat: &[f64],
    ln_poy: &[f64],
    ln_phi_v: &[f64],
    ln_p: f64,
    out: &mut [f64],
) {
    for i in 0..out.len() {
        out[i] = ln_gamma[i]
            + ln_psat[i]
            + ln_phi_sat[i]
            + ln_poy[i]
            - ln_phi_v[i]
            - ln_p;
    }
}
```

Only exponentiate `ln Kᵢ` where Rachford–Rice or final output actually requires `Kᵢ`.

## 3. Rachford–Rice needs preprocessing and stronger safeguards

The existing Halley kernel makes one division per component and computes `inv²` and `inv³` efficiently, which is good. Nevertheless, several numerical gaps remain in [isothermal.rs](/Users/migueljackson/dev/vle/engine/src/flash/isothermal.rs:55):

### Missing validation

It does not reject:

- Empty inputs.
- Non-finite `z` or `K`.
- Negative compositions.
- `K ≤ 0`.
- A feed that is not normalized.
- Trace components with `z = 0`.
- Degenerate `Kᵢ ≈ 1` values.
- A nonpositive or non-finite tolerance.

`fold(f64::MIN, f64::max)` also silently behaves poorly in the presence of NaNs.

### Pole nudging is not scale-safe

The fixed `1e-10 * span` pole offset can be:

- Too large when the interval is narrow.
- Too small to move the endpoint at large magnitude.
- Non-finite when extreme K-values produce an enormous span.

Use representable-neighbor movement or a relative endpoint guard.

### Halley acceptance is too permissive

`denom.abs() > 0.0` accepts subnormal and catastrophically ill-conditioned denominators. Acceptance should consider scale and require sufficient bracket reduction. Near a critical point, all `Kᵢ → 1`; then `f`, `f′`, and `f″` vanish at different rates and the Halley quotient becomes unreliable.

### Stop criteria are incomplete

Stopping only on `|f|` is scale-dependent. A small residual can coexist with a poorly determined β when `|f′|` is tiny. Require either a narrow bracket or a scaled Newton correction as well.

### Precompute `cᵢ = Kᵢ − 1`

`cᵢ` is invariant during one RR solve but currently recalculated during every Halley evaluation. Store `zᵢcᵢ` and `cᵢ`, dropping trace or near-unity terms.

```rust
struct RrTerm {
    c: f64,
    zc: f64,
}

#[inline]
fn rr_eval(terms: &[RrTerm], beta: f64) -> (f64, f64, f64) {
    let mut f = 0.0;
    let mut df = 0.0;
    let mut ddf = 0.0;

    for term in terms {
        let inv = (1.0 + beta * term.c).recip();
        let q = term.c * inv;
        let base = term.zc * inv;
        f = base.mul_add(1.0, f);
        df = (-base).mul_add(q, df);
        ddf = (2.0 * base).mul_add(q * q, ddf);
    }
    (f, df, ddf)
}

#[inline]
fn accept_halley(
    beta: f64,
    f: f64,
    df: f64,
    ddf: f64,
    lo: f64,
    hi: f64,
) -> Option<f64> {
    let denom = 2.0 * df * df - f * ddf;
    let scale = (2.0 * df * df).abs() + (f * ddf).abs();

    if !denom.is_finite() || denom.abs() <= 32.0 * f64::EPSILON * scale {
        return None;
    }

    let candidate = beta - 2.0 * f * df / denom;
    let guard = 0.05 * (hi - lo);

    (candidate.is_finite()
        && candidate > lo + guard
        && candidate < hi - guard)
        .then_some(candidate)
}
```

For a physical PT flash, first test the physical bracket directly:

```rust
let f0 = rr_eval(terms, 0.0).0;
let f1 = rr_eval(terms, 1.0).0;

if f0 <= 0.0 {
    return Phase::Liquid;
}
if f1 >= 0.0 {
    return Phase::Vapor;
}
// Only now solve beta in [0, 1].
```

Negative-flash support can remain a separate explicit API. It should not enlarge the hot physical-flash interval by default.

## 4. Rachford–Rice SIMD is useful, but EOS SIMD matters more

RR arrays are an excellent SIMD shape: three contiguous inputs and independent lanes. Stable Rust’s best portable approach without committing to a SIMD dependency is an unrolled contiguous loop that LLVM can vectorize.

```rust
#[inline(always)]
fn rr_f_unrolled(z: &[f64], c: &[f64], beta: f64) -> f64 {
    let mut s0 = 0.0;
    let mut s1 = 0.0;
    let mut s2 = 0.0;
    let mut s3 = 0.0;

    let chunks = z.len() / 4;
    for q in 0..chunks {
        let i = q * 4;
        s0 += z[i]     * c[i]     / (1.0 + beta * c[i]);
        s1 += z[i + 1] * c[i + 1] / (1.0 + beta * c[i + 1]);
        s2 += z[i + 2] * c[i + 2] / (1.0 + beta * c[i + 2]);
        s3 += z[i + 3] * c[i + 3] / (1.0 + beta * c[i + 3]);
    }

    let mut sum = (s0 + s1) + (s2 + s3);
    for i in chunks * 4..z.len() {
        sum += z[i] * c[i] / (1.0 + beta * c[i]);
    }
    sum
}
```

For explicit SIMD, use architecture-specific AVX2/AVX-512 or NEON kernels behind runtime dispatch, retaining the scalar kernel for small `n`. Do not expect SIMD to help binary or ternary mixtures; dispatch overhead can exceed the saved cycles.

More important SIMD candidates are:

- Pure-component `Aᵢ`, `Bᵢ`, and alpha evaluation.
- `sqrt(AᵢAⱼ)(1-kᵢⱼ)` matrix construction.
- Row dot products `Aij · x`.
- K residual/update loops.
- Batch calculations across state points.

Transcendentals (`exp`, `ln`, `sqrt`) will limit vectorization unless a vector math implementation or target-specific intrinsics are used. Measure accuracy requirements before enabling approximate vector math.

## 5. GDEM currently lacks trust-region protection

The implementation accepts any `μ ∈ (0,1)` and applies an amplification of `1 + μ/(1−μ)`. As `μ → 1`, the step tends to infinity. There is no:

- Maximum extrapolation factor.
- Residual-decrease test.
- Finite `ln K` bounds.
- Backtracking.
- Oscillation detection.
- Phase-composition validity check.

This can turn a slowly converging state near criticality into overflow.

```rust
fn guarded_gdem(
    ln_k: &[f64],
    r: &[f64],
    r_prev: &[f64],
    trial: &mut [f64],
) -> bool {
    let rr = dot(r, r_prev);
    let pp = dot(r_prev, r_prev);
    if pp <= f64::MIN_POSITIVE {
        return false;
    }

    let mu = rr / pp;
    if !(0.0..0.95).contains(&mu) {
        return false;
    }

    let gain = (1.0 / (1.0 - mu)).min(4.0);
    for i in 0..ln_k.len() {
        let candidate = ln_k[i] + gain * r[i];
        if !candidate.is_finite() || !(-80.0..=80.0).contains(&candidate) {
            return false;
        }
        trial[i] = candidate;
    }
    true
}
```

Evaluate the accelerated candidate and accept it only if the residual norm decreases sufficiently. Otherwise fall back to ordinary successive substitution.

## 6. The advertised full Newton finish is not implemented

The module documentation explicitly says the Newton polish remains future work. Current `flash_isothermal` is SS plus occasional scalar GDEM only.

The Newton unknowns should not naively be all `ln K` plus β without scaling. Near criticality, `ln K → 0` and the system becomes rank-deficient. Use:

- `n−1` independent composition variables per phase, or
- `ln K` with β eliminated through RR and an explicitly projected Jacobian.
- A trust-region or line-search step.
- An RR derivative used through the implicit function theorem.
- A condition-number or pivot-quality test before switching from SS.

For fixed `ln K`, implicit differentiation gives:

```text
dβ/d(ln Kⱼ) = −(∂f/∂ln Kⱼ)/(∂f/∂β)
```

```rust
#[inline]
fn d_beta_d_ln_k(
    z: &[f64],
    k: &[f64],
    beta: f64,
    out: &mut [f64],
) {
    let mut df_dbeta = 0.0;

    for i in 0..z.len() {
        let c = k[i] - 1.0;
        let inv = (1.0 + beta * c).recip();
        df_dbeta -= z[i] * c * c * inv * inv;
    }

    for j in 0..z.len() {
        let d = 1.0 + beta * (k[j] - 1.0);
        // ∂[z(K−1)/d]/∂lnK = z*K/d²
        let df_dlnk = z[j] * k[j] / (d * d);
        out[j] = -df_dlnk / df_dbeta;
    }
}
```

Use this to form the analytic equilibrium Jacobian rather than finite-differencing complete flash evaluations.

## 7. Stability testing is allocation-heavy and algorithmically fragile

Each stability iteration currently allocates:

- Normalized `wn`.
- `ln_phi`—and `min_gibbs_ln_phi` may evaluate both cubic roots.
- `w_new`.

See [stability.rs](/Users/migueljackson/dev/vle/engine/src/flash/stability.rs:58).

It also uses only two Wilson seeds. That can miss multiple local TPD minima in highly non-ideal or multicomponent mixtures. Components with `zᵢ = 0` cause `ln zᵢ`, `wnᵢ/zᵢ`, and trial-K failures.

### Required changes

- Reuse trial buffers.
- Operate on `ln W` to prevent overflow/underflow.
- Exclude inactive components below a configurable threshold.
- Normalize with log-sum-exp.
- Use multiple deterministic seeds: Wilson liquid/vapor, pure-component corners, and previous batch-point minima.
- Accelerate TPD SS with safeguarded GDEM or switch to Newton/BFGS on independent log-composition coordinates.
- Do not select the minimum-Gibbs root by independently recomputing the entire mixture state twice.

```rust
#[inline]
fn normalize_log_weights(ln_w: &[f64], w: &mut [f64]) -> f64 {
    let max = ln_w.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut sum = 0.0;

    for (dst, &v) in w.iter_mut().zip(ln_w) {
        *dst = (v - max).exp();
        sum += *dst;
    }

    let inv = sum.recip();
    for wi in w {
        *wi *= inv;
    }

    max + sum.ln()
}
```

For trace components:

```rust
let active: SmallVec<[usize; 16]> = z
    .iter()
    .enumerate()
    .filter_map(|(i, &zi)| (zi > 1e-14).then_some(i))
    .collect();
```

This also shrinks all stability linear algebra.

## 8. `min_gibbs_ln_phi` doubles work and allocation

The function evaluates the complete mixture fugacity path separately for liquid and vapor roots in [system.rs](/Users/migueljackson/dev/vle/engine/src/flash/system.rs:217). But both phases share:

- Pure `Aᵢ/Bᵢ/Uᵢ/Wᵢ`.
- Mixed parameters.
- Cubic coefficients.
- Most fugacity algebra.

Compute the cubic roots once and evaluate both candidate roots against the same prepared mixture state.

```rust
pub struct MixturePoint<const N: usize> {
    pub a: f64,
    pub b: f64,
    pub u: f64,
    pub w: f64,
    pub a_bar: [f64; N],
    pub b_bar: [f64; N],
    pub u_bar: [f64; N],
    pub w_bar: [f64; N],
    pub roots: [f64; 3],
    pub root_count: u8,
}

impl<const N: usize> MixturePoint<N> {
    pub fn ln_phi_at(&self, z: f64, out: &mut [f64; N]) {
        // Shared attractive integrals calculated once here.
        // Fill every component without rebuilding mixture parameters.
    }
}
```

A dynamic equivalent should use slices into a reusable workspace.

## 9. Phase-envelope continuation is dominated by finite differences

Each `(n+2)×(n+2)` Jacobian column calls the complete residual again in [envelope.rs](/Users/migueljackson/dev/vle/engine/src/flash/envelope.rs:96). Each residual:

- Allocates three component vectors.
- Computes feed fugacity again, even though feed composition is fixed.
- Computes both candidate roots for both feed and trial phases.
- Allocates a `DVector`.

The cost is effectively repeated nested `O(n²)` thermodynamics inside `O(n)` finite-difference columns.

### Replace it with analytic block assembly

At each envelope point cache:

- Feed-phase `ln φ(z)`.
- Feed derivatives with respect to T and P.
- Trial-phase `ln φ(w)`.
- Trial composition Jacobian.
- `K`, normalized `w`, and normalization derivatives.

```rust
fn assemble_envelope_jacobian(
    z: &[f64],
    k: &[f64],
    w: &[f64],
    dlnphi_w_dnw: &[f64], // flat row-major n*n
    dlnphi_w_dt: &[f64],
    dlnphi_w_dp: &[f64],
    dlnphi_z_dt: &[f64],
    dlnphi_z_dp: &[f64],
    j: &mut [f64],        // row-major (n+2)^2
) {
    let n = z.len();
    let m = n + 2;

    for row in 0..n {
        for col in 0..n {
            // ∂w_q/∂lnK_col = w_q(δ_qcol − w_col)
            let mut chain = 0.0;
            for q in 0..n {
                let dw = w[q] * (usize::from(q == col) as f64 - w[col]);
                chain += dlnphi_w_dnw[row * n + q] * dw;
            }
            j[row * m + col] = usize::from(row == col) as f64 + chain;
        }

        j[row * m + n] = dlnphi_w_dt[row] - dlnphi_z_dt[row];
        j[row * m + n + 1] = dlnphi_w_dp[row] - dlnphi_z_dp[row];
    }

    for col in 0..n {
        j[n * m + col] = z[col] * k[col];
    }
}
```

Apply log-variable chain factors for `ln T` and `ln P`: multiply temperature derivatives by `T` and pressure derivatives by `P`.

Use pseudo-arclength continuation and reuse the previous LU/QR factorization as a preconditioner. The current “largest tangent component” parameter switching is workable but not as robust near turning points and critical singularities.

# Part 2 — Global Package Optimization

## 1. `EosState` caches the wrong granularity for mixture workloads

`EosState` is a useful pure-component cache, but it is rebuilt for every component whenever mixture parameters are evaluated. It also stores `t` and `p` redundantly in every per-component state and includes fields not required by every consumer.

See [eos.rs](/Users/migueljackson/dev/vle/engine/src/eos.rs:945).

A flash point needs a two-level cache:

1. Immutable model cache: critical constants, EOS constants, flattened interaction matrices, activity parameters.
2. Per-`(T,P)` cache: pure-component alpha and dimensionless parameters.
3. Per-composition phase cache: mixed parameters, roots, fugacity, optional derivatives.

```rust
pub struct PreparedModel {
    pub n: usize,
    pub tc_inv: Box<[f64]>,
    pub pc_inv: Box<[f64]>,
    pub omega: Box<[f64]>,
    pub omega_a: Box<[f64]>,
    pub omega_b: Box<[f64]>,
    pub kij: Box<[f64]>, // flat row-major
}

pub struct TpCache {
    pub t: f64,
    pub p: f64,
    pub ai: Box<[f64]>,
    pub bi: Box<[f64]>,
    pub sqrt_ai: Box<[f64]>,
    pub d_ai_dt: Box<[f64]>,
    pub psat: Box<[f64]>,
    pub ln_psat: Box<[f64]>,
}

impl TpCache {
    pub fn update(&mut self, model: &PreparedModel, t: f64, p: f64) {
        self.t = t;
        self.p = p;

        for i in 0..model.n {
            let tr = t * model.tc_inv[i];
            let alpha = alpha_fast(tr, model.omega[i]);
            self.ai[i] = model.omega_a[i] * alpha
                * p * model.pc_inv[i] / (tr * tr);
            self.bi[i] = model.omega_b[i] * p * model.pc_inv[i] / tr;
            self.sqrt_ai[i] = self.ai[i].sqrt();
        }
    }
}
```

For a PT flash, `TpCache` is constructed once and reused by every liquid/vapor composition iteration.

## 2. `Component` is a cold, oversized AoS object

`Component` contains:

- A heap-backed `String`.
- A heap-backed `Vec<f64>`.
- Many fields unused by a particular EOS.
- Saturation and energy data interleaved with cubic-EOS data.

See [types.rs](/Users/migueljackson/dev/vle/engine/src/types.rs:69).

Iterating components to evaluate a cubic EOS drags cold metadata through cache lines. Convert the public component representation into a prepared hot Struct-of-Arrays representation.

```rust
#[repr(align(64))]
pub struct CubicComponentSoa {
    pub inv_tc: Box<[f64]>,
    pub inv_pc: Box<[f64]>,
    pub omega: Box<[f64]>,
    pub m_polar: Box<[f64]>,
    pub n_polar: Box<[f64]>,
    pub prsv_k1: Box<[f64]>,
}
```

Keep names, database metadata, and variable-length correlations in a separate cold structure. This improves:

- Sequential component evaluation.
- SIMD loads.
- Batch calculations.
- Cache-line sharing between Rayon workers.

## 3. `Vec<Vec<f64>>` must be removed from hot matrices

It appears throughout `kij`, `aij`, NRTL alpha, Jacobians, and virial matrices. Every row is a separate allocation and row traversal requires pointer chasing.

Examples include [types.rs](/Users/migueljackson/dev/vle/engine/src/types.rs:197), [virial.rs](/Users/migueljackson/dev/vle/engine/src/virial.rs:165), and [mixture.rs](/Users/migueljackson/dev/vle/engine/src/mixture.rs:818).

Use flat aligned storage:

```rust
#[derive(Clone)]
pub struct DenseMatrix {
    n: usize,
    data: Box<[f64]>,
}

impl DenseMatrix {
    #[inline(always)]
    pub fn at(&self, row: usize, col: usize) -> f64 {
        // Validate shape once when constructing the prepared model.
        unsafe { *self.data.get_unchecked(row * self.n + col) }
    }

    #[inline]
    pub fn row(&self, row: usize) -> &[f64] {
        &self.data[row * self.n..(row + 1) * self.n]
    }
}
```

For symmetric `kij`, packed triangular storage saves memory but harms vectorized row access. For small mixtures, the dense `n×n` representation is generally faster and small enough to remain cache-resident.

## 4. Classical mixing unnecessarily recomputes square roots

The inner closure currently evaluates:

```rust
(ai[i] * ai[j]).sqrt()
```

inside the `n²` mixing loop. Precompute `sqrt_ai[i]`; then:

```rust
#[inline]
fn classical_mix(
    x: &[f64],
    sqrt_ai: &[f64],
    bi: &[f64],
    kij: &[f64],
    a_bar: &mut [f64],
) -> (f64, f64) {
    let n = x.len();
    let mut a_mix = 0.0;
    let mut b_mix = 0.0;

    for i in 0..n {
        b_mix = x[i].mul_add(bi[i], b_mix);

        let row = &kij[i * n..(i + 1) * n];
        let mut ai_row = 0.0;
        for j in 0..n {
            let aij = sqrt_ai[i] * sqrt_ai[j] * (1.0 - row[j]);
            ai_row = x[j].mul_add(aij, ai_row);
        }

        a_bar[i] = 2.0 * ai_row;
        a_mix = x[i].mul_add(ai_row, a_mix);
    }

    (a_mix, b_mix)
}
```

For symmetric classical mixing, computing the upper triangle once can nearly halve multiplications. Whether it wins depends on `n` and vectorization; benchmark both kernels.

## 5. Activity models recompute invariant matrices inside nested loops

Wilson repeatedly evaluates `wilson_lambda_generic(k,j,...)` in both the sum and output loops. NRTL repeatedly evaluates `tau` and `exp(-alpha*tau)` in multiple nested loops in [activity.rs](/Users/migueljackson/dev/vle/engine/src/activity.rs:476).

At fixed temperature, cache:

- Wilson `Λᵢⱼ`.
- NRTL `τᵢⱼ`.
- NRTL `Gᵢⱼ`.
- NRTL `τᵢⱼGᵢⱼ`.

```rust
pub struct ActivityTpCache {
    pub lambda: Box<[f64]>,
    pub tau: Box<[f64]>,
    pub g: Box<[f64]>,
    pub tau_g: Box<[f64]>,
}

fn nrtl_into(
    n: usize,
    x: &[f64],
    cache: &ActivityTpCache,
    s: &mut [f64],
    c: &mut [f64],
    out: &mut [f64],
) {
    s.fill(0.0);
    c.fill(0.0);

    for j in 0..n {
        for k in 0..n {
            let idx = k * n + j;
            s[j] = x[k].mul_add(cache.g[idx], s[j]);
            c[j] = x[k].mul_add(cache.tau_g[idx], c[j]);
        }
    }

    for i in 0..n {
        let mut value = c[i] / s[i];
        let row = i * n;
        for j in 0..n {
            value += x[j] * cache.g[row + j] / s[j]
                * (cache.tau[row + j] - c[j] / s[j]);
        }
        out[i] = value;
    }
}
```

This removes large numbers of `exp` calls from every flash iteration.

## 6. `num-dual` is exact but the current derivative strategy scales poorly

For nonclassical composition derivatives, the implementation performs one complete dual evaluation per Jacobian column in [mixture.rs](/Users/migueljackson/dev/vle/engine/src/mixture.rs:818). That is effectively:

- `n` thermodynamic evaluations,
- each containing `O(n²)` mixing work,
- producing `O(n³)` cost,
- with repeated normalization and `SmallVec` construction.

For common `n ≤ 8`, use one of:

- Hand-derived analytic derivatives for all classical and common activity paths.
- A fixed-width multiderivative dual that carries all `n` derivatives in one evaluation.
- Reverse-mode differentiation when outputs and inputs are both large.
- Chunked forward mode, e.g. four derivative lanes per evaluation.

A lightweight fixed-width dual avoids dynamic derivative vectors:

```rust
#[derive(Clone, Copy)]
struct DualN<const N: usize> {
    re: f64,
    eps: [f64; N],
}

impl<const N: usize> DualN<N> {
    #[inline]
    fn variable(re: f64, index: usize) -> Self {
        let mut eps = [0.0; N];
        eps[index] = 1.0;
        Self { re, eps }
    }

    #[inline]
    fn exp(self) -> Self {
        let e = self.re.exp();
        let mut out = self;
        out.re = e;
        for d in &mut out.eps {
            *d *= e;
        }
        out
    }
}
```

Specialize through const-generic dispatch for common component counts:

```rust
match n {
    1 => derivative_kernel::<1>(...),
    2 => derivative_kernel::<2>(...),
    3 => derivative_kernel::<3>(...),
    4 => derivative_kernel::<4>(...),
    5..=8 => derivative_kernel::<8>(...),
    _ => dynamic_derivative_kernel(...),
}
```

This permits stack storage and compiler unrolling without making the public API const-generic.

## 7. `SmallVec<[D; 8]>` is not automatically cheap for dual numbers

For `f64`, eight inline values cost 64 bytes. For higher-order or vector duals, `[D; 8]` can become hundreds of bytes and is repeatedly moved or initialized. It can increase stack traffic and inhibit vectorization.

Use separate aliases by scalar type or pass caller-owned output/scratch slices. A generic `Buf<D>` should not define the performance policy for every scalar representation.

```rust
pub struct MixtureScratch<'a, D> {
    pub ai: &'a mut [D],
    pub bi: &'a mut [D],
    pub a_bar: &'a mut [D],
    pub b_bar: &'a mut [D],
    pub u_bar: &'a mut [D],
    pub w_bar: &'a mut [D],
}
```

This is faster and more predictable than constructing four `SmallVec`s for every `mixture_params` call.

## 8. `nalgebra::DMatrix/DVector` is too general for the dominant small systems

The envelope, critical, and Broyden paths use dynamic matrices. Broyden also explicitly computes the inverse Jacobian with `try_inverse` in [broyden.rs](/Users/migueljackson/dev/vle/engine/src/numerics/broyden.rs:230). Explicit inversion is slower and less stable than factor-and-solve.

For `n ≤ 8`:

- Use fixed-capacity stack matrices.
- Apply partial-pivot LU in place.
- Reuse matrix and pivot buffers.
- Solve systems; never form `J⁻¹`.
- For Broyden, update an approximate Jacobian and solve it, or use an inverse update only when profiling proves beneficial.

```rust
pub struct StackLu<const MAX: usize> {
    a: [[f64; MAX]; MAX],
    piv: [usize; MAX],
    n: usize,
}

impl<const MAX: usize> StackLu<MAX> {
    pub fn factor(&mut self) -> Result<(), ()> {
        for k in 0..self.n {
            let mut pivot = k;
            for i in k + 1..self.n {
                if self.a[i][k].abs() > self.a[pivot][k].abs() {
                    pivot = i;
                }
            }
            if self.a[pivot][k].abs() <= f64::EPSILON {
                return Err(());
            }
            self.a.swap(k, pivot);
            self.piv.swap(k, pivot);

            for i in k + 1..self.n {
                self.a[i][k] /= self.a[k][k];
                for j in k + 1..self.n {
                    self.a[i][j] -= self.a[i][k] * self.a[k][j];
                }
            }
        }
        Ok(())
    }
}
```

Retain `nalgebra` as the fallback for large systems and eigenproblems.

## 9. Virial evaluation rebuilds a fragmented matrix

`ln_phi_mix_virial` builds `Vec<Vec<f64>>` for every call in [virial.rs](/Users/migueljackson/dev/vle/engine/src/virial.rs:215). During a fixed-temperature flash, `Bᵢⱼ` is invariant.

Flatten and cache it in `TpCache`, then compute `row_dot = Bx` once:

```rust
fn virial_ln_phi_into(
    matrix: &[f64],
    n: usize,
    x: &[f64],
    factor: f64,
    row_dot: &mut [f64],
    out: &mut [f64],
) {
    for i in 0..n {
        let row = &matrix[i * n..(i + 1) * n];
        row_dot[i] = row.iter().zip(x).map(|(&b, &xj)| b * xj).sum();
    }

    let b_mix = x.iter().zip(row_dot.iter()).map(|(&xi, &r)| xi * r).sum::<f64>();

    for i in 0..n {
        out[i] = factor * (2.0 * row_dot[i] - b_mix);
    }
}
```

This changes the fugacity calculation from two matrix traversals plus allocation to one matrix-vector pass and two vector passes.

## 10. Rayon batch output uses an expensive AoS-to-SoA transpose

The Python batch implementation computes a vector of per-point result structs and then separately collects each output field. This:

- Stores nested per-point `Vec`s.
- Revisits results multiple times.
- Allocates every output independently.
- Produces poor cache behavior.
- Performs an AoS-to-SoA conversion after computation.

The NumPy API is zero-copy on input, but not output. Results are assembled in Rust vectors and transferred into newly allocated NumPy arrays.

A faster design preallocates final SoA output buffers and lets disjoint Rayon chunks write directly:

```rust
pub struct BatchOutput {
    beta: Vec<f64>,
    x: Vec<f64>, // point-major m*n
    y: Vec<f64>,
    k: Vec<f64>,
    iterations: Vec<u32>,
    flags: Vec<u8>,
}

fn run_chunk(
    point_start: usize,
    n: usize,
    beta: &mut [f64],
    x: &mut [f64],
    y: &mut [f64],
    k: &mut [f64],
    workspace: &mut FlashWorkspace,
) {
    for local in 0..beta.len() {
        let row = local * n;
        // Solve directly into x[row..row+n], etc.
        let _global_point = point_start + local;
        let _ = (&mut x[row..row + n], &mut y[row..row + n], &mut k[row..row + n]);
    }
}
```

Give each Rayon worker:

- A private `FlashWorkspace`.
- A contiguous range of output rows.
- Optional warm-start state local to that range.

Avoid one task per state point. Use chunks large enough to amortize Rayon scheduling—typically hundreds of points for cheap EOS calls, determined by benchmarks.

## 11. Parallel layout and false sharing

No hot-path locks were found in the inspected engine calculation code. Lock contention is therefore not the present problem. False sharing remains possible if adjacent Rayon jobs write scalar results such as β and flags on the same cache line.

Use coarse contiguous chunks. Do not pad every scalar output; chunk ownership already prevents simultaneous writes to most lines. Thread-local workspaces should be cache-line aligned if stored together:

```rust
#[repr(align(64))]
pub struct AlignedWorkspace(pub FlashWorkspace);
```

For batch state-point processing, SoA usually wins because consumers often request one property column and output arrays are naturally column-oriented. For a single state point, component data should also be SoA. The existing `Vec<Component>` AoS layout is unfavorable in both cases.

## 12. Fast-path dispatch must happen outside inner loops

The mixture core currently switches on EOS and mixing rules inside generic evaluation paths. Build a prepared model containing function pointers or an enum-selected kernel once:

```rust
type FugacityKernel = fn(
    model: &PreparedModel,
    tp: &TpCache,
    x: &[f64],
    phase: PhaseId,
    out: &mut [f64],
    scratch: &mut MixtureWorkspace,
) -> Result<(), MixError>;

pub struct PreparedSystem {
    pub model: PreparedModel,
    pub liquid_kernel: FugacityKernel,
    pub vapor_kernel: FugacityKernel,
}
```

This allows separate monomorphic kernels for:

- PR/SRK classical.
- Three-parameter classical.
- Wilson γ–φ.
- NRTL γ–φ.
- Exotic GE/dual-number fallback.

The common PR/SRK classical path should not pay abstraction or validation costs required by every exotic rule.

## 13. Separate checked APIs from unchecked hot kernels

Dimension validation currently traverses nested matrices during repeated mixture evaluation. Shape and model compatibility are immutable properties. Validate once when constructing `PreparedSystem`.

```rust
pub fn ln_phi_checked(
    model: &PreparedSystem,
    x: &[f64],
    out: &mut [f64],
) -> Result<(), MixError> {
    if x.len() != model.model.n || out.len() != model.model.n {
        return Err(MixError::Dimension("composition width mismatch".into()));
    }
    // Safety invariants established by PreparedSystem construction.
    unsafe { ln_phi_unchecked(model, x, out) }
}
```

Keep `unsafe` confined to a tiny internal layer. The performance gain comes from eliminating repeated bounds and shape checks, not from broadly using unchecked indexing.

## 14. Suggested implementation order

The likely payoff order is:

1. Introduce `PreparedSystem`, `TpCache`, `PhaseState`, and reusable workspaces.
2. Change fugacity/K APIs to `*_into`.
3. Keep K-values in logarithmic form.
4. Flatten every hot matrix.
5. Cache Wilson/NRTL/virial temperature-dependent matrices.
6. Compute both cubic roots from one shared mixture state.
7. Add analytic envelope and flash Jacobians.
8. Add small fixed-capacity LU.
9. Add const-generic kernels for common component counts.
10. Add explicit SIMD only after workspace and layout changes.
11. Write Rayon output directly into final SoA buffers.
12. Consider approximate/vector transcendental functions only behind an opt-in accuracy policy.

The first six items should deliver substantially more speed than substituting Halley for Newton in Rachford–Rice. The scalar RR solve is already cheap; the surrounding thermodynamics currently overwhelms it.