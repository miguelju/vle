# Appendix A -- Analyst Manual

## Description of Classes and Modules in the Project

> **Translation status**: Complete. See [Spanish original](../../../es/research-paper/markdown/programdocs/Analista.md).

---

## `clsActivityMulticomp`

Class designed to calculate mixture properties using activity-based models (Gamma Models).

### Enums

```vb
Public Enum TADiPGammaModel
    IdealSolutiong = 25
    VanLaarg = 21
    Wilsong = 22
    ScatchardHg = 23
    Margulesg = 24
End Enum
```

This enumeration allows selection of the thermodynamic model to be used within this class.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarActivityProps` | `colclsActivityProps` | Stores the compound properties needed for the class functions. |
| `mvarModel` | `TADiPGammaModel` | Stores the selected thermodynamic model. |
| `X()` | `Double` | Contains the molar compositions of the compounds in the mixture. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `ActivityProps` | Get/Set | Allows setting and retrieving the compound properties for the class. Type: `colclsActivityProps`. |
| `MolarFraction(Index As Integer)` | Get/Let | For compound `Index`, assigns or reads molar compositions to/from the variable `X()`. |
| `Model` | Get/Let | Assigns or reads the thermodynamic model to/from `mvarModel`. Type: `TADiPGammaModel`. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `CalcGamma(vntIndexKey As Integer, ByRef Gamma As Double)` | `Boolean` | Calculates the value of the activity coefficient (gamma). `vntIndexKey` is the compound number; `Gamma` returns the computed value. |
| `exGibbs(T As Double)` | `Double` | Calculates the excess Gibbs energy of a mixture at the temperature defined by parameter `T`. |
| `exEnthalpy(T As Double)` | `Double` | Calculates the excess enthalpy of a mixture at the temperature defined by parameter `T`. |
| `ChangeAij(T As Double)` | `Sub` | Recalculates binary parameters (`Aij`) as temperature (`T`) changes during phase equilibrium calculations. |

### Private Methods

| Method | Description |
|--------|-------------|
| `SumasWilson(vntIndexKey As Integer) As Double` | Performs the summations involved in Wilson model calculations. |
| `Class_Terminate()` | Destroys the contents of `mvarActivityProps`. |

---

## `clsActivityProps`

Class designed to store the property values of a component that are required for applying activity coefficient-based models.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvardelta` | `Double` | Contains the delta value for a compound. |
| `mvarvl` | `Double` | Contains the liquid volume value for a compound. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `vl` | Get/Let | Assigns or retrieves values for the variable `mvarvl` (liquid volume). |
| `delta` | Get/Let | Assigns or retrieves values for the variable `mvardelta` (solubility parameter delta). |

---

## `clsAllProps`

Class that holds instances of all objects whose purpose is to store compound and mixture properties.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarCriticalProps` | `colclsCriticalProps` | Stores a copy of the `colclsCriticalProps` collection. |
| `mvarOtherProps` | `colclsOtherProps` | Stores a copy of the `colclsOtherProps` collection. |
| `mvarLiquid1Model` | `TadipLiquidModels` | Stores the thermodynamic model used for liquid phase 1 calculations. |
| `mvarLiquid2Model` | `TadipLiquidModels` | Stores the thermodynamic model used for liquid phase 2 calculations. |
| `mvarVaporModel` | `TadipVaporModel` | Stores the thermodynamic model used for vapor phase calculations. |
| `mvarCpConsts` | `colclsCpConsts` | Stores a copy of the `colclsCpConsts` collection. |
| `mvarNumberOfComponents` | `Integer` | Stores the number of components in the mixture. |
| `mvarSatPressureModel` | `TADiPSatPressureModel` | Stores the correlation to be used for estimating saturation pressures. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `SatPressureModel` | Get/Let | Assigns or retrieves the saturation pressure model. Type: `TADiPSatPressureModel`. |
| `NumberOfComponents` | Get/Let | Assigns or retrieves the number of components. |
| `OtherProps` | Get/Set | Assigns or retrieves `mvarOtherProps`. Type: `colclsOtherProps`. |
| `CpConsts` | Get/Set | Assigns or retrieves `mvarCpConsts`. Type: `colclsCpConsts`. |
| `Liquid2Model` | Get/Let | Assigns or retrieves `mvarLiquid2Model`. Type: `TadipLiquidModels`. |
| `VaporModel` | Get/Let | Assigns or retrieves `mvarVaporModel`. Type: `TadipVaporModel`. |
| `Liquid1Model` | Get/Let | Assigns or retrieves `mvarLiquid1Model`. Type: `TadipLiquidModels`. |
| `CriticalProps` | Get/Set | Assigns or retrieves `mvarCriticalProps`. Type: `colclsCriticalProps`. |

### Private Methods

| Method | Description |
|--------|-------------|
| `Class_Terminate()` | Destroys the contents of `mvarCpConsts`, `mvarOtherProps`, and `mvarCriticalProps`. |

---

## `clsCpConsts`

Class that contains the coefficients required to calculate the Cp (heat capacity at constant pressure) of a given compound, according to the formula indicated through one of its properties.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarNbFormula` | `Integer` | Stores the formula number corresponding to the correlation used for Cp calculation. |
| `mvarCoef()` | `Double` | Stores the coefficients needed for Cp calculation according to the formula given by `mvarNbFormula`. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Coef(Index As Integer)` | Get/Let | Assigns or retrieves the coefficient values in `mvarCoef()`. |
| `NbFormula` | Get/Let | Assigns or retrieves the formula number in `mvarNbFormula`. |
| `NbCoef` | Get | Returns the number of coefficients contained in `mvarCoef()`. |

---

## `clsCriticalProps`

Class that stores the critical properties of a compound.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarTc` | `Double` | Critical temperature for a given compound in the mixture. |
| `mvarPc` | `Double` | Critical pressure for a given compound in the mixture. |
| `mvarW` | `Double` | Acentric factor for a given compound in the mixture. |
| `mvarZc` | `Double` | Critical compressibility factor for a given compound in the mixture. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Zc` | Get/Let | Assigns or retrieves the critical compressibility factor (`mvarZc`). |
| `w` | Get/Let | Assigns or retrieves the acentric factor (`mvarW`). |
| `Pc` | Get/Let | Assigns or retrieves the critical pressure (`mvarPc`). |
| `Tc` | Get/Let | Assigns or retrieves the critical temperature (`mvarTc`). |

---

## `clsFlow`

Class that represents a process stream, described through thermodynamic properties and mixture compositions.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarMolarFraction()` | `Double` | Stores the global composition of the compounds in the mixture. |
| `mvarTemperature` | `Double` | Temperature of the stream. |
| `mvarPressure` | `Double` | Pressure of the stream. |
| `mvarEnthalpy` | `Double` | Enthalpy of the stream. |
| `mvarEntropy` | `Double` | Entropy of the stream. |
| `mvarVaporFraction` | `Double` | Vapor fraction (beta) of the stream. |
| `mvarLiquid1Fraction` | `Double` | Fraction of liquid phase 1 present in the stream. |
| `mvarLiquid2Fraction` | `Double` | Fraction of liquid phase 2 present in the stream. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Liquid2Fraction` | Get/Let | Assigns or retrieves the liquid phase 2 fraction. |
| `Liquid1Fraction` | Get/Let | Assigns or retrieves the liquid phase 1 fraction. |
| `VaporFraction` | Get/Let | Assigns or retrieves the vapor fraction. |
| `Temperature` | Get/Let | Assigns or retrieves the temperature. |
| `Pressure` | Get/Let | Assigns or retrieves the pressure. |
| `Enthalpy` | Get/Let | Assigns or retrieves the enthalpy. |
| `Entropy` | Get/Let | Assigns or retrieves the entropy. |
| `MolarFraction(Index As Integer)` | Get/Let | Assigns or retrieves the molar fraction for compound `Index`. |

---

## `clsOtherProps`

Class that stores the miscellaneous (unclassified) properties of a compound.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvardelta` | `Double` | Solubility parameter of a substance. |
| `mvarTb` | `Double` | Normal boiling temperature of a compound. |
| `mvarvl` | `Double` | Liquid volume of the compound. |
| `mvarDipMoment` | `Double` | Dipole moment of the compound. |
| `mvarm` | `Double` | Modified cubic EOS parameter. |
| `mvarn` | `Double` | Modified cubic EOS parameter. |
| `mvarK1` | `Double` | Modified cubic EOS parameter. |
| `mvarPsatAtTr` | `Double` | Modified cubic EOS parameter. |
| `mvarg` | `Double` | Modified cubic EOS parameter. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `PsatAtTr` | Get/Let | Assigns or retrieves `mvarPsatAtTr`. |
| `K1` | Get/Let | Assigns or retrieves `mvarK1`. |
| `g` | Get/Let | Assigns or retrieves `mvarg`. |
| `n` | Get/Let | Assigns or retrieves `mvarn`. |
| `m` | Get/Let | Assigns or retrieves `mvarm`. |
| `DipMoment` | Get/Let | Assigns or retrieves the dipole moment (`mvarDipMoment`). |
| `Tb` | Get/Let | Assigns or retrieves the normal boiling temperature (`mvarTb`). |
| `vl` | Get/Let | Assigns or retrieves the liquid volume (`mvarvl`). |
| `delta` | Get/Let | Assigns or retrieves the solubility parameter (`mvardelta`). |

