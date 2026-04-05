ANEXO B

MANUAL DE USUARIO DE LA LIBRERÍA TERMODINÁMICA

B.1 Instalación de la librería.

B.1.1 Instalación en Microsoft VBA

1.  Abra Microsoft VBA desde cualquier programa de Microsoft Office que
    tenga acceso a éste, como por ejemplo, Microsoft^®^ Excel.

2.  En el menú Principal seleccione el menú Herramientas y seguidamente
    el menú Referencias.

3.  En la pantalla, seleccione el botón Examinar, busque el directorio
    donde se encuentra el archivo thrmStateSolver.dll y selecciónelo.

4.  Finalmente presione el botón aceptar y la librería será instalada.

B.1.2 Instalación en Microsoft VB

1.  Una vez abierto Microsoft VB, escoja en el menú Principal,
    seleccione el menú Proyecto y seguidamente el menú Referencias.

2.  En la pantalla, escoja ahora el botón Examinar, y busque el
    directorio donde se encuentra el archivo thrmStateSolver.dll y
    selecciónelo.

3.  Finalmente presione el botón aceptar y la librería será instalada.

B.2 Uso de las clases disponibles a través de la librería.

B.2.1 clsSatPressure

Esta clase permite el cálculo de presiones y temperaturas de saturación
a través de correlaciones existentes en la literatura (Riedel, Muller,
RPM).

Como primer paso se debe declarar una instancia de la clase y luego
llenarla con todas aquellas propiedades necesarias para el cálculo de la
presión de saturación de un compuesto mediante las correlaciones
disponibles. A continuación se muestran las líneas de código necesarias
para la utilización de esta clase :

Dim Sat as new clsSatPressure

Sat.CriticalConstants.Add Tc, Pc, w, Zc

Sat.OtherConstants.Add 0, 0, Tb, 0, 0, 0, 0, 0, 0

Ahora debe seleccionar el modelo como se muestra en la Figura 1.

![](media/image1.jpeg){width="2.486111111111111in"
height="0.8055555555555556in"}

Finalmente existen tres métodos que se pueden aplicar:

- Para calcular la presión de saturación de un componente i, debe
  utilizar la siguiente línea de código:

> Sat.SatPressure i, T, Psat
>
> donde T es la temperatura del sistema, y Psat la variable donde se
> obtendrá el resultado.

- Para calcular la temperatura de saturación de un componente i, debe
  utilizar la siguiente línea de código:

> Sat.SatTemperature i, P, Tsat
>
> donde P es la temperatura del sistema, y Tsat la variable donde se
> obtendrá el resultado.

- Para calcular la derivada de la presión de saturación de un componente
  i a una temperatura (T), debe utilizar la siguiente línea de código:

> dP = Sat.DSatPressure_T(i, T)
>
> donde dP es la variable donde se obtendrá el resultado.

B.2.2 clsVirial

Esta clase permite el cálculo de factores de compresibilidad,
coeficientes de fugacidad y propiedades energéticas residuales a través
de la ecuación Virial truncada en el segundo término.

Como primer paso se debe declarar una instancia de la clase y luego
introducir el valor del factor acéntrico de la sustancia utilizada:

Dim Vir As New clsVirial

Vir.AcentricFactor = w

Existen tres métodos que se pueden aplicar a esta clase:

- Para calcular el factor de compresibilidad, debe utilizar la siguiente
  línea de código: Z=Vir.CalculateZ(Tr, Pr)

<!-- -->

- Para calcular el coeficiente de fugacidad, debe utilizar la siguiente
  línea de código: fi=Vir.CalculateFugacity(Tr, Pr)

- El método HR_SR(Tr,Pr, Hr, Sr) calcula las propiedades residuales,
  tomando como valores la presión y temperatura reducida, y las
  variables donde van a se guardados los valores de las propiedades.

B.2.3 clsVirialMulticomp

La clase VirialMulticomp es la que se utiliza en el caso de mezclas
multicomponentes, en general el esquema de aplicación es similar al de
las clases anteriores es decir, se definen los parámetros que se
necesitan para los cálculos y luego se calculan las funciones deseadas.
A continuación se presenta unas líneas de código representativas del
funcionamiento de la clase para el caso de dos componentes:

Dim VirMC As New clsVirialMulticomp

'Parte donde se fijan los parámetros

VirMC.CriticalProperties.Add Tc1, Pc1, w1, 0

