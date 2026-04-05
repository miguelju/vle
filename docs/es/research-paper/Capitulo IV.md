**CONVALIDACIÓN DEL PAQUETE TERMODINÁMICO**

En el presente capítulo se presentan un conjunto de pruebas
representativas de los principales tipos de cálculos y modelos,
comparando los resultados obtenidos con diferentes fuentes encontradas
en la literatura.

Dado que la convalidación completa de un paquete termodinámico que
ofrece tantos modelos disponibles es una tarea ambiciosa. Con esta
finalidad, se creó un programa maestro que facilitara la interacción
entre el usuario y la librería termodinámica desarrollada (ver anexo C).

**4.1 Cálculos de puntos críticos.**

Peng y Robinson \[15\] reportaron resultados del cálculo del punto
crítico para diferentes mezclas. La tabla 4.1 contiene las composiciones
globales de los sistemas estudiados, mientras que en la tabla 4.2 se
presentan los valores reportados por Peng y Robinson \[15\] y los
calculados por el paquete desarrollado.

**Tabla 4.1 Composiciones globales del los sistemas estudiados.**

  -------------------------------------------------------------------------
    No. De la     C~1~    C~2~     C~3~    nC~4~    nC~5~    CO~2~   H~2~S
      Mezcla                                                        
  -------------- ------ -------- -------- -------- -------- ------- -------
        1                0,3414   0,3421            0,3165          

        2                         0,3276   0,3398   0,3326          

        3         0,07                                       0,616   0,314

        4                0,2542   0,2547   0,2554   0,2357          
  -------------------------------------------------------------------------

**Tabla 4.2 Comparación de los puntos críticos de mezcla calculados, con
los reportados por Peng y Robinson \[16\].**

+------------------+-------------------------+-------------------------+-------------------+
| **No. de la      | **Peng y Robinson       | **Este Proyecto**       | **Errores**       |
| Mezcla**         | \[16\]**                |                         |                   |
+:================:+:===========:+:=========:+:===========:+:=========:+:=======:+:=======:+
|                  | **Pc(kPa)** | **Tc(K)** | **Pc(kPa)** | **Tc(K)** | **%Pc** | **%Tc** |
+------------------+-------------+-----------+-------------+-----------+---------+---------+
| 1                | 5552        | 404,43    | 5620,32     | 405,47    | 1,231   | 0,256   |
+------------------+-------------+-----------+-------------+-----------+---------+---------+
| 2                | 4174        | 430,72    | 4173,9      | 430,71    | 0,002   | 0,002   |
+------------------+-------------+-----------+-------------+-----------+---------+---------+
| 3                | 8420        | 310,92    | 7999,33     | 315,88    | 4,996   | 1,595   |
+------------------+-------------+-----------+-------------+-----------+---------+---------+
| 4                | 5063        | 410,74    | 5094,55     | 411,26    | 0,623   | 0,126   |
+------------------+-------------+-----------+-------------+-----------+---------+---------+

Como se observa en la tabla 4.2, los resultados obtenidos a través de la
librería termodinámica desarrollada son bastante aproximados a los
reportados por Peng y Robinson \[15\].

Las discrepancias presentes se pueden deber a tres causas fundamentales:

- La probable diferencia existentes entre los valores de las propiedades
  críticas de los componentes de la mezcla utilizados por ambos
  proyectos, ya que Peng y Robinson \[15\] no reportan los mismos.

- Los valores calculados a través de la librería se obtuvieron sin tener
  en cuenta el coeficiente de interacción (k~ij~) entre las sustancias,
  lo cual sería causa primordial en las diferencias observadas, ya que
  estos parámetros sí son utilizados por Peng y Robinson \[15\] pero no
  reportados.

- La diferencia probable entre las precisiones de los cálculos de los
  proyectos en cuestión.

Es importante destacar que otros paquetes termodinámicos como PRMIX.EXE
(Sandler \[3\]) y Ekilib (Da Silva y Báez \[4\]) no permiten el cálculo
de puntos críticos de mezclas, por lo que no fue posible comparar los
resultados con estas fuentes.

Por otra parte el algoritmo utilizado (basado en el presentado por
Heidemann *et al*. \[16\] con la introducción de derivadas numéricas
como lo describieron Stockfleth *et al*. \[10\]) tiende a ser bastante
lento, si se compara con aquel que use derivadas analíticas, sin
embargo, permite el uso de diversas reglas de mezclado sin necesidad de
obtener las expresiones matemáticas (para cada EDEC) de los coeficientes
de fugacidad parcial de los componentes.

