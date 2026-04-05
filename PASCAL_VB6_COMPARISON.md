# Pascal vs VB6 Thermodynamic Program Comparison

## Overview

| Aspect | Pascal Program (`legacy/pascal/`) | VB6 Program (`legacy/vb6/`) |
|--------|-----------------------------------|-------------------------------|
| **Authors** | Francisco Avelino Da Silva, Luis Alberto Baez Linde | Carlos Mendible, Miguel Jackson |
| **Year** | 1989 (Caracas) | 1999 (Sartenejas, Venezuela) |
| **Platform** | Classic Macintosh (Mac Pascal, SANE FPU) | Windows (VB6 COM/DLL) |
| **Architecture** | 6 Pascal Units (TERMOI-TERMOVI) | ~27 class modules + 2 BAS modules |
| **Size** | ~2,500 lines | ~450KB / ~15,000+ lines |

---

## 1. Equations of State (Cubic EOS)

### Common (present in BOTH)

| EOS | Pascal (TERMOII) | VB6 (clsQbicsPure) | Notes |
|-----|-------------------|---------------------|-------|
| **Ideal Gas** | `GasIdeal` | `GasIdeal` | Identical: Z=1 |
| **Van der Waals** (variant) | `VWAdachi` (VdW-Adachi) | `VdW` (classic) + `VdWVald` (Valderrama) | Different VdW variants |
| **RKS (Soave 1972)** | `RKS` | `RKS` | **Identical** alpha: [1 + m(1-sqrt(Tr))]^2, m = 0.480+1.574w-0.176w^2 |
| **Peng-Robinson (1976)** | `PengR` | `PR` | **Identical** alpha: [1 + m(1-sqrt(Tr))]^2, m = 0.37464+1.54226w-0.26992w^2 |
| **Patel-Teja** | `PatelT`, `PatelTUSB` (2 mixing variants) | Not present | Pascal-only |
| **Schmidt-Wenzel** | `SchmidtW` | Not present | Pascal-only |
| **RKS-Polar** | `RKSPolar` (alpha = 1+(1-Tr)(m+n/Tr)) | `RKSmn` (same form) | **Same functional form** |

### VB6-only EOS (NOT in Pascal)

| EOS | Description |
|-----|-------------|
| **Berthelot (1899)** | Alpha = 1/T^0.5 |
| **RK (Redlich-Kwong, 1949)** | Alpha = 1/sqrt(Tr) |
| **RKSGD (1978)** | Soave variant with different m coefficients |
| **RKSL (1997)** | Soave variant with cubic w polynomial |
| **RP (Robinson-Poon, 1978)** | PR variant with cubic w polynomial |
| **PRL (1997)** | PR-Lohrenz with cubic w polynomial |
| **RKSATmn (1995)** | Exponential alpha with m, n, g params |
| **PRATmng (1997)** | PR exponential alpha with m, n, g params |
| **PRMmn (Mund, 1989)** | PR exponential alpha variant |
| **PRSV (Stryjek-Vera, 1986)** | PR with K1 correction parameter |
| **VdWOL, RKOL, PROL (Oracini-Leo, 1998)** | Series expansion alpha with h[0..4] |

### Pascal-only EOS (NOT in VB6)

| EOS | Description |
|-----|-------------|
| **Schmidt-Wenzel** | 3-parameter EOS with acentric factor in cubic |
| **Patel-Teja** (2 variants) | 3-parameter EOS with component-specific Zc |
| **Chao-Seader** | Liquid fugacity correlation (10+ parameter fit for normal, H2, methane) |

### Recommendation
- **Merge**: RKS, PR, RKS-Polar/RKSmn, Ideal Gas -- these are functionally identical
- **Add as separate**: All VB6-only variants (11 additional alpha functions) and Pascal-only variants (Schmidt-Wenzel, Patel-Teja, Chao-Seader) into a unified alpha function enum

---

## 2. Activity Coefficient Models

### Common (present in BOTH) -- ALL IDENTICAL

| Model | Pascal (TERMOIII) | VB6 (clsActivityMulticomp) | Status |
|-------|-------------------|---------------------------|--------|
| **Ideal Solution** | `SolucionIdeal` | `Ideal` | **Identical** (gamma=1) |
| **Scatchard-Hildebrand** | `ScatchardH` | `ScatchardHildebrand` | **Identical** (uses delta, vl) |
| **Margules** | `Margules` | `Margules` | **Identical** (binary-only, Aij) |
| **Van Laar** | `VanLaar` | `VanLaar` | **Identical** (Aij) |
| **Wilson** | `Wilson` | `Wilson` | **Identical** (temp-dependent Aij, vl) |