---

## `clsQbicsProps`

Class designed to store the properties of a compound that are required for applying cubic equations of state (CEOS).

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarCriticalTemperature` | `Double` | Critical temperature of a compound. |
| `mvarCriticalPressure` | `Double` | Critical pressure of a compound. |
| `mvarAcentricFactor` | `Double` | Acentric factor of a compound. |
| `mvarZc` | `Double` | Critical compressibility factor. |
| `mvarPRSVK1` | `Double` | PRSV modified cubic EOS parameter. |
| `mvarPsatAtTr` | `Double` | Modified cubic EOS parameter. |
| `mvarm` | `Double` | Modified cubic EOS parameter. |
| `mvarn` | `Double` | Modified cubic EOS parameter. |
| `mvarg` | `Double` | Modified cubic EOS parameter. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `PsatAtTr` | Get/Let | Assigns or retrieves `mvarPsatAtTr`. |
| `PRSVK1` | Get/Let | Assigns or retrieves `mvarPRSVK1`. |
| `m` | Get/Let | Assigns or retrieves `mvarm`. |
| `n` | Get/Let | Assigns or retrieves `mvarn`. |
| `g` | Get/Let | Assigns or retrieves `mvarg`. |
| `Zc` | Get/Let | Assigns or retrieves `mvarZc`. |
| `w` | Get/Let | Assigns or retrieves the acentric factor (`mvarAcentricFactor`). |
| `Pc` | Get/Let | Assigns or retrieves the critical pressure (`mvarCriticalPressure`). |
| `Tc` | Get/Let | Assigns or retrieves the critical temperature (`mvarCriticalTemperature`). |

---

## `clsReferencesSt`

Class that stores the reference state values used in the calculation of energy properties for multicomponent mixtures.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarTo` | `Double` | Reference temperature for all compounds in the mixture. |
| `mvarPo` | `Double` | Reference pressure for all compounds in the mixture. |
| `mvarSo` | `Double` | Reference entropy for all compounds in the mixture. |
| `mvarHo` | `Double` | Reference enthalpy for all compounds in the mixture. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Ho` | Get/Let | Assigns or retrieves the reference enthalpy (`mvarHo`). |
| `TTo` | Get/Let | Assigns or retrieves the reference temperature (`mvarTo`). |
| `So` | Get/Let | Assigns or retrieves the reference entropy (`mvarSo`). |
| `Po` | Get/Let | Assigns or retrieves the reference pressure (`mvarPo`). |

### Initialization

`Class_Initialize()` assigns the following default values (in SI units) to the internal variables:

```
mvarPo = 101325
mvarTTo = 273.15
mvarHo = 0
mvarSo = 0
```

---

## `clsSatPressure`

Class created to calculate saturation pressures and temperatures using correlations from the literature (Riedel, Muller, RPM).

### Enums

```vb
Public Enum TADiPSatPressureModel
    Riedel = 1
    Muller = 2
    RPM = 3
End Enum
```

This enumeration allows selection of the correlation to be used for calculating compound saturation pressures.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarCriticalProps` | `colclsCriticalProps` | Stores a copy of the `colclsCriticalProps` collection. |
| `mvarOtherProps` | `colclsOtherProps` | Stores a copy of the `colclsOtherProps` collection. |
| `mvarSatPressureModel` | `TADiPSatPressureModel` | Indicates which saturation pressure correlation will be used. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `SatPressureModel` | Get/Let | Assigns or retrieves the saturation pressure model. Type: `TADiPSatPressureModel`. |
| `CriticalConstants` | Get/Set | Assigns or retrieves `mvarCriticalProps`. Type: `colclsCriticalProps`. |
| `OtherConstants` | Get/Set | Assigns or retrieves `mvarOtherProps`. Type: `colclsOtherProps`. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `SatPressure(ByVal i As Integer, ByVal Temperature As Double, ByRef SatP As Double)` | `Long` | Calculates the saturation pressure of compound `i` in the mixture at the given `Temperature`, using the correlation stored in `mvarSatPressureModel`. The result is returned in `SatP`. |
| `DSatPressure_T(ByVal i As Integer, ByVal Temperature As Double)` | `Double` | Calculates the derivative of saturation pressure with respect to temperature for compound `i` at the given `Temperature`. |
| `SatTemperature(ByVal i As Integer, ByVal Prsat As Double, ByRef SatT As Double)` | `Long` | Calculates the saturation temperature of compound `i` at the reduced pressure given by `Prsat`. The result is returned in `SatT`. |

### Private Methods

| Method | Description |
|--------|-------------|
| `Regula_Falsi_M(Result, x1, x2, Precision, i)` | Applies the Modified Regula Falsi method to the `SatPressure` function in order to calculate the saturation temperature. |
| `ZZZ_Greater_Abs(x1, x2) As Double` | Auxiliary function that returns whichever of `x1` or `x2` has the greater absolute value. Used by `Regula_Falsi_M`. |
| `Funcion(X, i) As Double` | Auxiliary function used in the saturation pressure calculation within the Modified Regula Falsi numerical method. |