**4.2 Cálculos de *flash* adiabático.**

En la tabla 4.3 se presentan el cálculo de un *flash* adiabático para
una mezcla de cuatro compuestos (obtenida del ejemplo presentado por Da
Silva y Báez \[4\]; benceno, ciclohexano, metil-ciclohexano y n-hexano).
El modelo utilizado fue el de Peng y Robinson para ambas fases, sin
tomar en cuenta los parámetros de interacción binarios.

**Tabla 4.3 Datos de la alimentación del *flash* adiabático.**

  ---------------------------------------------------------------------
              **Datos**                          **Valor**
  ---------------------------------- ----------------------------------
            Presión (kPa)                           300

           Temperatura (K)                          420

                 Fase                             Líquida
  ---------------------------------------------------------------------

Como se puede observar en la tabla 4.4 los resultados obtenidos por el
paquete desarrollado son consistentes con los presentados por Da Silva y
Báez \[4\]. Las pequeñas diferencias existentes se deben a la precisión
utilizada así como las constantes empleadas.

Por otra parte, otros programas como PRMIX.EXE (Sandler \[3\]), a pesar
de utilizar la ecuación cúbica de Peng y Robinson, no permiten cálculos
de *flash* adiabático, con lo que se demuestra la poca versatilidad del
mismo.

**Tabla 4.4 Datos de la salida *flash* adiabático.**

  -----------------------------------------------------------------
     **Datos**        **Este     **Da Silva et       **Errores
                    Proyecto**    al. \[4\]**    calculados (%)**
  ---------------- ------------ --------------- -------------------
   Presión (kPa)       300            300       

  Temperatura (K)    394,263        394,285           0,00558
  -----------------------------------------------------------------

**Tabla 4.4 Datos de la salida *flash* adiabático. (cont.)**

  -----------------------------------------------------------------
     **Datos**        **Este     **Da Silva et       **Errores
                    Proyecto**    al. \[4\]**    calculados (%)**
  ---------------- ------------ --------------- -------------------
         β           0,194494       0,19864          0,020872

      Entalpía        -7151         -7033,5          1,670577
     (kJ/kmol)                                  

        x~1~          0,2466        0,24648          0,048685

        x~2~          0,2508        0,25084          0,015946

        x~3~          0,271         0,27144          0,162098

        x~4~          0,2316        0,23124          0,155682

        y~1~          0,2642        0,26421          0.003785

        y~2~          0,2465        0,24660          0,040552

        y~3~          0,1632        0,16351          0,189591

        y~4~          0,3261        0,32569          0,125887
  -----------------------------------------------------------------

**4.3 Cálculo del punto de burbuja conocida la temperatura del sistema
(Burbuja P).**

Utilizando el modelo de van Laar (cuyos parámetros se muestran en la
tabla 4.5) para la descripción de la fase líquida y el modelo de gas
ideal para la fase gaseosa, se calcularon diversos puntos de burbuja de
una mezcla binaria metanol(1)/agua(2) mediante el uso del paquete
desarrollado, comparándolos con aquellos obtenidos con el programa
AC.EXE (Orbey y Sandler \[21\]) a una temperatura de 298 K (ver tabla
4.6).

**Tabla 4.5 Parámetros utilizados con el modelo de van Laar (Orbey** **y
Sandler \[21\]).**

  ---------------------------------------------------------------------
              **Λ~12~**                          **Λ~21~**
  ---------------------------------- ----------------------------------
                0,5853                             0,3458

  ---------------------------------------------------------------------

**Tabla 4.6 Comparación de los cálculos de punto de burbuja con
temperatura conocida.**

+----------+-----------------------+-----------------------+-----------------------+
|          | **Orbey** **y Sandler | **Este proyecto**     | **Errores             |
|          | \[21\]**              |                       | calculados**          |
+:========:+:=========:+:=========:+:=========:+:=========:+:=========:+:=========:+
| **x~1~** | **y~1~**  | **P       | **y~1~**  | **P       | **% P**   | **% y**   |
|          |           | (kPa)**   |           | (kPa)**   |           |           |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0,0873   | 0,4416    | 5.1998    | 0.4419    | 5.2410    | 0.7921    | 0.0679    |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0,19     | 0,6287    | 7.0028    | 0.6245    | 7.0620    | 0.8460    | 0.6680    |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0,3417   | 0,7538    | 9.1151    | 0.7542    | 9.1962    | 0.8898    | 0.0531    |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0,4943   | 0,8334    | 10.9757   | 0.8338    | 11.0782   | 0.9343    | 0.0480    |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0,6919   | 0,909     | 13.2939   | 0.9092    | 13.4230   | 0.9714    | 0.0220    |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+
| 0,8492   | 0,9583    | 15.1678   | 0.9584    | 15.3184   | 0.9931    | 0.0104    |
+----------+-----------+-----------+-----------+-----------+-----------+-----------+

