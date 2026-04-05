# DESCRIPCIÓN DEL EQUILIBRIO LÍQUIDO VAPOR

Una ecuación de estado (EDE), es utilizada generalmente para describir
la fase gaseosa de una mezcla (por ejemplo, la ecuación Virial truncada
hasta el segundo término), a través del coeficiente de fugacidad parcial
de los compuestos químicos en solución (*φ)*, mientras que existen dos
métodos o formas diferentes para modelar la fase líquida; o se utiliza
la misma EDE usada en la descripción de la fase vapor (modelo *φ*-*φ*),
o se emplea el método de los coeficientes de actividad. Esta última
aproximación es conocida como modelo γ-*φ* (donde γ indica que el
coeficiente de actividad ha sido utilizado para describir la fase
líquida y *φ* para la descripción de la fase de vapor, calculado a
partir de una EDE).

A continuación se describen los métodos basados en las ecuaciones de
estado cúbicas (EDEC) y los que utilizan los coeficientes de actividad.

## 2.1 Ecuaciones de Estado Cúbicas 

Las EDEC se han convertido en una herramienta de uso común para la
solución de problemas de equilibrio de fases en sistemas
multicomponentes, no sólo por su simplicidad desde el punto de vista
computacional, sino también por las buenas aproximaciones que resultan
de su uso, especialmente a altas presiones, donde otros modelos tienden
a fallar.

Entre las EDEC más conocidas se encuentran la ecuación de van der Waals,
la de Redlich y Kwong, la modificación de esta última realizada por
Soave y la propuesta por Peng y Robinson.

Desde el punto de vista computacional, el uso de una expresión general
como la presentada por Abbott \[5\]:

![](media/image1.wmf) (2.1)

facilita la programación de tales modelos, ya que, como se observa en la
tabla (2.1), los parámetros a, b, k~1~, k~2~ y k~3,~ permiten establecer
la equivalencia entre la ecuación (2.1) y las EDEC nombradas con
anterioridad.

Tabla 2.1 Ecuaciones de estado cúbicas y su relación con la forma
general de Abbott .

+-----------------------+----------+----------+----------+-----------------------+-------+---------------+
| **P(T,v)**            | **k~1~** | **k~2~** | **k~3~** | **a**                 | **b** | **EDEC**      |
+:=====================:+:========:+:========:+:========:+:=====================:+=======+:=============:+
| ![](media/image2.wmf) | 0        | 0        | 1        | a                     | ### b | van der Waals |
+-----------------------+----------+----------+----------+-----------------------+-------+---------------+
| ![](media/image3.wmf) | 1        | 0        | 1        | ![](media/image4.wmf) | b     | Redlich y     |
|                       |          |          |          |                       |       | Kwong         |
+-----------------------+----------+----------+----------+-----------------------+-------+---------------+
| ![](media/image5.wmf) | 1        | 0        | 1        | ![](media/image4.wmf) | b     | Soave         |
+-----------------------+----------+----------+----------+-----------------------+-------+---------------+
| ![](media/image6.wmf) | 2        | -1       | 1        | ![](media/image7.wmf) | b     | Peng y        |
|                       |          |          |          |                       |       | Robinson      |
+-----------------------+----------+----------+----------+-----------------------+-------+---------------+

Otra simplificación importante corresponde a la utilización de la forma
adimensional de la ecuación (2.1) mediante la expresión:

![](media/image8.wmf) (2.2)

debido a que al ser sustituida en la ecuación (2.1) produce la siguiente
expresión:

![](media/image9.wmf) (2.3)

donde:

![](media/image10.wmf) (2.4)

y

![](media/image11.wmf) (2.5)

Los parámetros en las ecuaciones (2.4) y (2.5) pueden ser encontrados
para cada una de las ecuaciones de estado cúbicas nombradas, en la tabla
(2.2).

## Es importante resaltar que las ecuaciones reportadas en la tabla (2.2) para los parámetros A y B de las EDEC, son válidas sólo en el estudio de sustancias puras. Así, con el objetivo de utilizar una EDEC para la correlación y predicción de fases en sistemas multicomponentes, se debe introducir en estos parámetros una dependencia de la composición de las especies que componen la mezcla. Esta dependencia se logra a través de las reglas de mezclado, entre las cuales, la más sencilla es la de van der Waals:

