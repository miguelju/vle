##### **Capítulo I**

## INTRODUCCIÓN

##### En la actualidad existen gran cantidad de programas computacionales con la capacidad de realizar cálculos de equilibrio líquido vapor (ELV) de mezclas multicomponentes. Los programas de este tipo que se encuentran al alcance de profesores y estudiantes, provienen generalmente de paquetes sencillos incluidos en los libros de texto o de los propios proyectos de investigación y desarrollo generados dentro de la misma universidad.

Da Silva *et al*. \[1\] indican que la enseñanza de la termodinámica del
equilibrio se ha limitado al análisis de gráficos, estudio de ciertos
modelos, y situaciones lo suficientemente ideales, como para poder ser
resueltas en las clases o exámenes, quizás por la carencia de
herramientas adecuadas.

Por otra parte, tanto Da Silva *et al*. \[1\], como Jackson *et al.*
\[2\], mencionan que la mayoría de los programas desarrollados por
grupos de investigación universitarios, o aquéllos obtenidos a través de
los libros de texto, ilustran sólo ciertos tipos de problemas mediante
el uso de un solo modelo termodinámico, además de ser poco amigables
para el usuario y de no tener la capacidad de detectar errores en los
datos de entrada; por ejemplo, Sandler \[3\] proporciona únicamente el
programa que resuelve el ELV de mezclas multicomponentes con el modelo
de Peng y Robinson.

Existen varias plataformas o sistemas operativos, así como también
diversos lenguajes de programación con los cuales se pueden desarrollar
paquetes termodinámicos. En 1989, Da Silva y Báez \[4\] desarrollaron un
paquete con diversidad de innovaciones, entre las cuales destacan la
presencia de una interfaz gráfica para el usuario (GUI) y la capacidad
de usar diversos modelos termodinámicos para la solución de los
problemas de ELV más comunes. Lamentablemente, tal paquete se desarrolló
bajo el sistema operativo proporcionado por Apple^®^, cuyo uso no es en
la actualidad el más común a nivel mundial.

Es así como surge la necesidad de rescatar el trabajo realizado por Da
Silva y Báez \[4\], para poder utilizar sus ideas bajo la plataforma de
las computadoras personales IBM^®^ compatibles y el sistema operativo
Microsoft Windows^®^.

El proyecto realizado no sólo reproduce la mayoría de los cálculos del
trabajo mencionado con anterioridad, introduciendo mejoras en la
convergencia de varios de los algoritmos, sino que también posee la
versatilidad de poder ser utilizado por otros proyectos que necesiten
cálculos termodinámicos tanto de sustancias puras como de mezclas
multicomponentes, gracias a su desarrollo mediante la programación
orientada a objetos que proporciona el lenguaje de programación
Microsoft Visual Basic^®^ mediante el concepto de las denominadas
*clases*.

A continuación, en el Capítulo II se presenta la descripción de los
modelos termodinámicos y la metodología utilizada en la resolución de
problemas de equilibrio líquido-vapor de sistemas multicomponentes.
Seguidamente se explica el desarrollo y estructura del paquete
termodinámico (Capítulo III), para finalmente mostrar los resultados
obtenidos (Capítulo IV) y concluir sobre los mismos (Capítulo V).
