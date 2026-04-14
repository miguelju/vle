# Existen dos aspectos resaltantes en los programas computacionales:

1.  Una colección de algoritmos (instrucciones programadas para realizar
    ciertas tareas).

2.  Una colección de datos, sobre los cuales se aplican los algoritmos
    para obtener las soluciones deseadas.

Estos dos aspectos primarios, algoritmos y datos, han permanecido
invariables a través de la corta historia de la programación. Lo que se
ha desarrollado es la relación existente entre los mismos, a la cual se
le ha llamado paradigma de la programación.

En el paradigma de la programación secuencial, un problema es
directamente modelado por un conjunto de algoritmos. Desde este punto de
vista, un sistema para el cálculo del ELV de mezclas, sería representado
por una serie de procedimientos, mientras que los datos serían
almacenados separadamente, accesándolos de forma global o pasándolos
como parámetros a través de los procedimientos.

Ahora bien, recientemente el diseño de programas se ha desarrollado en
base al paradigma de las estructuras abstractas de datos (referido
generalmente como programación orientada a objetos). En este paradigma,
un problema es modelado por un conjunto de objetos abstractos
denominados *clases.* El modelaje del sistema para el cálculo del ELV
bajo esta visión, consistiría en la interacción de instancias de
*clases*, con la capacidad de representar por ejemplo, a la mezcla, los
modelos termodinámicos, las corrientes de un proceso, etc. Los
algoritmos asociados a cada *clase* son llamados la interfaz pública del
objeto, mientras que los datos son guardados de forma privada; el acceso
de los datos no es global al proyecto.

Las *clases*, dentro del ambiente de programación de Microsoft Visual
Basic^®^ son objetos que pueden poseer propiedades, eventos y métodos.
Los eventos, son las acciones que los objetos realizan luego de recibir
mensajes o instrucciones del sistema operativo. Las propiedades, son los
datos que caracterizan al objeto, y a los cuales se le aplican diversas
funciones o algoritmos, que son modelados a través de los métodos. Por
ejemplo, una sustancia o compuesto químico, posee propiedades como la
temperatura crítica, la presión crítica y el factor acéntrico. Es
importante resaltar, que las clases pueden ser agrupadas en otro tipo de
objetos denominados colecciones, por lo que, por ejemplo una mezcla
sería constituida por una colección de sustancias.

La resolución sistemática del ELV de mezclas multicomponentes, puede ser
planteada como un procedimiento compuesto por los siguientes pasos:

1.  Identificar el problema a resolver (*flash*, cálculo de la presión o
    temperatura de burbuja, etc.).

2.  Identificar las sustancias que componen la mezcla y en que
    proporción.

3.  Seleccionar los modelos termodinámicos a utilizar.

4.  Según los modelos escogidos, obtener las propiedades de los
    componentes necesarias para su aplicación.

5.  Aplicar el algoritmo de resolución.

Con el objetivo de reproducir computacionalmente y de forma natural la
visión global presentada con anterioridad, surge la necesidad de
utilizar la programación orientada a objetos, por lo cual se
desarrollaron diferentes tipos de *clases* con la capacidad de
representar:

- Flujos o corrientes de proceso.

- Modelos termodinámicos.

- Compuestos químicos y mezclas, y

- El problema global del ELV de mezclas multicomponentes.

> A continuación se explican con detalle los puntos mencionados.

**3.1 Flujos o corrientes de proceso.**

En los diversos procesos químicos existentes en la industria, es de gran
importancia la descripción de las corrientes del mismo. Una corriente,
se caracteriza mediante las siguientes propiedades: temperatura,
presión, entalpía, entropía y fracción vaporizada, además de las
proporciones en las cuales se encuentran los componentes químicos de la
misma. Así, una *clase* como la representada en la figura (3.1) es capaz
de modelar un flujo dentro de cualquier proceso químico (hasta con dos
fases líquidas).

\* Arreglo

**Figura 3.1 La *clase* de los flujos.**