![](media/image12.wmf) (2.6)

![](media/image13.wmf) (2.7)

## 

donde:

![](media/image14.wmf) (2.8)

y los parámetros k~ij~ se pueden obtener ajustando las predicciones
realizadas a través de la EDEC a datos volumétricos y de equilibrio
experimentales, o ser considerados nulos en el caso donde la interacción
molecular entre los componentes de la mezcla sea pequeña.

# 

# 

# 

# 

# Tabla 2.2 Parámetros de las ecuaciones de estado cúbicas.

  -----------------------------------------------------------------------------------------------------------------------------------------------------------------
         **P(T,v)**                 **a**                    **b**                   **Ω~a~**                 **Ω~b~**          ![](media/image15.wmf)   **EDEC**
  ------------------------ ------------------------ ------------------------ ------------------------ ------------------------ ------------------------ -----------
   ![](media/image2.wmf)    ![](media/image16.wmf)   ![](media/image17.wmf)   ![](media/image18.wmf)   ![](media/image19.wmf)             1               van der
                                                                                                                                                           Waals

   ![](media/image3.wmf)    ![](media/image20.wmf)   ![](media/image17.wmf)   ![](media/image21.wmf)   ![](media/image22.wmf)   ![](media/image23.wmf)   Redlich y
                                                                                                                                                           Kwong

   ![](media/image24.wmf)   ![](media/image16.wmf)   ![](media/image17.wmf)   ![](media/image25.wmf)   ![](media/image26.wmf)   ![](media/image27.wmf)     Soave

   ![](media/image28.wmf)   ![](media/image16.wmf)   ![](media/image17.wmf)   ![](media/image29.wmf)   ![](media/image30.wmf)   ![](media/image31.wmf)    Peng y
                                                                                                                                                         Robinson
  -----------------------------------------------------------------------------------------------------------------------------------------------------------------

## 2.1.1 Predicción del equilibrio líquido vapor mediante el uso de las EDEC.

La predicción adecuada del equilibrio líquido vapor de mezclas, mediante
las EDEC, es de gran importancia en la actualidad, debido a su gran uso
en diversas aplicaciones industriales como el modelaje de yacimientos,
diseño de procesos, separación de gases, etc. Sin embargo, existen,
según Fotouh y Shukla \[6\], dos problemas típicos asociados con el uso
de las EDEC en tales predicciones:

- El número de fases en equilibrio no es conocido a priori.

- En regiones cercanas al punto crítico de la mezcla, los cálculos
  dependen fuertemente de los valores iniciales, de las variables
  desconocidas y del método numérico utilizado, produciendo con
  frecuencia soluciones triviales (fases en equilibrio de idéntica
  composición).

En la aplicación de las EDEC a este tipo de problemas, una sola ecuación
es utilizada para modelar las propiedades termodinámicas de todas las
fases presentes en la mezcla. El equilibrio líquido vapor se logra
cuando se alcanzan las condiciones de estabilidad mecánica y térmica,
además de cumplir con los balances de masa del sistema. Las
estabilidades térmica y mecánica se cumplen cuando la temperatura y la
presión respectivamente, son iguales en todas las fases. Por otra parte
el cumplimiento de los balances de masa puede ser expresado en términos
de la energía libre de Gibbs de la mezcla, la cual alcanza un mínimo
global en el equilibrio.

A lo largo de los años, se han desarrollado diferentes métodos con el
fin de predecir el equilibrio de fases en mezclas. Entre estos métodos,
se encuentran:

- La minimización de la energía de Gibbs propuesto por Michelsen \[7\],
  en la cual se minimiza la función de esta energía de la mezcla
  respecto al número de moles o composición de los componentes en el
  fluido.

<!-- -->

- El método de las áreas propuesto por Eubank *et al*. \[8\], que
  consiste en calcular dos composiciones de la mezcla, tales que, la
  diferencia existente entre el área absoluta debajo de la línea recta
  que conecta estas composiciones, y el área de la curva de la energía
  de Gibbs existente entre las mismas, se encuentre en un máximo
  positivo.

<!-- -->

- El método de las fugacidades.

Este último, que por su sencillez, es el de uso más común, se expresa
mediante la siguiente ecuación:

![](media/image32.wmf) (i=1,2, \...,N) (2.9)

