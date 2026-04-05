# Chapter III — Development and Structure of the Thermodynamic Package

> **Translation status**: Complete. See [Spanish original](../../es/research-paper/Capitulo%20III.md).

---

There are two notable aspects in computer programs:

1. A collection of algorithms (programmed instructions to perform certain tasks).

2. A collection of data, upon which the algorithms are applied to obtain the desired solutions.

These two primary aspects, algorithms and data, have remained invariant throughout the short history of programming. What has evolved is the relationship between them, which has been called the *programming paradigm*.

In the sequential programming paradigm, a problem is directly modeled by a set of algorithms. From this viewpoint, a system for the calculation of multicomponent VLE would be represented by a series of procedures, while data would be stored separately, accessed globally or passed as parameters through the procedures.

More recently, however, program design has evolved around the paradigm of abstract data structures (generally referred to as object-oriented programming). In this paradigm, a problem is modeled by a set of abstract objects called *classes*. Modeling a VLE calculation system under this approach would consist of the interaction of *class* instances capable of representing, for example, the mixture, the thermodynamic models, the process streams, and so forth. The algorithms associated with each *class* are called the public interface of the object, while the data are stored privately; data access is not global to the project.

*Classes*, within the Microsoft Visual Basic programming environment, are objects that can possess properties, events, and methods. Events are the actions that objects perform upon receiving messages or instructions from the operating system. Properties are the data that characterize the object, and to which various functions or algorithms are applied, modeled through methods. For example, a substance or chemical compound possesses properties such as the critical temperature, the critical pressure, and the acentric factor. It is important to note that classes can be grouped into another type of object called a *collection*; thus, for instance, a mixture would be constituted by a collection of substances.

The systematic resolution of multicomponent VLE can be stated as a procedure composed of the following steps:

1. Identify the problem to be solved (flash, calculation of the bubble-point pressure or temperature, etc.).

2. Identify the substances composing the mixture and their proportions.

3. Select the thermodynamic models to be used.

4. According to the chosen models, obtain the component properties necessary for their application.

5. Apply the solution algorithm.

With the objective of computationally reproducing, in a natural manner, the global vision presented above, the need arose to employ object-oriented programming. To this end, different types of *classes* were developed with the capability to represent:

- Process flows or streams.
- Thermodynamic models.
- Chemical compounds and mixtures.
- The global multicomponent VLE problem.

Each of these is explained in detail in the sections that follow.

## 3.1 Process Flows

In the various chemical processes found in industry, the description of process streams is of great importance. A stream is characterized by the following properties: temperature, pressure, enthalpy, entropy, and vapor fraction, in addition to the proportions in which the chemical components are present. Thus, a *class* such as the one represented in Figure 3.1 is capable of modeling a flow within any chemical process (with up to two liquid phases).

> **Figure 3.1** — The `clsFlow` class. *(See the original Spanish document for the figure.)*

The `clsFlow` class encapsulates all the state variables that describe a single process stream. Each instance of `clsFlow` stores temperature, pressure, enthalpy, entropy, vapor fraction, and an array of component mole fractions. By creating multiple instances, the program can represent the feed, liquid product, and vapor product streams independently.

## 3.2 Thermodynamic Models

The classification of thermodynamic models is one of the most important factors in the use of objects for programming the problem at hand. In the present project, the classification of models was carried out by considering two general aspects:

- The thermodynamic and mathematical basis of the models.
- The applicability of the model (pure substances or mixtures) — that is, classification according to the requirements of the model.

In this manner, five representative *classes* were developed, with methods and properties, whose structures are shown in Figures 3.2 through 3.6.

**Activity coefficient models** — The class `clsActivityMulticomp` (Figure 3.2) implements activity-coefficient-based models for mixtures. It contains a collection of components and exposes methods for computing activity coefficients from models such as Margules, van Laar, Wilson, NRTL, and UNIQUAC, as discussed in Chapter II. Because activity coefficient models are inherently mixture models, only a multicomponent class was needed.

