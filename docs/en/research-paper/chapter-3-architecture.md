# Chapter III — Development and Structure of the Thermodynamic Package

> **Translation status**: Pending full translation. See [Spanish original](../../es/research-paper/Capitulo%20III.md).

---

There are two notable aspects in computer programs: a collection of algorithms (programmed instructions to perform certain tasks) and a collection of data, on which the algorithms are applied to obtain the desired solutions.

The project was developed using the object-oriented programming paradigm provided by Microsoft Visual Basic, creating different types of *classes* to represent: process flows/streams, thermodynamic models, chemical compounds and mixtures, and the global VLE problem.

## 3.1 Process Flows

A stream is characterized by: temperature, pressure, enthalpy, entropy, and vapor fraction, plus the proportions of chemical components. The `clsFlow` class models this (Figure 3.1).

## 3.2 Thermodynamic Models

Five representative classes were developed (Figures 3.2–3.6):
- Activity coefficient models (`clsActivityMulticomp`)
- CEOS for pure substances (`clsQbicsPure`)
- CEOS for mixtures (`clsQbicsMulticomp`)
- Virial equation for pure substances (`clsVirial`)
- Virial equation for mixtures (`clsVirialMulticomp`)
- Saturation pressure correlations (`clsSatPressureSolver`)

## 3.3 Modeling of Pure Substances and Mixtures

Properties classes (Figures 3.8–3.12): `clsCriticalProps` (Tc, Pc, w), `clsQbicsProps` (EOS-specific parameters), `clsCpConsts` (heat capacity coefficients), `clsOtherProps` (activity model parameters), grouped in `clsAllProps`.

## 3.4 The Global VLE Problem

The `clsLVE` class (Figures 3.13–3.15) integrates all other classes with four fundamental properties (feed stream, liquid stream, vapor stream, and mixture) and six methods (bubble T, bubble P, dew T, dew P, adiabatic flash, isothermal flash).

---

[Previous: Chapter II](chapter-2-vle-theory.md) | [Back to Table of Contents](README.md) | [Next: Chapter IV](chapter-4-validation.md)