Definiendo el coeficiente de fugacidad de la especie i en solución como:

![](media/image33.wmf) (2.10)

la ecuación (2.9) es equivalente a:

![](media/image34.wmf) (2.11)

Con fines de cálculo la ecuación (2.11) puede ser escrita de la
siguiente forma

![](media/image35.wmf) (2.12)

donde K~i~ es el coeficiente de equilibrio y está dado por:

![](media/image36.wmf) (2.13)

La combinación de las ecuaciones (2.12) y (2.13) se conoce con el nombre
del modelo *φ−φ* para la aproximación al equilibrio termodinámico de
sistemas multicomponentes, cuyo procedimiento de cálculo está basado en
la resolución simultánea del sistema de ecuaciones representado por la
ecuación (2.9).

Expresiones para los coeficientes de fugacidad en función de la presión
(o el volumen), la temperatura y composición del sistema deben ser
derivadas específicamente para cada EDEC y regla de mezclado utilizada.
En tal sentido, la expresión general propuesta por Müller *et al*. \[9\]
para coeficientes de fugacidad multicomponente, facilita el cálculo de
esta propiedad. Sin embargo, la evaluación de las derivadas
composicionales presentes en algunos de los términos de tal expresión
puede ser difícil, debido a los grandes esfuerzos analíticos y
algebraicos que deben realizarse para su cálculo. Estas dificultades
pueden ser solventadas evaluando este tipo de derivadas en forma
numérica como fue propuesto por Stockfleth y Dohrn \[10\]; la desventaja
de utilizar este tipo de método es que disminuye la rapidez de cálculo,
pero permite calcular las derivadas de los coeficientes de fugacidad de
cualquier modelo de EDEC y regla de mezclado.

A pesar de que el método de fugacidades satisface los balances de masa
del sistema, este puede presentar fallas en la predicción adecuada del
número de fases o simplemente provocar la obtención de soluciones
triviales. Este hecho se debe a que la igualdad de las fugacidades es
una condición necesaria pero no suficiente para la existencia de un
mínimo global en la energía de Gibbs del sistema, como lo describió Null
\[11\].

En particular, en la región crítica de la mezcla, las soluciones a los
problemas del ELV dependen considerablemente del valor inicial de las
composiciones desconocidas de la mezcla. Además, las técnicas numéricas
utilizadas para resolver el sistema de ecuaciones no lineales pueden
llevar a obtener mínimos locales y no el mínimo global de la energía de
Gibbs de la mezcla.

Se han planteado, en la literatura, diversos algoritmos basados en el
método de las fugacidades con el fin de evitar las soluciones triviales
y lograr buenas aproximaciones en zonas cercanas al punto crítico de la
mezcla. Así, por ejemplo, Poling y Prausnitz \[12\] y Gurdensen \[13\],
presentan dos algoritmos distintos para la obtención de los volúmenes de
las fases en equilibrio de forma adecuada, mientras que Asselineau *et
al*. \[14\] presentaron un algoritmo basado en el método numérico de
Newton-Raphson multivariable, para poder predecir adecuadamente el
equilibrio de fases en la zona crítica de las mezclas.

**2.1.2 Puntos críticos de mezclas multicomponentes.**

La predicción de las propiedades críticas reales de un sistema
multicomponente es un aspecto importante en el problema del cálculo de
fases, debido a que a partir de estos datos se puede predecir, por
ejemplo, la existencia del fenómeno conocido como condensación
retrograda.

En la literatura, existen diversos métodos para la predicción de puntos
críticos de mezcla, dentro de los cuales se pueden mencionar el
propuesto por Peng y Robinson \[15\] que, utilizando el criterio
enunciado por Gibbs, logra obtener una solución al problema de la
predicción de las propiedades críticas de una mezcla utilizando una
ecuación de estado cúbica de dos parámetros.

Otro método alternativo fue presentado por Heidemann y Khalil \[16\],
que se origina también en el criterio de Gibbs. El algoritmo de cálculo
está basado en el estudio de la estabilidad de las fases existentes. La
estabilidad puede ser descrita en términos matemáticos por las
siguientes ecuaciones:

![](media/image37.wmf) (2.14)

![](media/image38.wmf) (2.15)

