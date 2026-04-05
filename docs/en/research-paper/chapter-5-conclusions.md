# Chapter V — Conclusions and Recommendations

> **Translation status**: Complete. See [Spanish original](../../es/research-paper/Capitulo%20V.md).

---

## Conclusions

A thermodynamic library was developed using the object-oriented programming paradigm provided by Microsoft Visual Basic 5.0 for the calculation of properties of pure substances and multicomponent mixtures, as well as for the solution of common vapor--liquid equilibrium problems (bubble-point and dew-point calculations, together with adiabatic and isothermal flash calculations).

An interactive master program was designed to manage the thermodynamic library.

A novel algorithm was proposed for the calculation of bubble and dew points with the capability of converging in regions near the critical point.

The results obtained with the developed package are in agreement with those reported in the literature (Sandler [*3*](references.md#ref-3), Orbey and Sandler [*21*](references.md#ref-21), Da Silva and Baez [*4*](references.md#ref-4), and Smith *et al.* [*22*](references.md#ref-22)).

The developed thermodynamic library can be employed by other programs without requiring knowledge of its internal structure, owing to its object-oriented design.

## Recommendations

It is recommended that the developed thermodynamic library be applied to steady-state flow problems, such as the rigorous simulation of a distillation column.

Numerous parameters for the modifications of the Peng--Robinson and Redlich--Kwong--Soave equations of state remain to be incorporated into the existing database so that these modifications can be used in the various calculations.

Online help is an important feature of most software packages developed for the Windows environment; therefore, the creation of such tools for the master program is recommended in order to facilitate user interaction.

Finally, it is recommended that the thermodynamic library be restructured through the creation of new sub-classes that provide greater consistency to the classifications established for compound properties and thermodynamic models.

---

[Previous: Chapter IV](chapter-4-validation.md) | [Back to Table of Contents](README.md) | [References](references.md)