---

## `clsTolerances`

Class that stores the tolerances to be used in iterative calculation procedures.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarTolPressure` | `Double` | Tolerance to be used in pressure calculations. |
| `mvarTolTemperature` | `Double` | Tolerance to be used in temperature calculations. |
| `mvarMaxIter` | `Integer` | Maximum number of iterations allowed for a procedure. |
| `mvarTolMolarFraction` | `Double` | Tolerance accepted in molar fraction calculations. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `TolMolarFraction` | Get/Let | Assigns or retrieves the molar fraction tolerance. |
| `MaxIter` | Get/Let | Assigns or retrieves the maximum iteration count. |
| `TolTemperature` | Get/Let | Assigns or retrieves the temperature tolerance. |
| `TolPressure` | Get/Let | Assigns or retrieves the pressure tolerance. |

### Initialization

`Class_Initialize()` assigns the following default values to the internal variables:

```
mvarTolPressure = 1
mvarTolMolarFraction = 0.1
mvarTolTemperature = 1
mvarMaxIter = 0
```

---

## `clsVirial`

Class designed to calculate pure compound properties using the virial equation truncated at the second term.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarAcentricFactor` | `Double` | Acentric factor of a compound. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `AcentricFactor` | Get/Let | Assigns or retrieves `mvarAcentricFactor`. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `CalculateZ(Tr As Double, Pr As Double)` | `Double` | Calculates the compressibility factor of a compound at the reduced temperature and pressure given by `Tr` and `Pr`. |
| `CalculateFugacity(Tr As Double, Pr As Double)` | `Double` | Calculates the fugacity coefficient of a compound at the reduced temperature and pressure given by `Tr` and `Pr`. |

---

## `clsVirialMulticomp`

Class designed to calculate multicomponent mixture properties using the virial equation truncated at the second term.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarCriticalProperties` | `colclsCriticalProps` | Stores a copy of the `colclsCriticalProps` collection. |
| `mvarMolarFraction()` | `Double` | Stores the molar fractions of the compounds in the mixture. |
| `mvarKijParameters` | `Variant` | Stores the binary interaction parameters between pairs of compounds. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `KijParameters` | Get/Let/Set | Assigns or retrieves the binary interaction parameters (`mvarKijParameters`). |
| `MolarFraction(Index As Integer)` | Get/Let | Assigns or retrieves the molar fraction for compound `Index`. |
| `CriticalProperties` | Get/Set | Assigns or retrieves `mvarCriticalProperties`. Type: `colclsCriticalProps`. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `CalculatePartialFugacity(K As Integer, T As Double, P As Double)` | `Double` | Calculates the partial fugacity coefficient of compound `K` in the mixture at temperature `T` and pressure `P`. |
| `CalculateZ(T As Double, P As Double)` | `Double` | Calculates the compressibility factor of the mixture at temperature `T` and pressure `P`. |
| `HR(P As Double, T As Double)` | `Double` | Calculates the residual enthalpy at pressure `P` and temperature `T`. |
| `SR(P As Double, T As Double)` | `Double` | Calculates the residual entropy at pressure `P` and temperature `T`. |

### Private Methods

| Method | Description |
|--------|-------------|
| `calculateBij(i, j, T) As Double` | Auxiliary function for the virial equation applied to multicomponent mixtures. |
| `Deltaik(i, j, T) As Double` | Auxiliary function for the virial equation applied to multicomponent mixtures. |
| `CalculatedBijdT(i, j, T) As Double` | Auxiliary function for the virial equation applied to multicomponent mixtures. |
| `calculateB(T) As Double` | Auxiliary function for the virial equation applied to multicomponent mixtures. |
| `calculatedBdT(T) As Double` | Auxiliary function for the virial equation applied to multicomponent mixtures. |

---

## `colclsActivityProps`

Collection of compound properties needed for using activity coefficient-based thermodynamic models.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mCol` | `Collection` | Stores a collection of class instances. |
| `mvarAij()` | `Double` | Stores the `Aij` parameters of the Gamma models. |
| `mvarAijTemperature` | `Double` | Stores the temperature at which the `Aij` values were obtained. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Aij(i As Integer, j As Integer)` | Get/Let | Assigns or retrieves the `Aij` parameter values. |
| `AijTemperature` | Get/Let | Assigns or retrieves the temperature at which `Aij` values were obtained. |
| `Item(vntIndexKey As Variant)` | Get | Accesses an element in the collection. Type: `clsActivityProps`. |
| `Count` | Get | Returns the number of elements in the collection. |
| `NewEnum` | Get | Special collection property that enables use of the `For Each` statement. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `Add(delta As Double, vl As Double, Optional sKey As String)` | `clsActivityProps` | Adds an element to the collection. |
| `Remove(vntIndexKey As Variant)` | `Sub` | Removes an element from the collection. |

### Lifecycle

- `Class_Initialize()` -- Creates the `mCol` collection.
- `Class_Terminate()` -- Destroys the `mCol` collection.

---

## `colclsCpConsts`

Collection of the coefficients of all compounds needed for calculating each compound's Cp.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mCol` | `Collection` | Stores a collection of class instances. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Item(vntIndexKey As Variant)` | Get | Accesses an element in the collection. Type: `clsCpConsts`. |
| `Count` | Get | Returns the number of elements in the collection. |
| `NewEnum` | Get | Special collection property that enables use of the `For Each` statement. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `Add(Formula As Integer, Coef() As Double, Optional sKey As String)` | (void) | Adds an element to the collection. |
| `Remove(vntIndexKey As Variant)` | `Sub` | Removes an element from the collection. |

### Lifecycle

- `Class_Initialize()` -- Creates the `mCol` collection.
- `Class_Terminate()` -- Destroys the `mCol` collection.

---

## `colclsCriticalProps`

Collection of the critical properties of all compounds.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mCol` | `Collection` | Stores a collection of class instances. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Item(vntIndexKey As Variant)` | Get | Accesses an element in the collection. Type: `clsCriticalProps`. |
| `Count` | Get | Returns the number of elements in the collection. |
| `NewEnum` | Get | Special collection property that enables use of the `For Each` statement. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `Add(Tc As Double, Pc As Double, w As Double, Zc As Double, Optional sKey As String)` | `clsCriticalProps` | Adds an element to the collection. |
| `Remove(vntIndexKey As Variant)` | `Sub` | Removes an element from the collection. |

### Lifecycle

- `Class_Initialize()` -- Creates the `mCol` collection.
- `Class_Terminate()` -- Destroys the `mCol` collection.

---

## `colclsOtherProps`

Collection of miscellaneous (unclassified) properties of all compounds that are required for certain calculations.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mvarAij(1 To 100, 1 To 100)` | `Double` | Stores the `Aij` parameters of the Gamma models. |
| `mvarAijTemperature` | `Double` | Stores the temperature at which the `Aij` values were obtained. |
| `mCol` | `Collection` | Stores a collection of class instances. |
| `mvarKij` | `Variant` | Stores the binary interaction parameters between compound pairs. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Kij` | Get/Let/Set | Assigns or retrieves the binary interaction parameters (`mvarKij`). |
| `Aij(i As Integer, j As Integer)` | Get/Let | Assigns or retrieves the `Aij` parameter values. |
| `AijTemperature` | Get/Let | Assigns or retrieves the temperature at which `Aij` values were obtained. |
| `Item(vntIndexKey As Variant)` | Get | Accesses an element in the collection. Type: `clsOtherProps`. |
| `Count` | Get | Returns the number of elements in the collection. |
| `NewEnum` | Get | Special collection property that enables use of the `For Each` statement. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `Add(delta, vl, Tb, DipMoment, m, n, g, K1, PsatAtTr, Optional sKey)` | `clsOtherProps` | Adds an element to the collection. All numeric parameters are `Double`; `sKey` is an optional `String`. |
| `Remove(vntIndexKey As Variant)` | `Sub` | Removes an element from the collection. |

### Lifecycle

- `Class_Initialize()` -- Creates the `mCol` collection.
- `Class_Terminate()` -- Destroys the `mCol` collection.

---

## `colclsQbicsProps`

Collection of compound properties needed for using cubic equation of state-based thermodynamic models.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `mCol` | `Collection` | Stores a collection of class instances. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Item(vntIndexKey As Variant)` | Get | Accesses an element in the collection. Type: `clsQbicsProps`. |
| `Count` | Get | Returns the number of elements in the collection. |
| `NewEnum` | Get | Special collection property that enables use of the `For Each` statement. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `Add(Tc, Pc, w, Zc, m, n, g, PsatAtTr, Optional sKey)` | `clsQbicsProps` | Adds an element to the collection. All numeric parameters are `Double`; `sKey` is an optional `String`. |
| `Remove(vntIndexKey As Variant)` | `Sub` | Removes an element from the collection. |

