# Chapter IV — Validation of the Thermodynamic Package

> **Translation status**: Complete. See [Spanish original](../../es/research-paper/pdf/Capitulo%20IV.pdf).

---

This chapter presents a set of representative tests of the main types of calculations and models, comparing the results obtained with different sources found in the literature.

Given that the complete validation of a thermodynamic package offering so many available models is an ambitious task, a master program was created to facilitate the interaction between the user and the developed thermodynamic library (see Appendix C).

## 4.1 Critical Point Calculations

Peng and Robinson [*15*](references.md#ref-15) reported results for critical point calculations of different mixtures. Table 4.1 contains the global compositions of the systems studied, while Table 4.2 presents the values reported by Peng and Robinson [*15*](references.md#ref-15) and those calculated by the developed package.

**Table 4.1** Global compositions of the studied systems.

| Mixture No. | C₁   | C₂     | C₃     | nC₄    | nC₅    | CO₂   | H₂S   |
|:-----------:|:-----:|:------:|:------:|:------:|:------:|:-----:|:-----:|
| 1           |       | 0.3414 | 0.3421 |        | 0.3165 |       |       |
| 2           |       |        | 0.3276 | 0.3398 | 0.3326 |       |       |
| 3           | 0.07  |        |        |        |        | 0.616 | 0.314 |
| 4           |       | 0.2542 | 0.2547 | 0.2554 | 0.2357 |       |       |

**Table 4.2** Comparison of calculated mixture critical points with those reported by Peng and Robinson [*16*](references.md#ref-16).

| Mixture No. | Pc (kPa) [*16*](references.md#ref-16) | Tc (K) [*16*](references.md#ref-16) | Pc (kPa) This Project | Tc (K) This Project | % Pc  | % Tc  |
|:-----------:|:------:|:------:|:------:|:------:|:-----:|:-----:|
| 1           | 5552   | 404.43 | 5620.32 | 405.47 | 1.231 | 0.256 |
| 2           | 4174   | 430.72 | 4173.9  | 430.71 | 0.002 | 0.002 |
| 3           | 8420   | 310.92 | 7999.33 | 315.88 | 4.996 | 1.595 |
| 4           | 5063   | 410.74 | 5094.55 | 411.26 | 0.623 | 0.126 |

As shown in Table 4.2, the results obtained through the developed thermodynamic library are quite close to those reported by Peng and Robinson [*15*](references.md#ref-15).

The existing discrepancies can be attributed to three fundamental causes:

- The probable difference between the values of the critical properties of the mixture components used by both projects, since Peng and Robinson [*15*](references.md#ref-15) do not report these values.

- The values calculated through the library were obtained without taking into account the binary interaction coefficient (k_ij) between the substances, which would be the primary cause of the observed differences, since these parameters are used by Peng and Robinson [*15*](references.md#ref-15) but not reported.

- The probable difference between the calculation precisions of the two projects.

It is important to note that other thermodynamic packages such as PRMIX.EXE (Sandler [*3*](references.md#ref-3)) and Ekilib (Da Silva and Báez [*4*](references.md#ref-4)) do not allow the calculation of mixture critical points, making it impossible to compare results with these sources.

On the other hand, the algorithm used (based on that presented by Heidemann *et al.* [*16*](references.md#ref-16) with the introduction of numerical derivatives as described by Stockfleth *et al.* [*10*](references.md#ref-10)) tends to be rather slow compared to one using analytical derivatives. However, it allows the use of various mixing rules without the need to obtain the mathematical expressions (for each cubic EOS) of the partial fugacity coefficients of the components.

## 4.2 Adiabatic Flash Calculations

Table 4.3 presents the adiabatic flash calculation for a four-component mixture (taken from the example presented by Da Silva and Báez [*4*](references.md#ref-4): benzene, cyclohexane, methylcyclohexane, and n-hexane). The Peng-Robinson model was used for both phases, without considering binary interaction parameters.

**Table 4.3** Feed data for the adiabatic flash.

| Data             | Value  |
|:----------------:|:------:|
| Pressure (kPa)   | 300    |
| Temperature (K)  | 420    |
| Phase             | Liquid |

As shown in Table 4.4, the results obtained by the developed package are consistent with those presented by Da Silva and Báez [*4*](references.md#ref-4). The small existing differences are due to the precision used as well as the constants employed.

Furthermore, other programs such as PRMIX.EXE (Sandler [*3*](references.md#ref-3)), despite using the Peng-Robinson cubic equation, do not allow adiabatic flash calculations, demonstrating the limited versatility of that program.

**Table 4.4** Adiabatic flash output data.

| Data              | This Project | Da Silva *et al.* [*4*](references.md#ref-4) | Calculated Errors (%) |
|:-----------------:|:------------:|:---------------------------------------------:|:---------------------:|
| Pressure (kPa)    | 300          | 300                                           |                       |
| Temperature (K)   | 394.263      | 394.285                                       | 0.00558               |
| β                 | 0.194494     | 0.19864                                       | 0.020872              |
| Enthalpy (kJ/kmol)| −7151        | −7033.5                                       | 1.670577              |
| x₁                | 0.2466       | 0.24648                                       | 0.048685              |
| x₂                | 0.2508       | 0.25084                                       | 0.015946              |
| x₃                | 0.271        | 0.27144                                       | 0.162098              |
| x₄                | 0.2316       | 0.23124                                       | 0.155682              |
| y₁                | 0.2642       | 0.26421                                       | 0.003785              |
| y₂                | 0.2465       | 0.24660                                       | 0.040552              |
| y₃                | 0.1632       | 0.16351                                       | 0.189591              |
| y₄                | 0.3261       | 0.32569                                       | 0.125887              |

## 4.3 Bubble Point Pressure

Using the van Laar model (whose parameters are shown in Table 4.5) for the description of the liquid phase and the ideal gas model for the vapor phase, several bubble points of a binary methanol(1)/water(2) mixture were calculated using the developed package, and compared with those obtained with the program AC.EXE (Orbey and Sandler [*21*](references.md#ref-21)) at a temperature of 298 K (see Table 4.6).

**Table 4.5** Parameters used with the van Laar model (Orbey and Sandler [*21*](references.md#ref-21)).

| Λ₁₂   | Λ₂₁   |
|:------:|:------:|
| 0.5853 | 0.3458 |

**Table 4.6** Comparison of bubble point calculations at known temperature.

| x₁     | y₁ [*21*](references.md#ref-21) | P (kPa) [*21*](references.md#ref-21) | y₁ This Project | P (kPa) This Project | % P    | % y    |
|:------:|:------:|:------:|:------:|:------:|:------:|:------:|
| 0.0873 | 0.4416 | 5.1998 | 0.4419 | 5.2410 | 0.7921 | 0.0679 |
| 0.19   | 0.6287 | 7.0028 | 0.6245 | 7.0620 | 0.8460 | 0.6680 |
| 0.3417 | 0.7538 | 9.1151 | 0.7542 | 9.1962 | 0.8898 | 0.0531 |
| 0.4943 | 0.8334 | 10.9757| 0.8338 | 11.0782| 0.9343 | 0.0480 |
| 0.6919 | 0.909  | 13.2939| 0.9092 | 13.4230| 0.9714 | 0.0220 |
| 0.8492 | 0.9583 | 15.1678| 0.9584 | 15.3184| 0.9931 | 0.0104 |

As shown in Table 4.6, the pressure values calculated through the developed package differ on average by less than 1% from the values reported by the consulted source. This error is explained by the differences between the constants used in the correlation employed for predicting the saturation pressures of the components, as well as the difference in the precision with which the calculations were performed.

## 4.4 Dew Point Calculations

For the binary system 2-propanol(1)/water(2), the dew point calculation was performed using the Wilson model for the liquid phase and the ideal gas model for the vapor phase.

Tables 4.7 and 4.8 present the results obtained.

**Table 4.7** Comparison of the calculated dew point given the pressure.

| y₁  | P (kPa) | T (K) Smith *et al.* [*22*](references.md#ref-22) | x₁ Smith *et al.* [*22*](references.md#ref-22) | T (K) This Project | x₁ This Project | % T   | % x    |
|:---:|:-------:|:------:|:------:|:------:|:------:|:-----:|:------:|
| 0.4 | 101.33  | 360.61 | 0.0639 | 360.918| 0.0824 | 0.085 | 28.95  |

As shown in Table 4.7, the percentage error in temperature is less than 1% and can be explained by the difference in calculation precision. However, the error in composition (28.95%) indicates that the calculation of the liquid molar volumes of the components has significantly affected the results, since in the calculations performed by Smith *et al.* [*22*](references.md#ref-22), the variation of this property with respect to temperature is not taken into account, whereas this is considered in the calculations of the developed package.

On the other hand, when comparing the results obtained with those calculated using the Ekilib package (Da Silva and Báez [*4*](references.md#ref-4)), they agree up to the second decimal place, since although both projects use the same equations, there are differences in the precisions used (greater precision in the developed project).

Table 4.8 presents the results for the dew point calculation of the system at a temperature of 353.15 K. As shown, the results in this case are closer to those presented by Smith *et al.* [*22*](references.md#ref-22) (less than 5%), since the liquid molar volume of each component is calculated only once at the mixture temperature, unlike the previous case in which the volume is recalculated at each iteration with temperature.

**Table 4.8** Comparison of the calculated dew point given the temperature.

| y₁  | T (K)  | P (kPa) Smith *et al.* [*22*](references.md#ref-22) | x₁ Smith *et al.* [*22*](references.md#ref-22) | P (kPa) This Project | x₁ This Project | % P  | % x  |
|:---:|:------:|:------:|:------:|:------:|:------:|:----:|:----:|
| 0.6 | 353.15 | 96.72  | 0.449  | 92.64  | 0.471  | 4.2  | 4.8  |

## 4.5 Bubble Point Temperature

In this case, a four-component mixture (see Table 4.9) was studied using Raoult's law as the thermodynamic model. As shown, the results obtained through the developed thermodynamic library are absolutely consistent with those obtained through the package designed by Da Silva and Báez [*4*](references.md#ref-4).

**Table 4.9** Comparison of the calculated bubble point given the pressure.

| P (kPa)     |        | T (K) This Project | T (K) Da Silva and Báez [*4*](references.md#ref-4) |
|:-----------:|:------:|:------------------:|:---------------------------------------------------:|
| 101.325      |        | 131.51             | 131.51                                              |

| Component   | x_i    | y_i This Project   | y_i Da Silva and Báez [*4*](references.md#ref-4) | % y_i |
|:-----------:|:------:|:------------------:|:-------------------------------------------------:|:-----:|
| methane     | 0.25   | 0.99461            | 0.99461                                           | 0     |
| ethane      | 0.35   | 0.00536            | 0.00536                                           | 0     |
| propane     | 0.15   | 0.00003            | 0.00003                                           | 0     |
| butane      | 0.25   | 0.00000            | 0.00000                                           | 0     |

## 4.6 Isothermal Flash Calculations

An equimolar mixture of n-heptane(1) and butane(2) was selected. For the description of both phases, the Redlich-Kwong-Soave cubic EOS was used, neglecting binary interaction parameters. Table 4.10 presents the results obtained.

**Table 4.10** Comparison of results obtained in the resolution of an isothermal flash.

| T (K) | P (kPa) | x₁ Ekilib [*4*](references.md#ref-4) | y₁ Ekilib [*4*](references.md#ref-4) | β Ekilib [*4*](references.md#ref-4) | x₁ This Project | y₁ This Project | β This Project |
|:-----:|:-------:|:------:|:------:|:--------:|:------:|:------:|:--------:|
| 300   | 100     | 0.6135 | 0.04284| 0.19889  | 0.6135 | 0.04283| 0.198821 |

As can be observed, differences appear from the third decimal place onward when compared with the values obtained using the package developed by Da Silva and Báez [*4*](references.md#ref-4). Once again, such discrepancies originate from the precision with which the calculations were performed, as previously mentioned.

## 4.7 Binary Interaction Parameter Calculation

For the calculation of binary interaction parameters (k_ij), a CO₂(1)/butane(2) mixture was chosen. Table 4.11 presents the pressure versus composition data at constant temperature used for solving the stated problem, while Table 4.12 shows the values calculated by the developed program as well as those presented by other authors.

The results obtained are in agreement with those presented by both consulted sources.

**Table 4.11** Experimental equilibrium data for the CO₂-butane system (T = 357.57 K) (Da Silva and Báez [*4*](references.md#ref-4)).

| P (bar) | x₁      | y₁      |
|:-------:|:-------:|:-------:|
| 14.824  | 0.02967 | 0.21649 |
| 19.029  | 0.06228 | 0.36217 |
| 23.511  | 0.0959  | 0.45773 |
| 27.441  | 0.1283  | 0.51818 |
| 31.164  | 0.15673 | 0.56447 |
| 36.404  | 0.19636 | 0.60283 |
| 42.885  | 0.25027 | 0.64685 |
| 49.573  | 0.30421 | 0.67849 |
| 56.399  | 0.35904 | 0.69727 |
| 63.569  | 0.41871 | 0.71069 |
| 70.671  | 0.49255 | 0.7139  |
| 75.428  | 0.5352  | 0.71155 |
| 77.91   | 0.56473 | 0.69855 |
| 79.289  | 0.5745  | 0.68927 |

**Table 4.12** Comparison of results obtained in the calculation of binary interaction parameters.

|        | Ekilib (Da Silva and Báez [*4*](references.md#ref-4)) | Sandler [*3*](references.md#ref-3) | This Project |
|:------:|:-----------------------------------------------------:|:-----------------------------------:|:------------:|
| k_ij   | 0.1359                                                | 0.135                               | 0.1357       |

---

[Previous: Chapter III](chapter-3-architecture.md) | [Back to Table of Contents](README.md) | [Next: Chapter V](chapter-5-conclusions.md)
