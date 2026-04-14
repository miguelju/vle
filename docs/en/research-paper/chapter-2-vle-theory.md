# Chapter II -- Description of Vapor-Liquid Equilibrium

> **Translation status**: Complete. See [Spanish original](../../es/research-paper/pdf/Capitulo%20II.pdf).

---

An equation of state (EOS) is generally used to describe the gas phase of a mixture (for example, the Virial equation truncated at the second term), through the partial fugacity coefficient of the chemical compounds in solution (φ), while there exist two different methods or approaches for modeling the liquid phase: either the same EOS used for the vapor phase description is employed (the φ-φ model), or the activity coefficient method is used. This latter approach is known as the γ-φ model (where γ indicates that the activity coefficient has been used to describe the liquid phase and φ for the description of the vapor phase, calculated from an EOS).

The methods based on cubic equations of state (CEOS) and those using activity coefficients are described below.

## 2.1 Cubic Equations of State

Cubic equations of state (CEOS) have become a commonly used tool for solving phase equilibrium problems in multicomponent systems, not only for their computational simplicity, but also for the good approximations that result from their use, especially at high pressures, where other models tend to fail.

Among the most well-known CEOS are the van der Waals equation, the Redlich-Kwong equation, the modification of the latter by Soave, and the one proposed by Peng and Robinson.