### Lifecycle

- `Class_Initialize()` -- Creates the `mCol` collection.
- `Class_Terminate()` -- Destroys the `mCol` collection.

---

## `clsQbicsMulticomp`

Class designed to apply cubic equations of state (CEOS) to multicomponent mixture problems.

### Enums

```vb
Public Enum TADiPPhiModel
    IdealSolutionp = 25
    IdealGas = -2
    PR1976_Qbicp = 0
    RK1949_Qbicp = 1
    RKS1972_Qbicp = 2
    VdW1870_Qbicp = 3
    PRL1997_Qbicp = 4
    RKSL1997_Qbicp = 5
    RKSGD1978_Qbicp = 6
    RP1978_Qbicp = 7
    Berth1899_Qbicp = 8
    VdWAda1984_Qbicp = 9
    VdWVald1989_Qbicp = 10
    RKSmn1980_Qbicp = 11
    RKSATmn1995_Qbicp = 12
    PRATmng1997_Qbicp = 13
    PRMmn1989_Qbicp = 14
    PRSV1986_Qbicp = 15
    VdWOL1998_Qbicp = 16
    RKOL1998_Qbicp = 17
    PROL1998_Qbicp = 18
End Enum
```

Enumeration that allows selection of the cubic equation of state.

```vb
Public Enum TADiPPhaseID
    PhaseIDvapor = 0
    PhaseIDLiquid = 1
End Enum
```

Enumeration that selects whether the mixture phase is liquid or vapor.

```vb
Public Enum TADiPdim
    WithDim = 0
    DimLess = 1
End Enum
```

Enumeration that selects whether the cubic EOS parameters are calculated in their dimensionless form or not.

### Private Types

```vb
Private Type ParametersEOS
    A As Double
    B As Double
    C As Double
End Type
```

Structure containing the values of the cubic equation of state coefficients.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `gintK1`, `gIntK2`, `gintK3` | `Integer` | Parameters describing the cubic equations. |
| `gdblOmA`, `gdblOmB`, `gdblOmC` | `Double` | Parameters describing the cubic equations. |
| `gdblH(0 To 4)` | `Double` | Coefficients for saturation pressure calculation using TADiP correlations. |
| `mvarModel` | `TADiPPhiModel` | Selected cubic equation of state. |
| `mParameter` | `ParametersEOS` | EOS parameters calculated for the mixture. |
| `mParameteri()` | `ParametersEOS` | EOS parameters calculated for component `i`. |
| `mvarMixingRule` | `Integer` | Selected mixing rule. |
| `mvarPhaseOfMixture` | `TADiPPhaseID` | Phase in which the mixture is found. |
| `mvarMolarFraction()` | `Double` | Molar fractions of components in the phase indicated by `mvarPhaseOfMixture`. |
| `mvarProperties` | `colclsQbicsProps` | Copy of the `colclsQbicsProps` collection. |
| `mvarKij` | `Variant` | Binary interaction parameter values for the mixture components. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `Kij` | Get/Let/Set | Assigns or retrieves binary interaction parameters. |
| `Properties` | Get/Set | Assigns or retrieves `mvarProperties`. Type: `colclsQbicsProps`. |
| `PhaseOfMixture` | Get/Let | Assigns or retrieves the phase identifier. Type: `TADiPPhaseID`. |
| `MixingRule` | Get/Let | Assigns or retrieves the mixing rule. |
| `Model` | Get/Let | Assigns or retrieves the equation of state model. Type: `TADiPPhiModel`. |
| `MolarFraction(Index As Integer)` | Get/Let | Assigns or retrieves the molar fraction for compound `Index`. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `PartialFugacityCoeficientInSolution(Component, Temperature, Pressure, ByRef PartialFugacityCoeficient, Optional Z)` | `Long` | Calculates the partial fugacity coefficient of `Component` in the mixture at the given temperature and pressure. Result returned in `PartialFugacityCoeficient`. |
| `ZSolver(Optional Phase As TADiPPhaseID)` | `Double` | Calculates the compressibility factor of the mixture in the phase described by the optional `Phase` parameter. |
| `PWithDimensions(T As Double, v As Double)` | `Double` | Calculates the mixture pressure based on volume (not compressibility factor), i.e., using the dimensional form of the cubic EOS parameters. |
| `psCriticalProperties(ByRef tpsc, ByRef vpsc, ByRef ppsc)` | `Sub` | Calculates the pseudo-critical properties of the mixture: pseudo-critical temperature (`tpsc`), volume (`vpsc`), and pressure (`ppsc`). |
| `CriticalProperties(ByRef Tc, ByRef Pc, ByRef vc)` | `Boolean` | Calculates the true critical properties of the mixture. |
| `ResidualProperties(T, P, Z, ByRef HR, ByRef SR)` | `Boolean` | Calculates the residual properties (residual enthalpy `HR` and residual entropy `SR`) at conditions `T`, `P`, and `Z`. |
| `Kij_ReadExperimentalData(FilePath, ByRef Data(), ByRef datasets, ByRef Dato, ByRef isoterm)` | `Sub` | Reads a text file to obtain experimental data needed for calculating binary interaction coefficients. |
| `Kij_GoldenSearch(ax, bx, cx, tol, Xmin, Dato, datasets, ByRef Data(), isoterm)` | (variant) | Golden section search for calculating `kij` parameters. |
| `Kij_BracketMin(...)` | `Double` | Bracket minimum search for `kij` parameter calculation. |
| `Kij_Qf(mmkij, ByRef Data(), datasets, Dato, isoterm)` | `Double` | Objective function for `kij` parameter calculation. |

