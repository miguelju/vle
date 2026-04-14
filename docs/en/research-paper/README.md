# Development of a Computer Program for the Calculation of Vapor-Liquid Equilibrium of Multicomponent Mixtures under the Windows Environment

> **Original title**: *"Desarrollo de un Programa Computacional para el Cálculo del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente Windows"*

**Authors**: Miguel Roberto Jackson Ugueto and Carlos Fernando Mendible Porras

**Thesis Project** (*Proyecto de Grado*), presented to Universidad Simón Bolívar as partial requirement for the degree of Chemical Engineer.

**Advisors**: Prof. Coray M. Colina and Prof. Jean-Marie Ledanois

**Date**: Sartenejas, April 1999

**Approved with Honors** (*Aprobado con Mención de Honor*)

---

## Abstract

A thermodynamic library and a master program were developed with the capability of utilizing the library. The thermodynamic library enables VLE calculation through the combination of different models, including cubic equations of state (CEOS), the truncated Virial equation, and activity coefficient-based models. The library includes algorithms commonly used in the study of VLE of multicomponent systems: bubble and dew point calculations, flash calculations, and the calculation of critical points using CEOS. Since the library was designed under the object-oriented programming paradigm provided by Microsoft Visual Basic, it can be easily used within other projects. The results obtained are fully consistent with those known in the literature (Sandler (*3*)(references.md#ref-3), Orbey and Sandler (*21*)(references.md#ref-21), Da Silva and Báez (*4*)(references.md#ref-4), and Smith et al. (*22*)(references.md#ref-22)).

**Keywords**: Thermodynamic Library, Critical Points, Object-Oriented Programming

---

## Table of Contents

### Front Matter
- [List of Symbols](list-of-symbols.md)
- [List of Figures](list-of-figures.md)
- [List of Tables](list-of-tables.md)

### Chapters

- **[Chapter I — Introduction](chapter-1-introduction.md)**

- **[Chapter II — Description of Vapor-Liquid Equilibrium](chapter-2-vle-theory.md)**
  - [2.1 Cubic Equations of State](chapter-2-vle-theory.md#21-cubic-equations-of-state)
    - [2.1.1 VLE Prediction using CEOS](chapter-2-vle-theory.md#211-vle-prediction-using-ceos)
    - [2.1.2 Critical Points of Multicomponent Mixtures](chapter-2-vle-theory.md#212-critical-points-of-multicomponent-mixtures)
    - [2.1.3 Energy Properties based on CEOS](chapter-2-vle-theory.md#213-energy-properties-based-on-ceos)
  - [2.2 Activity Coefficient Models](chapter-2-vle-theory.md#22-activity-coefficient-models)
    - [2.2.1 Energy Properties using Activity Coefficient Models](chapter-2-vle-theory.md#221-energy-properties-using-activity-coefficient-models)
  - [2.3 Applications of VLE Study](chapter-2-vle-theory.md#23-applications-of-vle-study)
    - [2.3.1 Bubble and Dew Point Calculations](chapter-2-vle-theory.md#231-bubble-and-dew-point-calculations)
    - [2.3.2 Flash Calculations](chapter-2-vle-theory.md#232-flash-calculations)
    - [2.3.3 Proposed Algorithms](chapter-2-vle-theory.md#233-proposed-algorithms)

- **[Chapter III — Development and Structure of the Thermodynamic Package](chapter-3-architecture.md)**
  - [3.1 Process Flows / Streams](chapter-3-architecture.md#31-process-flows)
  - [3.2 Thermodynamic Models](chapter-3-architecture.md#32-thermodynamic-models)
  - [3.3 Modeling of Pure Substances and Mixtures](chapter-3-architecture.md#33-modeling-of-pure-substances-and-mixtures)
  - [3.4 The Global VLE Problem for Multicomponent Mixtures](chapter-3-architecture.md#34-the-global-vle-problem)

- **[Chapter IV — Validation of the Thermodynamic Package](chapter-4-validation.md)**
  - [4.1 Critical Point Calculations](chapter-4-validation.md#41-critical-point-calculations)
  - [4.2 Adiabatic Flash Calculations](chapter-4-validation.md#42-adiabatic-flash-calculations)
  - [4.3 Bubble Point Pressure (Bubble P)](chapter-4-validation.md#43-bubble-point-pressure)
  - [4.4 Dew Point Calculations (Dew T and Dew P)](chapter-4-validation.md#44-dew-point-calculations)
  - [4.5 Bubble Point Temperature (Bubble T)](chapter-4-validation.md#45-bubble-point-temperature)
  - [4.6 Isothermal Flash Calculations](chapter-4-validation.md#46-isothermal-flash-calculations)
  - [4.7 Binary Interaction Parameter Calculation (kij)](chapter-4-validation.md#47-binary-interaction-parameter-calculation)

- **[Chapter V — Conclusions and Recommendations](chapter-5-conclusions.md)**

### Back Matter
- **[References](references.md)**
- **Appendices**
  - [Appendix A — Analyst Manual (Class and Module Descriptions)](appendices/appendix-a-analyst-manual.md)
  - [Appendix B �� User Manual for the Thermodynamic Library](appendices/appendix-b-user-manual.md)

---

## Spanish Original

The original Spanish text is preserved in [`docs/es/research-paper/`](../../es/research-paper/README.md) (PDFs with Markdown fallbacks).

## Related Documents

- [Modernization Plan](../../../MODERNIZATION_PLAN.md) — How this thesis is being modernized into Rust + Python
- [Pascal vs VB6 Comparison](../../../PASCAL_VB6_COMPARISON.md) — Detailed comparison of the two legacy codebases
- [Legacy VB6 Source](../../../legacy/vb6/) — Original VB6 code from this thesis
- [Legacy Pascal Source](../../../legacy/pascal/) — Original Pascal code from Da Silva & Báez (*4*)(references.md#ref-4)