Como se observa en la tabla 4.5, los valores de la presión calculados a
través del paquete desarrollado difieren en promedio de menos del 1%
respecto a los valores reportados por la fuente consultada. Tal error se
explica debido a las diferencias existentes entre las constantes
utilizadas en la correlación empleada para la predicción de las
presiones de saturación de los componentes y la diferencia entre las
precisiones con la cual se realizaron los cálculos.

**4.4 Cálculo del punto de rocío (Rocío T y Rocío P)**

Para el sistema binario 2-propanol(1)/agua(2), se realizó el cálculo del
punto de rocío, utilizando el modelo de Wilson para la fase líquida y el
de gas ideal para la fase gaseosa.

En la tablas 4.7 y 4.8 se presentan los resultados obtenidos.

**Tabla 4.7 Comparación del punto de rocío calculado dada la presión.**

+-------------------+----------------------+--------------------+---------------------+
| **Datos**         | **Smith *et al*.     | **Este Proyecto**  | **Errores           |
|                   | \[22\]**             |                    | calculados**        |
+:=======:+:=======:+:=======:+:==========:+:=======:+:========:+:========:+:========:+
| **y1**  | **P     | **T     | **x~1\ ~** | **T     | **x~1~** | **% T**  | **% x**  |
|         | (kPa)** | (K)**   |            | (K)**   |          |          |          |
+---------+---------+---------+------------+---------+----------+----------+----------+
| 0,4     | 101,33  | 360,61  | 0,0639     | 360,918 | 0,0824   | 0,085    | 28,95    |
+---------+---------+---------+------------+---------+----------+----------+----------+

Como se observa en la tabla 4.7, el error porcentual en temperatura es
menor al 1%, y puede ser explicado por la diferencia en la precisión de
los cálculos, sin embargo, el error presentado en la composición (28.95
%), indica que el cálculo de los volúmenes molares líquidos de los
componentes, ha afectado significativamente los resultados, ya que en
los cálculos realizados por Smith *et al.*\[22\], no se toma en cuanta
la variación de dicha propiedad respecto a la temperatura, cuestión que
si esta contemplada en los cálculos del paquete desarrollado.

Por otra parte, al comparar los resultados obtenidos con aquellos
calculados mediante el paquete Ekilib (Da Silva y Báez \[4\]), éstos
coinciden hasta la segunda cifra decimal, ya que aunque ambos proyectos
utilizan las misma ecuaciones, existen diferencias en las precisiones
utilizadas (mayor precisión en el proyecto desarrollado).

En la tabla 4.8 se presentan los resultados para el cálculo de el punto
de rocío del sistema a una temperatura de 353.15 K. Como se observa, los
resultados en este caso se acercan más a aquéllos presentado por Smith
*et al.*\[22\] (menores al 5%), ya que el volumen molar líquido de cada
componente se calcula una sola vez a la temperatura de la mezcla, a
diferencia del caso anterior en el cual el volumen es recalculado en
cada iteración realizada con la temperatura.

**Tabla 4.8 Comparación del punto de rocío calculado dada la
temperatura.**

+-------------------+----------------------+--------------------+---------------------+
| **Datos**         | **Smith *et al*.     | **Este Proyecto**  | **Errores           |
|                   | \[22\]**             |                    | calculados**        |
+:=======:+:=======:+:=======:+:==========:+:=======:+:========:+:========:+:========:+
| **y1**  | **T     | **P     | **x~1\ ~** | **P     | **x~1~** | **% P**  | **% x**  |
|         | (K)**   | (kPa)** |            | (kPa)** |          |          |          |
+---------+---------+---------+------------+---------+----------+----------+----------+
| 0,6     | 353,15  | 96,72   | 0,449      | 92,64   | 0,471    | 4,2      | 4,8      |
+---------+---------+---------+------------+---------+----------+----------+----------+

**4.5 Cálculo del punto de burbuja conocida la presión del sistema
(Burbuja T)**

En este caso se estudió una mezcla de cuatro componentes (ver tabla 4.9)
utilizando la ley de Raoult como modelo termodinámico. Como se observa
los resultados obtenidos mediante la librería termodinámica desarrollada
son absolutamente consistentes con aquéllos obtenido a través del
paquete diseñado por Da Silva y Báez \[4\].

