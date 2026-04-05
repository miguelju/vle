# Chapter II — Description of Vapor-Liquid Equilibrium

> **Translation status**: Pending full translation. See [Spanish original](../../es/research-paper/Capitulo%20II.md).

---

## 2.1 Cubic Equations of State

Cubic equations of state (CEOS) have become a commonly used tool for solving phase equilibrium problems in multicomponent systems, not only for their computational simplicity, but also for the good approximations that result from their use, especially at high pressures, where other models tend to fail.

Among the most well-known CEOS are the van der Waals equation, the Redlich-Kwong equation, the modification of the latter by Soave, and the one proposed by Peng and Robinson.

From a computational standpoint, the use of a general expression as presented by Abbott (*5*)(references.md#ref-5) facilitates the programming of such models (see Table 2.1 and Table 2.2 in the original).

### 2.1.1 VLE Prediction using CEOS

According to Fotouh and Shukla (*6*)(references.md#ref-6), there are two typical problems associated with the use of CEOS in VLE predictions: the number of phases in equilibrium is not known a priori, and near the critical point of the mixture, calculations depend strongly on initial values.

The general expression proposed by Müller et al. (*9*)(references.md#ref-9) for multicomponent fugacity coefficients facilitates the calculation of this property. The numerical evaluation of compositional derivatives as proposed by Stockfleth and Dohrn (*10*)(references.md#ref-10) decreases calculation speed but allows computing fugacity coefficient derivatives for any CEOS and mixing rule model.

### 2.1.2 Critical Points of Multicomponent Mixtures

The method presented by Heidemann and Khalil (*16*)(references.md#ref-16) originates from Gibbs' criterion. The algorithm is based on the study of the stability of existing phases (Equations 2.14–2.23 of the thesis). See also Peng and Robinson (*15*)(references.md#ref-15) for an alternative approach.

### 2.1.3 Energy Properties based on CEOS

Müller et al. (*9*)(references.md#ref-9) presented general expressions for residual enthalpy and entropy (Equations 2.28–2.34) that allow calculation for any CEOS with k3=1 and any mixing rule.

## 2.2 Activity Coefficient Models

For cases where simple CEOS fail to predict liquid phase properties (Assael et al. (*17*)(references.md#ref-17)), activity coefficient models provide better results. Models include Wilson, Scatchard-Hildebrand, Margules, and van Laar (Table 2.3 in the original).

Liquid volumes needed for the Poynting factor and Wilson/Scatchard-Hildebrand equations can be obtained through correlations such as that presented by Hankinson and Thomson (*18*)(references.md#ref-18).

### 2.2.1 Energy Properties using Activity Coefficient Models

Excess enthalpy and entropy can be calculated from the temperature derivative of excess Gibbs energy (Equations 2.44–2.48).

## 2.3 Applications of VLE Study

### 2.3.1 Bubble and Dew Point Calculations

General framework following Assael et al. (*17*)(references.md#ref-17) (Figure 2.2). Temperature/pressure updates can use parabolic extrapolation as presented by Da Silva and Báez (*4*)(references.md#ref-4) or multivariable Newton-Raphson as presented by Asselineau et al. (*14*)(references.md#ref-14).

### 2.3.2 Flash Calculations

Isothermal flash uses the Rachford-Rice equation (Eq. 2.49). Michelsen (*19*)(references.md#ref-19) proposes stability analysis as a first step. The adiabatic flash algorithm is shown in Figure 2.6 (Assael et al. (*17*)(references.md#ref-17)).

### 2.3.3 Proposed Algorithms

A two-stage method was developed combining Da Silva and Báez (*4*)(references.md#ref-4) (first stage, fast convergence) with Asselineau et al. (*14*)(references.md#ref-14) (second stage, near critical). The second stage follows the approach of Anderson and Prausnitz (*20*)(references.md#ref-20) (Figure 2.7).

---

[Previous: Chapter I](chapter-1-introduction.md) | [Back to Table of Contents](README.md) | [Next: Chapter III](chapter-3-architecture.md)