VirMC.CriticalProperties.Add Tc2, Pc2, w2, 0

VirMC.MolarFraction(1) = y1

VirMC.MolarFraction(1) = y2

Set VirMC.KijParameters = a(i,j)

'Parte de cálculos deseados

fi = VirMC.CalculatePartialFugacity(i, T, P)

Z = VirMC.CalculateZ(T, P)

HR = VirMC.HR(P, T)

SR = VirMC.SR(P, T)

B.2.4 clsQbicsPure

Esta clase como su nombre lo indica es para el cálculo de: residuales
(HR,SR), factor de compresibilidad así como el equilibrio líquido vapor
de sustancias puras. Esta clase tiene una gran cantidad de métodos,
debido a la variedad de combinaciones posibles de datos. A partir de la
especificación de dos variables independientes (P,T ó v) se puede
determinar el estado de la sustancia. También existe la posibilidad de
utilizar dos adimensionalizaciones distintas para la EDEC utilizada, las
funciones que contienen vr y otras para Z. A continuación se presenta
unas líneas de código representativas de la aplicación del criterio de
Maxwell:

Dim Qbic as New clsQbicsPure

Dim pr1 As Double, zf As Double, zg As Double, a As Integer

'Qbic.Model= PR1976_Qbic

Propiedades que necesita el modelo

Qbic.AcentricFactor = w

Qbic.MaxwellTestGivenTr(tr, pr1, zg, zf)

PrMaxwell = pr1

B.2.5 clsQbicsMulticomp

Esta clase permite el cálculo de propiedades para mezclas a partir de
las ecuaciones de estado cúbicas (también incluye secciones para el
cálculo de puntos críticos y de los parámetros de interacción binarios),
la forma general del planteamiento de cálculo es similar a los ejemplos
anteriores. A continuación se muestran, unas líneas de código
representativas del cálculo de un punto crítico:

Dim QBicMulticomp As New clsQbicsMulticomp

QBicMulticomp.Properties.Add Tc1, Pc1, w1, zc1, 0, 0, 0, 0

QBicMulticomp.Properties.Add Tc1, Pc1, w2, zc2, 0, 0, 0, 0

QBicMulticomp.MolarFraction(1)=x1

QBicMulticomp.MolarFraction(2)=x2

QBicMulticomp.CriticalProperties Tc, Pc, vc

B.2.6 clsLVE

Esta clase encierra todas las posibles combinaciones del cálculo del
equilibrio liquido vapor de mezclas, mediante los modelos γ-*φ* y *φ-φ*.
El funcionamiento general de la clase es el siguiente: las propiedades
necesarias se definen a través de Mixture, luego de especificar. A
continuación se muestra un ejemplo típico para el cálculo de punto de
burbuja:

Private LVE As New clsLVE

'Esta sección es común para todos los cálculos, aquí se llenan las
propiedades para el cálculo

LVE.Tolerances.MaxIter = 30

LVE.Tolerances.TolMolarFraction = 0.0001

LVE.Tolerances.TolTemperature = 0.0001

LVE.Tolerances.TolPressure = 0.0001

LVE.ReferenceState.TTo = 273.15

LVE.ReferenceState.Po = 10325

LVE.ReferenceState.Ho = 0

LVE.ReferenceState.So = 0

LVE.Mixture.NumberOfComponents =2

LVE.Mixture.CriticalProps.Add Tc1, Pc1, w1, Zc1

LVE.Mixture.CriticalProps.Add Tc2, Pc2, w2, Zc2

LVE.Mixture.OtherProps.Add 0, Tb1, 0, 0, 0, 0, 0, 0

LVE.Mixture.OtherProps.Add 0, Tb2, 0, 0, 0, 0, 0, 0

LVE.Mixture.SatPressureModel = RPM

LVE.Mixture.vaporModel = PR1976_Qbicv

LVE.Mixture.Liquid1Model = PR1976_Qbicl

'Esta sección especifica las condiciones del flujo de entrada

LVE.FeedFlow.MolarFraction(1)=z1

LVE.FeedFlow.MolarFraction(2)=z2

LVE.FeedFlow.Temperature = 300

'Esta sección es donde se invoca el método a calcular

LVE.BubblePointPressure

'Sección de resultados

y1=LVE.VaporFlow.MolarFraction(1)

y2= LVE.VaporFlow.MolarFraction(2)

P= LVE.VaporFlow.Pressure