**Tabla 4.9 Comparación del punto de burbuja calculado dada la
presión.**

+----------------+----------+------------+-----------------------------+
|                |          | **Este     | **Da Silva y Báez \[4\]**   |
|                |          | Proyecto** |                             |
+:==============:+:========:+:==========:+:==============:+:==========:+
| **P (kPa)**    |          | **T (K)**  | **T (K)**                   |
+----------------+----------+------------+-----------------------------+
| 101,325        |          | 131.51     | 131.51                      |
+----------------+----------+------------+----------------+------------+
| **Componente** | **x~i~** | **y~i~**   | **y~I~**       | **%y~1~**  |
+----------------+----------+------------+----------------+------------+
| metano         | 0,25     | 0,99461    | 0,99461        | 0          |
+----------------+----------+------------+----------------+------------+
| etano          | 0,35     | 0,00536    | 0,00536        | 0          |
+----------------+----------+------------+----------------+------------+
| propano        | 0,15     | 0,00003    | 0,00003        | 0          |
+----------------+----------+------------+----------------+------------+
| butano         | 0,25     | 0,00000    | 0,00000        | 0          |
+----------------+----------+------------+----------------+------------+

**4.6 Cálculos de *flash* isotérmico.**

Se seleccionó una mezcla equimolar de n-heptano(1) y butano(2). En la
descripción de ambas fases, se utilizó la EDEC de Redlich-Kwong-Soave,
despreciando los parámetros de interacción binarios. En la tabla 4.10 se
presentan los resultados obtenidos.

**Tabla 4.10 Comparación de los resultados obtenidos en la resolución de
un *flash* isotérmico.**

+-----------------------+--------------------------------+--------------------------------+
| **Datos**             | **Ekilib (Da Silva y Báez*.*   | **Este Proyecto**              |
|                       | \[4\])**                       |                                |
+:========:+:==========:+:========:+:========:+:========:+:========:+:========:+:========:+
| **T(K)** | **P(kPa)** | **x~1~** | **y~1~** | **β**    | **x~1~** | **y~1~** | **β**    |
+----------+------------+----------+----------+----------+----------+----------+----------+
| 300      | 100        | 0,6135   | 0,04284  | 0,19889  | 0,6135   | 0,04283  | 0,198821 |
+----------+------------+----------+----------+----------+----------+----------+----------+

Como se puede observar, existen diferencias a partir de la tercera cifra
decimal, si se comparan con los valores obtenidos utilizando el paquete
desarrollado por Da Silva y Báez \[4\]. Una vez más tales discrepancias
provienen de la precisión con la que se realizaron los cálculos como se
ha mencionado con anterioridad.

**4.7 Cálculo de parámetros de interacción binaria (kij).**

Para el cálculo de los parámetros de interacción binaria (k~ij~), se
tomó una mezcla de CO~2~(1)/Butano(2). En la tabla 4.11 se presentan los
datos de presión contra composición, a temperatura constante utilizados
para la resolución del problema planteado, mientras que en la tabla 4.12
se pueden observar los valores calculados por el programa desarrollado,
así como los presentados por otros autores.

Los resultados obtenidos concuerdan con aquéllos presentados por las dos
fuentes consultadas.

**Tabla 4.11 Datos experimentales de equilibrio para el sistema
CO~2~-Butano (T=357.57 K) (Da Silva y Báez \[4\]).**

  -----------------------------------------------------------------------
        **P (bar)**              **x~1~**                **y~1~**
  ----------------------- ----------------------- -----------------------
          14,824                  0,02967                 0,21649

          19,029                  0,06228                 0,36217

          23,511                  0,0959                  0,45773

          27,441                  0,1283                  0,51818

          31,164                  0,15673                 0,56447

          36,404                  0,19636                 0,60283

          42,885                  0,25027                 0,64685

          49,573                  0,30421                 0,67849

          56,399                  0,35904                 0,69727

          63,569                  0,41871                 0,71069

          70,671                  0,49255                 0,7139

          75,428                  0,5352                  0,71155

           77,91                  0,56473                 0,69855

          79,289                  0,5745                  0,68927
  -----------------------------------------------------------------------

**Tabla 4.12 Comparación de los resultados obtenidos en el cálculo de
los parámetros de interacción binarios.**

  ---------------------------------------------------------
           **Ekilib (Da Silva y    **Sandler     **Este
               Báez\[4\])**         \[3\]**    Proyecto**
  ------- ----------------------- ----------- -------------
   k~ij~          0,1359             0,135       0,1357

  ---------------------------------------------------------
