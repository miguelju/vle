# Chapter I — Introduction

> **Translation status**: Pending. See [Spanish original](../../es/research-paper/Capitulo%20I.md).

---

At present, there are a large number of computer programs capable of performing vapor-liquid equilibrium (VLE) calculations for multicomponent mixtures. Programs of this type that are accessible to professors and students generally come from simple packages included in textbooks or from research and development projects generated within the university itself.

Da Silva et al. (*1*)(references.md#ref-1) indicate that the teaching of equilibrium thermodynamics has been limited to the analysis of graphs, study of certain models, and situations sufficiently ideal to be solved in classes or exams, perhaps due to the lack of adequate tools.

Furthermore, both Da Silva et al. (*1*)(references.md#ref-1) and Jackson et al. (*2*)(references.md#ref-2) mention that most programs developed by university research groups, or those obtained through textbooks, illustrate only certain types of problems using a single thermodynamic model, in addition to being user-unfriendly and lacking the ability to detect errors in input data; for example, Sandler (*3*)(references.md#ref-3) provides only the program that solves the VLE of multicomponent mixtures with the Peng-Robinson model.

There are various platforms or operating systems, as well as different programming languages with which thermodynamic packages can be developed. In 1989, Da Silva and Báez (*4*)(references.md#ref-4) developed a package with a diversity of innovations, among which the presence of a graphical user interface (GUI) and the ability to use various thermodynamic models for the solution of the most common VLE problems stand out. Unfortunately, this package was developed under the operating system provided by Apple, whose use is not currently the most common worldwide.

Thus arises the need to rescue the work carried out by Da Silva and Báez (*4*)(references.md#ref-4), in order to use their ideas under the platform of IBM-compatible personal computers and the Microsoft Windows operating system.

The project carried out not only reproduces most of the calculations of the aforementioned work, introducing improvements in the convergence of several of the algorithms, but also has the versatility of being used by other projects that need thermodynamic calculations of both pure substances and multicomponent mixtures, thanks to its development through object-oriented programming provided by the Microsoft Visual Basic programming language through the concept of *classes*.

In Chapter II, the description of the thermodynamic models and the methodology used in solving vapor-liquid equilibrium problems of multicomponent systems is presented. The development and structure of the thermodynamic package is then explained (Chapter III), to finally show the results obtained (Chapter IV) and draw conclusions (Chapter V).

---

[Back to Table of Contents](README.md) | [Next: Chapter II](chapter-2-vle-theory.md)