En la ecuación (2.14) la presión P~o~ y los potenciales químicos μ~io~
son evaluados en un punto de prueba (estado inicial) y A-A~o~ es la
diferencia entre la energía libre de Helmholtz entre el estado inicial y
un estado subsiguiente. Si las ecuaciones (2.14) y (2.15) no son
satisfechas para cualquier cambio de fase en una región alrededor del
punto de prueba, existe una energía interna menor, accesible a la mezcla
mediante su separación en dos o más fases.

La energía libre de Helmholtz puede ser expandida en series de Taylor
alrededor de un punto tomando constante el volumen, obteniéndose la
siguiente expresión:

![](media/image39.wmf)

![](media/image40.wmf) (2.16)

La estabilidad del punto de prueba requiere que la ecuación anterior sea
positiva para cualquier valor arbitrario de ![](media/image41.wmf).

Por otra parte, para que un punto esté en el límite de estabilidad debe
cumplirse que el determinante de la matriz **Q** con elementos

![](media/image42.wmf) (2.17)

sea igual a cero :

Det(**Q**)=0 (2.18)

o que exista un vector

![](media/image43.wmf) (2.19)

que cumpla con la ecuación

**Q**![](media/image44.wmf) (2.20)

Un punto crítico es aquel que se encuentra en el límite de estabilidad,
lo cual significa, desde el punto de vista matemático, que existe un
vector ![](media/image45.wmf) que satisface la ecuación (2.20) y que
este vector, al ser sustituido en la ecuación (2.16) anule el término
cúbico, es decir:

![](media/image46.wmf) (2.21)

A partir de las ecuaciones (2.18), (2.20) y (2.21) Heidemann y Khalil
\[16\] plantearon un procedimiento para el cálculo de puntos críticos
como el que se observa en el diagrama de flujo que se muestra en la
figura (2.1). En el caso de ecuaciones de estado cúbicas, las derivadas
de la energía de Helmholtz que se encuentran en las ecuaciones (2.16) y
(2.21), pueden ser calculadas mediante las siguientes ecuaciones:

![](media/image47.wmf) (2.22)

![](media/image48.wmf) (2.23)

donde las derivadas de las fugacidades de primer y segundo orden, pueden
ser evaluadas numéricamente como se mencionó en la sección anterior.

![](media/image49.wmf)

# 

# Figura 2.1 Algoritmo para el cálculo del punto critico de una mezcla

**2.1.3 Propiedades energéticas basadas en las EDEC.**

En el cálculo de las propiedades energéticas mediante el uso de las
EDEC, es necesario introducir el concepto de propiedad residual:

![](media/image50.wmf) (2.24)

donde M es el valor de la propiedad termodinámica del fluido y M^gi^ es
el valor que la misma propiedad tendría si el fluido se comportara como
un gas ideal a la misma temperatura y presión de M.

Tomando la expresión básica que relaciona la energía de Gibbs, con la
temperatura, presión y potenciales químicos:

![](media/image51.wmf) (2.25)

es posible obtener las expresiones para la entalpía y entropía residual:

![](media/image52.wmf) (2.26)

![](media/image53.wmf) (2.27)

Müller *et al.* \[9\] presentaron expresiones generales para éstas y
otras propiedades residuales, que permiten su cálculo para cualquier
EDEC con k~3~=1 y regla de mezclado utilizada.

![](media/image54.wmf) (2.28)

![](media/image55.wmf) (2.29)

donde:

![](media/image56.wmf) (2.30)

y

![](media/image57.wmf) si ![](media/image58.wmf) (2.31)

![](media/image59.wmf) si ![](media/image60.wmf) (2.32)

![](media/image61.wmf) si ![](media/image62.wmf) (2.33)

![](media/image63.wmf) (2.34)

Las ecuaciones (2.28) y (2.29), junto con las expresiones conocidas para
el cálculo de las capacidades calóricas de gases ideales, permiten el
cálculo de la entalpía y entropía de una fase del sistema
multicomponente, tomando como estado de referencia el gas ideal para
cada uno de los componentes de la mezcla (T~ref~, P~ref~, H^o^ y S^o^):

![](media/image64.wmf) (2.35)

![](media/image65.wmf) (2.36)

## 2.2 Modelos de Coeficientes de Actividad.

