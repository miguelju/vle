# Chapter I — Introduction

> **Translation status**: Complete. See [Spanish original](../../es/research-paper/pdf/Capitulo%20I.pdf).

---

At present, a large number of computer programs are capable of performing vapor-liquid equilibrium (VLE) calculations for multicomponent mixtures. Programs of this type that are accessible to professors and students generally originate from simple packages bundled with textbooks or from research and development projects undertaken within the university itself.

Da Silva *et al.* [(*1*)](references.md#ref-1) note that the teaching of equilibrium thermodynamics has been limited to the analysis of graphs, the study of certain models, and situations sufficiently idealized to be solved in the classroom or on examinations, perhaps owing to the lack of adequate computational tools.

Furthermore, both Da Silva *et al.* [(*1*)](references.md#ref-1) and Jackson *et al.* [(*2*)](references.md#ref-2) observe that most programs developed by university research groups, or those distributed with textbooks, illustrate only certain types of problems through the use of a single thermodynamic model. In addition, such programs tend to be user-unfriendly and lack the ability to detect errors in input data. For example, Sandler [(*3*)](references.md#ref-3) provides only the program that solves VLE for multicomponent mixtures using the Peng-Robinson model.

Several platforms, operating systems, and programming languages are available for the development of thermodynamic software packages. In 1989, Da Silva and Baez [(*4*)](references.md#ref-4) developed a package incorporating a number of innovations, most notably a graphical user interface (GUI) and the capacity to employ various thermodynamic models for solving the most common VLE problems. Unfortunately, this package was developed for the Apple operating system, which is not the most widely used platform worldwide.

Thus arose the need to build upon the work of Da Silva and Baez [(*4*)](references.md#ref-4) so that their ideas could be applied on IBM-compatible personal computers running the Microsoft Windows operating system.

The resulting project not only reproduces the majority of the calculations from the earlier work while introducing improvements in the convergence of several algorithms, but also offers the versatility to be employed by other projects requiring thermodynamic calculations for both pure substances and multicomponent mixtures. This versatility is achieved through object-oriented programming in the Microsoft Visual Basic language, specifically through the use of *classes*.

The remainder of this work is organized as follows. Chapter II presents the thermodynamic models and the methodology used to solve vapor-liquid equilibrium problems for multicomponent systems. Chapter III describes the development and structure of the thermodynamic software package. Chapter IV presents the results obtained, and Chapter V provides the conclusions.

---

[Back to Table of Contents](README.md) | [Next: Chapter II](chapter-2-vle-theory.md)
