Se desarrolló, mediante el esquema de programación orientada a objetos
que proporciona Microsoft Visual Basic^®^ 5.0, una librería
termodinámica para el cálculo de propiedades de sustancias puras y
mezclas multicomponentes, así como también la resolución de problemas
comunes de equilibrio líquido vapor (cálculos de punto de burbuja y
rocío, al igual que la resolución de problemas de evaporación
instantánea adiabática e isotérmica).

Se diseñó un programa maestro interactivo que maneja la librería
termodinámica.

Se planteó un nuevo algoritmo para el cálculo del punto de burbuja y
rocío con la capacidad de converger en zonas cercanas al punto crítico.

Los resultados obtenidos con el paquete desarrollado concuerdan con los
presentados en la literatura (Sandler \[3\], Hasan y Sandler \[21\], Da
Silva y Báez \[4\] y Smith *et al.* \[22\]).

La librería termodinámica desarrollada puede ser utilizada en otros
programas sin la necesidad de conocer la estructura interna de la misma
(debido a su diseño orientado a objetos).

Se recomienda la utilización de la librería termodinámica desarrollada
en la resolución de problemas de flujo en estado estacionario, como
podría ser el cálculo riguroso de una torre de destilación.

Existen numerosos parámetros de las modificaciones de las EDEC de Peng y
Robinson y Redlich Kwong Soave, que deben ser incorporados a la base de
datos existente para poder utilizarlas en los diferentes cálculos.

Las ayudas en línea son una característica importante de la mayoría de
los paquetes computacionales desarrollados bajo ambiente Windows^®^ por
lo que se recomienda la creación de este tipo de herramientas para el
programa maestro, con la finalidad de facilitar la interacción entre
este y el usuario.

Por último se recomienda la reconstrucción de la librería termodinámica,
mediante la creación de nuevas sub-*clases* que le den mayor
consistencia a las clasificaciones planteadas para las propiedades de
los compuestos y modelos termodinámicos.