El uso apropiado de una EDEC, ofrece una manera conveniente para el
cálculo del equilibrio de fases. Sin embargo, las EDEC simples son sólo
aplicables a mezclas de moléculas que no poseen interacciones
específicas fuertes, y tienden a fallar en la predicción de las
propiedades de la fase líquida (Assael *et al.* \[17\]). En tales casos,
se obtienen mejores resultados cuando la fugacidad de cada componente en
la fase líquida es estimada mediante un modelo de coeficiente de
actividad.

La actividad ![](media/image66.wmf) de una sustancia i en una mezcla de
n componentes, es definida como:

![](media/image67.wmf) (2.37)

y se expresa en términos del correspondiente coeficiente de actividad:

![](media/image68.wmf) (2.38)

donde ![](media/image69.wmf)es la fugacidad del líquido i puro a la
temperatura del sistema y una presión de referencia.

El fin del uso de un modelo de coeficiente de actividad, es representar
la dependencia de γ~i~ con la temperatura, presión y composición del
sistema. El efecto de la presión es tomado en cuenta sabiendo que:

![](media/image70.wmf) (2.39)

donde ![](media/image71.wmf)es el volumen molar parcial del componente
i.

La fugacidad de un componente en la mezcla puede ser entonces escrita
como:

![](media/image72.wmf) (2.40)

donde F~i~ es el factor de Poynting que viene dado por la siguiente
expresión:

![](media/image73.wmf) (2.41)

Finalmente si se usa la ecuación (2.10) para describir la fase vapor de
la mezcla, y la ecuación (2.39) para describir la fase líquida, se
obtiene la expresión conocida como el modelo γ-*φ* para la aproximación
termodinámica del equilibrio líquido vapor de sistemas multicomponentes:

![](media/image74.wmf) (2.42)

donde el coeficiente de actividad debe ser calculado mediante el uso de
un modelo termodinámico, la presión de saturación a través una
correlación y ![](media/image75.wmf)con la misma EDE utilizada en la
descripción de la fase gaseosa. En tal sentido en la tabla (2.3) se
pueden observar las diferentes expresiones para los coeficientes de
actividad, según los modelos de Wilson, Scatchard-Hildebrand, Margules,
van Laar, siendo los dos últimos sólo válidos para mezclas binarias.

Tabla 2.3 Modelos basados en el coeficiente de actividad.

+--------------------------+------------------------+------------------------+
| **Modelo**               | **Parámetros**         | ![](media/image76.wmf) |
+==========================+:======================:+:======================:+
| #### Wilson              | ![](media/image77.wmf) | ![](media/image78.wmf) |
+--------------------------+------------------------+------------------------+
| ### Scatchard-Hildebrand | ![](media/image79.wmf) | ![](media/image80.wmf) |
+--------------------------+------------------------+------------------------+
| Margules                 | ![](media/image81.wmf) | ![](media/image82.wmf) |
+--------------------------+------------------------+------------------------+
| van Laar                 | ![](media/image83.wmf) | ![](media/image85.wmf) |
|                          |                        |                        |
|                          | ![](media/image84.wmf) |                        |
+--------------------------+------------------------+------------------------+

Finalmente, es de importancia señalar, que los volúmenes líquidos
necesarios para el cálculo tanto del factor de Poynting, como de las
ecuaciones de Wilson y Scatchard-Hildebrand, pueden ser obtenidos a
través del uso de correlaciones, como la presentada por Hankinson y
Thomson \[18\].

**2.2.1 Propiedades energéticas mediante el uso de modelos de
coeficiente de actividad.**

Para lograr la predicción de propiedades energéticas mediante el uso de
los modelos basados en el coeficiente de actividad, es necesario
introducir el concepto de propiedad en exceso. Una propiedad en exceso
se define como la diferencia entre el valor de una propiedad
termodinámica (M~i~) y el valor de la misma, si el fluido se comportara
como una solución ideal (M~i~^SI^), a las mismas condiciones de
temperatura, presión y composición. Por lo tanto:

![](media/image86.wmf) (2.43)

donde M~i~^E^, es la propiedad en exceso del componente i en solución.

Una vez conocido el concepto de propiedad en exceso, y análogamente al
caso en que se definen las propiedades residuales, el uso de ecuaciones
fundamentales, conduce a la siguiente formulación:

![](media/image87.wmf) (2.44)