### Excess property calculations

| Property | Pascal | VB6 | Status |
|----------|--------|-----|--------|
| Excess Gibbs | `Gibbs_Exceso` | `exGibbs` | **Identical**: R*T*sum(x_i*ln(gamma_i)) |
| Excess Enthalpy | `Entalpia_Exceso` | `exEnthalpy` | Pascal: analytical per model; VB6: numerical derivative |
| Excess Entropy | `Entropia_Exceso` | `exEntropy` | Both: (H_ex - G_ex)/T for Wilson; 0 for others |

### Liquid molar volume

| Model | Pascal | VB6 | Status |
|-------|--------|-----|--------|
| Rackett | Not separate (uses vl from component data) | Rackett equation (Model 1) | VB6 has explicit Rackett correlation |
| Thomson/COSTALD | Not present | Model 2 | VB6-only |

### Recommendation
- **Merge completely**: All 5 activity models are identical in both programs. Single implementation serves both.
- **Add from VB6**: Rackett and Thomson liquid molar volume as explicit models (Pascal uses pre-computed vl values)

---

## 3. Mixing Rules

| Mixing Rule | Pascal (TERMOII) | VB6 (clsQbicsMulticomp) | Status |
|-------------|-------------------|-------------------------|--------|
| **Classical (IVDW)** | Yes (default, in `CalcularA_B_CdeMezcla`) | `IVDW` | **Identical**: a_mix = sum(xi*xj*(1-kij)*sqrt(ai*aj)), b_mix = sum(xi*bi) |
| **IIVDW** | No | Yes | VB6-only |
| **Wong-Sandler** | No | `WS` | VB6-only |
| **Huron-Vidal (original)** | No | `HOV` | VB6-only |
| **Huron-Vidal (simplified)** | No | `HVS` | VB6-only |
| **MHV1** | No | `MHV1` | VB6-only |
| **MHV2** | No | `MHV2` | VB6-only |
| **Clasica_I** | No | `Clasica_I` | VB6-only |

### Special Pascal mixing for 3-parameter EOS
Pascal implements special mixing rules for Schmidt-Wenzel and Patel-Teja (mixing of the "C" parameter), which are not in VB6 because VB6 doesn't have those EOS.

### Recommendation
- **Merge**: Classical IVDW mixing rule (identical)
- **Add from VB6**: 7 additional mixing rules as new variants
- **Add from Pascal**: Schmidt-Wenzel and Patel-Teja C-parameter mixing (tied to those EOS)

---

## 4. Saturation Pressure Models

| Model | Pascal (TERMOI) | VB6 (clsSatPressureSolver) | Status |
|-------|-----------------|---------------------------|--------|
| **Antoine** | `Antoine` | Not in VB6 | Pascal-only |
| **Riedel** | `Riedel` | `Riedel` | **Same algorithm** |
| **RPM** | `RPM` | `RPM` | **Same algorithm** |
| **Muller** | `Muller` | `Muller` | **Same algorithm** |
| **DBPsat** (polynomial database) | No | `DBPsat` | VB6-only |
| **Maxwell** (from EOS) | No | `Maxwell` | VB6-only |

### Supporting functions

| Function | Pascal | VB6 | Status |
|----------|--------|-----|--------|
| Boiling temperature | `TEbullicion` (iterative via PseudoAntoine) | `SatTemperature` (Newton-Raphson) | Same goal, different algorithms |
| dPsat/dT | `DPrVapor_DT` (analytical for Antoine, numerical otherwise) | `DSatPressure_T` | Same approach |
| Poynting correction | `CorrectPoynting` | Used inline in VB6 | **Identical formula** |

### Recommendation
- **Merge**: Riedel, RPM, Muller (identical), Poynting correction
- **Add from Pascal**: Antoine correlation
- **Add from VB6**: DBPsat polynomial, Maxwell EOS-based method

---

## 5. Virial Equation of State

| Feature | Pascal | VB6 (clsVirial, clsVirialMulticomp) | Status |
|---------|--------|--------------------------------------|--------|
| Pitzer correlation (B0, B1) | **Not present** | Yes: B0 = 0.083 - 0.422/Tr^1.6, B1 = 0.139 - 0.172/Tr^4.2 | VB6-only |
| Pure component Z, fugacity | **Not present** | Yes | VB6-only |
| Mixture Z, partial fugacity | **Not present** | Yes (Bij mixing with kij) | VB6-only |
| Residual H, S (virial) | **Not present** | Yes (via dB/dT) | VB6-only |