**3.2 Modelos termodinámicos.**

La clasificación de los modelos termodinámicos, es uno de los factores
más importantes en el uso de objetos para la programación del problema
planteado. La clasificación de los modelos, en el proyecto desarrollado,
se realizó tomando dos aspectos generales.

- La base termodinámica y matemática de los modelos.

- La aplicabilidad del modelo (sustancias puras o mezclas). Es decir la
  clasificación a través de las necesidades del modelo.

De esta forma, se desarrollaron cinco *clases* representativas, con
métodos y propiedades, cuyas estructuras se muestran en las figuras
(3.2), (3.3), (3.4), (3.5) y (3.6).

\* Arreglo

![\*\* Colección](media/image2.emf){width="3.8055555555555554in"
height="1.9444444444444444in"}

**Figura 3.2 *Clase* de los modelos basados en los coeficientes de
actividad.**

![](media/image3.emf){width="3.8055555555555554in"
height="6.222222222222222in"}

**Figura 3.3 *Clase* de las EDEC aplicadas a sustancias puras.**

\* Arreglo

![\*\* Colección](media/image4.emf){width="3.8055555555555554in"
height="2.4444444444444446in"}

**Figura 3.4 *Clase* de las EDEC aplicadas a mezclas.**

![](media/image5.emf){width="3.8055555555555554in"
height="1.9444444444444444in"}

**Figura 3.5 *Clase* de la EVT aplicada a sustancias puras.**

\* Arreglo

![\*\* Colección](media/image6.emf){width="3.8055555555555554in"
height="1.9444444444444444in"}

**Figura 3.6 *Clase* de la EVT aplicada a mezclas.**

Por otra parte, existen correlaciones que permiten el cálculo de ciertas
propiedades de los compuestos, y que complementan el conjunto de modelos
o ecuaciones termodinámicas utilizadas en la solución de cualquier
problema de ELV. Por esto, surge la necesidad de crear una nueva *clase*
que contiene las correlaciones para el cálculo de la presión de
saturación de sustancias puras, la cual se ilustra en la figura (3.7).

![\*\* Colección](media/image7.emf){width="3.8055555555555554in"
height="1.6944444444444444in"}

**Figura 3.7 *Clase* para el cálculo de presiones de saturación de
sustancias puras.**

**3.3 Modelaje de sustancias puras y mezclas.**

Un proceso químico no puede, evidentemente, realizarse sin la presencia
de los compuestos o mezclas que se utilizarán en el mismo. De allí, la
importancia de realizar este modelaje correctamente dentro de un esquema
orientado a objetos.

Debido a la gran cantidad de propiedades y parámetros que se conocen de
una sustancia, es difícil tratar de modelar la misma a través de una
*clase* única, ya que no se estarían tomando en cuenta las necesidades
de los modelos termodinámicos como se explicó en el apartado anterior.

Entonces, se desarrollaron *clases* que contienen las propiedades
necesarias para la aplicación de ciertos modelos, para luego agruparlas
en colecciones, dando lugar al modelaje de las mezclas. Por ejemplo,
para utilizar las EDEC en la solución de problemas de sistemas
multicomponentes, es necesario conocer las temperaturas y presiones
críticas así como el factor acéntrico de todas las sustancias de la
mezcla. Es por esto, que dentro del presente proyecto existe una *clase*
que contiene las tres propiedades principales de una sustancia pura, al
igual que una colección con la capacidad de agruparlas.

Las estructuras de las *clases* que representan a una sustancia según
las necesidades de los modelos termodinámicos, se presentan en las
figuras (3.8), (3.9), (3.10) y (3.11).

![](media/image8.emf){width="2.638888888888889in"
height="1.9444444444444444in"}

**Figura 3.8 Estructura de la *clase clsCriticalProps.***

![](media/image9.emf){width="2.638888888888889in"
height="3.2083333333333335in"}

**Figura 3.9 Estructura de la *clase clsQbicsProps.***

