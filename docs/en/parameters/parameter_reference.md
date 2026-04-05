# Parameter Reference

Comprehensive reference for all parameters required by the VLE thermodynamic calculator, organized by category. Cross-references legacy source files and recommended data sources.

---

## Critical Properties

Required by all cubic EOS and most correlations.

| Parameter | Symbol | Units | Description | Sources |
|-----------|--------|-------|-------------|---------|
| Critical temperature | Tc | K | Temperature at the critical point | Reid, Prausnitz & Poling (4th ed); DIPPR 801; NIST WebBook |
| Critical pressure | Pc | kPa | Pressure at the critical point | Reid, Prausnitz & Poling; DIPPR 801; NIST WebBook |
| Acentric factor | w (omega) | — | Pitzer acentric factor | Reid, Prausnitz & Poling; DIPPR 801 |
| Critical compressibility | Zc | — | Z at the critical point | Reid, Prausnitz & Poling; DIPPR 801 |
| Critical volume | Vc | cm³/mol | Molar volume at the critical point | Reid, Prausnitz & Poling; DIPPR 801 |

**Legacy sources**: `clsCriticalProps` (VB6), component data arrays (Pascal)

---

## EOS Family Constants

Parameters that define each cubic equation of state in Abbott's (*5*)(../research-paper/references.md#ref-5) general form.

