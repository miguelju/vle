# Chapter V — Conclusions and Recommendations

> **Translation status**: Pending full translation. See [Spanish original](../../es/research-paper/Capitulo%20V.md).

---

A thermodynamic library was developed, using the object-oriented programming paradigm provided by Microsoft Visual Basic 5.0, for the calculation of properties of pure substances and multicomponent mixtures, as well as the solution of common vapor-liquid equilibrium problems (bubble and dew point calculations, and adiabatic and isothermal flash calculations).

An interactive master program was designed to operate the thermodynamic library.

A new algorithm was proposed for the calculation of bubble and dew points with the ability to converge in regions near the critical point.

The results obtained with the developed package are consistent with those presented in the literature (Sandler (*3*)(references.md#ref-3), Orbey and Sandler (*21*)(references.md#ref-21), Da Silva and Báez (*4*)(references.md#ref-4), and Smith et al. (*22*)(references.md#ref-22)).

The developed thermodynamic library can be used in other programs without needing to know its internal structure (due to its object-oriented design).

### Recommendations

- Use of the thermodynamic library for steady-state flow problems, such as rigorous distillation column calculations.
- Incorporation of parameters for EOS modifications (Peng-Robinson and RKS variants) into the existing database.
- Creation of online help tools for the master program.
- Reconstruction of the thermodynamic library through new sub-classes for greater consistency in property and model classifications.

---

[Previous: Chapter IV](chapter-4-validation.md) | [Back to Table of Contents](README.md) | [References](references.md)