a partir de la cual se obtienen las expresiones de la entalpía y
entropía en exceso:

![](media/image88.wmf) (2.45)

![](media/image89.wmf) (2.46)

Como se observa en las ecuaciones (2.44), (2.45) y (2.46), una vez
escogido el modelo de actividad, el cálculo de las propiedades en exceso
es bastante sencillo, ya que la energía de Gibbs en exceso (G^E^) es
función directa de los coeficientes de actividad, y la derivada presente
en las expresiones para la entalpía y entropía en exceso puede ser
calculada numérica o analíticamente, según la complejidad del modelo
utilizado.

Finalmente, al definir un estado de referencia de gas ideal común a
todas las especies presentes en la mezcla, la entalpía y entropía de la
fase líquida pueden ser calculadas mediante las siguientes fórmulas:

![](media/image90.wmf)(2.47)

![](media/image91.wmf)

![](media/image92.wmf) (2.48)

**2.3 Aplicaciones del estudio del equilibrio líquido vapor (ELV).**

La predicción del equilibrio de fases es uno de los problemas centrales
de la termodinámica en ingeniería química, ya que es un aspecto de gran
importancia en el diseño de procesos químicos. La cantidad infinita de
posibles mezclas, y el amplio rango de temperaturas y presiones
encontradas en los procesos industriales es tal que ningún modelo
termodinámico en la actualidad, es aplicable a todos los casos.

A continuación, se presentan varios algoritmos que permiten el cálculo
del ELV de mezclas.

1.  **Cálculo de puntos de burbuja y rocío.**

La base de la mayoría de los cálculos de equilibrio de fases, es un
algoritmo con la capacidad de calcular el punto de burbuja o el punto de
rocío para una mezcla de determinada composición. Por lo general, los
cálculos de estos puntos se realizan a través de algoritmos con dos
lazos de cálculo. El lazo externo, que controla la temperatura o presión
desconocida del sistema y el interno, mediante el que se ajustan las
composiciones incógnitas del proceso.

En la figura (2.2), se presenta un diagrama de flujo que representa un
algoritmo genérico para el cálculo de puntos de burbuja de sistemas
multicomponentes.

El algoritmo que generalmente se utiliza para el cálculo del punto de
rocío, es similar al anterior (figura (2.2)). En este caso la
composición de vapor es conocida, por lo que el lazo interno de cálculo
se realiza para ajustar las composiciones de la fase líquida.

Es importante resaltar que en tales algoritmos, la nueva temperatura o
presión en cada iteración puede ser calculada a través de distintos
métodos numéricos, entre los cuales se encuentran, por ejemplo, los
presentados por Da Silva *et al*. \[4\] (extrapolación parabólica) y
Asselineau *et al*. \[14\] (Newton-Raphson multivariable).

2.  **Cálculos de evaporación instantánea (*flash*)*.***

La simulación de los procesos de evaporación instantánea, es una de las
aplicaciones más importantes de la termodinámica en ingeniería química.
En este proceso, una corriente de flujo cuyas composiciones globales son
conocidas, pasa a través de un "tambor" donde la fase vapor y la líquida
son separadas. Tal proceso puede ser operado bajo diferentes
condiciones, entre las cuales se encuentran:

- Temperatura y presión constantes (*Flash* isotérmico)

- Presión y entalpía constantes (*Flash* isentálpico)

![Figura 2.2 Algoritmo genérico para el cálculo de puntos de burbuja de
sistemas multicomponentes (Assael *et al.*
\[17\]).](media/image93.emf){width="6.236111111111111in"
height="5.791666666666667in"}

En el proceso ilustrado en la figura (2.3) la alimentación a la
temperatura T~F~ y la presión P~F~, pasa a través de una válvula, para
luego entrar al "tambor" de evaporación instantánea donde las fases se
separan. La presión P de la unidad es controlada, y el calor Q se
transfiere mediante un intercambiador de calor, de manera que la
temperatura permanezca constante. De esta forma los cálculos
relacionados con el *flash* isotérmico tienen como objetivo fundamental
determinar las composiciones (y~I~, x~i~) y la fracción vaporizada (β)
del flujo de entrada F.

![](media/image94.emf){width="3.3194444444444446in" height="1.875in"}

**Figura 2.3 *Flash* Isotérmico.**