| Parameter | Description | Typical Values | Source |
|-----------|-------------|----------------|--------|
| k₁ | First cubic parameter | 0 (VdW), 1 (RKS), 2 (PR) | Abbott (*5*)(../research-paper/references.md#ref-5) |
| k₂ | Second cubic parameter | 0 (VdW, RKS), -1 (PR) | Abbott (*5*)(../research-paper/references.md#ref-5) |
| k₃ | Third cubic parameter | 1 (all standard EOS) | Abbott (*5*)(../research-paper/references.md#ref-5) |
| Ωa | a-parameter constant | 0.42748 (RKS), 0.45724 (PR) | Original EOS papers |
| Ωb | b-parameter constant | 0.08664 (RKS), 0.07780 (PR) | Original EOS papers |
| h[0..4] | Oracini-Leo series coefficients | EOS-specific | VB6 `McommonFunctions.bas` |

**Legacy sources**: `McommonFunctions.bas:273` (VB6), `TERMOII.PAS` (Pascal)

### Schmidt-Wenzel Parameters (4)

| Parameter | Description | Formula |
|-----------|-------------|---------|
| Beta | Acentric-factor-dependent covolume | 0.25989 - 0.02143w + 0.00337w² |
| m | Alpha function parameter | Complex function of w |

### Patel-Teja Parameters (4)

| Parameter | Description | Formula |
|-----------|-------------|---------|
| Ωb | b-parameter constant | Component-specific via Zc |
| ZZc | Critical Z correlation | 0.32903 - 0.07680w + 0.02119w² |
| Num2i | Second parameter | 0.08517 - 0.02641w + 0.00789w² |

### Chao-Seader Coefficients (4)

| Parameter | Description | Count |
|-----------|-------------|-------|
| ANor[0..9] | Normal compound coefficients | 10 |
| Aele[10..14] | Electrolyte coefficients | 5 |
| AH2[0..9] | Hydrogen-specific coefficients | 10 |
| AMe[0..9] | Methane-specific coefficients | 10 |

**Legacy source**: Hardcoded in Pascal `TERMOII.PAS:646`

---

## Alpha Function Parameters

Temperature-dependent correction to the a-parameter of cubic EOS.

| EOS Variant | Parameters | Formula Type |
|-------------|-----------|--------------|
| RKS (1972) | m = f(w) | [1 + m(1 - √Tr)]² |
| PR (1976) | m = f(w) | [1 + m(1 - √Tr)]² |
| PRSV (1986) | m, K1 | [1 + m(1 - √Tr) + K1(1 + √Tr)(0.7 - Tr)]² |
| RKSmn, PRATmng | m, n, g | Exponential: exp[m(1-Tr) + n(1-√Tr)² + g(1-Tr)⁴] |
| Oracini-Leo | h[0..4] | Series expansion with 5 coefficients |
| RKS-Polar | m, n | 1 + (1 - Tr)(m + n/Tr) |

**Data sources**: Stryjek & Vera (1986) for PRSV K1; Mathias & Copeman for MC params; original EOS papers

---

## Binary Interaction Parameters (kij)

| Parameter | Symbol | Description | Sources |
|-----------|--------|-------------|---------|
| Binary interaction | kij | Symmetric matrix (kij = kji, kii = 0) | Experimental regression; Knapp et al. (1982); DECHEMA |

**Legacy sources**: `clsQbicsMulticomp.cls` (VB6), `Kij` array (Pascal)

---

## Activity Model Parameters

| Model | Parameters | Description | Sources |
|-------|-----------|-------------|---------|
| Wilson | Λij | Binary interaction energy parameters (asymmetric) | DECHEMA VLE Data Collection; Gmehling & Onken |
| van Laar | A12, A21 | Binary Margules-type parameters | DECHEMA; experimental regression |
| Margules | A12, A21 | Symmetric or asymmetric binary parameters | DECHEMA; experimental regression |
| Scatchard-Hildebrand | δi, Vli | Solubility parameters and liquid molar volumes | Hildebrand & Scott; CRC Handbook |
| Ideal | — | No parameters (γ = 1) | — |

**Legacy sources**: `clsActivityMulticomp.cls` (VB6), `TERMOIII.PAS` (Pascal)

---

## Heat Capacity (Cp) Polynomials

Ideal gas heat capacity as polynomial in temperature:

Cp⁰ = A + B·T + C·T² + D·T³ (+ E·T⁴ for some sources)

| Parameter | Units | Description | Sources |
|-----------|-------|-------------|---------|
| A, B, C, D | Various | Polynomial coefficients | Reid, Prausnitz & Poling; DIPPR; Yaws |

**Note**: Temperature in K; Cp in kJ/(kmol·K). Coefficients vary by source — ensure consistent units.

---

## Liquid Molar Volume

| Model | Parameters | Description | Source |
|-------|-----------|-------------|--------|
| Rackett | ZRA | Modified Rackett parameter | Spencer & Danner (1972) |
| Thomson/COSTALD | Vs, wSRK | Characteristic volume and SRK acentric factor | Hankinson & Thomson (*18*)(../research-paper/references.md#ref-18) |

**Legacy sources**: `clsActivityMulticomp.cls` (VB6 — both models), component `vl` field (Pascal)

---

## Saturation Pressure

| Model | Parameters | Description | Source |
|-------|-----------|-------------|--------|
| Antoine | a1, a2, a3 | ln(P/Pc) = a1 - a2/(a3 + T) | NIST WebBook; DIPPR |
| Riedel | Tc, Pc, Tb | Correlation using critical + boiling point | Reid, Prausnitz & Poling |
| RPM | Tc, Pc, w | Reduced pressure model | Reid, Prausnitz & Poling |
| Muller | Tc, Pc, w, Tb | Acentric factor + boiling point | Muller correlation |
| Database polynomial | Coefficients | Multi-term polynomial fit | VB6-only (`DBPsat`) |

**Legacy sources**: `clsSatPressureSolver.cls` (VB6), `TERMOI.PAS` (Pascal)

---

## Other Properties

| Parameter | Symbol | Units | Description | Source |
|-----------|--------|-------|-------------|--------|
| Normal boiling point | Tb | K | Boiling temperature at 1 atm | CRC Handbook; DIPPR |
| Dipole moment | μ | Debye | Molecular dipole moment (for RKS-Polar) | McClellan (1963) |
| Liquid molar volume at Tb | Vl | cm³/mol | For Scatchard-Hildebrand model | CRC Handbook |
| Solubility parameter | δ | (cal/cm³)^0.5 | For Scatchard-Hildebrand model | Hildebrand & Scott |

---

## Universal Constants

| Constant | Value | Units | Used In |
|----------|-------|-------|---------|
| R (gas constant) | 8.31451 | kJ/(kmol·K) | All EOS, thermo properties |
| R (for activity) | 0.23898 | cal/(mol·K) | Scatchard-Hildebrand (Pascal) |
| Golden ratio | 0.618034 | — | kij optimization (legacy) |

---

[Back to Research Paper](../research-paper/README.md)