> **Figure 3.2** — The `clsActivityMulticomp` class: activity-coefficient models for mixtures. *(See the original Spanish document for the figure.)*

**Cubic equations of state for pure substances** — The class `clsQbicsPure` (Figure 3.3) implements cubic equations of state (CEOS) applied to pure substances. It provides methods for calculating compressibility factors, fugacity coefficients, molar volumes, and departure functions for a single component using equations such as van der Waals, Redlich-Kwong, Soave-Redlich-Kwong, and Peng-Robinson.

> **Figure 3.3** — The `clsQbicsPure` class: CEOS applied to pure substances. *(See the original Spanish document for the figure.)*

**Cubic equations of state for mixtures** — The class `clsQbicsMulticomp` (Figure 3.4) extends cubic EOS capabilities to multicomponent systems. It incorporates mixing rules and binary interaction parameters to compute mixture fugacity coefficients, which are essential for VLE calculations via the equation-of-state approach.

> **Figure 3.4** — The `clsQbicsMulticomp` class: CEOS applied to mixtures. *(See the original Spanish document for the figure.)*

**Virial equation of state for pure substances** — The class `clsVirial` (Figure 3.5) implements the truncated virial equation of state for pure substances.

> **Figure 3.5** — The `clsVirial` class: virial EOS applied to pure substances. *(See the original Spanish document for the figure.)*

**Virial equation of state for mixtures** — The class `clsVirialMulticomp` (Figure 3.6) extends the virial equation to multicomponent mixtures, using appropriate mixing rules for the second virial coefficient.

> **Figure 3.6** — The `clsVirialMulticomp` class: virial EOS applied to mixtures. *(See the original Spanish document for the figure.)*

Furthermore, there exist correlations that allow the calculation of certain compound properties, complementing the set of thermodynamic models or equations used in solving any VLE problem. For this reason, a new *class* was created containing correlations for the calculation of saturation pressures of pure substances, illustrated in Figure 3.7.

**Saturation pressure correlations** — The class `clsSatPressureSolver` (Figure 3.7) provides methods for computing the vapor pressure of a pure substance at a given temperature using correlations such as the Antoine equation and other empirical expressions. These correlations serve as auxiliary tools in VLE algorithms, particularly when the gamma-phi approach requires the pure-component saturation pressure.

> **Figure 3.7** — The `clsSatPressureSolver` class: saturation pressure calculations for pure substances. *(See the original Spanish document for the figure.)*

## 3.3 Modeling of Pure Substances and Mixtures

A chemical process evidently cannot be carried out without the presence of the compounds or mixtures to be used in it. Hence the importance of performing this modeling correctly within an object-oriented framework.

Due to the large number of properties and parameters known for a substance, it is difficult to model it through a single *class*, since the requirements of the thermodynamic models, as explained in the preceding section, would not be taken into account.

Therefore, *classes* were developed that contain the properties necessary for the application of certain models, which are then grouped into collections, giving rise to the modeling of mixtures. For example, to use cubic equations of state in solving multicomponent system problems, it is necessary to know the critical temperatures, critical pressures, and acentric factors of all substances in the mixture. For this reason, the present project includes a *class* that contains the three principal properties of a pure substance, as well as a collection capable of grouping them.

The structures of the *classes* that represent a substance according to the requirements of the thermodynamic models are presented in Figures 3.8 through 3.11.

**`clsCriticalProps`** (Figure 3.8) — Stores the critical temperature (*T*_c), critical pressure (*P*_c), and acentric factor (*omega*) for a single pure component. These three properties are required by all cubic equations of state.

> **Figure 3.8** — Structure of the `clsCriticalProps` class. *(See the original Spanish document for the figure.)*

**`clsQbicsProps`** (Figure 3.9) — Stores additional equation-of-state-specific parameters for a pure component, extending the information held by `clsCriticalProps` with data needed by particular CEOS variants and their mixing rules.

> **Figure 3.9** — Structure of the `clsQbicsProps` class. *(See the original Spanish document for the figure.)*

**`clsCpConsts`** (Figure 3.10) — Contains the polynomial coefficients for ideal-gas heat capacity correlations, stored as an array. These constants are used in enthalpy and entropy departure calculations.

