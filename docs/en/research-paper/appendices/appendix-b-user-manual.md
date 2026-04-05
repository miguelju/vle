# Appendix B — User Manual for the Thermodynamic Library

> **Translation status**: Complete. See [Spanish original](../../../es/research-paper/programdocs/dllManual.md).

---

## B.1 Installation of the Library

### B.1.1 Installation in Microsoft VBA

1. Open Microsoft VBA from any Microsoft Office program that has access to it, such as Microsoft Excel.
2. In the main menu, select the **Tools** menu and then the **References** menu.
3. On the screen, select the **Browse** button, find the directory where the `thrmStateSolver.dll` file is located, and select it.
4. Finally, press the **OK** button and the library will be installed.

### B.1.2 Installation in Microsoft VB

1. Once Microsoft VB is open, choose from the main menu, select the **Project** menu and then the **References** menu.
2. On the screen, choose the **Browse** button, and find the directory where the `thrmStateSolver.dll` file is located, and select it.
3. Finally, press the **OK** button and the library will be installed.

---

## B.2 Using the Classes Available Through the Library

### B.2.1 `clsSatPressure`

This class allows the calculation of saturation pressures and temperatures through correlations found in the literature (Riedel, Muller, RPM).

As a first step, an instance of the class must be declared and then filled with all the properties necessary for calculating the saturation pressure of a compound using the available correlations:

```vb
Dim Sat As New clsSatPressure

Sat.CriticalConstants.Add Tc, Pc, w, Zc
Sat.OtherConstants.Add 0, 0, Tb, 0, 0, 0, 0, 0, 0
```

Then select the model (Riedel, Muller, or RPM).

There are three methods that can be applied:

- **Calculate saturation pressure** of component *i* at temperature *T*:
  ```vb
  Sat.SatPressure i, T, Psat
  ```
  where `T` is the system temperature, and `Psat` is the variable where the result will be stored.

- **Calculate saturation temperature** of component *i* at pressure *P*:
  ```vb
  Sat.SatTemperature i, P, Tsat
  ```
  where `P` is the system pressure, and `Tsat` is the variable where the result will be stored.

- **Calculate the derivative of saturation pressure** of component *i* at temperature *T*:
  ```vb
  dP = Sat.DSatPressure_T(i, T)
  ```
  where `dP` is the variable where the result will be stored.

### B.2.2 `clsVirial`

This class allows the calculation of compressibility factors, fugacity coefficients, and residual energy properties through the second virial equation truncated at the second term.

As a first step, declare an instance of the class and then set the acentric factor of the substance:

```vb
Dim Vir As New clsVirial

Vir.AcentricFactor = w
```

There are three methods that can be applied to this class:

- **Calculate the compressibility factor**:
  ```vb
  Z = Vir.CalculateZ(Tr, Pr)
  ```

- **Calculate the fugacity coefficient**:
  ```vb
  fi = Vir.CalculateFugacity(Tr, Pr)
  ```

- **Calculate residual properties** (enthalpy and entropy):
  ```vb
  Vir.HR_SR Tr, Pr, Hr, Sr
  ```
  Takes reduced pressure and temperature as input, and stores the residual property values in the provided variables.

### B.2.3 `clsVirialMulticomp`

This class is used for multicomponent mixtures. The general application scheme is similar to the previous classes — define the necessary parameters and then calculate the desired functions. Below are representative code lines for a two-component case:

```vb
Dim VirMC As New clsVirialMulticomp

' Set parameters
VirMC.CriticalProperties.Add Tc1, Pc1, w1, 0
VirMC.CriticalProperties.Add Tc2, Pc2, w2, 0

VirMC.MolarFraction(1) = y1
VirMC.MolarFraction(2) = y2

Set VirMC.KijParameters = a(i, j)

' Desired calculations
fi = VirMC.CalculatePartialFugacity(i, T, P)
Z  = VirMC.CalculateZ(T, P)
HR = VirMC.HR(P, T)
SR = VirMC.SR(P, T)
```

### B.2.4 `clsQbicsPure`

This class, as its name indicates, is for calculating residual properties (H^R, S^R), compressibility factors, and vapor-liquid equilibrium of pure substances. This class has a large number of methods due to the variety of possible data combinations. From the specification of two independent variables (P, T or v), the state of the substance can be determined. There is also the possibility of using two different dimensionless forms for the CEOS used — functions containing v_r and others for Z.

Below are representative code lines for applying the Maxwell criterion:

```vb
Dim Qbic As New clsQbicsPure
Dim pr1 As Double, zf As Double, zg As Double

Qbic.Model = PR1976_Qbic

' Properties needed by the model
Qbic.AcentricFactor = w

Qbic.MaxwellTestGivenTr tr, pr1, zg, zf

PrMaxwell = pr1
```

### B.2.5 `clsQbicsMulticomp`

This class allows the calculation of mixture properties using cubic equations of state (it also includes sections for calculating critical points and binary interaction parameters). The general calculation approach is similar to the previous examples.

Below are representative code lines for calculating a critical point:

```vb
Dim QBicMulticomp As New clsQbicsMulticomp

QBicMulticomp.Properties.Add Tc1, Pc1, w1, zc1, 0, 0, 0, 0
QBicMulticomp.Properties.Add Tc2, Pc2, w2, zc2, 0, 0, 0, 0

QBicMulticomp.MolarFraction(1) = x1
QBicMulticomp.MolarFraction(2) = x2

QBicMulticomp.CriticalProperties Tc, Pc, vc
```

### B.2.6 `clsLVE`

This class encompasses all possible combinations for calculating vapor-liquid equilibrium of mixtures, using the gamma-phi and phi-phi models. The general operation of the class is as follows: the necessary properties are defined through `Mixture`, then the calculation conditions are specified.

Below is a typical example for calculating a bubble point:

```vb
Private LVE As New clsLVE

' --- Common section for all calculations: fill properties ---

LVE.Tolerances.MaxIter = 30
LVE.Tolerances.TolMolarFraction = 0.0001
LVE.Tolerances.TolTemperature = 0.0001
LVE.Tolerances.TolPressure = 0.0001

LVE.ReferenceState.TTo = 273.15
LVE.ReferenceState.Po = 10325
LVE.ReferenceState.Ho = 0
LVE.ReferenceState.So = 0

LVE.Mixture.NumberOfComponents = 2

LVE.Mixture.CriticalProps.Add Tc1, Pc1, w1, Zc1
LVE.Mixture.CriticalProps.Add Tc2, Pc2, w2, Zc2

LVE.Mixture.OtherProps.Add 0, Tb1, 0, 0, 0, 0, 0, 0
LVE.Mixture.OtherProps.Add 0, Tb2, 0, 0, 0, 0, 0, 0

LVE.Mixture.SatPressureModel = RPM
LVE.Mixture.vaporModel = PR1976_Qbicv
LVE.Mixture.Liquid1Model = PR1976_Qbicl

' --- Specify feed flow conditions ---

LVE.FeedFlow.MolarFraction(1) = z1
LVE.FeedFlow.MolarFraction(2) = z2
LVE.FeedFlow.Temperature = 300

' --- Invoke the calculation method ---

LVE.BubblePointPressure

' --- Results ---

y1 = LVE.VaporFlow.MolarFraction(1)
y2 = LVE.VaporFlow.MolarFraction(2)
P  = LVE.VaporFlow.Pressure
```

---

[Back to Table of Contents](../README.md)