### Private Methods

| Method | Description |
|--------|-------------|
| `MixingRules(T, P, Dimensions)` | Calculates EOS parameters according to the selected mixing rule. |
| `AAi(i, T, P, Dimensions) As Double` | Auxiliary: calculates EOS parameter `A` for component `i`. |
| `BBi(i, T, P, Dimensions) As Double` | Auxiliary: calculates EOS parameter `B` for component `i`. |
| `CalculateJacobianP(T, v, ByRef Jacob())` | Calculates the Jacobian needed for pseudo-critical property computation. |
| `FirstDerivateP(T, v) As Double` | First derivative of pressure with respect to volume. |
| `SecondDerivateP(T, v) As Double` | Second derivative of pressure with respect to volume. |
| `CP_Calculate_T(ByRef T, v, n, ByRef Dn())` | Calculates temperature within the mixture critical properties algorithm. |
| `CP_FillQ(ByRef Q(), T, v, n)` | Auxiliary function for multicomponent critical point calculation. |
| `AAij(i, j) As Double` | Auxiliary function for mixing rule calculations. |
| `CP_D_lnFugacity_nt(i, j, ...)` | First derivative of the fugacity coefficient of component `i` with respect to moles of component `j`. |
| `CP_SD_ln_Fug_nt(i, j, K, T, v, n)` | Second derivative of the fugacity coefficient of component `i` with respect to moles of component `j`. |
| `CP_Calculate_C(T, v, ByRef Dn(), n)` | Calculates `C` in the mixture critical properties algorithm. |
| `EvaluateF(X(), ByRef f())` | Evaluates pressure derivatives with respect to volume in the pseudo-critical properties algorithm. |
| `Xnew(ByRef X(), deltaX(), n)` | Calculates new variable values in Newton-Raphson methods. |
| `a_T(T) As Double` | Auxiliary for mixing rule calculations. |
| `b_T(T) As Double` | Auxiliary for mixing rule calculations. |
| `ai__bi(i, T, a_i, b_i, n)` | Auxiliary for mixing rule calculations. |
| `Class_Terminate()` | Destroys the contents of `mvarProperties`. |

---

## `clsQbicsPure`

Class designed to apply cubic equations of state to pure substance problems.

### Enums

```vb
Public Enum TADiPEDC
    PR1976_Qbic = 0
    RK1949_Qbic = 1
    RKS1972_Qbic = 2
    VdW1870_Qbic = 3
    PRL1997_Qbic = 4
    RKSL1997_Qbic = 5
    RKSGD1978_Qbic = 6
    RP1978_Qbic = 7
    Berth1899_Qbic = 8
    VdWAda1984_Qbic = 9
    VdWVald1989_Qbic = 10
    RKSmn1980_Qbic = 11
    RKSATmn1995_Qbic = 12
    PRATmng1997_Qbic = 13
    PRMmn1989_Qbic = 14
    PRSV1986_Qbic = 15
    VdWOL1998_Qbic = 16
    RKOL1998_Qbic = 17
    PROL1998_Qbic = 18
End Enum
```

Enumeration indicating the available cubic equations of state.

### Private Types

```vb
Private Type ParametersEOS
    A As Double
    B As Double
    C As Double
End Type
```

Structure containing the values of the cubic EOS coefficients.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `gintK1`, `gIntK2`, `gintK3` | `Integer` | Parameters describing the cubic equations. |
| `gdblOmA`, `gdblOmB`, `gdblOmC` | `Double` | Parameters describing the cubic equations. |
| `gdblH(0 To 4)` | `Double` | Coefficients for saturation pressure calculation using TADiP correlations. |
| `mvarCubicCoef1`, `mvarCubicCoef2`, `mvarCubicCoef3` | `Double` | Coefficients of the cubic volume expansion of the CEOS. |
| `mParameter` | `ParametersEOS` | Instance of the `ParametersEOS` structure. |
| `mvarZc` | `Double` | Critical compressibility factor. |
| `Quality` | `Double` | Quality (vapor fraction in pure substance VLE). |
| `mvarPRSVK1` | `Double` | Modified cubic EOS parameter. |
| `mvarPsatAtTr` | `Double` | Modified cubic EOS parameter. |
| `gintRaices_encontradas` | `Boolean` | Indicates whether two real roots were found in the Maxwell criterion calculations. |
| `mvarQbicsWinEQofState` | `TADiPEDC` | Selected cubic equation of state. |
| `mvarAcentricFactor` | `Double` | Acentric factor of the pure substance. |
| `mvarm`, `mvarn`, `mvarg` | `Double` | Modified cubic EOS parameters. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `m` | Get/Let | Assigns or retrieves `mvarm`. |
| `g` | Get/Let | Assigns or retrieves `mvarg`. |
| `n` | Get/Let | Assigns or retrieves `mvarn`. |
| `Model` | Get/Let | Assigns or retrieves the equation of state. Type: `TADiPEDC`. |
| `PRSVK1` | Get/Let | Assigns or retrieves `mvarPRSVK1`. |
| `PsatAtTr` | Get/Let | Assigns or retrieves `mvarPsatAtTr`. |
| `AcentricFactor` | Get/Let | Assigns or retrieves the acentric factor. |
| `Zc` | Get/Let | Assigns or retrieves the critical compressibility factor. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `PrXGivenTrvr(ReducedTemperature, ReducedVolume, ByRef ReducedPressure, ByRef Quality)` | `Boolean` | Calculates reduced pressure and quality given reduced temperature and reduced volume. |
| `FugacityCoeficient(ReducedTemperature, ReducedPressure, CompresibilityFactor, ByRef FugacityCoef)` | `Boolean` | Calculates the fugacity coefficient using reduced temperature, reduced pressure, and compressibility factor. |
| `ResidualEnthalpy(ReducedTemperature, ReducedPressure, CompresibilityFactor, ByRef HR)` | `Boolean` | Calculates the residual enthalpy at the given reduced conditions. |
| `ResidualEntropy(ReducedTemperature, ReducedPressure, CompresibilityFactor, ByRef SR)` | `Boolean` | Calculates the residual entropy at the given reduced conditions. |
| `TrXGivenPrZ(ReducedPressure, CompresibilityFactor, ByRef ReducedTemperature, ByRef Quality)` | `Boolean` | Calculates reduced temperature and quality from reduced pressure and compressibility factor. |
| `TrZGivenPrX(ReducedPressure, Quality, ByRef ReducedTemperature, ByRef CompresibilityFactor)` | `Boolean` | Calculates reduced temperature and compressibility factor from reduced pressure and quality. |
| `TrvrGivenPrX(ReducedPressure, Quality, ByRef ReducedTemperature, ByRef ReducedVolume)` | `Boolean` | Calculates reduced temperature and reduced volume from reduced pressure and quality. |
| `TrPrGivenvrX(ReducedVolume, Quality, ByRef ReducedTemperature, ByRef ReducedPressure)` | `Boolean` | Calculates reduced temperature and reduced pressure from reduced volume and quality. |
| `PrZGivenTrX(ReducedTemperature, Quality, ByRef ReducedPressure, ByRef CompresibilityFactor)` | `Boolean` | Calculates reduced pressure and compressibility factor from reduced temperature and quality. |
| `PrvrGivenTrX(ReducedTemperature, Quality, ByRef ReducedPressure, ByRef ReducedVolume)` | `Boolean` | Calculates reduced pressure and reduced volume from reduced temperature and quality. |
| `MaxwellTestGivenPr(ReducedPressure, ByRef ReducedTemperature, ByRef VaporCompresibilityFactor, ByRef LiquidCompresibilityFactor)` | `Boolean` | Applies the Maxwell criterion using reduced pressure as input to obtain the VLE reduced temperature, saturated vapor and saturated liquid compressibility factors. |
| `MaxwellTestGivenTr(ReducedTemperature, ByRef ReducedPressure, ByRef VaporCompresibilityFactor, ByRef LiquidCompresibilityFactor)` | `Boolean` | Applies the Maxwell criterion using reduced temperature as input to obtain the VLE reduced pressure, saturated vapor and saturated liquid compressibility factors. |
| `MaxwellTestGivenTr_vr(ReducedTemperature, ByRef ReducedPressure, ByRef vrg, ByRef vrf)` | `Boolean` | Applies the Maxwell criterion using reduced temperature to obtain reduced pressure and saturated vapor/liquid reduced volumes. |
| `MaxwellTestGivenPr_Vr(ReducedPressure, ByRef ReducedTemperature, ByRef vrg, ByRef vrf)` | `Boolean` | Applies the Maxwell criterion using reduced pressure to obtain reduced temperature and saturated vapor/liquid reduced volumes. |
| `PrXGivenTrZ(ReducedTemperature, CompresibilityFactor, ByRef ReducedPressure, ByRef Quality)` | `Boolean` | Calculates reduced pressure and quality from reduced temperature and compressibility factor. |
| `TrXGivenPrVr(ReducedPressure, ReducedVolume, ByRef ReducedTemperature, ByRef Quality)` | `Boolean` | Calculates reduced temperature and quality from reduced pressure and reduced volume. |
| `ZGivenTrPr(ReducedTemperature, ReducedPressure, ByRef CompresibilityFactor)` | `Boolean` | Calculates compressibility factor from reduced temperature and reduced pressure. |
| `vrGivenTrPr(ReducedTemperature, ReducedPressure, ByRef ReducedVolume)` | `Boolean` | Calculates reduced volume from reduced temperature and reduced pressure. |
| `TrPrGivenZX(CompresibilityFactor, Quality, ByRef ReducedTemperature, ByRef ReducedPressure)` | `Boolean` | Calculates reduced temperature and pressure from compressibility factor and quality. |