> **Figure 3.10** — Structure of the `clsCpConsts` class. *(See the original Spanish document for the figure.)*

**`clsOtherProps`** (Figure 3.11) — Holds additional properties required by activity-coefficient models and other auxiliary correlations, such as liquid molar volumes and model-specific binary interaction parameters.

> **Figure 3.11** — Structure of the `clsOtherProps` class. *(See the original Spanish document for the figure.)*

Finally, it became necessary to collect multiple instances of the objects described above, which was accomplished through the creation of the `clsAllProps` object, whose structure is shown below.

**`clsAllProps`** (Figure 3.12) — This aggregating class groups collections of `clsCriticalProps`, `clsQbicsProps`, `clsCpConsts`, and `clsOtherProps` instances. A single instance of `clsAllProps` thus represents the complete property data set for an entire multicomponent mixture, organized according to the requirements of each thermodynamic model.

> **Figure 3.12** — Structure of the `clsAllProps` class. *(See the original Spanish document for the figure.)*

## 3.4 The Global VLE Problem

Once all the objects created for solving VLE problems are known, it is necessary to group them into a new object that operates under the global and systematic vision presented earlier. Thus, the *class* designated `clsLVE` was developed with four fundamental properties and six methods or functions, through which the solutions to the problems under study are obtained.

The first three properties represent a feed stream, a liquid stream, and a vapor stream, through different instances of the `clsFlow` class, while the fourth represents the multicomponent mixture through a single instance of the `clsAllProps` class.

The six methods, in turn, represent the solution algorithms for bubble points, dew points, adiabatic flash, and isothermal flash:

| Method | Description |
|---|---|
| Bubble *P* | Calculate bubble-point pressure at a given temperature |
| Bubble *T* | Calculate bubble-point temperature at a given pressure |
| Dew *P* | Calculate dew-point pressure at a given temperature |
| Dew *T* | Calculate dew-point temperature at a given pressure |
| Adiabatic flash | Determine phase split under adiabatic conditions |
| Isothermal flash | Determine phase split at fixed temperature and pressure |

The `clsLVE` object can be viewed as a box with two inputs, as shown in Figure 3.13, containing six solution possibilities (methods) and two outputs.

> **Figure 3.13** — Inputs, outputs, and methods of the `clsLVE` object. *(See the original Spanish document for the figure.)*

The class operates and groups the other classes according to a scheme such as that presented in Figure 3.14. There, one can observe the relationships among the methods, thermodynamic models, auxiliary correlations, the mixture, and the input and output flows.

The mixture, as described in previous sections, derives from collecting the properties that the thermodynamic models use, which in turn are employed by the methods as computational tools.

To better understand how the `clsLVE` object operates (whose structure is shown in Figure 3.14), suppose one wishes to calculate, at a certain temperature and set of overall compositions, the bubble-point pressure of a multicomponent system using the Peng-Robinson CEOS. The first step would be to obtain the properties required by the model, according to the relationships in the diagram of Figure 3.15 — namely, the critical temperature, critical pressure, and acentric factor of each component, as well as the binary interaction parameters *k*_ij — thereby fully specifying the mixture. Next, the object would be informed of the temperature and overall compositions via the feed stream, to finally apply the appropriate method (Bubble *P*) and obtain the desired results through the output streams.

Upon applying the method, it is assisted by the classes derived from the selected thermodynamic model — in this case, a two-parameter cubic equation of state applied to multicomponent systems (`clsQbicsMulticomp`).

Finally, the description and usage of each of the properties and methods presented throughout this chapter can be found in Appendices A and B.

> **Figure 3.14** — Structure of the `clsLVE` class. *(See the original Spanish document for the figure.)*

> **Figure 3.15** — Explanatory diagram of the `clsLVE` class. *(See the original Spanish document for the figure.)*

---

[Previous: Chapter II](chapter-2-vle-theory.md) | [Back to Table of Contents](README.md) | [Next: Chapter IV](chapter-4-validation.md)