From a computational standpoint, the use of a general expression such as the one presented by Abbott [(*5*)](references.md#ref-5):

`P = RT/(v - b) - a·α(T) / [(v + k₁b)(v + k₂b)]`  (2.1)

facilitates the programming of such models, since, as shown in Table 2.1, the parameters *a*, *b*, *k*₁, *k*₂, and *k*₃ establish the equivalence between Equation (2.1) and the aforementioned CEOS.

**Table 2.1** -- Cubic equations of state and their relationship to Abbott's general form.

| CEOS | P(T, v) | k₁ | k₂ | k₃ | a | b |
|---|---|:---:|:---:|:---:|---|---|
| van der Waals | `P = RT/(v - b) - a/v²` | 0 | 0 | 1 | *a* | *b* |
| Redlich-Kwong | `P = RT/(v - b) - a/[T^(1/2) · v(v + b)]` | 1 | 0 | 1 | `a(T) = a · T^(-1/2)` | *b* |
| Soave (SRK) | `P = RT/(v - b) - a·α(T)/[v(v + b)]` | 1 | 0 | 1 | `a(T) = a · α(T)` | *b* |
| Peng-Robinson | `P = RT/(v - b) - a·α(T)/[v(v + b) + b(v - b)]` | 2 | -1 | 1 | `a(T) = a · α(T)` | *b* |

Another important simplification corresponds to the use of the dimensionless form of Equation (2.1) through the expression:

`Z = Pv/(RT)`  (2.2)

which, upon substitution into Equation (2.1), produces the following expression:

`Z³ - (1 + B - k₁B)Z² + (A - k₁B - k₂B²)Z - (AB + k₂B² + k₂B³) = 0`  (2.3)

where:

`A = aP/(R²T²)`  (2.4)

and

`B = bP/(RT)`  (2.5)

The parameters in Equations (2.4) and (2.5) can be found for each of the named cubic equations of state in Table 2.2.

**Table 2.2** -- Parameters of the cubic equations of state.

| CEOS | a | b | Ω_a | Ω_b | α(T_r) |
|---|---|---|---|---|---|
| van der Waals | `Ω_a · R²T_c²/P_c` | `Ω_b · RT_c/P_c` | 27/64 | 1/8 | 1 |
| Redlich-Kwong | `Ω_a · R²T_c^(5/2)/P_c` | `Ω_b · RT_c/P_c` | 0.42748 | 0.08664 | `T_r^(-1/2)` |
| Soave (SRK) | `Ω_a · R²T_c²/P_c` | `Ω_b · RT_c/P_c` | 0.42748 | 0.08664 | `[1 + m(1 - T_r^(1/2))]²` where `m = 0.480 + 1.574ω - 0.176ω²` |
| Peng-Robinson | `Ω_a · R²T_c²/P_c` | `Ω_b · RT_c/P_c` | 0.45724 | 0.07780 | `[1 + m(1 - T_r^(1/2))]²` where `m = 0.37464 + 1.54226ω - 0.26992ω²` |

It is important to emphasize that the equations reported in Table 2.2 for the parameters *A* and *B* of the CEOS are valid only for the study of pure substances. Thus, in order to use a CEOS for the correlation and prediction of phases in multicomponent systems, a composition dependence of the species comprising the mixture must be introduced into these parameters. This dependence is achieved through mixing rules, the simplest of which is the van der Waals mixing rule:

`a_mix = ΣᵢΣⱼ xᵢ xⱼ aᵢⱼ`  (2.6)

`b_mix = Σᵢ xᵢ bᵢ`  (2.7)

where:

`aᵢⱼ = (aᵢ aⱼ)^(1/2) (1 - kᵢⱼ)`  (2.8)

and the parameters *k*ᵢⱼ can be obtained by fitting the predictions made through the CEOS to experimental volumetric and equilibrium data, or can be considered zero in cases where the molecular interaction between the components of the mixture is small.

### 2.1.1 VLE Prediction using CEOS

The adequate prediction of vapor-liquid equilibrium of mixtures using CEOS is of great importance today, due to its extensive use in diverse industrial applications such as reservoir modeling, process design, gas separation, etc. However, according to Fotouh and Shukla [(*6*)](references.md#ref-6), there are two typical problems associated with the use of CEOS in such predictions:

- The number of phases in equilibrium is not known a priori.
- In regions near the critical point of the mixture, calculations depend strongly on the initial values of the unknown variables and the numerical method used, frequently producing trivial solutions (equilibrium phases of identical composition).

In the application of CEOS to this type of problem, a single equation is used to model the thermodynamic properties of all phases present in the mixture. Vapor-liquid equilibrium is achieved when the conditions of mechanical and thermal stability are reached, in addition to satisfying the mass balances of the system. Thermal and mechanical stability are fulfilled when the temperature and pressure, respectively, are equal in all phases. Furthermore, the satisfaction of mass balances can be expressed in terms of the Gibbs free energy of the mixture, which reaches a global minimum at equilibrium.

Over the years, different methods have been developed to predict phase equilibrium in mixtures. Among these methods are:

- The **Gibbs energy minimization** proposed by Michelsen [(*7*)](references.md#ref-7), in which the Gibbs energy function of the mixture is minimized with respect to the number of moles or composition of the components in the fluid.

- The **area method** proposed by Eubank *et al.* [(*8*)](references.md#ref-8), which consists of calculating two compositions of the mixture such that the difference between the absolute area under the straight line connecting these compositions and the area of the Gibbs energy curve between them is at a positive maximum.

- The **fugacity method**.

The latter, which by virtue of its simplicity is the most commonly used, is expressed through the following equation:

`f̂ᵢᴸ = f̂ᵢⱽ`  (i = 1, 2, ..., N)  (2.9)

Defining the fugacity coefficient of species *i* in solution as:

`φ̂ᵢ = f̂ᵢ / (xᵢ P)`  (2.10)

Equation (2.9) is equivalent to:

`xᵢ φ̂ᵢᴸ = yᵢ φ̂ᵢⱽ`  (2.11)

For calculation purposes, Equation (2.11) can be written in the following form:

`yᵢ = Kᵢ xᵢ`  (2.12)

where *K*ᵢ is the equilibrium coefficient given by:

`Kᵢ = φ̂ᵢᴸ / φ̂ᵢⱽ`  (2.13)

The combination of Equations (2.12) and (2.13) is known as the **φ-φ model** for the approximation to thermodynamic equilibrium of multicomponent systems, whose calculation procedure is based on the simultaneous solution of the system of equations represented by Equation (2.9).

Expressions for the fugacity coefficients as a function of pressure (or volume), temperature, and system composition must be derived specifically for each CEOS and mixing rule used. In this regard, the general expression proposed by Muller *et al.* [(*9*)](references.md#ref-9) for multicomponent fugacity coefficients facilitates the calculation of this property. However, the evaluation of the compositional derivatives present in some terms of such an expression can be difficult, due to the considerable analytical and algebraic effort required for their computation. These difficulties can be overcome by evaluating such derivatives numerically, as proposed by Stockfleth and Dohrn [(*10*)](references.md#ref-10); the disadvantage of using this type of method is that it decreases calculation speed, but it allows computing fugacity coefficient derivatives for any CEOS and mixing rule model.

Although the fugacity method satisfies the mass balances of the system, it can fail in the adequate prediction of the number of phases or simply lead to trivial solutions. This is because the equality of fugacities is a necessary but not sufficient condition for the existence of a global minimum in the Gibbs energy of the system, as described by Null [(*11*)](references.md#ref-11).

In particular, in the critical region of the mixture, the solutions to VLE problems depend considerably on the initial values of the unknown mixture compositions. Furthermore, the numerical techniques used to solve the system of nonlinear equations can lead to local minima rather than the global minimum of the Gibbs energy of the mixture.

Various algorithms based on the fugacity method have been proposed in the literature in order to avoid trivial solutions and achieve good approximations in zones near the critical point of the mixture. For example, Poling and Prausnitz [(*12*)](references.md#ref-12) and Gundersen [(*13*)](references.md#ref-13) present two different algorithms for adequately obtaining the volumes of the phases in equilibrium, while Asselineau *et al.* [(*14*)](references.md#ref-14) presented an algorithm based on the multivariable Newton-Raphson numerical method to adequately predict phase equilibrium in the critical zone of the mixtures.

### 2.1.2 Critical Points of Multicomponent Mixtures

The prediction of the actual critical properties of a multicomponent system is an important aspect of the phase calculation problem, since from these data one can predict, for example, the existence of the phenomenon known as retrograde condensation.

In the literature, there exist various methods for the prediction of mixture critical points, among which one can mention that proposed by Peng and Robinson [(*15*)](references.md#ref-15), which, using the criterion stated by Gibbs, achieves a solution to the problem of predicting the critical properties of a mixture using a two-parameter cubic equation of state.

An alternative method was presented by Heidemann and Khalil [(*16*)](references.md#ref-16), which also originates from Gibbs' criterion. The calculation algorithm is based on the study of the stability of existing phases. Stability can be described in mathematical terms by the following equations:

`A - A₀ - P₀(V - V₀) - Σᵢ μᵢ₀(nᵢ - nᵢ₀) ≥ 0`  (2.14)

`dU = dA + T dS ≥ 0`  (2.15)

In Equation (2.14), the pressure *P*₀ and the chemical potentials μᵢ₀ are evaluated at a trial point (initial state), and *A - A*₀ is the difference between the Helmholtz free energy of the initial state and a subsequent state. If Equations (2.14) and (2.15) are not satisfied for any phase change in a region around the trial point, there exists a lower internal energy accessible to the mixture through its separation into two or more phases.

The Helmholtz free energy can be expanded in a Taylor series around a point at constant volume, yielding the following expression:

`A - A₀ = (1/2!) Σᵢ Σⱼ (∂²A/∂nᵢ∂nⱼ) Δnᵢ Δnⱼ + (1/3!) Σᵢ Σⱼ Σₖ (∂³A/∂nᵢ∂nⱼ∂nₖ) Δnᵢ Δnⱼ Δnₖ + ...`  (2.16)

The stability of the trial point requires that the above equation be positive for any arbitrary value of Δnᵢ.

On the other hand, for a point to lie on the stability limit, the determinant of the matrix **Q** with elements:

`Qᵢⱼ = ∂²A / ∂nᵢ∂nⱼ`  (2.17)

must equal zero:

`det(Q) = 0`  (2.18)

or there must exist a vector:

`Δn = (Δn₁, Δn₂, ..., ΔnN)`  (2.19)

that satisfies the equation:

`Q · Δn = 0`  (2.20)

A critical point is one that lies on the stability limit, which means, from a mathematical standpoint, that there exists a vector **Δn** that satisfies Equation (2.20) and that, when substituted into Equation (2.16), causes the cubic term to vanish, that is:

`Σᵢ Σⱼ Σₖ (∂³A/∂nᵢ∂nⱼ∂nₖ) Δnᵢ Δnⱼ Δnₖ = 0`  (2.21)

From Equations (2.18), (2.20), and (2.21), Heidemann and Khalil [(*16*)](references.md#ref-16) formulated a procedure for calculating critical points as shown in the flow diagram presented in Figure 2.1 (see original Spanish document). In the case of cubic equations of state, the Helmholtz energy derivatives found in Equations (2.16) and (2.21) can be calculated using the following equations:

`∂²A/∂nᵢ∂nⱼ = RT [δᵢⱼ/nᵢ + (1/RT)(∂ ln f̂ᵢ / ∂nⱼ)]`  (2.22)

`∂³A/∂nᵢ∂nⱼ∂nₖ = RT [-δᵢⱼ/nᵢ² + (1/RT)(∂² ln f̂ᵢ / ∂nⱼ∂nₖ)]`  (2.23)

where the first- and second-order fugacity derivatives can be evaluated numerically as mentioned in the previous section.

> **Figure 2.1**: Algorithm for calculating the critical point of a mixture (see original Spanish document).

### 2.1.3 Energy Properties based on CEOS

In the calculation of energy properties through the use of CEOS, it is necessary to introduce the concept of a residual property:

`Mᴿ = M - Mᵍⁱ`  (2.24)

where *M* is the value of the thermodynamic property of the fluid and *M*ᵍⁱ is the value that the same property would have if the fluid behaved as an ideal gas at the same temperature and pressure as *M*.

Taking the fundamental expression that relates the Gibbs energy to temperature, pressure, and chemical potentials:

`G = H - TS`  (2.25)

it is possible to obtain the expressions for the residual enthalpy and entropy:

`Hᴿ = -T² ∂(Gᴿ/T)/∂T |_P`  (2.26)

`Sᴿ = -∂Gᴿ/∂T |_P`  (2.27)

Muller *et al.* [(*9*)](references.md#ref-9) presented general expressions for these and other residual properties, which allow their calculation for any CEOS with *k*₃ = 1 and any mixing rule used:

`Hᴿ/(RT) = (Z - 1) + [T(∂A/∂T) - A] · I / B`  (2.28)

`Sᴿ/R = ln(Z - B) + T(∂A/∂T) · I / B`  (2.29)

where:

`D = (k₁² - 4k₂)^(1/2)`  (2.30)

and the auxiliary integral *I* depends on the discriminant *k*₁² - 4*k*₂:

`I = (1/D) ln[(2Z + B(k₁ - D)) / (2Z + B(k₁ + D))]`  if  `k₁² - 4k₂ > 0`  (2.31)

`I = (2/D) arctan[(2Z + k₁B) / (BD)]`  if  `k₁² - 4k₂ < 0`  (2.32)

`I = -2B / (2Z + k₁B)`  if  `k₁² - 4k₂ = 0`  (2.33)

`T(∂A/∂T) = (P / R²T²)[T(da/dT) - 2a]`  (2.34)

Equations (2.28) and (2.29), together with the known expressions for ideal gas heat capacities, allow the calculation of the enthalpy and entropy of a phase of the multicomponent system, taking as a reference state the ideal gas for each of the components of the mixture (*T*_ref, *P*_ref, *H*⁰ and *S*⁰):

`H(T, P, x) = Σᵢ xᵢ [Hᵢ⁰ + ∫(T_ref→T) Cpᵢᵍⁱ dT] + Hᴿ(T, P, x)`  (2.35)

`S(T, P, x) = Σᵢ xᵢ [Sᵢ⁰ + ∫(T_ref→T) (Cpᵢᵍⁱ/T) dT - R ln(P/P_ref)] + Sᴿ(T, P, x) - R Σᵢ xᵢ ln xᵢ`  (2.36)

## 2.2 Activity Coefficient Models

The appropriate use of a CEOS offers a convenient way to calculate phase equilibrium. However, simple CEOS are only applicable to mixtures of molecules that do not possess strong specific interactions, and they tend to fail in predicting liquid phase properties (Assael *et al.* [(*17*)](references.md#ref-17)). In such cases, better results are obtained when the fugacity of each component in the liquid phase is estimated using an activity coefficient model.

The activity â_i of a substance *i* in a mixture of *n* components is defined as:

`âᵢ = f̂ᵢᴸ / fᵢ⁰`  (2.37)

and is expressed in terms of the corresponding activity coefficient:

`âᵢ = γᵢ · xᵢ`  (2.38)

where *f*ᵢ⁰ is the fugacity of pure liquid *i* at the system temperature and a reference pressure.

The purpose of using an activity coefficient model is to represent the dependence of γᵢ on the temperature, pressure, and composition of the system. The pressure effect is taken into account by noting that:

`∂ ln γᵢ / ∂P = V̄ᵢᴸ / (RT)`  (2.39)

where V̄ᵢᴸ is the partial molar volume of component *i*.

The fugacity of a component in the mixture can then be written as:

`f̂ᵢᴸ = xᵢ γᵢ fᵢ⁰ Fᵢ`  (2.40)

where *F*ᵢ is the Poynting factor given by the following expression:

`Fᵢ = exp[(1/RT) ∫(Pᵢˢᵃᵗ→P) V̄ᵢᴸ dP]`  (2.41)

Finally, if Equation (2.10) is used to describe the vapor phase of the mixture, and Equation (2.40) for the liquid phase, one obtains the expression known as the **γ-φ model** for the thermodynamic approximation of vapor-liquid equilibrium of multicomponent systems:

`yᵢ φ̂ᵢⱽ P = xᵢ γᵢ Pᵢˢᵃᵗ φᵢˢᵃᵗ Fᵢ`  (2.42)

where the activity coefficient must be calculated using a thermodynamic model, the saturation pressure through a correlation, and φᵢˢᵃᵗ with the same EOS used in the description of the gas phase. In this regard, Table 2.3 presents the different expressions for activity coefficients according to the Wilson, Scatchard-Hildebrand, Margules, and van Laar models, the latter two being valid only for binary mixtures.

**Table 2.3** -- Activity coefficient models.

| Model | Parameters | ln γᵢ |
|---|---|---|
| Wilson | `Λᵢⱼ = (Vⱼᴸ/Vᵢᴸ) exp[-(λᵢⱼ - λᵢᵢ)/(RT)]` | `ln γᵢ = -ln(Σⱼ xⱼΛᵢⱼ) + 1 - Σₖ [xₖΛₖᵢ / Σⱼ(xⱼΛₖⱼ)]` |
| Scatchard-Hildebrand | δᵢ = solubility parameter; Vᵢᴸ = liquid molar volume | `ln γᵢ = Vᵢᴸ(δᵢ - δ_mix)²/(RT)` where `δ_mix = Σᵢ Φᵢδᵢ` and `Φᵢ = xᵢVᵢᴸ / Σⱼ xⱼVⱼᴸ` |
| Margules (binary) | A₁₂, A₂₁ | `ln γ₁ = x₂²[A₁₂ + 2(A₂₁ - A₁₂)x₁]`; `ln γ₂ = x₁²[A₂₁ + 2(A₁₂ - A₂₁)x₂]` |
| van Laar (binary) | A₁₂, A₂₁ | `ln γ₁ = A₁₂/[1 + (A₁₂x₁)/(A₂₁x₂)]²`; `ln γ₂ = A₂₁/[1 + (A₂₁x₂)/(A₁₂x₁)]²` |

Finally, it is important to note that the liquid volumes needed for the calculation of both the Poynting factor and the Wilson and Scatchard-Hildebrand equations can be obtained through the use of correlations such as the one presented by Hankinson and Thomson [(*18*)](references.md#ref-18).

### 2.2.1 Energy Properties using Activity Coefficient Models

To predict energy properties using activity coefficient-based models, it is necessary to introduce the concept of an excess property. An excess property is defined as the difference between the value of a thermodynamic property (*M*ᵢ) and the value of the same property if the fluid behaved as an ideal solution (*M*ᵢˢᴵ), at the same conditions of temperature, pressure, and composition. Therefore:

`Mᵢᴱ = Mᵢ - Mᵢˢᴵ`  (2.43)

where *M*ᵢᴱ is the excess property of component *i* in solution.

Once the concept of excess property is understood, and analogously to the case where residual properties are defined, the use of fundamental equations leads to the following formulation:

`Gᴱ = RT Σᵢ xᵢ ln γᵢ`  (2.44)

from which the expressions for excess enthalpy and entropy are obtained:

`Hᴱ = -T² ∂(Gᴱ/T)/∂T |_{P,x}`  (2.45)

`Sᴱ = -∂Gᴱ/∂T |_{P,x}`  (2.46)

As can be seen from Equations (2.44), (2.45), and (2.46), once the activity model is chosen, the calculation of excess properties is quite straightforward, since the excess Gibbs energy (*G*ᴱ) is a direct function of the activity coefficients, and the derivative present in the expressions for excess enthalpy and entropy can be calculated numerically or analytically, depending on the complexity of the model used.

Finally, by defining an ideal gas reference state common to all species present in the mixture, the enthalpy and entropy of the liquid phase can be calculated using the following formulas:

`Hᴸ(T, P, x) = Σᵢ xᵢ [Hᵢ⁰ + ∫(T_ref→T) Cpᵢᵍⁱ dT - ΔHᵢᵛᵃᵖ] + Hᴱ(T, P, x)`  (2.47)

`Sᴸ(T, P, x) = Σᵢ xᵢ [Sᵢ⁰ + ∫(T_ref→T) (Cpᵢᵍⁱ/T) dT - R ln(P/P_ref) - ΔSᵢᵛᵃᵖ] + Sᴱ(T, P, x) - R Σᵢ xᵢ ln xᵢ`  (2.48)

## 2.3 Applications of VLE Study

The prediction of phase equilibrium is one of the central problems of thermodynamics in chemical engineering, as it is of great importance in the design of chemical processes. The infinite number of possible mixtures, and the wide range of temperatures and pressures encountered in industrial processes, is such that no current thermodynamic model is applicable to all cases.

Several algorithms that allow the calculation of VLE for mixtures are presented below.

### 2.3.1 Bubble and Dew Point Calculations

The basis of most phase equilibrium calculations is an algorithm with the capability to calculate the bubble point or the dew point for a mixture of a given composition. In general, these point calculations are performed through algorithms with two calculation loops. The outer loop controls the unknown temperature or pressure of the system, and the inner loop adjusts the unknown compositions of the process.

In Figure 2.2 (see original Spanish document), a flow diagram is presented that represents a generic algorithm for bubble point calculation of multicomponent systems.

The algorithm generally used for dew point calculation is similar to the one described above (Figure 2.2). In this case, the vapor composition is known, and therefore the inner calculation loop is performed to adjust the liquid phase compositions.

It is important to note that in such algorithms, the new temperature or pressure at each iteration can be calculated through various numerical methods, among which are those presented by Da Silva *et al.* [(*4*)](references.md#ref-4) (parabolic extrapolation) and Asselineau *et al.* [(*14*)](references.md#ref-14) (multivariable Newton-Raphson).

### 2.3.2 Flash Calculations

The simulation of flash (instantaneous vaporization) processes is one of the most important applications of thermodynamics in chemical engineering. In this process, a flow stream whose overall compositions are known passes through a "drum" where the vapor and liquid phases are separated. Such a process can be operated under different conditions, including:

- Constant temperature and pressure (isothermal flash)
- Constant pressure and enthalpy (isenthalpic flash)

> **Figure 2.2**: Generic algorithm for the calculation of bubble points of multicomponent systems (Assael *et al.* [(*17*)](references.md#ref-17)). See original Spanish document.

In the process illustrated in Figure 2.3 (see original Spanish document), the feed at temperature *T*_F and pressure *P*_F passes through a valve and then enters the flash drum where the phases separate. The pressure *P* of the unit is controlled, and heat *Q* is transferred through a heat exchanger so that the temperature remains constant. In this way, the calculations related to the isothermal flash have as their fundamental objective the determination of the compositions (*y*ᵢ, *x*ᵢ) and the vaporized fraction (β) of the inlet flow *F*.

> **Figure 2.3**: Isothermal flash (see original Spanish document).

In the isothermal flash, since only the overall compositions of the mixture are known, the mass balances must be expressed in a form where the dependence on flows disappears, yielding the Rachford-Rice equation:

`Σᵢ [zᵢ(Kᵢ - 1) / (1 + β(Kᵢ - 1))] = 0`  (2.49)

This equation must be solved by a numerical method to obtain β as a result.

Michelsen [(*19*)](references.md#ref-19) proposes as a first step for solving a flash the study of phase stability using the Gibbs tangent plane criterion, in order to avoid trivial solutions. He then proposes an algorithm whose starting point is the values obtained as a result of the stability analysis, to subsequently obtain rapid solutions, even near the critical point of the mixture, using a second-order convergence method.

Other algorithms have been proposed in the literature, such as the one presented by Asselineau *et al.* [(*14*)](references.md#ref-14), in which the starting point, in certain cases, is temperature or pressure values obtained through the calculation of the pseudo-critical point of the mixture. However, the most commonly employed solution method, due to the ease of its application, is the one shown in Figure 2.4 (Assael *et al.* [(*17*)](references.md#ref-17); see original Spanish document).

> **Figure 2.4**: Algorithm for isothermal flash calculation (see original Spanish document).

On the other hand, the isenthalpic flash is illustrated in Figure 2.5 (see original Spanish document). The goal is to find the temperature, the vaporized fraction (β), and the molar fractions of each phase resulting from the process.

> **Figure 2.5**: Isenthalpic flash (see original Spanish document).

One of the algorithms employed for solving this type of problem is shown in Figure 2.6 (see original Spanish document), where an adiabatic operating condition (*Q* = 0) has been imposed.

> **Figure 2.6**: Algorithm for adiabatic flash calculation (see original Spanish document).

In the case where the process is not adiabatic, the objective function of the algorithm presented in Figure 2.6 would be determined by the energy balance expression of the system:

`H_F - (1 - β)Hᴸ - βHⱽ - Q = 0`  (2.50)

where the heat is a known variable.

### 2.3.3 Proposed Algorithms

In order to avoid trivial solutions and guarantee convergence in zones near the critical point of the mixture, a method was developed for the calculation of bubble or dew points, based on the combination of several algorithms mentioned or presented in previous sections.

The developed algorithm is composed of two stages:

1. **First stage**: An algorithm based on the one proposed by Da Silva and Baez [(*4*)](references.md#ref-4), similar to the one presented in Figure 2.2, is used in order to obtain rapid solutions. If a trivial solution is obtained, the procedure moves to the second stage.

2. **Second stage**: A bubble or dew point is calculated at a low pressure or temperature, below the pseudo-critical point (below which the CEOS presents a single real root), in order to guarantee the convergence of the method proposed by Asselineau *et al.* [(*14*)](references.md#ref-14). Once this point is obtained, another point of greater magnitude is estimated (at a certain ΔT or ΔP), taking as initial values those of the previous point. Then, by calculating the slope between them, the next point is estimated through the equations presented below, where the derivatives are calculated numerically:

`T_{n+1} = T_n + (dT/dP)_n · ΔP`  (2.51)

`P_{n+1} = P_n + (dP/dT)_n · ΔT`  (2.52)

> with the purpose of traversing the phase envelope differentially and thus achieving the desired point.

The second stage of the proposed process is illustrated in Figure 2.7 (see original Spanish document), as presented by Anderson and Prausnitz [(*20*)](references.md#ref-20):

> **Figure 2.7**: Second stage of the proposed algorithm, where point *a* represents the starting point of the algorithm and point *b* the desired final value (see original Spanish document).

---

[Previous: Chapter I](chapter-1-introduction.md) | [Back to Table of Contents](README.md) | [Next: Chapter III](chapter-3-architecture.md)