### Recommendation
- **Add entirely from VB6**: The virial equation module is completely absent from Pascal

---

## 6. Flash / Phase Equilibrium Calculations

### Common flash types

| Calculation | Pascal (TERMOIV) | VB6 (clsLVE) | Status |
|-------------|-------------------|---------------|--------|
| **Bubble Point Temperature** | `TemperaturaBR(Burbuja=True)` | `BubblePointTemperature()` | **Same algorithm**: parabolic interpolation on ln(sum(Ki*xi)) |
| **Bubble Point Pressure** | `PresionBR(Burbuja=True)` | `BubblePointPressure()` | **Same algorithm** |
| **Dew Point Temperature** | `TemperaturaBR(Burbuja=False)` | `DewPointTemperature()` | **Same algorithm** |
| **Dew Point Pressure** | `PresionBR(Burbuja=False)` | `DewPointPressure()` | **Same algorithm** |
| **Isothermal Flash** | `FlashIsotermico` | `IsoThermicFlash()` | **Same core**: Rachford-Rice + K-value iteration |
| **Adiabatic Flash** | `FlashAdiabatico` | `AdiabaticFlash()` | **Same core**: nested T-loop with enthalpy balance |
| **Mixture Critical Point** | `ZCriticoMezcla` | Not directly in clsLVE | Pascal estimates Tc,Pc for mixture |

### Key common subroutines

| Subroutine | Pascal | VB6 | Status |
|------------|--------|-----|--------|
| K-value estimation (Raoult) | `Estimados` | Initial K from Psat/P | **Identical** |
| K-value calculation (Ki) | `CalcularKi` | K-value loop in flash | **Same phi-phi and gamma-phi logic** |
| Rachford-Rice (CV) | `CalcularCV` | Newton-Raphson on R-R | **Identical algorithm** |
| Parabolic interpolation | `Parabola`, `NuevoIncremento` | `NuevoIncremento` in VB6 | **Identical** (named same!) |
| Composition normalization | `Normalice` | Inline normalization | **Identical** |
| High-pressure handling | `AltaPresion` procedure | HP algorithm in VB6 | **Same concept** |

### VB6-only flash features

| Feature | Description |
|---------|-------------|
| **MonocompTP** | Pure component flash at T,P |
| **MonocompHV/HL** | Enthalpy-based pure component flash |
| **Heidemann critical point** | Full Newton-Raphson critical point algorithm |