### Friend Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `Prsat(Tr As Double)` | (variant) | Calculates the reduced saturation pressure based on TADiP correlations, using reduced temperature `Tr`. |
| `I_z(Z, K1, K2, B)` | `Double` | Auxiliary function used in fugacity coefficient calculations for both pure substances and mixtures. |
| `Alpha(ReducedTemperature)` | `Double` | Calculates the alpha parameter of the cubic EOS. |
| `D_Alpha(Tr)` | `Double` | Calculates the temperature derivative of the alpha parameter. |

### Private Methods

| Method | Description |
|--------|-------------|
| `Prvr(Tr, vr, Alphac) As Double` | Calculates the reduced pressure. |
| `Maxwell_RM(alphat, vg, vf) As Double` | Auxiliary function for the Maxwell criterion using reduced volume. |
| `Regula_Falsi_M(Result, x1, x2, Precision, i)` | Modified Regula Falsi numerical method. |
| `ZZZ_Greater_Abs(x1, x2) As Double` | Returns whichever of `x1`, `x2` has the greater absolute value. |
| `Funcion(X, i) As Double` | Function to be solved by the Modified Regula Falsi. |
| `T_despejada(ReducedPressure, CompresibilityFactor, initTemperature)` | Returns reduced temperature given reduced pressure and compressibility factor. |
| `Tr_despejada(ReducedPressure, ReducedVolume, IniT)` | Returns reduced temperature given reduced pressure and reduced volume. |
| `F_de_PyT(ReducedTemperature, CompresibilityFactor, Pressure) As Double` | Evaluates the cubic expansion of the CEOS in compressibility factor. |
| `F_de_PryTrVr(Tr, ReducedVolume, Pr) As Double` | Evaluates the cubic expansion of the CEOS in reduced volume. |
| `P_despejada(ReducedTemperature, CompresibilityFactor, initPressure) As Double` | Returns pressure given temperature and compressibility factor. |
| `Fugacity(Z) As Double` | Calculates fugacity using the compressibility factor. |
| `Coef_Cubicas(Pr, Tr, alfac)` | Calculates the cubic expansion coefficients of the CEOS in terms of compressibility factor. |
| `Coef_Cubicasvr(P0, alphat)` | Calculates the cubic expansion coefficients of the CEOS in terms of reduced volume. |

---

## `clsLVE`

Class designed to solve common vapor-liquid equilibrium (VLE) problems for multicomponent mixtures, using both cubic equation of state-based models and activity coefficient-based models.

### Enums

```vb
Public Enum TadipLiquidModels
    IdealSolution = 25
    PR1976_Qbicl = 0
    RK1949_Qbicl = 1
    RKS1972_Qbicl = 2
    VdW1870_Qbicl = 3
    PRL1997_Qbicl = 4
    RKSL1997_Qbicl = 5
    RKSGD1978_Qbicl = 6
    RP1978_Qbicl = 7
    Berth1899_Qbicl = 8
    VdWAda1984_Qbicl = 9
    VdWVald1989_Qbicl = 10
    RKSmn1980_Qbicl = 11
    RKSATmn1995_Qbicl = 12
    PRATmng1997_Qbicl = 13
    PRMmn1989_Qbicl = 14
    PRSV1986_Qbicl = 15
    VdWOL1998_Qbicl = 16
    RKOL1998_Qbicl = 17
    PROL1998_Qbicl = 18
    vanLaar = 21
    Wilson = 22
    ScatchardHild = 23
    Margules = 24
End Enum
```

Enumeration of all models applicable to the liquid phase of a mixture.

```vb
Public Enum TadipVaporModel
    IdealGasv = -2
    PR1976_Qbicv = 0
    RK1949_Qbicv = 1
    RKS1972_Qbicv = 2
    VdW1870_Qbicv = 3
    PRL1997_Qbicv = 4
    RKSL1997_Qbicv = 5
    RKSGD1978_Qbicv = 6
    RP1978_Qbicv = 7
    Berth1899_Qbicv = 8
    VdWAda1984_Qbicv = 9
    VdWVald1989_Qbicv = 10
    RKSmn1980_Qbicv = 11
    RKSATmn1995_Qbicv = 12
    PRATmng1997_Qbicv = 13
    PRMmn1989_Qbicv = 14
    PRSV1986_Qbicv = 15
    VdWOL1998_Qbicv = 16
    RKOL1998_Qbicv = 17
    PROL1998_Qbicv = 18
    virial = -1
End Enum
```