![\* Arreglo](media/image10.emf){width="2.638888888888889in"
height="1.6944444444444444in"}

**Figura 3.10 Estructura de la *clase clsCpConsts.***

![](media/image11.emf){width="2.638888888888889in"
height="2.6944444444444446in"}

**Figura 3.11 Estructura de la *clase clsOtherProps.***

Finalmente, se hizo necesario coleccionar múltiples instancias de los
objetos ilustrados con anterioridad, lo cual se logró, mediante la
creación del objeto *clsAllProps* y cuya estructura se muestra a
continuación.

![\*\* Colección](media/image12.emf){width="2.638888888888889in"
height="2.9583333333333335in"}

Figura 3.12 Estructura de la *clase* *clsAllProps*.

**3.4 El problema global del ELV de mezclas multicomponentes.**

Una vez conocidos todos los objetos creados para la resolución de un
problema de ELV, es necesario agruparlos en uno nuevo que funcione bajo
la visión global y sistemática presentada con anterioridad. Así, se
desarrolló la *clase* denominada *clsLVE* con cuatro propiedades
fundamentales y seis métodos o funciones, mediante los cuales se
obtienen las soluciones a los problemas estudiados.

Las tres primeras propiedades, representan una corriente de
alimentación, una líquida y otra de vapor, a través de instancias
diferentes de la clase *clsFlow*, mientras que la cuarta representa a la
mezcla multicomponente a través de una única instancia de la *clase
clsAllProps*.

Por otra parte, los seis métodos, representan los algoritmos de solución
para los puntos de burbuja, los puntos de rocío, el *flash* adiabático y
finalmente el *flash* isotérmico.

El objeto, *clsLVE* puede ser visto como una caja con dos entradas como
se muestra en la figura (3.13) conteniendo seis posibilidades de
solución (métodos) y dos únicas salidas.

![](media/image13.emf){width="3.7777777777777777in"
height="2.7916666666666665in"}

**Figura 3.13 Entradas, salidas y métodos del objeto *clsLVE*.**

La *clase*, funciona y agrupa a las demás según un esquema como el
presentado en la figura (3.14). Allí se observan las relaciones
existentes entre los métodos, modelos termodinámicos, correlaciones
auxiliares, mezcla y los flujo de entrada y salida.

La mezcla, como se ha descrito en secciones anteriores, deriva de
coleccionar las propiedades que los modelos termodinámicos utilizan, los
cuales a su vez son usados por los métodos como herramientas de cálculo.

Para comprender mejor la forma en que opera el objeto *clsLVE*, cuya
estructura se muestra en la figura (3.14), suponga que se desea calcular
a cierta temperatura, y composiciones globales, la presión de burbuja de
un sistema multicomponente utilizando la EDEC de Peng y Robinson. El
primer paso sería obtener las propiedades necesitadas por el modelo,
según las relaciones del diagrama de la figura (3.15), es decir, la
temperatura crítica, la presión crítica y el factor acéntrico de cada
componente, además de los parámetros de interacción k~ij~, con lo cual
quedaría determinada la mezcla. Luego se informaría al objeto sobre la
temperatura y las composiciones globales, mediante el flujo de
alimentación, para finalmente aplicar el método adecuado (Burbuja P),
para obtener los resultados deseados a través de las corrientes de
salida.

Ahora bien, al aplicar el método, este es auxiliado por las clases que
derivan del modelo termodinámico seleccionado, en este caso una EDEC de
dos parámetros aplicada a sistemas multicomponentes
(*clsQbicsMulticomp*).

Finalmente, la descripción y forma de uso de cada una de las propiedades
y métodos presentados a lo largo de este capítulo pueden encontrarse en
los anexos A y B.

![](media/image14.emf){width="3.8055555555555554in"
height="2.4444444444444446in"}

**Figura 3.14 Estructura de la *clase clsLVE*.**

![](media/image15.emf){width="7.430555555555555in"
height="5.486111111111111in"}

**Figura 3.15 Diagrama explicativo de la *clase clsLVE***