### Recommendation
- **Merge**: All 6 flash calculation types (bubble T/P, dew T/P, isothermal, adiabatic) -- same algorithms
- **Merge**: Rachford-Rice, K-value calculation, parabolic interpolation -- identical code logic
- **Add from VB6**: Heidemann critical point algorithm (more complete than Pascal's ZCriticoMezcla)

---

## 7. Thermodynamic Properties (Enthalpy / Entropy)

### Common

| Property | Pascal | VB6 | Status |
|----------|--------|-----|--------|
| **Ideal gas Cp integration** | `EntalpiaIdeal` (polynomial Cp[0..4]) | Polynomial Cp integration | **Identical**: sum(Cp_k * (T^k - Tref^k)/k) |
| **Ideal gas entropy** | `EntropiaIdeal` (with mixing entropy) | Entropy integration | **Identical**: includes -R*sum(xi*ln(xi)) |
| **Residual enthalpy (cubic EOS)** | `EntalpiaRes` via `Ental_Entro` | `ResidualProperties` | **Same formula**: R*T*(Z-1-I*(a_T+1)) |
| **Residual entropy (cubic EOS)** | `EntropiaRes` via `Ental_Entro` | `ResidualProperties` | **Same formula** |
| **dA/dT (temperature derivative)** | `Aa_T` (analytical per EOS) | `a_T` (numerical 5-point) | Same goal, different method |
| **Reference state** | `AsignarTyP_Ho_SoReferencia` | `clsReferencesSt` | **Same concept** |
| **Condensation enthalpy** | `EntalpiadeCondensacion` (Clausius-Clapeyron) | Via Clausius-Clapeyron | **Same approach** |
| **Liquid residual enthalpy** | `EntalpiaResLiq` (via condensation + excess) | Via flash + departure | **Same thermodynamic path** |

### Recommendation
- **Merge completely**: Ideal Cp integration, departure functions, reference state handling -- all identical

---

## 8. Binary Interaction Parameter Regression (kij / Aij)

### Common

| Feature | Pascal | VB6 | Status |
|---------|--------|-----|--------|
| **kij golden section search** | `KBinarioAureo` (TERMOVI) | `Kij_GoldenSearch` | **Same algorithm**: golden ratio L=0.618034 |
| **Aij regression (activity models)** | `AijBinario` (TERMOV) | Not in VB6 (VB6 regresses kij, not Aij) | Pascal-only in this form |
| **Objective function** | Minimize sum(abs(xi*phi_L - yi*phi_V)/(yi*phi_V)) | `Kij_Qf` | **Same type** of objective |
| **Initial estimate via polynomial** | `HayKInicial` (4th-degree fit + root finding) | `Kij_BracketMin` | Similar concept, different methods |
| **Gaussian elimination** | `Gauss` (TERMOVI, for polynomial fitting) | `MatrizGauss` (McommonFunctions) | **Same algorithm** |

### Pascal-only: Aij regression
Pascal (TERMOV) has a full Newton-Raphson regression for activity model binary parameters (A12, A21) for Margules, Van Laar, and Wilson, including:
- Analytical Jacobian with second derivatives (DGamiDA12, DGamiDA21)
- Experimental gamma calculation from VLE data
- Correlation factor analysis for initial estimates

### Recommendation
- **Merge**: Golden section kij search (identical)
- **Add from Pascal**: Aij regression for activity models (unique capability not in VB6)

---

## 9. Numerical Methods

| Method | Pascal | VB6 | Status |
|--------|--------|-----|--------|
| **Cubic root solver (Cardano)** | `Raices` (TERMOI) | `CubicSolver` | **Same algorithm** (discriminant-based, 3 real roots sorted) |
| **Gaussian elimination** | `Gauss` (TERMOVI, normalized + pivoting) | `MatrizGauss` (pivoting) | **Same algorithm** |
| **Parabolic interpolation** | `Parabola` (TERMOIV) | `NuevoIncremento` | **Identical** |
| **Newton-Raphson** | Used inline (CalcParBinWilson, etc.) | Used inline (various) | **Same approach** |
| **LU decomposition** | Not present | `LUDecomp`, `LUbksb` | VB6-only |
| **NR with numerical Jacobian** | Not present | Full 2n+4 system NR | VB6-only (for flash) |

### Recommendation
- **Merge**: Cubic solver, Gaussian elimination, parabolic interpolation
- **Add from VB6**: LU decomposition, full Newton-Raphson with numerical Jacobian

---

## 10. Data I/O and Experimental Data Handling

| Feature | Pascal (TERMOV) | VB6 | Status |
|---------|-----------------|-----|--------|
| **Isothermal data file reading** | `Leisot` (structured text files) | `Kij_ReadExperimentalData` | Both read T/P/x/y data |
| **Isothermal/isobaric flag** | `Iso` ('T' or 'P') | Same concept | **Same** |
| **Data source tracking** | `FUENTE` (2 strings per isotherm) | Not tracked | Pascal-only |

### Recommendation
- **Merge**: Common data format into JSON-based component/experimental database

---

## Summary: Merge Strategy

### Fully Mergeable (identical algorithms)
1. Cubic EOS: RKS, PR, Ideal Gas
2. All 5 activity models (Ideal, Scatchard-Hildebrand, Margules, Van Laar, Wilson)
3. Classical IVDW mixing rule
4. Saturation pressure: Riedel, RPM, Muller
5. All 6 flash types (bubble/dew T/P, isothermal, adiabatic flash)
6. Rachford-Rice solver, K-value calculation
7. Ideal Cp integration, residual H/S, reference state
8. Golden section kij search
9. Cubic solver (Cardano), Gaussian elimination, parabolic interpolation

### Add from Pascal (unique functionality)
1. **Schmidt-Wenzel EOS** (3-parameter, acentric factor dependent)
2. **Patel-Teja EOS** (2 mixing variants, component-specific Zc)
3. **Chao-Seader** liquid fugacity correlation (with H2/methane special cases)
4. **Antoine** vapor pressure correlation
5. **Aij regression** for activity model parameters (Newton-Raphson with analytical Jacobian)
6. **Mixture critical point estimation** (ZCriticoMezcla)

### Add from VB6 (already in plan, not in Pascal)
1. 11 additional EOS alpha functions (RKSGD, PRL, PRSV, etc.)
2. 7 additional mixing rules (WS, HOV, HVS, MHV1, MHV2, IIVDW, Clasica_I)
3. Virial equation (pure + multicomponent)
4. DBPsat and Maxwell saturation pressure
5. Rackett and Thomson liquid molar volume
6. Heidemann critical point algorithm
7. LU decomposition, full NR with numerical Jacobian