Enumeration of all models applicable to the vapor phase of a mixture.

### Private Enums and Types

```vb
Private Enum TypeCalculation
    DewP = 1
    DewT = 2
    BubP = 3
    BubT = 4
    Flash = 5
End Enum
```

Enumeration of the possible algorithms related to mixture VLE.

```vb
Private Type DatosFlash
    T As Double
    P As Double
    Nl As Double
    Nv As Double
    Nt As Double
    Alpha As Double
    Beta As Double
    Alphac As Double
    Betac As Double
End Type
```

Structure containing data used in multiple VLE calculations.

### Private Variables

| Variable | Type | Description |
|----------|------|-------------|
| `DBO` | `clsDataBase` | Instance of the TADiP group database class. |
| `mvarLiquidFlow` | `clsFlow` | Properties of the liquid phase of the mixture. |
| `mvarVaporFlow` | `clsFlow` | Properties of the vapor phase of the mixture. |
| `mvarFeedFlow` | `clsFlow` | Properties of the feed stream to which the calculation algorithms will be applied. |
| `LiquidSolver` | `Object` | Object providing the solution pathway for the liquid phase. |
| `VaporSolver` | `Object` | Object providing the solution pathway for the vapor phase. |
| `SatPressureCalc` | `clsSatPressure` | Instance needed for calculating pure compound saturation pressures. |
| `mvarMixture` | `clsAllProps` | Instance holding all substance data and thermodynamic models for VLE resolution. |
| `mvarBoolgammaphi` | `Boolean` | Indicates whether the thermodynamic model combination corresponds to the gamma-phi approach. |
| `mvarTolerances` | `clsTolerances` | Tolerances for the calculation algorithms. |
| `mvarReferenceState` | `clsReferencesSt` | Reference state values for energy property calculations. |

### Public Properties

| Property | Access | Description |
|----------|--------|-------------|
| `ReferenceState` | Get/Set | Assigns or retrieves the reference state. Type: `clsReferencesSt`. |
| `Tolerances` | Get/Set | Assigns or retrieves the tolerances. Type: `clsTolerances`. |
| `Mixture` | Get/Set | Assigns or retrieves the mixture properties. Type: `clsAllProps`. |
| `FeedFlow` | Get/Set | Assigns or retrieves the feed stream. Type: `clsFlow`. |
| `LiquidFlow` | Get/Set | Assigns or retrieves the liquid stream. Type: `clsFlow`. |
| `VaporFlow` | Get/Set | Assigns or retrieves the vapor stream. Type: `clsFlow`. |

### Public Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `BubblePointTemperature()` | (variant) | Calculates the bubble point temperature of a multicomponent mixture from the pressure and global composition data in `mvarFeedFlow`. |
| `DewPointTemperature()` | (variant) | Calculates the dew point temperature of a multicomponent mixture from the pressure and global composition data in `mvarFeedFlow`. |
| `DewPointPressure()` | (variant) | Calculates the dew point pressure of a multicomponent mixture from the temperature and global composition data in `mvarFeedFlow`. |
| `BubblePointPressure()` | `Boolean` | Calculates the bubble point pressure of a multicomponent mixture from the temperature and global composition data in `mvarFeedFlow`. |
| `AdiabaticFlash(Heat As Double, IntPressure As Double)` | `Integer` | Calculates the adiabatic flash with internal pressure equal to `IntPressure` and heat equal to `Heat`. |
| `IsoThermicFlash(ByRef Betac As Double)` | `Sub` | Calculates the isothermal flash. |

### Private Methods

| Method | Description |
|--------|-------------|
| `NR_JacobianMatrix(n, ByRef Jac(), Datos, ByRef X(), ByRef Y(), ByRef Ni())` | Auxiliary for Newton-Raphson algorithms where system pressure is unknown. |
| `NR_JacobianMatrixBDT(n, ByRef Jac(), Datos, ByRef X(), ByRef Y(), ByRef Ni())` | Auxiliary for Newton-Raphson algorithms where system temperature is unknown. |
| `CalculatePoynting(i, T, P)` | Calculates the Poynting correction factor. |
| `CalculatePhiSati(i, T) As Double` | Calculates the pure component fugacity coefficient at saturation conditions via `clsQbicsPure`. |
| `Class_Terminate()` | Destroys `mvarLiquidFlow`, `mvarVaporFlow`, `mvarFeedFlow`, `mvarMixture`, and `DBO`. |
| `BubblePointTemperatureHP(n)` | Auxiliary for `BubblePointTemperature`: numerically traverses the T-P envelope. |
| `NR_DownloadX(ByRef Xanswer(), ByRef Datos, ByRef X(), ByRef Y(), n)` | Transfers contents of `Xanswer` array to the `Datos` structure. |
| `NR_UploadX(ByRef Xanswer(), ByRef Datos, ByRef X(), ByRef Y(), n)` | Transfers contents of the `Datos` structure to the `Xanswer` array. |
| `NR_Xnew(ByRef X(), deltaX())` | Calculates new variable values iteration by iteration in Newton-Raphson. |
| `NR_EvaluateFi(ByRef f(), ByRef X(), ByRef Y(), T, P, n)` | Evaluates the objective function for all components in Newton-Raphson. |
| `PE_Hgf(n, T, P) As Double` | Calculates energy properties using the Clapeyron equation. |
| `NR_UploadLiquidMolarFraction(ByRef X(), n)` | Transfers `X()` to `mvarLiquidSolver.MolarFraction()`. |
| `NR_UploadVaporMolarFraction(ByRef Y(), n)` | Transfers `Y()` to `mvarVaporSolver.MolarFraction()`. |
| `NR_Fi(ByRef X(), ByRef Y(), T, P, i) As Double` | Evaluates the objective function for component `i`. |
| `NR_DewPointPressure(firstP, firstX(), n)` | Dew point pressure via multivariable Newton-Raphson, given temperature and global composition. |
| `DimSolverObjects(n)` | Specifies the class type for `LiquidSolver` and `VaporSolver` and transfers compound properties. |
| `ClearSolverObjects()` | Destroys the contents of `LiquidSolver` and `VaporSolver`. |
| `NR_BubblePointPressure(firstP, firstY(), n)` | Bubble point pressure via multivariable Newton-Raphson, given temperature and global composition. |
| `NR_BubblePointTemperature(firstT, firstY(), n)` | Bubble point temperature via multivariable Newton-Raphson, given pressure and global composition. |
| `NR_DewPointTemperature(firstT, firstX(), n)` | Dew point temperature via multivariable Newton-Raphson, given pressure and global composition. |
| `FillSatPressureSolver(n)` | Transfers mixture data needed by `SatPressureCalc`. |
| `CalculateKi(n, T, P, ByRef Ki())` | Calculates equilibrium ratios `Ki` in VLE algorithms. |
| `FranctionsEquals(ByRef x1(), ByRef x2(), ByRef y1(), ByRef y2()) As Boolean` | Determines whether compositions in consecutive iterations are equal within tolerance. |
| `Flash_CalculateXi(Beta, ByRef Ki(), ByRef X(), n) As Boolean` | Calculates liquid phase molar compositions in flash calculations. |
| `Flash_CalculateYi(ByRef X(), ByRef Y(), ByRef Ki(), n) As Boolean` | Calculates vapor phase molar compositions in flash calculations. |
| `Flash_RachfordRice_NR(initialguess, ByRef answer, ByRef Ki(), n) As Boolean` | Solves the Rachford-Rice equation via Newton-Raphson. |
| `Flash_RachfordRice(Beta, ByRef Ki(), n)` | Evaluates the Rachford-Rice equation. |
| `Flash_DerivateRachfordRice(Beta, ByRef Ki(), n)` | Evaluates the derivative of the Rachford-Rice equation with respect to vapor fraction. |
| `RaoultFirstAprox(TypeOfCalc, n)` | Calculates VLE via Raoult's law as a first approximation for subsequent algorithms. |
| `TempIni(TypeOfCalc, n) As Double` | Calculates the initial temperature estimate for VLE algorithms. |
| `FillVaporFraction(vData, Optional Val)` | Transmits the vapor fraction value to each `clsFlow` instance according to the algorithm. |
| `BubblePointPressureHP(n)` | Auxiliary for `BubblePointPressure`: numerically traverses the T-P envelope. |
| `DewpointPressureHP(n)` | Auxiliary for `DewPointPressure`: numerically traverses the T-P envelope. |
| `DewPointTemperatureHP(n)` | Auxiliary for `DewPointTemperature`: numerically traverses the T-P envelope. |
| `Ekilib_BubblePointPressure(n, firstP, firstY())` | Bubble point pressure via the Ekilib method, given temperature and global composition. |
| `Ekilib_LazoK_Y(ByRef sumaY, n, T, P)` | Inner loop for calculating vapor molar compositions when unknown. |
| `Enthalpy_Entropy(n, T, P, Beta, ByRef h, ByRef s)` | Calculates enthalpy and entropy of a mixture in VLE at the given `T`, `P`, and vaporized fraction `Beta`. |
| `Ekilib_DewPointPressure(n, firstP, firstX())` | Dew point pressure via the Ekilib method, given temperature and global composition. |
| `IdealEnthalpy(n, T, Tref, Ho, ByRef lH, ByRef vH)` | Calculates the ideal gas enthalpy change from reference temperature `Tref` to system temperature `T`. |
| `IdealEntropy(n, T, Tref, P, Pref, So, ByRef lS, ByRef vS)` | Calculates the ideal gas entropy change from reference conditions (`Tref`, `Pref`) to system conditions (`T`, `P`). |
| `Ekilib_BubblePointTemperature(n, firstT, firstY())` | Bubble point temperature via the Ekilib method, given pressure and global composition. |
| `Ekilib_DewPointTemperature(n, firstT, firstX())` | Dew point temperature via the Ekilib method, given pressure and global composition. |