En el *flash* isotérmico, debido a que sólo se conocen las composiciones
globales de la mezcla, los balances de masa se han de expresar de forma
que la dependencia de los flujos desaparezca, obteniéndose la ecuación
de Rachford-Rice:

![](media/image95.wmf) (2.49)

ecuación que debe ser resuelta por un método numérico para obtener como
resultado β.

Michelsen \[19\], plantea como primer paso para la resolución de un
*flash* el estudio de la estabilidad de las fases mediante el uso del
criterio del plano tangente de Gibbs, para así evitar las soluciones
triviales. Luego propone un algoritmo cuyo punto de partida son los
valores obtenidos como resultado del análisis de estabilidad, para luego
obtener soluciones rápidas, aún cerca del punto crítico de la mezcla,
utilizando un método de convergencia de segundo orden.

Otros algoritmos han sido propuestos en la literatura como el presentado
por Asselineau *et al*. \[14\], en el cual se utiliza como punto de
partida, en determinados casos, valores de temperatura o presión
obtenidos mediante el cálculo del punto pseudocrítico de la mezcla. Sin
embargo el método de resolución más empleado, debido a la facilidad de
su aplicación, es el que se muestra en la figura (2.4) (Assael *et al*
\[17\]).

Por otra parte, el *flash* isentálpico es ilustrado en la figura (2.5).
El fin es conseguir la temperatura, la fracción vaporizada (β) y las
fracciones molares de cada fase resultante del proceso.

![Figura 2.4 Algoritmo para el cálculo del *flash*
isotérmico.](media/image96.emf){width="1.7777777777777777in"
height="4.694444444444445in"}

![](media/image97.emf){width="3.8194444444444446in"
height="2.388888888888889in"}

# Figura 2.5 *Flash* isentálpico.

Uno de los algoritmos empleados para la resolución este tipo de
problemas, se muestra en la figura (2.6), donde se observa que se ha
impuesto una condición de operación adiabática (Q=0).

![](media/image98.emf){width="2.3055555555555554in"
height="3.2777777777777777in"}

# Figura 2.6 Algoritmo para el cálculo del *flash* adiabático.

En caso contrario a una condición adiabática en el proceso, la función
objetivo del algoritmo presentado en la figura (2.6) sería determinada
por la expresión del balance energético del sistema:

![](media/image99.wmf) (2.50)

donde el calor es una variable conocida.

1.  **Algoritmos propuestos.**

Con el fin de evitar las soluciones triviales y garantizar la
convergencia en zonas cercanas al punto crítico de la mezcla, se
desarrolló un método para el cálculo del punto de burbuja o rocío,
basado en la combinación de varios algoritmos mencionados o presentados
en secciones anteriores.

El algoritmo desarrollado está compuesto de dos etapas:

1.  Primera etapa: se utiliza un algoritmo basado en el propuesto por Da
    Silva y Báez \[4\], similar al presentado en la figura (2.2), con el
    fin de obtener soluciones rápidas. En caso de obtener una solución
    trivial se procede a la segunda etapa.

2.  Segunda etapa: se calcula un punto de burbuja o rocío a una presión
    o temperatura baja, por debajo del punto pseudocrítico (a partir del
    cual la EDEC presenta una sola raíz real) con el fin de garantizar
    la convergencia del método propuesto por Asselineau *et al*. \[14\].
    Una vez conseguido este punto, se estima otro de mayor magnitud (en
    cierto ![](media/image100.wmf) o ![](media/image101.wmf)) tomando
    como valores iniciales los del punto anterior. Entonces, calculando
    la pendiente entre los mismos, se logra estimar el siguiente, a
    través de las ecuaciones que se presentan a continuación, donde las
    derivadas son calculadas en forma numérica:

![](media/image102.wmf) (2.51)

![](media/image103.wmf) (2.52)

> con el fin de ir recorriendo el domo en forma diferencial y así lograr
> obtener el punto deseado.

La segunda etapa del proceso planteado, se ilustra en la figura (2.7)
como fue presentado por Anderson y Prausnitz \[20\]:

![](media/image104.emf){width="3.388888888888889in"
height="2.8055555555555554in"}

**Figura 2.7 Segunda etapa del algoritmo planteado.**

donde a representa el punto de partida del algoritmo y b el valor final
deseado.
