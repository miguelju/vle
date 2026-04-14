# Desarrollo de un Programa Computacional para el Cálculo del Equilibrio Líquido Vapor de Mezclas Multicomponentes bajo el Ambiente Windows

**Autores**: Miguel Roberto Jackson Ugueto y Carlos Fernando Mendible Porras

**Proyecto de Grado**, presentado a la Universidad Simón Bolívar como requisito parcial para optar al título de Ingeniero Químico.

**Tutores**: Prof. Coray M. Colina y Prof. Jean-Marie Ledanois

**Fecha**: Sartenejas, abril de 1999

**Aprobado con Mención de Honor**

---

## Resumen

Se desarrolló una librería termodinámica y un programa maestro con la capacidad de utilizar la librería. La librería termodinámica permite el cálculo del ELV mediante la combinación de distintos modelos, incluyendo ecuaciones de estado cúbicas (EDEC), la ecuación del Virial truncada y modelos basados en coeficientes de actividad. La librería incluye los algoritmos comúnmente utilizados en el estudio del ELV de sistemas multicomponentes: cálculos de puntos de burbuja y rocío, cálculos de evaporación instantánea (*flash*) y el cálculo de puntos críticos mediante EDEC. Como la librería fue diseñada bajo el paradigma de programación orientada a objetos provisto por Microsoft Visual Basic, puede ser fácilmente utilizada dentro de otros proyectos. Los resultados obtenidos son completamente consistentes con los conocidos en la literatura.

**Palabras clave**: Librería Termodinámica, Puntos Críticos, Programación Orientada a Objetos

---

## Tabla de Contenido (PDFs)

### Material Preliminar
- [Índice general](pdf/Indice%20general.pdf)
- [Lista de Símbolos](pdf/Lista%20de%20simbolos.pdf)
- [Lista de Tablas](pdf/Lista%20de%20Tablas.pdf)
- [Lista de Figuras](markdown/Lista%20de%20Figuras.md) *(solo disponible en Markdown)*
- [Portada](markdown/Portada.md) *(solo disponible en Markdown)*
- [Resumen](markdown/Resumen.md) *(solo disponible en Markdown)*

### Capítulos

- **[Capítulo I — Introducción](pdf/Capitulo%20I.pdf)**

- **[Capítulo II — Descripción del Equilibrio Líquido Vapor](pdf/Capitulo%20II.pdf)**
  - 2.1 Ecuaciones de Estado Cúbicas
    - 2.1.1 Predicción del ELV mediante el uso de las EDEC
    - 2.1.2 Puntos Críticos de Mezclas Multicomponentes
    - 2.1.3 Propiedades Energéticas basadas en las EDEC
  - 2.2 Modelos de Coeficientes de Actividad
    - 2.2.1 Propiedades Energéticas mediante Modelos de Coeficiente de Actividad
  - 2.3 Aplicaciones del Estudio del ELV
    - 2.3.1 Cálculo de Puntos de Burbuja y Rocío
    - 2.3.2 Cálculos de Evaporación Instantánea (*flash*)
    - 2.3.3 Algoritmos Propuestos

- **[Capítulo III — Desarrollo y Estructura del Paquete Termodinámico](pdf/Capitulo%20III.pdf)**
  - 3.1 Flujos o Corrientes de Proceso
  - 3.2 Modelos Termodinámicos
  - 3.3 Modelaje de Sustancias Puras y Mezclas
  - 3.4 El Problema Global del ELV de Mezclas Multicomponentes

- **[Capítulo IV — Convalidación del Paquete Termodinámico](pdf/Capitulo%20IV.pdf)**
  - 4.1 Cálculos de Puntos Críticos
  - 4.2 Cálculos de *flash* Adiabático
  - 4.3 Cálculo del Punto de Burbuja conocida la Temperatura (Burbuja P)
  - 4.4 Cálculo del Punto de Rocío (Rocío T y Rocío P)
  - 4.5 Cálculo del Punto de Burbuja conocida la Presión (Burbuja T)
  - 4.6 Cálculos de *flash* Isotérmico
  - 4.7 Cálculo de Parámetros de Interacción (kij)

- **[Capítulo V — Conclusiones y Recomendaciones](pdf/Capitulo%20V.pdf)**

### Material Final
- **[Referencias](pdf/REFERENCIAS.pdf)**
- **Anexos** *(solo disponibles en Markdown)*
  - [Anexo A — Manual del Analista (Descripción de las clases y módulos)](markdown/programdocs/Analista.md)
  - [Anexo B — Manual de Usuario de la Librería Termodinámica](markdown/programdocs/dllManual.md)

---

## Fuentes Markdown

Las transcripciones en Markdown del texto original (utilizadas para la traducción al inglés) se conservan en [`markdown/`](markdown/).

## Traducción al Inglés

La traducción navegable al inglés está disponible en [`docs/en/research-paper/`](../../en/research-paper/README.md).