---

## `McommonFunctions`

Module containing a variety of functions and numerical methods used by several of the classes that comprise the library.

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `UnivGasConst` | `8.31451` | Universal gas constant. |

### Declarations

| Declaration | Description |
|-------------|-------------|
| `CopyMemory Lib "kernel32" Alias "RtlMoveMemory"` | Windows API function that allows copying one array to another without a `For...Next` loop. |

### Module-Level Variables

| Variable | Type | Description |
|----------|------|-------------|
| `Filter` | `clsJMLFilter` | Instance of `clsJMLFilter`, used to filter numeric input in the program. |

### Public Functions and Subroutines

| Function/Sub | Return Type | Description |
|--------------|-------------|-------------|
| `SWAPN(ByRef A1 As Variant, ByRef A2 As Variant)` | (variant) | Swaps the value of `A1` with `A2`. |
| `DeterDiag(n, ByRef A(), ByRef P(), d)` | `Double` | Calculates the determinant of a matrix. |
| `SumFrac(ByRef X(), ByRef SUM)` | (variant) | Calculates the summation of molar fractions `X()`. |
| `Compara(X(), Y(), E) As Boolean` | `Boolean` | Compares the values of `X()` and `Y()` to rule out a trivial solution. |
| `IncrementarYRodar(ByRef x1, ByRef x2, ByRef x3, Incremento, ByRef f1, ByRef f2, ByRef f3)` | `Sub` | Assigns `x2` to `x1`, `x3` to `x2`, then increments `x3`. |
| `Parabola(x1, x2, x3, fx1, fx2, fx3, ByRef Incremento, Lineal)` | `Sub` | Predicts a new value in iterative algorithms using a parabolic approximation. |
| `CopySign(A, B)` | `Double` | Changes the sign of `A` to match the sign of `B`. |
| `NuevoIncremento(x1, x2, x3, f1, f2, f3, Iteracion)` | `Double` | Calculates new increments in various iterative algorithms. |
| `Max(x1, x2)` | `Double` | Returns the larger of `x1` and `x2`. |
| `GeneralConstantsEOS(intEOS, ByRef K1, ByRef K2, ByRef K3, ByRef OmA, ByRef OmB, ByRef OmC, ByRef h())` | `Sub` | Given a cubic EOS (`intEOS`), returns the parameter values needed for calculations using the Abbott equation. |
| `CubicSolver(A0, A1, A2, A3, ByRef y1, ByRef y2, ByRef y3)` | `Integer` | Solves a cubic equation of the form: `A0 + A1*x + A2*x^2 + A3*x^3`. |
| `CopyX(ByRef X(), ByRef XC())` | `Sub` | Copies the contents of `X()` to `XC()` using `CopyMemory`. |
| `LUbksb(ByRef A(), ByRef n, NP, ByRef INDX(), ByRef B())` | (variant) | LU back-substitution for solving linear systems. |
| `LUDecomp(n, NP, ByRef A(), ByRef INDX(), ByRef d)` | `Boolean` | LU decomposition of a matrix. |
| `SolveAx(n, A(), B(), ByRef X())` | `Boolean` | Solves the system of equations `A() * X() = B()`. |
| `CheckFunConv(ByRef f(), E)` | `Boolean` | Verifies convergence of algorithms in the Newton-Raphson numerical method (function residual check). |
| `CheckRootConv(ByRef X(), E)` | `Boolean` | Verifies convergence of algorithms in the Newton-Raphson numerical method (root convergence check). |
| `FirstDerivate(fo, f1, h)` | `Double` | Calculates the first numerical derivative given `f(x)` (`fo`), `f(x+h)` (`f1`), and step `h`. |
| `SecondDerivate(fo, f1, f_1, h)` | `Double` | Calculates the second numerical derivative given `f(x)` (`fo`), `f(x+h)` (`f1`), `f(x-h)` (`f_1`), and step `h`. |
| `CubicSolverNR(A, B, C, ByRef x1, ByRef x2, ByRef x3)` | `Integer` | Solves a cubic equation of the form: `A0 + A1*x + A2*x^2 + A3*x^3`. |

### Private Functions

| Function | Description |
|----------|-------------|
| `SWAP(i, j)` | Swaps the value of `i` with `j`. |
| `Arccos(alfa) As Double` | Calculates the arccosine of `alfa`. |
| `XALAY(X, Y) As Double` | Computes `X` raised to the power `Y`. |

---

[Back to Table of Contents](../README.md)
