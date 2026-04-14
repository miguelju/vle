## ANEXO A {#anexo-a .MyHeader}

## MANUAL DEL ANALISTA {#manual-del-analista .MyHeader}

## (DESCRIPCIÓN DE LAS *CLASES* Y MODULOS PRESENTES EN EL PROYECTO) {#descripción-de-las-clases-y-modulos-presentes-en-el-proyecto .MyHeader}

## clsActivityMulticomp {#clsactivitymulticomp .MyHeader}

*Clase* diseñada con el objetivo de calcular las propiedades de mezcla a
través de los modelos basados en la actividad (Modelos Gamma)

## Public Enum TADiPGammaModel {#public-enum-tadipgammamodel .MyHeader}

##  IdealSolutiong = 25 {#idealsolutiong-25 .MyHeader}

##  VanLaarg = 21 {#vanlaarg-21 .MyHeader}

##  Wilsong = 22 {#wilsong-22 .MyHeader}

##  ScatchardHg = 23 {#scatchardhg-23 .MyHeader}

##  Margulesg = 24 {#margulesg-24 .MyHeader}

## End Enum {#end-enum .MyHeader}

Esta es la enumeración que permite la escogencia del modelo
termodinámico a utilizar dentro de esta *clase*.

## Private mvarActivityProps As colclsActivityProps {#private-mvaractivityprops-as-colclsactivityprops .MyHeader}

Esta variable guarda las propiedades de los compuestos necesarias para
el cálculo de las funciones de la *clase*.

## Private mvarModel As TADiPGammaModel {#private-mvarmodel-as-tadipgammamodel .MyHeader}

Variable que guarda el modelo termodinámico seleccionado.

## Private X() As Double {#private-x-as-double .MyHeader}

Variable que contiene las composiciones molares de los compuestos de la
mezcla.

## Public Property Get ActivityProps() As colclsActivityProps {#public-property-get-activityprops-as-colclsactivityprops .MyHeader}

## Public Property Set ActivityProps(vData As colclsActivityProps) {#public-property-set-activitypropsvdata-as-colclsactivityprops .MyHeader}

Son las propiedades de la *clase* que permiten introducir, las
propiedades de los compuestos a la misma.

## Private Sub Class_Terminate() {#private-sub-class_terminate .MyHeader}

Destruye los contenidos de la variable mvarActivityProps.

## Public Function CalcGamma(vntIndexKey As Integer, ByRef Gamma As Double) As Boolean {#public-function-calcgammavntindexkey-as-integer-byref-gamma-as-double-as-boolean .MyHeader}

Esta función, es la que calcula el valor de γ. vntIndexKey es el número
del compuesto al cual se le calculará la propiedad, y Gamma, la variable
donde se devolverá el valor obtenido.

## Private Function SumasWilson(vntIndexKey As Integer) As Double {#private-function-sumaswilsonvntindexkey-as-integer-as-double .MyHeader}

Realiza las sumatorias existentes dentro de los cálculos relacionados
con el modelo de Wilson.

## Public Property Get MolarFraction(Index As Integer) As Double {#public-property-get-molarfractionindex-as-integer-as-double .MyHeader}

## Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As Double) {#public-property-let-molarfractionindex-as-integer-byval-molarfraction-as-double .MyHeader}

Estas son las propiedades que permiten para el compuesto Index asignar y
leer las composiciones molares a la variable X().

## Public Property Let Model(ByVal vData As TADiPGammaModel) {#public-property-let-modelbyval-vdata-as-tadipgammamodel .MyHeader}

## Public Property Get Model() As TADiPGammaModel {#public-property-get-model-as-tadipgammamodel .MyHeader}

Propiedades que permiten asignar y leer, el modelo termodinámico a la
variable mvarModel.

## Public Function exGibbs(T As Double) As Double {#public-function-exgibbst-as-double-as-double .MyHeader}

Función que calcula el valor de la energía de Gibbs en exceso de una
mezcla, a la temperatura definida por el parámetro T.

## Public Function exEnthalpy(T As Double) As Double {#public-function-exenthalpyt-as-double-as-double .MyHeader}

Función que calcula el valor de la entalpía en exceso de una mezcla, a
la temperatura definida por el parámetro T.

## Public Sub ChangeAij(T As Double) {#public-sub-changeaijt-as-double .MyHeader}

Este es un procedimiento desarrollado con la finalidad de recalcular los
parámetros binarios (Aij) a medida que la temperatura (T) varía a través
de los cálculos de los equilibrios de fase

## clsActivityProps {#clsactivityprops .MyHeader}

*Clase* desarrollada con el objetivo de guardar en su interior los
valores de las propiedades necesarias de un componente, para poder
aplicar a éste los modelos basados en los coeficientes de actividad.

## Private mvardelta As Double {#private-mvardelta-as-double .MyHeader}

Variable que contiene el valor de delta para un compuesto.

## Private mvarvl As Double {#private-mvarvl-as-double .MyHeader}

Variable que contiene el valor del volumen líquido correspondiente a un
compuesto.

## Public Property Let vl(ByVal vData As Double) {#public-property-let-vlbyval-vdata-as-double .MyHeader}

## Public Property Get vl() As Double {#public-property-get-vl-as-double .MyHeader}

Propiedades que permiten la asignación de los valores a la variable
mvarvl.

## Public Property Let delta(ByVal vData As Double) {#public-property-let-deltabyval-vdata-as-double .MyHeader}

## Public Property Get delta() As Double {#public-property-get-delta-as-double .MyHeader}

Propiedades que permiten la asignación de los valores a la variable
mvardelta.

## clsAllProps {#clsallprops .MyHeader}

*Clase* que posee instancias de todos los objetos que tengan como
objetivo el de guardar propiedades de los compuestos y de las mezclas.

## Private mvarCriticalProps As colclsCriticalProps {#private-mvarcriticalprops-as-colclscriticalprops .MyHeader}

Esta variable, permite guardat una copia de la colección
colclsCriticalProps

## Private mvarOtherProps As colclsOtherProps {#private-mvarotherprops-as-colclsotherprops .MyHeader}

Variable en la cual se guarda una copia de la colección colclsOtherProps

## Private mvarLiquid1Model As TadipLiquidModels {#private-mvarliquid1model-as-tadipliquidmodels .MyHeader}

## Private mvarLiquid2Model As TadipLiquidModels {#private-mvarliquid2model-as-tadipliquidmodels .MyHeader}

Permite guardar el modelo termodinámico utilizado para los cálculos
relacionados con las fases líquidas de la mezcla.

## Private mvarVaporModel As TadipVaporModel {#private-mvarvapormodel-as-tadipvapormodel .MyHeader}

Permite guardar el modelo termodinámico utilizado para los cálculos
relacionados con la fase vapor de la mezcla.

## Private mvarCpConsts As colclsCpConsts {#private-mvarcpconsts-as-colclscpconsts .MyHeader}

Permite guardar una copia de la colección colclsCpConsts.

## Private mvarNumberOfComponents As Integer {#private-mvarnumberofcomponents-as-integer .MyHeader}

Variable que guarda el número de componentes existentes en la mezcla.

## Private mvarSatPressureModel As TADiPSatPressureModel {#private-mvarsatpressuremodel-as-tadipsatpressuremodel .MyHeader}

Permite guardar la correlación que se utilizará para los estimados de
las presiones de saturación de los compuestos de la mezcla.

## Public Property Let SatPressureModel(ByVal vData As TADiPSatPressureModel) {#public-property-let-satpressuremodelbyval-vdata-as-tadipsatpressuremodel .MyHeader}

## Public Property Get SatPressureModel() As TADiPSatPressureModel {#public-property-get-satpressuremodel-as-tadipsatpressuremodel .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarSatPressureModel.

## Public Property Let NumberOfComponents(ByVal vData As Integer) {#public-property-let-numberofcomponentsbyval-vdata-as-integer .MyHeader}

## Public Property Get NumberOfComponents() As Integer {#public-property-get-numberofcomponents-as-integer .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarNumberOfComponents.

## Public Property Get OtherProps() As colclsOtherProps {#public-property-get-otherprops-as-colclsotherprops .MyHeader}

## Public Property Set OtherProps(vData As colclsOtherProps) {#public-property-set-otherpropsvdata-as-colclsotherprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarOtherProps.

## Public Property Get CpConsts() As colclsCpConsts {#public-property-get-cpconsts-as-colclscpconsts .MyHeader}

## Public Property Set CpConsts(vData As colclsCpConsts) {#public-property-set-cpconstsvdata-as-colclscpconsts .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCpConsts.

## Public Property Let Liquid2Model(ByVal vData As TadipLiquidModels) {#public-property-let-liquid2modelbyval-vdata-as-tadipliquidmodels .MyHeader}

## Public Property Get Liquid2Model() As TadipLiquidModels {#public-property-get-liquid2model-as-tadipliquidmodels .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarLiquid2Model.

## Public Property Let VaporModel(ByVal vData As TadipVaporModel) {#public-property-let-vapormodelbyval-vdata-as-tadipvapormodel .MyHeader}

## Public Property Get VaporModel() As TadipVaporModel {#public-property-get-vapormodel-as-tadipvapormodel .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarVaporModel.

## Public Property Let Liquid1Model(ByVal vData As TadipLiquidModels) {#public-property-let-liquid1modelbyval-vdata-as-tadipliquidmodels .MyHeader}

## Public Property Get Liquid1Model() As TadipLiquidModels {#public-property-get-liquid1model-as-tadipliquidmodels .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarLiquid1Model.

## Public Property Get CriticalProps() As colclsCriticalProps {#public-property-get-criticalprops-as-colclscriticalprops .MyHeader}

## Public Property Set CriticalProps(vData As colclsCriticalProps) {#public-property-set-criticalpropsvdata-as-colclscriticalprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProps.

## Private Sub Class_Terminate()  {#private-sub-class_terminate-1 .MyHeader}

Este procedimiento, destruye el contenido de las siguientes variables:
mvarCpConsts, mvarOtherProps y mvarCriticalProps.

## clsCpConsts {#clscpconsts .MyHeader}

*Clase* que contiene, los coeficientes necesarios para calcular del Cp
de un compuesto determinado, según la fórmula indicada, a través de una
de las propiedades.

## Private mvarNbFormula As Integer {#private-mvarnbformula-as-integer .MyHeader}

Variable que guarda el número de fórmula que corresponde a la
correlación utilizada para el cálculo del Cp de un compuesto.

## Private mvarCoef() As Double {#private-mvarcoef-as-double .MyHeader}

Variable que guarda los coeficientes necesarios para el cálculo del Cp
de un compuesto, según la fórmula dada por la variable mvarNbFormula.

## Public Property Let Coef(Index As Integer, vData As Double) {#public-property-let-coefindex-as-integer-vdata-as-double .MyHeader}

## Public Property Get Coef(Index As Integer) As Double {#public-property-get-coefindex-as-integer-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCoef().

## Public Property Get NbFormula() As Integer {#public-property-get-nbformula-as-integer .MyHeader}

## Public Property Let NbFormula(vData As Integer) {#public-property-let-nbformulavdata-as-integer .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarNbFormula.

## Public Property Get NbCoef() As Integer {#public-property-get-nbcoef-as-integer .MyHeader}

Propiedad que permite conocer la cantidad de coeficientes contenidos en
la variable mvarCoef().

## clsCriticalProps {#clscriticalprops .MyHeader}

*Clase* que guarda en su interior las propiedades críticas de un
compuesto.

## Private mvarTc As Double {#private-mvartc-as-double .MyHeader}

Variable que guarda el valor de la temperatura crítica para determinado
compuesto de una mezcla.

## Private mvarPc As Double {#private-mvarpc-as-double .MyHeader}

Variable que guarda el valor de la presión crítica para determinado
compuesto de una mezcla.

## Private mvarW As Double {#private-mvarw-as-double .MyHeader}

Variable que guarda el valor del factor acéntrico para determinado
compuesto de una mezcla.

## Private mvarZc As Double {#private-mvarzc-as-double .MyHeader}

Variable que guarda el valor del factor de compresibilidad crítico para
determinado compuesto de una mezcla.

## Public Property Let Zc(ByVal vData As Double) {#public-property-let-zcbyval-vdata-as-double .MyHeader}

## Public Property Get Zc() As Double {#public-property-get-zc-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarZc.

## Public Property Let w(ByVal vData As Double) {#public-property-let-wbyval-vdata-as-double .MyHeader}

## Public Property Get w() As Double {#public-property-get-w-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarW.

## Public Property Let Pc(ByVal vData As Double) {#public-property-let-pcbyval-vdata-as-double .MyHeader}

## Public Property Get Pc() As Double {#public-property-get-pc-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPc.

## Public Property Let Tc(ByVal vData As Double) {#public-property-let-tcbyval-vdata-as-double .MyHeader}

## ublic Property Get Tc() As Double {#ublic-property-get-tc-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarTc.

##  {#section .MyHeader}

##  {#section-1 .MyHeader}

##  {#section-2 .MyHeader}

##  {#section-3 .MyHeader}

##  {#section-4 .MyHeader}

##  {#section-5 .MyHeader}

##  {#section-6 .MyHeader}

##  {#section-7 .MyHeader}

##  {#section-8 .MyHeader}

##  {#section-9 .MyHeader}

##  {#section-10 .MyHeader}

##  {#section-11 .MyHeader}

##  {#section-12 .MyHeader}

##  {#section-13 .MyHeader}

##  {#section-14 .MyHeader}

##  {#section-15 .MyHeader}

##  {#section-16 .MyHeader}

##  {#section-17 .MyHeader}

##  {#section-18 .MyHeader}

##  {#section-19 .MyHeader}

##  {#section-20 .MyHeader}

##  {#section-21 .MyHeader}

##  {#section-22 .MyHeader}

##  {#section-23 .MyHeader}

##  {#section-24 .MyHeader}

##  {#section-25 .MyHeader}

##  {#section-26 .MyHeader}

##  {#section-27 .MyHeader}

##  {#section-28 .MyHeader}

##  {#section-29 .MyHeader}

##  {#section-30 .MyHeader}

##  {#section-31 .MyHeader}

##  {#section-32 .MyHeader}

##  {#section-33 .MyHeader}

##  {#section-34 .MyHeader}

##  {#section-35 .MyHeader}

##  {#section-36 .MyHeader}

##  {#section-37 .MyHeader}

## clsFlow {#clsflow .MyHeader}

*Clase* que representa una corriente de un proceso, descrita a través de
propiedades termodinámicas y composiciones de mezcla.

## Private mvarMolarFraction() As Double {#private-mvarmolarfraction-as-double .MyHeader}

Variable que guarda el valor de la composición global de los compuestos
en la mezcla.

## Private mvarTemperature As Double {#private-mvartemperature-as-double .MyHeader}

Variable que guarda el valor de la temperatura de la corriente que
representa esta *clase*.

## Private mvarPressure As Double {#private-mvarpressure-as-double .MyHeader}

Variable que guarda el valor de la presión de la corriente que
representa esta *clase*.

## Private mvarEnthalpy As Double {#private-mvarenthalpy-as-double .MyHeader}

Variable que guarda el valor de la entalpía de la corriente que
representa esta *clase*.

## Private mvarEntropy As Double {#private-mvarentropy-as-double .MyHeader}

Variable que guarda el valor de la entropía de la corriente que
representa esta *clase*.

## Private mvarVaporFraction As Double {#private-mvarvaporfraction-as-double .MyHeader}

Variable que guarda el valor del β (fracción de vapor) de la corriente
que representa esta *clase*.

## Private mvarLiquid1Fraction As Double {#private-mvarliquid1fraction-as-double .MyHeader}

## Private mvarLiquid2Fraction As Double {#private-mvarliquid2fraction-as-double .MyHeader}

Variables que guardan el valor de la fracción de cada líquido que esta
presente en la corriente que representa esta *clase*.

## Public Property Let Liquid2Fraction(ByVal vData As Double) {#public-property-let-liquid2fractionbyval-vdata-as-double .MyHeader}

## Public Property Get Liquid2Fraction() As Double {#public-property-get-liquid2fraction-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarLiquid2Fraction.

## Public Property Let Liquid1Fraction(ByVal vData As Double) {#public-property-let-liquid1fractionbyval-vdata-as-double .MyHeader}

## Public Property Get Liquid1Fraction() As Double {#public-property-get-liquid1fraction-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarLiquid1Fraction.

## Public Property Let VaporFraction(ByVal vData As Double) {#public-property-let-vaporfractionbyval-vdata-as-double .MyHeader}

## Public Property Get VaporFraction() As Double {#public-property-get-vaporfraction-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarVaporFraction.

## Public Property Get Temperature() As Double {#public-property-get-temperature-as-double .MyHeader}

## Public Property Let Temperature(T As Double) {#public-property-let-temperaturet-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarTemperature.

## Public Property Get Pressure() As Double {#public-property-get-pressure-as-double .MyHeader}

## Public Property Let Pressure(P As Double) {#public-property-let-pressurep-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPressure.

## Public Property Get Enthalpy() As Double {#public-property-get-enthalpy-as-double .MyHeader}

## Public Property Let Enthalpy(h As Double) {#public-property-let-enthalpyh-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarEnthalpy.

## Public Property Get Entropy() As Double {#public-property-get-entropy-as-double .MyHeader}

## Public Property Let Entropy(s As Double) {#public-property-let-entropys-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarEntropy.

## Public Property Get MolarFraction(Index As Integer) As Double {#public-property-get-molarfractionindex-as-integer-as-double-1 .MyHeader}

## Public Property Let MolarFraction(Index As Integer, ByVal NewValue As Double) {#public-property-let-molarfractionindex-as-integer-byval-newvalue-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarMolarFarction.

## clsOtherProps {#clsotherprops .MyHeader}

> *Clase* que guarda las propiedades no clasificadas de un compuesto.

## Private mvardelta As Double {#private-mvardelta-as-double-1 .MyHeader}

Variable que guarda el valor del parámetro de solubilidad de una
sustancia.

## Private mvarTb As Double {#private-mvartb-as-double .MyHeader}

Variable que guarda el valor de la temperatura de ebullición normal de
un compuesto.

## Private mvarm As Double {#private-mvarm-as-double .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Private mvarn As Double {#private-mvarn-as-double .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Private mvarK1 As Double {#private-mvark1-as-double .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Private mvarPsatAtTr As Double {#private-mvarpsatattr-as-double .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Private mvarg As Double {#private-mvarg-as-double .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Public Property Let PsatAtTr(ByVal vData As Double) {#public-property-let-psatattrbyval-vdata-as-double .MyHeader}

## Public Property Get PsatAtTr() As Double {#public-property-get-psatattr-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPsatAtTr.

## Public Property Let K1(ByVal vData As Double) {#public-property-let-k1byval-vdata-as-double .MyHeader}

## Public Property Get K1() As Double {#public-property-get-k1-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarK1.

## Public Property Let g(ByVal vData As Double) {#public-property-let-gbyval-vdata-as-double .MyHeader}

## Public Property Get g() As Double {#public-property-get-g-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarg.

## Public Property Let n(ByVal vData As Double) {#public-property-let-nbyval-vdata-as-double .MyHeader}

## Public Property Get n() As Double {#public-property-get-n-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarn.

## Public Property Let m(ByVal vData As Double) {#public-property-let-mbyval-vdata-as-double .MyHeader}

## Public Property Get m() As Double {#public-property-get-m-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarm.

## Public Property Let DipMoment(ByVal vData As Double) {#public-property-let-dipmomentbyval-vdata-as-double .MyHeader}

## Public Property Get DipMoment() As Double {#public-property-get-dipmoment-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarDipMoment.

## Public Property Let Tb(ByVal vData As Double) {#public-property-let-tbbyval-vdata-as-double .MyHeader}

## Public Property Get Tb() As Double {#public-property-get-tb-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarTb.

## Public Property Let vl(ByVal vData As Double) {#public-property-let-vlbyval-vdata-as-double-1 .MyHeader}

## Public Property Get vl() As Double {#public-property-get-vl-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarvl.

## Public Property Let delta(ByVal vData As Double) {#public-property-let-deltabyval-vdata-as-double-1 .MyHeader}

## Public Property Get delta() As Double {#public-property-get-delta-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvardelta.

## clsQbicsProps {#clsqbicsprops .MyHeader}

*Clase* diseñada para guardar las propiedades necesarias de un compuesto
para aplicar a éste las ecuaciones de estado cúbicas.

## Private mvarCriticalTemperature As Double {#private-mvarcriticaltemperature-as-double .MyHeader}

Variable que guarda el valor de la temperatura crítica de un compuesto.

## Private mvarCriticalPressure As Double {#private-mvarcriticalpressure-as-double .MyHeader}

Variable que guarda el valor de la presión crítica de un compuesto.

## Private mvarAcentricFactor As Double {#private-mvaracentricfactor-as-double .MyHeader}

Variable que guarda el valor del factor acéntrico de un compuesto.

## Public Property Let PsatAtTr(ByVal vData As Double) {#public-property-let-psatattrbyval-vdata-as-double-1 .MyHeader}

## Public Property Get PsatAtTr() As Double {#public-property-get-psatattr-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPsatAtTr.

## Public Property Let PRSVK1(ByVal vData As Double) {#public-property-let-prsvk1byval-vdata-as-double .MyHeader}

## Public Property Get PRSVK1() As Double {#public-property-get-prsvk1-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPRSVK1.

## Public Property Let m(ByVal vData As Double) {#public-property-let-mbyval-vdata-as-double-1 .MyHeader}

## Public Property Get m() As Double {#public-property-get-m-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarm.

## Public Property Let n(ByVal vData As Double) {#public-property-let-nbyval-vdata-as-double-1 .MyHeader}

## Public Property Get n() As Double {#public-property-get-n-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarn.

## Public Property Let g(ByVal vData As Double) {#public-property-let-gbyval-vdata-as-double-1 .MyHeader}

## Public Property Get g() As Double {#public-property-get-g-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarg.

## Public Property Let Zc(ByVal vData As Double) {#public-property-let-zcbyval-vdata-as-double-1 .MyHeader}

## Public Property Get Zc() As Double {#public-property-get-zc-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarZc.

## Public Property Let w(ByVal vData As Double) {#public-property-let-wbyval-vdata-as-double-1 .MyHeader}

## Public Property Get w() As Double {#public-property-get-w-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAcentricFactor.

## Public Property Let Pc(ByVal vData As Double) {#public-property-let-pcbyval-vdata-as-double-1 .MyHeader}

## Public Property Get Pc() As Double {#public-property-get-pc-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalPressure.

## Public Property Let Tc(ByVal vData As Double) {#public-property-let-tcbyval-vdata-as-double-1 .MyHeader}

## Public Property Get Tc() As Double {#public-property-get-tc-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalTemperature.

## clsReferencesSt {#clsreferencesst .MyHeader}

*Clase* que guarda los valores de los estados de referencia utilizados
en el cálculo de propiedades energéticas de las mezclas
multicomponentes.

## Private mvarTo As Double {#private-mvarto-as-double .MyHeader}

Variable que guarda el valor de la temperatura de referencia para todo
los compuestos de la mezcla.

## Private mvarPo As Double {#private-mvarpo-as-double .MyHeader}

Variable que guarda el valor de la presión de referencia para todo los
compuestos de la mezcla.

## Private mvarSo As Double {#private-mvarso-as-double .MyHeader}

Variable que guarda el valor de la entropía de referencia para todo los
compuestos de la mezcla.

## Private mvarHo As Double {#private-mvarho-as-double .MyHeader}

Variable que guarda el valor de la entalpía de referencia para todo los
compuestos de la mezcla.

## Public Property Let Ho(ByVal vData As Double) {#public-property-let-hobyval-vdata-as-double .MyHeader}

## Public Property Get Ho() As Double {#public-property-get-ho-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarHo.

## Public Property Let TTo(ByVal vData As Double) {#public-property-let-ttobyval-vdata-as-double .MyHeader}

## Public Property Get TTo() As Double {#public-property-get-tto-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarTo.

## Public Property Let So(ByVal vData As Double) {#public-property-let-sobyval-vdata-as-double .MyHeader}

## Public Property Get So() As Double {#public-property-get-so-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarSo.

## Public Property Let Po(ByVal vData As Double) {#public-property-let-pobyval-vdata-as-double .MyHeader}

## Public Property Get Po() As Double {#public-property-get-po-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPo.

## Private Sub Class_Initialize() {#private-sub-class_initialize .MyHeader}

Este procedimiento asigna los siguiente valores (en unidades SI) por
defecto a las variables internas de la *clase*:

mvarPo = 101325 mvarTTo = 273.15 mvarHo = 0 mvarSo =0

## clsSatPressure {#clssatpressure .MyHeader}

*Clase* creada con el objetivo de poder calcular presiones y
temperaturas de saturación a través de correlaciones existentes en la
literatura (Riedel, Müller, RPM).

## Public Enum TADiPSatPressureModel {#public-enum-tadipsatpressuremodel .MyHeader}

##  Riedel = 1 {#riedel-1 .MyHeader}

##  Muller = 2 {#muller-2 .MyHeader}

##  RPM = 3  {#rpm-3 .MyHeader}

## End Enum {#end-enum-1 .MyHeader}

Enumeración que permite la escogencia de la correlación a utilizar para
el cálculo de las presiones de saturación de los compuestos.

## Private mvarCriticalProps As colclsCriticalProps {#private-mvarcriticalprops-as-colclscriticalprops-1 .MyHeader}

Variable que guarda una copia de la colección colclsCriticalProps.

## Private mvarOtherProps As colclsOtherProps {#private-mvarotherprops-as-colclsotherprops-1 .MyHeader}

Variable que guarda una copia de la colección colclsOtherProps.

## Private mvarSatPressureModel As TADiPSatPressureModel {#private-mvarsatpressuremodel-as-tadipsatpressuremodel-1 .MyHeader}

Variable que guarda el valor que indica que correlación para la presión
de saturación se utilizará para el cálculo de la propiedad.

## Public Property Let SatPressureModel(ByVal vData As TADiPSatPressureModel) {#public-property-let-satpressuremodelbyval-vdata-as-tadipsatpressuremodel-1 .MyHeader}

## Public Property Get SatPressureModel() As TADiPSatPressureModel {#public-property-get-satpressuremodel-as-tadipsatpressuremodel-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarSatPressureModel.

## Public Property Get CriticalConstants() As colclsCriticalProps {#public-property-get-criticalconstants-as-colclscriticalprops .MyHeader}

## Public Property Set CriticalConstants(vData As colclsCriticalProps) {#public-property-set-criticalconstantsvdata-as-colclscriticalprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProps.

## Public Property Get OtherConstants() As colclsOtherProps {#public-property-get-otherconstants-as-colclsotherprops .MyHeader}

## Public Property Set OtherConstants(vData As colclsCriticalProps) {#public-property-set-otherconstantsvdata-as-colclscriticalprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarOtherProps.

## Public Function SatPressure(ByVal i As Integer, ByVal Temperature As Double, ByRef SatP As Double) As Long {#public-function-satpressurebyval-i-as-integer-byval-temperature-as-double-byref-satp-as-double-as-long .MyHeader}

Función que calcula la presión de saturación del compuesto i de la
mezcla a la temperatura dada por el parámetro Temperature y la
correlación contenida en la variable mvarSatPressureModel. El resultado
de la función es arrojado en la variable SatP

## Public Function DSatPressure_T(ByVal i As Integer, ByVal Temperature As Double) As Double {#public-function-dsatpressure_tbyval-i-as-integer-byval-temperature-as-double-as-double .MyHeader}

Función que calcula la derivada de la presión de saturación respecto a
la temperatura del compuesto i de la mezcla, a la temperatura dada por
el parámetro Temperature y la correlación contenida en la variable
mvarSatPressureModel.

## Public Function SatTemperature(ByVal i As Integer, ByVal Prsat As Double, ByRef SatT As Double) As Long {#public-function-sattemperaturebyval-i-as-integer-byval-prsat-as-double-byref-satt-as-double-as-long .MyHeader}

Función que calcula la temperatura de saturación del compuesto i de la
mezcla a la presión reducida dada por el parámetro Prsta y la
correlación contenida en la variable mvarSatPressureModel. El resultado
de la función es arrojado en la variable SatT.

## Private Function Regula_Falsi_M(Result As Double, ByVal x1 As Double, ByVal x2 As Double, Precision As Double, i As Integer) {#private-function-regula_falsi_mresult-as-double-byval-x1-as-double-byval-x2-as-double-precision-as-double-i-as-integer .MyHeader}

Esta función aplica el método de Regula Falsi modificado a la función
SatPressure con el objetivo de calcular la temperatura de saturación.

## Private Function ZZZ_Greater_Abs(x1 As Double, x2 As Double) As Double {#private-function-zzz_greater_absx1-as-double-x2-as-double-as-double .MyHeader}

Esta función escoge entre x1 y x2 para obtener el que presente mayor
valor absoluto. Es una función auxiliar utilizada por la función
Regula_Falsi_M.

## Private Function Funcion(X As Double, i As Integer) As Double {#private-function-funcionx-as-double-i-as-integer-as-double .MyHeader}

Función auxiliar utilizada para el cálculo de la presión de saturación
dentro del método numérico de Regula Falsi Modificado.

## clsTolerances {#clstolerances .MyHeader}

*Clase* que guarda las tolerancias a utilizar en los procedimientos
iterativos de cálculo.

## Private mvarTolPressure As Double {#private-mvartolpressure-as-double .MyHeader}

Variable que guarda el valor de la tolerancia a utilizar en los cálculos
de presión.

## Private mvarTolTemperature As Double {#private-mvartoltemperature-as-double .MyHeader}

Variable que guarda el valor de la tolerancia a utilizar en los cálculos
de temperatura.

## Private mvarMaxIter As Integer {#private-mvarmaxiter-as-integer .MyHeader}

Variable que guarda el valor del número máximo de iteraciones que un
procedimiento pueda manejar.

## Private mvarTolMolarFraction As Double {#private-mvartolmolarfraction-as-double .MyHeader}

Variable que guarda el valor de la tolerancia aceptada en los cálculos
de las fracciones molares de los compuestos

## Public Property Let TolMolarFraction (ByVal vData As Double) {#public-property-let-tolmolarfraction-byval-vdata-as-double .MyHeader}

## Public Property Get TolMolarFraction() As Double {#public-property-get-tolmolarfraction-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProps.

## Public Property Let MaxIter(ByVal vData As Integer) {#public-property-let-maxiterbyval-vdata-as-integer .MyHeader}

## Public Property Get MaxIter() As Integer {#public-property-get-maxiter-as-integer .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProps.

## Public Property Let TolTemperature(ByVal vData As Double) {#public-property-let-toltemperaturebyval-vdata-as-double .MyHeader}

## Public Property Get TolTemperature() As Double {#public-property-get-toltemperature-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProps.

## Public Property Let TolPressure(ByVal vData As Double) {#public-property-let-tolpressurebyval-vdata-as-double .MyHeader}

## Public Property Get TolPressure() As Double {#public-property-get-tolpressure-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProps.

## Private Sub Class_Initialize() {#private-sub-class_initialize-1 .MyHeader}

Este procedimiento asigna los siguientes valores por defecto a las
variables internas de la *clase*:

mvarTolPressure = 1 mvarMolarFraction = 0.1 mvarTolTemperature = 1
mvarMaxIter =0

##  clsVirial {#clsvirial .MyHeader}

*Clase* diseñada para calcular propiedades de compuestos puros a través
de la ecuación virial truncada hasta el segundo término.

## Private mvarAcentricFactor As Double {#private-mvaracentricfactor-as-double-1 .MyHeader}

Variable que guarda el valor del factor acéntrico de un compuesto.

## Public Property Let AcentricFactor(ByVal vData As Double) {#public-property-let-acentricfactorbyval-vdata-as-double .MyHeader}

## Public Property Get AcentricFactor() As Double {#public-property-get-acentricfactor-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAcentricFactor.

## Public Function CalculateZ(Tr As Double, Pr As Double) As Double {#public-function-calculateztr-as-double-pr-as-double-as-double .MyHeader}

Función que calcula el factor de compresibilidad de un compuesto a las
condiciones de temperatura y presión reducidas dadas por los parámetros
Tr y Pr.

## Public Function CalculateFugacity(Tr As Double, Pr As Double) As Double {#public-function-calculatefugacitytr-as-double-pr-as-double-as-double .MyHeader}

Función que calcula el coeficiente de fugacidad de un compuesto a las
condiciones de temperatura y presión reducidas dadas por los parámetros
Tr y Pr.

## clsVirialMulticomp {#clsvirialmulticomp .MyHeader}

*Clase* diseñada para calcular propiedades de mezclas multicomponentes a
través de la ecuación virial truncada hasta el segundo término.

## Private mvarCriticalProperties As colclsCriticalProps {#private-mvarcriticalproperties-as-colclscriticalprops .MyHeader}

Variable que guarda una copia de la colección colclsCriticalProps.

## Private mvarMolarFraction() As Double {#private-mvarmolarfraction-as-double-1 .MyHeader}

Variable que guarda las fracciones molares de los compuesto en la
mezcla.

## Private mvarKijParameters As Variant {#private-mvarkijparameters-as-variant .MyHeader}

Variable que guarda los parámetros de interacción binaria existentes
entre los pares de compuestos de la mezcla.

## Public Property Let KijParameters(ByVal vData As Variant) {#public-property-let-kijparametersbyval-vdata-as-variant .MyHeader}

## Public Property Get KijParameters() As Variant {#public-property-get-kijparameters-as-variant .MyHeader}

## Public Property Set KijParameters(ByVal vData As Object) {#public-property-set-kijparametersbyval-vdata-as-object .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarKijParameters.

## Public Property Get MolarFraction(Index As Integer) As Double {#public-property-get-molarfractionindex-as-integer-as-double-2 .MyHeader}

## Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As Double) {#public-property-let-molarfractionindex-as-integer-byval-molarfraction-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarMolarFraction.

## Public Property Set CriticalProperties(ByVal vData As colclsCriticalProps) {#public-property-set-criticalpropertiesbyval-vdata-as-colclscriticalprops .MyHeader}

## Public Property Get CriticalProperties() As colclsCriticalProps {#public-property-get-criticalproperties-as-colclscriticalprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarCriticalProperties.

## Public Function CalculatePartialFugacity(K As Integer, T As Double, P As Double) As Double {#public-function-calculatepartialfugacityk-as-integer-t-as-double-p-as-double-as-double .MyHeader}

Función que calcula el coeficiente de fugacidad parcial del compuesto k
dentro de la mezcla, a las condiciones dadas por la temperatura y
presión representadas por los parámetros T y P respectivamente.

## Public Function CalculateZ(T As Double, P As Double) As Double {#public-function-calculatezt-as-double-p-as-double-as-double .MyHeader}

Función que calcula el factor de compresibilidad de la mezcla a las
condiciones dadas por la temperatura y presión representadas por los
parámetros T y P respectivamente.

## Public Function HR(P As Double, T As Double) As Double {#public-function-hrp-as-double-t-as-double-as-double .MyHeader}

Función que calcula la entalpía residual a las condiciones dadas por la
temperatura y presión representadas por los parámetros T y P
respectivamente.

## Public Function SR(P As Double, T As Double) As Double {#public-function-srp-as-double-t-as-double-as-double .MyHeader}

Función que calcula la entropía residual a las condiciones dadas por la
temperatura y presión representadas por los parámetros T y P
respectivamente.

## Private Function calculateBij(i As Integer, j As Integer, T As Double) As Double {#private-function-calculatebiji-as-integer-j-as-integer-t-as-double-as-double .MyHeader}

## Private Function Deltaik(i As Integer, j As Integer, T As Double) As Double {#private-function-deltaiki-as-integer-j-as-integer-t-as-double-as-double .MyHeader}

## Private Function CalculatedBijdT(i As Integer, j As Integer, T As Double) As Double {#private-function-calculatedbijdti-as-integer-j-as-integer-t-as-double-as-double .MyHeader}

## Private Function calculateB(T As Double) As Double {#private-function-calculatebt-as-double-as-double .MyHeader}

## Private Function calculatedBdT(T As Double) As Double {#private-function-calculatedbdtt-as-double-as-double .MyHeader}

Funciones auxiliares en la aplicación de la EVT a mezclas
multicomponentes.

## colclsActivityProps {#colclsactivityprops .MyHeader}

Colección de las propiedades de todos los compuestos necesarias para el
uso de los modelos termodinámicos basados en el coeficiente de
actividad.

## Private mCol As Collection {#private-mcol-as-collection .MyHeader}

Variable que guarda una colección de *clase*s.

## Private mvarAij() As Double {#private-mvaraij-as-double .MyHeader}

Variable que guarda los parámetros Aij de los modelos Gamma.

## Private mvarAijTemperature As Double {#private-mvaraijtemperature-as-double .MyHeader}

Variable que guarda la temperatura a la cual se obtuvieron los valores
de Aij introducidos en la variable explicada anteriormente .

## Public Property Let Aij(i As Integer, j As Integer, ByVal vData As Double) {#public-property-let-aiji-as-integer-j-as-integer-byval-vdata-as-double .MyHeader}

## Public Property Get Aij(i As Integer, j As Integer) As Double {#public-property-get-aiji-as-integer-j-as-integer-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAij().

## Public Function Add(delta As Double, vl As Double, Optional sKey As String) As clsActivityProps {#public-function-adddelta-as-double-vl-as-double-optional-skey-as-string-as-clsactivityprops .MyHeader}

Función que permite agregar un elemento a la colección.

## Public Property Get Item(vntIndexKey As Variant) As clsActivityProps {#public-property-get-itemvntindexkey-as-variant-as-clsactivityprops .MyHeader}

Propiedad que permite acceder a uno de los elementos existentes dentro
de la colección.

## Public Property Get Count() As Long {#public-property-get-count-as-long .MyHeader}

Propiedad que permite conocer el número de elementos presentes en la
colección.

## Public Sub Remove(vntIndexKey As Variant) {#public-sub-removevntindexkey-as-variant .MyHeader}

Procedimiento que permite borrar un elemento de la colección

## Public Property Get NewEnum() As IUnknown {#public-property-get-newenum-as-iunknown .MyHeader}

Esta es una propiedad especial de las colecciones que permite utilizar
la sentencia "For Each" dentro del cuerpo del programa.

## Private Sub Class_Initialize() {#private-sub-class_initialize-2 .MyHeader}

Crea la colección mCol.

## Private Sub Class_Terminate() {#private-sub-class_terminate-2 .MyHeader}

Destruye la colección mCol.

## Public Property Let AijTemperature(ByVal vData As Double) {#public-property-let-aijtemperaturebyval-vdata-as-double .MyHeader}

## Public Property Get AijTemperature() As Double {#public-property-get-aijtemperature-as-double .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAijTemperature.

## colclsCpConsts {#colclscpconsts .MyHeader}

Colección de los coeficientes de todos los compuestos necesarios para el
cálculo de los Cp de cada uno de ellos.

## Private mCol As Collection {#private-mcol-as-collection-1 .MyHeader}

Variable que guarda una colección de *clase*s.

## Public Function Add(Formula As Integer, Coef() As Double, Optional sKey As String)  {#public-function-addformula-as-integer-coef-as-double-optional-skey-as-string .MyHeader}

Función que permite agregar un elemento a la colección.

## Public Property Get Item(vntIndexKey As Variant) As clsCpConsts {#public-property-get-itemvntindexkey-as-variant-as-clscpconsts .MyHeader}

Propiedad que permite acceder a uno de los elementos existentes dentro
de la colección.

## Public Property Get Count() As Long {#public-property-get-count-as-long-1 .MyHeader}

Propiedad que permite conocer el número de elementos presentes en la
colección.

## Public Sub Remove(vntIndexKey As Variant) {#public-sub-removevntindexkey-as-variant-1 .MyHeader}

Procedimiento que permite borrar un elemento de la colección

## Public Property Get NewEnum() As IUnknown {#public-property-get-newenum-as-iunknown-1 .MyHeader}

Esta es una propiedad especial de las colecciones que permite utilizar
las sentencia "For Each" dentro del cuerpo del programa.

## Private Sub Class_Initialize() {#private-sub-class_initialize-3 .MyHeader}

Crea la colección mCol.

## Private Sub Class_Terminate() {#private-sub-class_terminate-3 .MyHeader}

Destruye la colección mCol.

## colclsCriticalProps {#colclscriticalprops .MyHeader}

Colección de las propiedades críticas de todos los compuestos.

## Private mCol As Collection {#private-mcol-as-collection-2 .MyHeader}

Variable que guarda una colección de *clase*s.

## Public Function Add(Tc As Double, Pc As Double, w As Double, Zc As Double, Optional sKey As String) As clsCriticalProps {#public-function-addtc-as-double-pc-as-double-w-as-double-zc-as-double-optional-skey-as-string-as-clscriticalprops .MyHeader}

Función que permite agregar un elemento a la colección.

## Public Property Get Item(vntIndexKey As Variant) As clsCpConsts {#public-property-get-itemvntindexkey-as-variant-as-clscpconsts-1 .MyHeader}

Propiedad que permite acceder a uno de los elementos existentes dentro
de la colección.

## Public Property Get Count() As Long {#public-property-get-count-as-long-2 .MyHeader}

Propiedad que permite conocer el número de elementos presentes en la
colección.

## Public Sub Remove(vntIndexKey As Variant) {#public-sub-removevntindexkey-as-variant-2 .MyHeader}

Procedimiento que permite borrar un elemento de la colección

## Public Property Get NewEnum() As IUnknown {#public-property-get-newenum-as-iunknown-2 .MyHeader}

Esta es una propiedad especial de las colecciones que permite utilizar
las sentencia "For Each" dentro del cuerpo del programa.

## Private Sub Class_Initialize() {#private-sub-class_initialize-4 .MyHeader}

Crea la colección mCol.

## Private Sub Class_Terminate() {#private-sub-class_terminate-4 .MyHeader}

Destruye la colección mCol.

## colclsOtherProps {#colclsotherprops .MyHeader}

Colección de las propiedades que no fueron clasificadas de todos los
compuestos y que son necesarias en ciertos cálculos.

## Private mvarAij(1 To 100, 1 To 100) As Double {#private-mvaraij1-to-100-1-to-100-as-double .MyHeader}

Variable que guarda los parámetros Aij de los modelos Gamma.

## Private mvarAijTemperature As Double {#private-mvaraijtemperature-as-double-1 .MyHeader}

Variable que guarda la temperatura a la cual se obtuvieron los valores
de Aij introducidos en la variable explicada anteriormente .

## Private mCol As Collection {#private-mcol-as-collection-3 .MyHeader}

Variable que guarda una colección de *clase*s.

## Private mvarKij As Variant {#private-mvarkij-as-variant .MyHeader}

Variable que guarda los parámetros de interacción binaria existentes
entre los pares de compuestos de la mezcla.

## Public Property Let Kij(ByVal vData As Variant) {#public-property-let-kijbyval-vdata-as-variant .MyHeader}

## Public Property Set Kij(ByVal vData As Object) {#public-property-set-kijbyval-vdata-as-object .MyHeader}

## Public Property Get Kij() As Variant {#public-property-get-kij-as-variant .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarKij.

## Public Property Let Aij(i As Integer, j As Integer, ByVal vData As Double) {#public-property-let-aiji-as-integer-j-as-integer-byval-vdata-as-double-1 .MyHeader}

## Public Property Get Aij(i As Integer, j As Integer) As Double {#public-property-get-aiji-as-integer-j-as-integer-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAij().

## Public Function Add(delta As Double, vl As Double, Tb As Double, DipMoment As Double, m As Double, n As Double, g As Double, K1 As Double, PsatAtTr As Double, Optional sKey As String) As clsOtherProps {#public-function-adddelta-as-double-vl-as-double-tb-as-double-dipmoment-as-double-m-as-double-n-as-double-g-as-double-k1-as-double-psatattr-as-double-optional-skey-as-string-as-clsotherprops .MyHeader}

Función que permite agregar un elemento a la colección.

## Public Property Get Item(vntIndexKey As Variant) As clsCpConsts {#public-property-get-itemvntindexkey-as-variant-as-clscpconsts-2 .MyHeader}

Propiedad que permite acceder a uno de los elementos existentes dentro
de la colección.

## Public Property Get Count() As Long {#public-property-get-count-as-long-3 .MyHeader}

Propiedad que permite conocer el número de elementos presentes en la
colección.

## Public Sub Remove(vntIndexKey As Variant) {#public-sub-removevntindexkey-as-variant-3 .MyHeader}

Procedimiento que permite borrar un elemento de la colección

## Public Property Get NewEnum() As IUnknown {#public-property-get-newenum-as-iunknown-3 .MyHeader}

Esta es una propiedad especial de las colecciones que permite utilizar
la sentencia "For Each" dentro del cuerpo del programa.

## Private Sub Class_Initialize() {#private-sub-class_initialize-5 .MyHeader}

Crea la colección mCol.

## Private Sub Class_Terminate() {#private-sub-class_terminate-5 .MyHeader}

Destruye la colección mCol.

## Public Property Let AijTemperature(ByVal vData As Double) {#public-property-let-aijtemperaturebyval-vdata-as-double-1 .MyHeader}

## Public Property Get AijTemperature() As Double {#public-property-get-aijtemperature-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAijTemperature.

## colclsQbicsProps {#colclsqbicsprops .MyHeader}

Colección de las propiedades de todos los compuestos necesarias para el
uso de los modelos termodinámicos basados en las EDEC.

## Private mCol As Collection {#private-mcol-as-collection-4 .MyHeader}

Variable que guarda una colección de *clase*s.

## Public Function Add(Tc As Double, Pc As Double, w As Double, Zc As Double, m As Double, n As Double, g As Double, PsatAtTr As Double, Optional sKey As String) As clsQbicsProps {#public-function-addtc-as-double-pc-as-double-w-as-double-zc-as-double-m-as-double-n-as-double-g-as-double-psatattr-as-double-optional-skey-as-string-as-clsqbicsprops .MyHeader}

Función que permite agregar un elemento a la colección.

## Public Property Get Item(vntIndexKey As Variant) As clsCpConsts {#public-property-get-itemvntindexkey-as-variant-as-clscpconsts-3 .MyHeader}

Propiedad que permite acceder a uno de los elementos existentes dentro
de la colección.

## Public Property Get Count() As Long {#public-property-get-count-as-long-4 .MyHeader}

Propiedad que permite conocer el número de elementos presentes en la
colección.

## Public Sub Remove(vntIndexKey As Variant) {#public-sub-removevntindexkey-as-variant-4 .MyHeader}

Procedimiento que permite borrar un elemento de la colección

## Public Property Get NewEnum() As IUnknown {#public-property-get-newenum-as-iunknown-4 .MyHeader}

Esta es una propiedad especial de las colecciones que permite utilizar
la sentencia "For Each" dentro del cuerpo del programa.

## Private Sub Class_Initialize() {#private-sub-class_initialize-6 .MyHeader}

Crea la colección mCol.

## Private Sub Class_Terminate() {#private-sub-class_terminate-6 .MyHeader}

Destruye la colección mCol.

## clsQbicsMulticomp {#clsqbicsmulticomp .MyHeader}

*Clase* diseñada con el objetivo de aplicar las EDEC a problemas de
mezclas multicomponentes.

## Public Enum TADiPPhiModel {#public-enum-tadipphimodel .MyHeader}

##  IdealSolutionp = 25 {#idealsolutionp-25 .MyHeader}

##  IdealGas = -2 {#idealgas--2 .MyHeader}

##  PR1976_Qbicp = 0 {#pr1976_qbicp-0 .MyHeader}

##  RK1949_Qbicp = 1 {#rk1949_qbicp-1 .MyHeader}

##  RKS1972_Qbicp = 2 {#rks1972_qbicp-2 .MyHeader}

##  VdW1870_Qbicp = 3 {#vdw1870_qbicp-3 .MyHeader}

##  PRL1997_Qbicp = 4 {#prl1997_qbicp-4 .MyHeader}

##  RKSL1997_Qbicp = 5 {#rksl1997_qbicp-5 .MyHeader}

##  RKSGD1978_Qbicp = 6 {#rksgd1978_qbicp-6 .MyHeader}

##  RP1978_Qbicp = 7 {#rp1978_qbicp-7 .MyHeader}

##  Berth1899_Qbicp = 8 {#berth1899_qbicp-8 .MyHeader}

##  VdWAda1984_Qbicp = 9 {#vdwada1984_qbicp-9 .MyHeader}

##  VdWVald1989_Qbicp = 10 {#vdwvald1989_qbicp-10 .MyHeader}

##  RKSmn1980_Qbicp = 11 {#rksmn1980_qbicp-11 .MyHeader}

##  RKSATmn1995_Qbicp = 12 {#rksatmn1995_qbicp-12 .MyHeader}

##  PRATmng1997_Qbicp = 13 {#pratmng1997_qbicp-13 .MyHeader}

##  PRMmn1989_Qbicp = 14 {#prmmn1989_qbicp-14 .MyHeader}

##  PRSV1986_Qbicp = 15 {#prsv1986_qbicp-15 .MyHeader}

##  VdWOL1998_Qbicp = 16 {#vdwol1998_qbicp-16 .MyHeader}

##  RKOL1998_Qbicp = 17 {#rkol1998_qbicp-17 .MyHeader}

##  PROL1998_Qbicp = 18 {#prol1998_qbicp-18 .MyHeader}

##  End Enum {#end-enum-2 .MyHeader}

Enumeración que permite la selección de la ecuación de estado cúbica.

## Public Enum TADiPPhaseID {#public-enum-tadipphaseid .MyHeader}

##  PhaseIDvapor = 0 {#phaseidvapor-0 .MyHeader}

##  PhaseIDLiquid = 1 {#phaseidliquid-1 .MyHeader}

## End Enum {#end-enum-3 .MyHeader}

Enumeración que permite seleccionar si la fase de la mezcla es liquida o
vapor.

## Public Enum TADiPdim {#public-enum-tadipdim .MyHeader}

##  WithDim = 0 {#withdim-0 .MyHeader}

##  DimLess = 1 {#dimless-1 .MyHeader}

## End Enum {#end-enum-4 .MyHeader}

Enumeración que permite escoger si los parámetros de las ecuaciones de
estado cúbicas se calcularán en su forma adimensional o no.

## Private gintK1 As Integer, gIntK2 As Integer, gintK3 As Integer {#private-gintk1-as-integer-gintk2-as-integer-gintk3-as-integer .MyHeader}

## Private gdblOmA As Double, gdblOmB As Double, gdblOmC As Double {#private-gdbloma-as-double-gdblomb-as-double-gdblomc-as-double .MyHeader}

## Private gdblH(0 To 4) As Double {#private-gdblh0-to-4-as-double .MyHeader}

Estas variables guardan los valores de los parámetros que describen las
ecuaciones cúbicas. En especial la variable gdblH() guarda los
coeficientes necesarios para el cálculo de la presión de saturación
mediante las correlaciones desarrolladas por el grupo TADiP.

## Private Type ParametersEOS {#private-type-parameterseos .MyHeader}

##  A As Double {#a-as-double .MyHeader}

##  B As Double {#b-as-double .MyHeader}

##  C As Double {#c-as-double .MyHeader}

## End Type {#end-type .MyHeader}

Estructura que contiene los valores de los coeficientes de las
ecuaciones cúbicas de estado.

## Private mvarModel As TADiPPhiModel {#private-mvarmodel-as-tadipphimodel .MyHeader}

Variable que guarda la ecuación cúbica de estado que se utilizará en el
cálculo de las funciones presentes en la *clase*.

## Private mParameter As ParametersEOS {#private-mparameter-as-parameterseos .MyHeader}

Instancia de la estructura ParametersEOS utilizada para guardar los
valores calculados para la mezcla.

## Private mParameteri() As ParametersEOS {#private-mparameteri-as-parameterseos .MyHeader}

Instancia de la estructura ParametersEOS utilizada para guardar los
valores calculados para el componente i.

## Private mvarMixingRule As Integer {#private-mvarmixingrule-as-integer .MyHeader}

Variable donde se guarda la escogencia de la regla de mezclado a
utilizar en el uso de la *clase*.

## Private mvarPhaseOfMixture As TADiPPhaseID {#private-mvarphaseofmixture-as-tadipphaseid .MyHeader}

Instancia de la estructura TADiPPhaseID, utilizada para guardar el valor
que indica la fase en la cual se encuentra la mezcla.

## Private mvarMolarFraction() As Double {#private-mvarmolarfraction-as-double-2 .MyHeader}

Variable donde se guardan las fracciones molares de los componentes de
la mezcla en la fase indicada por la variable mvarPhaseOfMixture.

##  Private mvarProperties As colclsQbicsProps {#private-mvarproperties-as-colclsqbicsprops .MyHeader}

Variable donde se guarda una copia de la colección colclsQbicsProps .

## Private mvarKij As Variant  {#private-mvarkij-as-variant-1 .MyHeader}

Variable donde se guardan los valores de los parámetros de interacción
binaria de los componentes existentes en la mezcla.

## Public Property Let Kij(ByVal vData As Variant) {#public-property-let-kijbyval-vdata-as-variant-1 .MyHeader}

## Public Property Set Kij(ByVal vData As Object) {#public-property-set-kijbyval-vdata-as-object-1 .MyHeader}

## Public Property Get Kij() As Variant {#public-property-get-kij-as-variant-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarKij.

## Public Property Get Properties() As colclsQbicsProps {#public-property-get-properties-as-colclsqbicsprops .MyHeader}

## Public Property Set Properties(vData As colclsQbicsProps) {#public-property-set-propertiesvdata-as-colclsqbicsprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarProperties.

## Public Property Let PhaseOfMixture(ByVal vData As TADiPPhaseID) {#public-property-let-phaseofmixturebyval-vdata-as-tadipphaseid .MyHeader}

## Public Property Get PhaseOfMixture() As TADiPPhaseID {#public-property-get-phaseofmixture-as-tadipphaseid .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPhaseOfMixture.

## Public Property Let MixingRule(ByVal vData As Integer) {#public-property-let-mixingrulebyval-vdata-as-integer .MyHeader}

## Public Property Get MixingRule() As Integer {#public-property-get-mixingrule-as-integer .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarMixingRule.

## Public Property Get Model() As TADiPPhiModel {#public-property-get-model-as-tadipphimodel .MyHeader}

## Public Property Let Model(ByVal vData As TADiPPhiModel) {#public-property-let-modelbyval-vdata-as-tadipphimodel .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarModel.

## Public Property Get MolarFraction(Index As Integer) As Double {#public-property-get-molarfractionindex-as-integer-as-double-3 .MyHeader}

## Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As Double) {#public-property-let-molarfractionindex-as-integer-byval-molarfraction-as-double-2 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarMolarFraction().

## Public Function PartialFugacityCoeficientInSolution(Component As Integer, Temperature As Double, Pressure As Double, ByRef PartialFugacityCoeficient As Double, Optional Z As Double) As Long {#public-function-partialfugacitycoeficientinsolutioncomponent-as-integer-temperature-as-double-pressure-as-double-byref-partialfugacitycoeficient-as-double-optional-z-as-double-as-long .MyHeader}

Función que calcula el coeficiente de fugacidad parcial del compuesto
Component dentro de la mezcla a las condiciones de temperatura y presión
dadas. El resultado de la función se devuelve en la variable
PartialFugacityCoeficient.

## Private Sub MixingRules(ByVal T As Double, P As Double, Dimensions As TADiPdim) {#private-sub-mixingrulesbyval-t-as-double-p-as-double-dimensions-as-tadipdim .MyHeader}

Este procedimiento permite el cálculo de los parámetros de las
ecuaciones de estado cúbicas según la regla de mezclado seleccionada.

## Private Function AAi(i As Integer, T As Double, P As Double, Dimensions As TADiPdim) As Double {#private-function-aaii-as-integer-t-as-double-p-as-double-dimensions-as-tadipdim-as-double .MyHeader}

## Private Function BBi(ByVal i As Integer, ByVal T As Double, ByVal P As Double, Dimensions As TADiPdim) As Double {#private-function-bbibyval-i-as-integer-byval-t-as-double-byval-p-as-double-dimensions-as-tadipdim-as-double .MyHeader}

Funciones auxiliares que calculan los parámetros de las ecuaciones de
estado cúbicas para el componente i de la mezcla.

## Public Function ZSolver(Optional Phase As TADiPPhaseID) As Double {#public-function-zsolveroptional-phase-as-tadipphaseid-as-double .MyHeader}

Calcula el factor de compresibilidad de la mezcla en la fase descrita
por la variable opcional Phase.

## Private Sub Class_Terminate() {#private-sub-class_terminate-7 .MyHeader}

Este procedimiento destruye el contenido de la variable mvarProperties.

## Public Function PWithDimensions(ByVal T As Double, ByVal v As Double) As Double {#public-function-pwithdimensionsbyval-t-as-double-byval-v-as-double-as-double .MyHeader}

Función que calcula la presión de la mezcla basada en el volumen y no en
el factor de compresibilidad de la misma, es decir, que no se usa la
forma adimensional de los parámetros de la ecuación cubica.

## Private Sub CalculateJacobianP(ByVal T As Double, ByVal v As Double, ByRef Jacob() As Double) {#private-sub-calculatejacobianpbyval-t-as-double-byval-v-as-double-byref-jacob-as-double .MyHeader}

Procedimiento que calcula el Jacobiano necesario para el cálculo de las
propiedades pseudocríticas.

## Private Function FirstDerivateP(T As Double, v As Double) As Double {#private-function-firstderivatept-as-double-v-as-double-as-double .MyHeader}

Función que calcula la primera derivada de la presión respecto al
volumen y la evalúa a la temperatura y presión dadas.

## Private Function SecondDerivateP(T As Double, v As Double) As Double {#private-function-secondderivatept-as-double-v-as-double-as-double .MyHeader}

Función que calcula la segunda derivada de la presión respecto al
volumen y la evalúa a la temperatura y presión dadas.

## Public Sub psCriticalProperties(ByRef tpsc As Double, ByRef vpsc As Double, ByRef ppsc As Double) {#public-sub-pscriticalpropertiesbyref-tpsc-as-double-byref-vpsc-as-double-byref-ppsc-as-double .MyHeader}

Función que calcula las propiedades pseudocríticas de la mezcla y las
devuelve en las variables tpsc, ppsc y vpsc (temperatura, presión y
volumen pseudocríticos respectivamente)

## Public Function CriticalProperties(ByRef Tc As Double, ByRef Pc As Double, ByRef vc As Double) As Boolean {#public-function-criticalpropertiesbyref-tc-as-double-byref-pc-as-double-byref-vc-as-double-as-boolean .MyHeader}

Función que calcula las propiedades críticas de la mezcla y las devuelve
en las variables Tc, Pc y vc.

## Private Function CP_Calculate_T(ByRef T As Double, ByVal v As Double, ByVal n As Integer, ByRef Dn() As Double) As Boolean {#private-function-cp_calculate_tbyref-t-as-double-byval-v-as-double-byval-n-as-integer-byref-dn-as-double-as-boolean .MyHeader}

Función que calcula la temperatura dentro del algoritmo de cálculo de
las propiedades críticas de mezcla.

## Private Function CP_FillQ(ByRef Q() As Double, ByVal T As Double, ByVal v As Double, n As Integer) As Boolean {#private-function-cp_fillqbyref-q-as-double-byval-t-as-double-byval-v-as-double-n-as-integer-as-boolean .MyHeader}

Función auxiliar en el cálculo de puntos críticos de sistemas
multicomponentes.

## Private Function AAij(ByVal i As Integer, ByVal j As Integer) As Double {#private-function-aaijbyval-i-as-integer-byval-j-as-integer-as-double .MyHeader}

Esta es una función auxiliar utilizada en el cálculo de los parámetros
de las EDEC mediante las reglas de mezclado.

## Private Function CP_D_lnFugacity_nt(ByVal i As Integer, ByVal j As Integer, ByVal  {#private-function-cp_d_lnfugacity_ntbyval-i-as-integer-byval-j-as-integer-byval .MyHeader}

Función que calcula la primera derivada del coeficiente de fugacidad del
componente i variando los moles del componente j de la mezcla.

## Private Function CP_SD_ln_Fug_nt(ByVal i As Integer, ByVal j As Integer, ByVal K As Integer, ByVal T As Double, ByVal v As Double, ByVal n As Integer) As Double {#private-function-cp_sd_ln_fug_ntbyval-i-as-integer-byval-j-as-integer-byval-k-as-integer-byval-t-as-double-byval-v-as-double-byval-n-as-integer-as-double .MyHeader}

Función que calcula la segunda derivada del coeficiente de fugacidad del
componente i variando los moles del componente j de la mezcla.

## Private Function CP_Calculate_C(T As Double, v As Double, ByRef Dn() As Double, n As Integer) As Double {#private-function-cp_calculate_ct-as-double-v-as-double-byref-dn-as-double-n-as-integer-as-double .MyHeader}

Permite calcular C en el algoritmo de cálculo de la propiedades críticas
de la mezcla.

## Private Sub EvaluateF(X() As Double, ByRef f() As Double) {#private-sub-evaluatefx-as-double-byref-f-as-double .MyHeader}

Función que evalúa las derivadas de la presión respecto a el volumen en
el algoritmo de cálculo de las propiedades pseudocríticas.

## Private Function Xnew(ByRef X() As Double, deltaX() As Double, ByVal n As Integer) {#private-function-xnewbyref-x-as-double-deltax-as-double-byval-n-as-integer .MyHeader}

Función que calcula los nuevos valores de las variable involucradas en
los métodos numéricos de Newton- Raphson.

## Public Function ResidualProperties(ByVal T As Double, ByVal P As Double, ByVal Z As Double, ByRef HR As Double, ByRef SR As Double) As Boolean {#public-function-residualpropertiesbyval-t-as-double-byval-p-as-double-byval-z-as-double-byref-hr-as-double-byref-sr-as-double-as-boolean .MyHeader}

Función que calcula las propiedades residuales de la mezcla (entalpía
residual y entropía residual) a las condiciones descritas por los
parámetros T, P y Z.

## Private Function a_T(T As Double) As Double {#private-function-a_tt-as-double-as-double .MyHeader}

## Private Function b_T(T As Double) As Double {#private-function-b_tt-as-double-as-double .MyHeader}

## Private Function ai\_\_bi(i As Integer, T As Double, a_i As Double, b_i As Double, ByVal n As Integer) {#private-function-ai__bii-as-integer-t-as-double-a_i-as-double-b_i-as-double-byval-n-as-integer .MyHeader}

Funciones auxiliares para el uso de las reglas de mezclado utilizadas
por las EDEC.

## Public Sub Kij_ReadExperimentalData(ByVal FilePath As String, ByRef Data() As Double, ByRef datasets As Integer, ByRef Dato As Double, ByRef isoterm As Boolean) {#public-sub-kij_readexperimentaldatabyval-filepath-as-string-byref-data-as-double-byref-datasets-as-integer-byref-dato-as-double-byref-isoterm-as-boolean .MyHeader}

Función que lee un archivo de texto para obtener de éste los datos
experimentales necesarios para el cálculo de los coeficientes de
interacción binarios existentes entre dos componentes.

## Public Function Kij_GoldenSearch(ax As Double, bx As Double, cx As Double, tol As Double, Xmin As Double, Dato As Double, datasets As Integer, ByRef Data() As Double, isoterm As Boolean) {#public-function-kij_goldensearchax-as-double-bx-as-double-cx-as-double-tol-as-double-xmin-as-double-dato-as-double-datasets-as-integer-byref-data-as-double-isoterm-as-boolean .MyHeader}

## Public Function Kij_BracketMin(ByRef ax As Double, ByRef bx As Double, ByRef cx As Double, ByRef fa As Double, ByRef fb As Double, ByRef fc As Double, datasets As Integer, Dato As Double, ByRef Data() As Double, isoterm As Boolean) As Double {#public-function-kij_bracketminbyref-ax-as-double-byref-bx-as-double-byref-cx-as-double-byref-fa-as-double-byref-fb-as-double-byref-fc-as-double-datasets-as-integer-dato-as-double-byref-data-as-double-isoterm-as-boolean-as-double .MyHeader}

## Public Function Kij_Qf(mmkij As Double, ByRef Data() As Double, datasets As Integer, Dato As Double, isoterm As Boolean) As Double {#public-function-kij_qfmmkij-as-double-byref-data-as-double-datasets-as-integer-dato-as-double-isoterm-as-boolean-as-double .MyHeader}

Funciones que permiten el cálculo de los parámteros de interacción
k~ij~.

## clsQbicsPure {#clsqbicspure .MyHeader}

*Clase* diseñada con el objetivo de aplicar las ecuaciones de estado
cúbicas a problemas de sustancias puras.

## Public Enum TADiPEDC {#public-enum-tadipedc .MyHeader}

##  PR1976_Qbic = 0 {#pr1976_qbic-0 .MyHeader}

##  RK1949_Qbic = 1 {#rk1949_qbic-1 .MyHeader}

##  RKS1972_Qbic = 2 {#rks1972_qbic-2 .MyHeader}

##  VdW1870_Qbic = 3 {#vdw1870_qbic-3 .MyHeader}

##  PRL1997_Qbic = 4 {#prl1997_qbic-4 .MyHeader}

##  RKSL1997_Qbic = 5 {#rksl1997_qbic-5 .MyHeader}

##  RKSGD1978_Qbic = 6 {#rksgd1978_qbic-6 .MyHeader}

##  RP1978_Qbic = 7 {#rp1978_qbic-7 .MyHeader}

##  Berth1899_Qbic = 8 {#berth1899_qbic-8 .MyHeader}

##  VdWAda1984_Qbic = 9 {#vdwada1984_qbic-9 .MyHeader}

##  VdWVald1989_Qbic = 10 {#vdwvald1989_qbic-10 .MyHeader}

##  RKSmn1980_Qbic = 11 {#rksmn1980_qbic-11 .MyHeader}

##  RKSATmn1995_Qbic = 12 {#rksatmn1995_qbic-12 .MyHeader}

##  PRATmng1997_Qbic = 13 {#pratmng1997_qbic-13 .MyHeader}

##  PRMmn1989_Qbic = 14 {#prmmn1989_qbic-14 .MyHeader}

##  PRSV1986_Qbic = 15 {#prsv1986_qbic-15 .MyHeader}

##  VdWOL1998_Qbic = 16 {#vdwol1998_qbic-16 .MyHeader}

##  RKOL1998_Qbic = 17 {#rkol1998_qbic-17 .MyHeader}

##  PROL1998_Qbic = 18 {#prol1998_qbic-18 .MyHeader}

## End Enum {#end-enum-5 .MyHeader}

Enumeración que indica las ecuaciones de estado cúbicas disponibles.

## Private gintK1 As Integer, gIntK2 As Integer, gintK3 As Integer {#private-gintk1-as-integer-gintk2-as-integer-gintk3-as-integer-1 .MyHeader}

## Private gdblOmA As Double, gdblOmB As Double, gdblOmC As Double {#private-gdbloma-as-double-gdblomb-as-double-gdblomc-as-double-1 .MyHeader}

Variables que guardan los valores de los parámetros que describen las
ecuaciones cúbicas.

## Private gdblH(0 To 4) As Double {#private-gdblh0-to-4-as-double-1 .MyHeader}

Variable que guarda los coeficientes necesarios para el cálculo de la
presión de saturación mediante las correlaciones desarrolladas por el
grupo TADiP.

## Private mvarCubicCoef1 As Double, mvarCubicCoef2 As Double, mvarCubicCoef3 As Double {#private-mvarcubiccoef1-as-double-mvarcubiccoef2-as-double-mvarcubiccoef3-as-double .MyHeader}

Variables que contienen los valores de los coeficientes del desarrollo
cúbico en volumen de las EDEC.

## Private Type ParametersEOS {#private-type-parameterseos-1 .MyHeader}

##  A As Double {#a-as-double-1 .MyHeader}

##  B As Double {#b-as-double-1 .MyHeader}

##  C As Double {#c-as-double-1 .MyHeader}

## End Type {#end-type-1 .MyHeader}

Estructura que contiene los valores de los coeficientes de las EDEC.

## Private mParameter As ParametersEOS {#private-mparameter-as-parameterseos-1 .MyHeader}

Instancia de la estructura ParametersEOS.

## Private mvarZc As Double {#private-mvarzc-as-double-1 .MyHeader}

Variable que contiene el factor de compresibilidad crítico para una
sustancia dada.

## Private Quality As Double {#private-quality-as-double .MyHeader}

Variable que contiene el valor de la calidad (cantidad de vapor
existente en el equilibrio líquido vapor de las sustancias puras).

## Private mvarPRSVK1 As Double {#private-mvarprsvk1-as-double .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Private mvarPsatAtTr As Double {#private-mvarpsatattr-as-double-1 .MyHeader}

Variable que contiene un parámetro de EDEC modificada.

## Private gintRaices_encontradas As Boolean {#private-gintraices_encontradas-as-boolean .MyHeader}

Variable que indica si se obtuvieron o no dos raíces reales dentro de
los cálculos del criterio de Maxwell.

## Private mvarQbicsWinEQofState As TADiPEDC {#private-mvarqbicswineqofstate-as-tadipedc .MyHeader}

Variable que contiene la ecuación cúbica de estado escogida.

## Private mvarAcentricFactor As Double {#private-mvaracentricfactor-as-double-2 .MyHeader}

Variable que contiene el factor acéntrico de la sustancia pura a tratar
dentro de la *clase*.

##  {#section-38 .MyHeader}

## Public Property Get m() As Double {#public-property-get-m-as-double-2 .MyHeader}

## Public Property Let m(ByVal vData As Double) {#public-property-let-mbyval-vdata-as-double-2 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarm.

## Public Property Get g() As Double {#public-property-get-g-as-double-2 .MyHeader}

## Public Property Let g(ByVal vData As Double) {#public-property-let-gbyval-vdata-as-double-2 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarg.

## Public Property Get n() As Double {#public-property-get-n-as-double-2 .MyHeader}

## Public Property Let n(ByVal vData As Double) {#public-property-let-nbyval-vdata-as-double-2 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarn.

## Public Property Get Model() As TADiPEDC {#public-property-get-model-as-tadipedc .MyHeader}

## Public Property Let Model(ByVal vData As TADiPEDC) {#public-property-let-modelbyval-vdata-as-tadipedc .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarQbicsWinEQofState.

## Public Property Let PRSVK1(ByVal vData As Double) {#public-property-let-prsvk1byval-vdata-as-double-1 .MyHeader}

## Public Property Get PRSVK1() As Double {#public-property-get-prsvk1-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPRSVK1.

## Public Property Let PsatAtTr(ByVal vData As Double) {#public-property-let-psatattrbyval-vdata-as-double-2 .MyHeader}

## Public Property Get PsatAtTr() As Double {#public-property-get-psatattr-as-double-2 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarPsatAtTTr.

## Public Property Let AcentricFactor(ByVal vData As Double) {#public-property-let-acentricfactorbyval-vdata-as-double-1 .MyHeader}

## Public Property Get AcentricFactor() As Double {#public-property-get-acentricfactor-as-double-1 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarAcentricFactor.

##  Public Property Get Zc() As Double {#public-property-get-zc-as-double-2 .MyHeader}

## Public Property Let Zc(ByVal vData As Double) {#public-property-let-zcbyval-vdata-as-double-2 .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarZc.

## Public Function PrXGivenTrvr(ReducedTemperature As Double, ReducedVolume As Double, ByRef ReducedPressure As Double, ByRef Quality As Double) As Boolean {#public-function-prxgiventrvrreducedtemperature-as-double-reducedvolume-as-double-byref-reducedpressure-as-double-byref-quality-as-double-as-boolean .MyHeader}

Función que calcula la presión reducida y la calidad, tomando como datos
la temperatura y el volumen reducidos del sistema.

## Private Function Prvr(Tr As Double, vr As Double, Alphac As Double) As Double {#private-function-prvrtr-as-double-vr-as-double-alphac-as-double-as-double .MyHeader}

Función que calcula la presión reducida.

## PublicFunction FugacityCoeficient(ReducedTemperature As Double, ReducedPressure As Double, CompresibilityFactor As Double, ByRef FugacityCoef As Double) As Boolean {#publicfunction-fugacitycoeficientreducedtemperature-as-double-reducedpressure-as-double-compresibilityfactor-as-double-byref-fugacitycoef-as-double-as-boolean .MyHeader}

Función que calcula el coeficiente de fugacidad de una sustancia pura
utilizando los datos de temperatura reducida, presión reducida y el
factor de compresibilidad a tales condiciones.

## Public Function ResidualEnthalpy(ReducedTemperature As Double, ReducedPressure As Double, CompresibilityFactor As Double, ByRef HR As Double) As Boolean {#public-function-residualenthalpyreducedtemperature-as-double-reducedpressure-as-double-compresibilityfactor-as-double-byref-hr-as-double-as-boolean .MyHeader}

Función que calcula la entalpía residual utilizando los datos de
temperatura reducida, presión reducida y el factor de compresibilidad a
tales condiciones.

## Public Function ResidualEntropy(ReducedTemperature As Double, ReducedPressure As Double, CompresibilityFactor As Double, ByRef SR As Double) As Boolean {#public-function-residualentropyreducedtemperature-as-double-reducedpressure-as-double-compresibilityfactor-as-double-byref-sr-as-double-as-boolean .MyHeader}

Función que calcula la entropía residual utilizando los datos de
temperatura reducida, presión reducida y el factor de compresibilidad a
tales condiciones.

## Public Function TrXGivenPrZ(ReducedPressure As Double, CompresibilityFactor As Double, ByRef ReducedTemperature As Double, ByRef Quality As Double) As Boolean {#public-function-trxgivenprzreducedpressure-as-double-compresibilityfactor-as-double-byref-reducedtemperature-as-double-byref-quality-as-double-as-boolean .MyHeader}

Función que calcula la temperatura reducida y la calidad utilizando los
datos de presión reducida y el factor de compresibilidad.

## Public Function TrZGivenPrX(ReducedPressure As Double, Quality As Double, ByRef ReducedTemperature As Double, ByRef CompresibilityFactor As Double) As Boolean {#public-function-trzgivenprxreducedpressure-as-double-quality-as-double-byref-reducedtemperature-as-double-byref-compresibilityfactor-as-double-as-boolean .MyHeader}

Función que calcula la temperatura reducida y el factor de
compresibilidad utilizando los datos de presión reducida y la calidad.

## Public Function TrvrGivenPrX(ReducedPressure As Double, Quality As Double, ByRef ReducedTemperature As Double, ByRef ReducedVolume As Double) As Boolean {#public-function-trvrgivenprxreducedpressure-as-double-quality-as-double-byref-reducedtemperature-as-double-byref-reducedvolume-as-double-as-boolean .MyHeader}

Función que calcula la temperatura reducida y el volumen reducido
utilizando los datos de presión reducida y la calidad.

## Public Function TrPrGivenvrX(RduceVolume As Double, ByVal Quality As Double, ByRef ReducedTemperature As Double, ByRef ReducedPressure As Double) As Boolean {#public-function-trprgivenvrxrducevolume-as-double-byval-quality-as-double-byref-reducedtemperature-as-double-byref-reducedpressure-as-double-as-boolean .MyHeader}

Función que calcula la temperatura reducida y la presión reducia
utilizando los datos de la calidad y el volumen reducido.

## Public Function PrZGivenTrX(ReducedTemperature As Double, Quality As Double, ByRef ReducedPressure As Double, ByRef CompresibilityFactor As Double) As Boolean {#public-function-przgiventrxreducedtemperature-as-double-quality-as-double-byref-reducedpressure-as-double-byref-compresibilityfactor-as-double-as-boolean .MyHeader}

Función que calcula la presión reducida y el factor de compresibilidad
utilizando los datos de calidad y la temperatura reducida.

## Public Function PrvrGivenTrX(ReducedTemperature As Double, Quality As Double, ByRef ReducedPressure As Double, ByRef ReducedVolume As Double) As Boolean {#public-function-prvrgiventrxreducedtemperature-as-double-quality-as-double-byref-reducedpressure-as-double-byref-reducedvolume-as-double-as-boolean .MyHeader}

Función que calcula la presión reducida y el volumen reducido utilizando
los datos la calidad y la temperatura reducida.

## Public Function MaxwellTestGivenPr(ReducedPressure As Double, ByRef ReducedTemperature As Double, ByRef VaporCompresibilityFactor As Double, ByRef LiquidCompresibilityFactor As Double) As Boolean {#public-function-maxwelltestgivenprreducedpressure-as-double-byref-reducedtemperature-as-double-byref-vaporcompresibilityfactor-as-double-byref-liquidcompresibilityfactor-as-double-as-boolean .MyHeader}

Función que aplica el criterio de Maxwell usando como dato la presión
reducida para obtener como resultados la temperatura reducida del
equilibrio líquido vapor y los factores de compresibilidad de vapor
saturado y líquido saturado.

## Public Function MaxwellTestGivenTr(ReducedTemperature As Double, ByRef ReducedPressure As Double, ByRef VaporCompresibilityFactor As Double, ByRef LiquidCompresibilityFactor As Double) As Boolean {#public-function-maxwelltestgiventrreducedtemperature-as-double-byref-reducedpressure-as-double-byref-vaporcompresibilityfactor-as-double-byref-liquidcompresibilityfactor-as-double-as-boolean .MyHeader}

Función que aplica el criterio de Maxwell usando como dato la
temperatura reducida para obtener como resultados la presión reducida
del equilibrio líquido vapor y los factores de compresibilidad de vapor
saturado y líquido saturado.

## Private Function Maxwell_RM(ByVal alphat As Double, ByVal vg As Double, ByVal vf As Double) As Double {#private-function-maxwell_rmbyval-alphat-as-double-byval-vg-as-double-byval-vf-as-double-as-double .MyHeader}

Función auxiliar utilizada en la aplicación de el criterio de Maxwell
utilizando el volumen reducido.

## Public Function MaxwellTestGivenTr_vr(ReducedTemperature As Double, ByRef ReducedPressure As Double, ByRef vrg As Double, ByRef vrf As Double) As Boolean {#public-function-maxwelltestgiventr_vrreducedtemperature-as-double-byref-reducedpressure-as-double-byref-vrg-as-double-byref-vrf-as-double-as-boolean .MyHeader}

Función que aplica el criterio de Maxwell usando como dato la
temperatura reducida para obtener como resultados la presión reducida
del equilibrio líquido vapor y los volúmenes reducidos de vapor saturado
y líquido saturado.

## Public Function PrXGivenTrZ(ReducedTemperature As Double, CompresibilityFactor As Double, ByRef ReducedPressure As Double, ByRef Quality As Double) As Boolean {#public-function-prxgiventrzreducedtemperature-as-double-compresibilityfactor-as-double-byref-reducedpressure-as-double-byref-quality-as-double-as-boolean .MyHeader}

Función que calcula la presión reducida y la calidad utilizando como
datos la temperatura reducida y el factor de compresibilidad.

## Public Function TrXGivenPrVr(ReducedPressure As Double, ReducedVolume As Double, ByRef ReducedTemperature As Double, ByRef Quality As Double) As Boolean {#public-function-trxgivenprvrreducedpressure-as-double-reducedvolume-as-double-byref-reducedtemperature-as-double-byref-quality-as-double-as-boolean .MyHeader}

Función que calcula la temperatura reducida y la calidad utilizando como
datos la presión reducida y el volumen reducido.

## Public Function ZGivenTrPr(ReducedTemperature As Double, ReducedPressure As Double, ByRef CompresibilityFactor As Double) As Boolean {#public-function-zgiventrprreducedtemperature-as-double-reducedpressure-as-double-byref-compresibilityfactor-as-double-as-boolean .MyHeader}

Función que calcula el factor de compresibilidad utilizando como datos
la presión reducida y la temperatura reducida.

## Public Function vrGivenTrPr(ReducedTemperature As Double, ReducedPressure As Double, ByRef ReducedVolume As Double) As Boolean {#public-function-vrgiventrprreducedtemperature-as-double-reducedpressure-as-double-byref-reducedvolume-as-double-as-boolean .MyHeader}

Función que calcula el volumen reducido utilizando como datos la presión
reducida y la temperatura reducida.

## Public Function MaxwellTestGivenPr_Vr(ReducedPressure As Double, ByRef ReducedTemperature As Double, ByRef vrg As Double, ByRef vrf As Double) As Boolean {#public-function-maxwelltestgivenpr_vrreducedpressure-as-double-byref-reducedtemperature-as-double-byref-vrg-as-double-byref-vrf-as-double-as-boolean .MyHeader}

Función que aplica el criterio de Maxwell usando como dato la presión
reducida para obtener como resultados la temperatura reducida del
equilibrio líquido vapor y los volúmenes reducidos de vapor saturado y
líquido saturado.

## Public Function TrPrGivenZX(CompresibilityFactor As Double, Quality As Double, ByRef ReducedTemperature As Double, ByRef ReducedPressure As Double) As Boolean {#public-function-trprgivenzxcompresibilityfactor-as-double-quality-as-double-byref-reducedtemperature-as-double-byref-reducedpressure-as-double-as-boolean .MyHeader}

Función que calcula la temperatura y presión reducida utilizando como
datos la calidad y el factor de compresibilidad.

## Private Function Regula_Falsi_M(Result As Double, ByVal x1 As Double, ByVal x2 As Double, Precision As Double, i As Integer) {#private-function-regula_falsi_mresult-as-double-byval-x1-as-double-byval-x2-as-double-precision-as-double-i-as-integer-1 .MyHeader}

Método numérico de Regula Falsi Modificado.

## Private Function ZZZ_Greater_Abs(x1 As Double, x2 As Double) As Double {#private-function-zzz_greater_absx1-as-double-x2-as-double-as-double-1 .MyHeader}

Función auxiliar utilizada por la función Regula_Falsi_M que compara los
valores absolutos de x1 y x2, devolviendo cual de los dos es más grande.

## Private Function Funcion(X As Double, i As Integer) As Double {#private-function-funcionx-as-double-i-as-integer-as-double-1 .MyHeader}

Esta es la función a resolver por el Regula Falsi Modificado.

## Private Function T_despejada(ReducedPressure As Double, CompresibilityFactor As Double, initTemperature As Double) {#private-function-t_despejadareducedpressure-as-double-compresibilityfactor-as-double-inittemperature-as-double .MyHeader}

Función que devuelve la temperatura reducida dadas la presión reducida y
el factor de compresibilidad.

## Private Function Tr_despejada(ReducedPressure As Double, ReducedVolume As Double, IniT As Double) {#private-function-tr_despejadareducedpressure-as-double-reducedvolume-as-double-init-as-double .MyHeader}

Función que devuelve la temperatura reducida dadas la presión reducida y
volumen reducido.

## Private Function F_de_PyT(ReducedTemperature As Double, CompresibilityFactor As Double, Pressure As Double) As Double {#private-function-f_de_pytreducedtemperature-as-double-compresibilityfactor-as-double-pressure-as-double-as-double .MyHeader}

Función que evalúa la expresión del desarrollo cúbico en factor de
compresibilidad de las EDEC.

## Private Function F_de_PryTrVr(Tr As Double, ReducedVolume As Double, Pr As Double) As Double {#private-function-f_de_prytrvrtr-as-double-reducedvolume-as-double-pr-as-double-as-double .MyHeader}

Función que devuelve el valor de la expresión del desarrollo cúbico en
volumen reducido de las EDEC.

## Private Function P_despejada(ReducedTemperature As Double, CompresibilityFactor As Double, initPressure As Double) As Double {#private-function-p_despejadareducedtemperature-as-double-compresibilityfactor-as-double-initpressure-as-double-as-double .MyHeader}

Función que devuelve la presión usando como datos la temperatura y el
factor de compresibilidad.

## Private Function Fugacity(Z As Double) As Double {#private-function-fugacityz-as-double-as-double .MyHeader}

Función que calcula la fugacidad usando como dato el factor de
compresibilidad.

## Private Sub Coef_Cubicas(Pr As Double, Tr As Double, alfac As Double) {#private-sub-coef_cubicaspr-as-double-tr-as-double-alfac-as-double .MyHeader}

Procedmiento que calcula el valor de los coeficientes del desarrollo
cúbico de las EDEC en función del factor de compresibilidad.

## Private Sub Coef_Cubicasvr(P0 As Double, alphat As Double) {#private-sub-coef_cubicasvrp0-as-double-alphat-as-double .MyHeader}

Procedmiento que calcula el valor de los coeficientes del desarrollo
cúbico de las ecuaciones de estado en función del volumen reducido.

## Friend Function Prsat(Tr As Double) {#friend-function-prsattr-as-double .MyHeader}

Función que calcula la presión de saturación reducida basada en las
correlaciones desarrolladas por el grupo TADiP, utilizando como datos la
temperatura reducida Tr.

## Friend Function I_z(Z As Double, K1 As Integer, K2 As Integer, B As Double) As Double {#friend-function-i_zz-as-double-k1-as-integer-k2-as-integer-b-as-double-as-double .MyHeader}

Función auxiliar utilizada en el cálculo de los coeficientes de
fugacidad, tanto de sustancias puras como de mezclas.

## Friend Function Alpha(ReducedTemperature As Double) As Double {#friend-function-alphareducedtemperature-as-double-as-double .MyHeader}

Función que calcula el parámetro α de las EDEC.

## Friend Function D_Alpha(Tr As Double) As Double {#friend-function-d_alphatr-as-double-as-double .MyHeader}

Función que calcula la derivada respecto de la temperatura del parámetro
α de las EDEC.

## clsLVE {#clslve .MyHeader}

*Clase* diseñada con el objetivo de resolver problemas comunes del
equilibrio líquido vapor de mezcla multicomponentes, utilizando tanto
los modelos basados en las ecuaciones de estado cúbicas como aquéllos
basados en los coeficientes de actividad.

## Private DBO As New clsDataBase {#private-dbo-as-new-clsdatabase .MyHeader}

Instancia de la *clase* de la base de datos del grupo TADiP.

## Public Enum TadipLiquidModels {#public-enum-tadipliquidmodels .MyHeader}

##  IdealSolution = 25 {#idealsolution-25 .MyHeader}

##  PR1976_Qbicl = 0 {#pr1976_qbicl-0 .MyHeader}

##  RK1949_Qbicl = 1 {#rk1949_qbicl-1 .MyHeader}

##  RKS1972_Qbicl = 2 {#rks1972_qbicl-2 .MyHeader}

##  VdW1870_Qbicl = 3 {#vdw1870_qbicl-3 .MyHeader}

##  PRL1997_Qbicl = 4 {#prl1997_qbicl-4 .MyHeader}

##  RKSL1997_Qbicl = 5 {#rksl1997_qbicl-5 .MyHeader}

##  RKSGD1978_Qbicl = 6 {#rksgd1978_qbicl-6 .MyHeader}

##  RP1978_Qbicl = 7 {#rp1978_qbicl-7 .MyHeader}

##  Berth1899_Qbicl = 8 {#berth1899_qbicl-8 .MyHeader}

##  VdWAda1984_Qbicl = 9 {#vdwada1984_qbicl-9 .MyHeader}

##  VdWVald1989_Qbicl = 10 {#vdwvald1989_qbicl-10 .MyHeader}

##  RKSmn1980_Qbicl = 11 {#rksmn1980_qbicl-11 .MyHeader}

##  RKSATmn1995_Qbicl = 12 {#rksatmn1995_qbicl-12 .MyHeader}

##  PRATmng1997_Qbicl = 13 {#pratmng1997_qbicl-13 .MyHeader}

##  PRMmn1989_Qbicl = 14 {#prmmn1989_qbicl-14 .MyHeader}

##  PRSV1986_Qbicl = 15 {#prsv1986_qbicl-15 .MyHeader}

##  VdWOL1998_Qbicl = 16 {#vdwol1998_qbicl-16 .MyHeader}

##  RKOL1998_Qbicl = 17 {#rkol1998_qbicl-17 .MyHeader}

##  PROL1998_Qbicl = 18 {#prol1998_qbicl-18 .MyHeader}

##  vanLaar = 21 {#vanlaar-21 .MyHeader}

##  Wilson = 22 {#wilson-22 .MyHeader}

##  ScatchardHild = 23 {#scatchardhild-23 .MyHeader}

##  Margules = 24 {#margules-24 .MyHeader}

## End Enum {#end-enum-6 .MyHeader}

Enumeración de todos los modelos aplicables a la fase líquida de una
mezcla.

## Public Enum TadipVaporModel {#public-enum-tadipvapormodel .MyHeader}

##  IdealGasv = -2 {#idealgasv--2 .MyHeader}

##  PR1976_Qbicv = 0 {#pr1976_qbicv-0 .MyHeader}

##  RK1949_Qbicv = 1 {#rk1949_qbicv-1 .MyHeader}

##  RKS1972_Qbicv = 2 {#rks1972_qbicv-2 .MyHeader}

##  VdW1870_Qbicv = 3 {#vdw1870_qbicv-3 .MyHeader}

##  PRL1997_Qbicv = 4 {#prl1997_qbicv-4 .MyHeader}

##  RKSL1997_Qbicv = 5 {#rksl1997_qbicv-5 .MyHeader}

##  RKSGD1978_Qbicv = 6 {#rksgd1978_qbicv-6 .MyHeader}

##  RP1978_Qbicv = 7 {#rp1978_qbicv-7 .MyHeader}

##  Berth1899_Qbicv = 8 {#berth1899_qbicv-8 .MyHeader}

##  VdWAda1984_Qbicv = 9 {#vdwada1984_qbicv-9 .MyHeader}

##  VdWVald1989_Qbicv = 10 {#vdwvald1989_qbicv-10 .MyHeader}

##  RKSmn1980_Qbicv = 11 {#rksmn1980_qbicv-11 .MyHeader}

##  RKSATmn1995_Qbicv = 12 {#rksatmn1995_qbicv-12 .MyHeader}

##  PRATmng1997_Qbicv = 13 {#pratmng1997_qbicv-13 .MyHeader}

##  PRMmn1989_Qbicv = 14 {#prmmn1989_qbicv-14 .MyHeader}

##  PRSV1986_Qbicv = 15 {#prsv1986_qbicv-15 .MyHeader}

##  VdWOL1998_Qbicv = 16 {#vdwol1998_qbicv-16 .MyHeader}

##  RKOL1998_Qbicv = 17 {#rkol1998_qbicv-17 .MyHeader}

##  PROL1998_Qbicv = 18 {#prol1998_qbicv-18 .MyHeader}

##  virial = -1 {#virial--1 .MyHeader}

## End Enum {#end-enum-7 .MyHeader}

Enumeración de todos los modelos aplicables a la fase vapor de una
mezcla.

## Private Enum TypeCalculation {#private-enum-typecalculation .MyHeader}

##  DewP = 1 {#dewp-1 .MyHeader}

##  DewT = 2 {#dewt-2 .MyHeader}

##  BubP = 3 {#bubp-3 .MyHeader}

##  BubT = 4 {#bubt-4 .MyHeader}

##  Flash = 5 {#flash-5 .MyHeader}

## End Enum {#end-enum-8 .MyHeader}

Enumeración que contiene los posibles algoritmos relacionados con el
equilibrio líquido vapor de mezclas.

## Private Type DatosFlash {#private-type-datosflash .MyHeader}

##  T As Double {#t-as-double .MyHeader}

##  P As Double {#p-as-double .MyHeader}

##  Nl As Double {#nl-as-double .MyHeader}

##  Nv As Double {#nv-as-double .MyHeader}

##  Nt As Double {#nt-as-double .MyHeader}

##  Alpha As Double {#alpha-as-double .MyHeader}

##  Beta As Double {#beta-as-double .MyHeader}

##  Alphac As Double {#alphac-as-double .MyHeader}

##  Betac As Double {#betac-as-double .MyHeader}

## End Type {#end-type-2 .MyHeader}

Estructrura que contiene los datos utilizados en múltiples cálculos
relacionados con el ELV de mezclas.

## Private mvarLiquidFlow As clsFlow {#private-mvarliquidflow-as-clsflow .MyHeader}

Instancia de la *clase* clsFlow, que contiene las propiedades de las
fase líquida de la mezcla.

## Private mvarVaporFlow As clsFlow {#private-mvarvaporflow-as-clsflow .MyHeader}

Instancia de la *clase* clsFlow, que contiene las propiedades de las
fase vapor de la mezcla.

## Private mvarFeedFlow As clsFlow {#private-mvarfeedflow-as-clsflow .MyHeader}

Instancia de la *clase* clsFlow, que contiene las propiedades de la
corriente de alimentación a la cual se le aplicaran los algoritmos de
cálculo.

## Private LiquidSolver As Object {#private-liquidsolver-as-object .MyHeader}

Objeto destinado a proporcionar la vía de solución para la fase líquida
de la mezcla.

## Private VaporSolver As Object {#private-vaporsolver-as-object .MyHeader}

Objeto destinado a proporcionar la vía de solución para la fase vapor de
la mezcla.

## Private SatPressureCalc As New clsSatPressure {#private-satpressurecalc-as-new-clssatpressure .MyHeader}

Instancia de la *clase* clsSatPressure necesaria para el cálculo de las
presiones de saturación de los compuestos puros que conforman la mezcla.

## Private mvarMixture As clsAllProps {#private-mvarmixture-as-clsallprops .MyHeader}

Instancia de la colección clsAllProps necesaria para tener los datos de
todas las sustancias presentes en la mezcla, así como los modelos
termodinámicos a utilizar en la resolución de los problemas de ELV.

## Private mvarBoolgammaphi As Boolean {#private-mvarboolgammaphi-as-boolean .MyHeader}

Variable que indica si la combinación de modelos termodinámicos
corresponde o no a la aproximación γ-φ.

## Private mvarTolerances As clsTolerances {#private-mvartolerances-as-clstolerances .MyHeader}

Instancia de la *clase* clsTolerances que permite conocer en todo
momento la tolerancia con que se deben realizar los cálculos a través de
todos los algoritmos.

## Private mvarReferenceState As clsReferencesSt {#private-mvarreferencestate-as-clsreferencesst .MyHeader}

Instancia de la *clase* clsReferencesSt que permite establecer los
valores de los estados de referencia para el cálculo de las propiedades
energéticas de las fases en equilibrio.

## Public Property Set ReferenceState(ByVal vData As Object) {#public-property-set-referencestatebyval-vdata-as-object .MyHeader}

## Public Property Get ReferenceState() As clsReferencesSt {#public-property-get-referencestate-as-clsreferencesst .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarReferenceState.

## Public Property Set Tolerances(ByVal vData As Object) {#public-property-set-tolerancesbyval-vdata-as-object .MyHeader}

## Public Property Get Tolerances() As clsTolerances {#public-property-get-tolerances-as-clstolerances .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarTolerances.

## Public Property Set Mixture(vData As clsAllProps) {#public-property-set-mixturevdata-as-clsallprops .MyHeader}

## Public Property Get Mixture() As clsAllProps {#public-property-get-mixture-as-clsallprops .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarMixture.

## Public Property Set FeedFlow(vData As clsFlow) {#public-property-set-feedflowvdata-as-clsflow .MyHeader}

## Public Property Get FeedFlow() As clsFlow {#public-property-get-feedflow-as-clsflow .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarFeedFlow.

## Public Property Set LiquidFlow(vData As clsFlow) {#public-property-set-liquidflowvdata-as-clsflow .MyHeader}

## Public Property Get LiquidFlow() As clsFlow {#public-property-get-liquidflow-as-clsflow .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarLiquidFlow.

## Public Property Set VaporFlow(vData As clsFlow) {#public-property-set-vaporflowvdata-as-clsflow .MyHeader}

## Public Property Get VaporFlow() As clsFlow {#public-property-get-vaporflow-as-clsflow .MyHeader}

Propiedades que permiten asignar u obtener los valores de la variable
mvarVaporFlow.

## Private Function NR_JacobianMatrix(ByVal n As Integer, ByRef Jac() As Double, Datos As DatosFlash, ByRef X() As Double, ByRef Y() As Double, ByRef Ni() As Double) {#private-function-nr_jacobianmatrixbyval-n-as-integer-byref-jac-as-double-datos-as-datosflash-byref-x-as-double-byref-y-as-double-byref-ni-as-double .MyHeader}

Función auxiliar, utilizada por los algoritmos de Newton Raphson, en los
cuales, se desconoce la presión del sistema.

## Private Function NR_JacobianMatrixBDT(ByVal n As Integer, ByRef Jac() As Double, Datos As DatosFlash, ByRef X() As Double, ByRef Y() As Double, ByRef Ni() As Double) {#private-function-nr_jacobianmatrixbdtbyval-n-as-integer-byref-jac-as-double-datos-as-datosflash-byref-x-as-double-byref-y-as-double-byref-ni-as-double .MyHeader}

Función auxiliar, utilizada por los algoritmos de Newton Raphson, en los
cuales, se desconoce la temperatura del sistema.

## Private Function CalculatePoynting(ByVal i As Integer, ByVal T As Double, ByVal P As Double) {#private-function-calculatepoyntingbyval-i-as-integer-byval-t-as-double-byval-p-as-double .MyHeader}

Función que calcula el factor correctivo de Poynting.

## Private Function CalculatePhiSati(ByVal i As Integer, ByVal T As Double) As Double {#private-function-calculatephisatibyval-i-as-integer-byval-t-as-double-as-double .MyHeader}

Función que calcula internamente y mediante una llamada a la *clase*
clsQbicsPure, el coeficiente de fugacidad del componente puro en estado
de saturación, a las condiciones dadas de temperatura.

## Private Sub Class_Terminate() {#private-sub-class_terminate-8 .MyHeader}

Procedimiento que destruye los contenidos de las siguientes variables:
mvarLiquidFlow , mvarVaporFlow, mvarFeedFlow, mvarMixture, DBO.

## Public Function BubblePointTemperature() {#public-function-bubblepointtemperature .MyHeader}

Función que permite el cálculo del punto de burbuja de una mezcla
multicomponente, a partir de los datos de presión y composición
(global), insertados en la variable mvarFeedFlow.

## Private Function BubblePointTemperatureHP(ByVal n As Integer) {#private-function-bubblepointtemperaturehpbyval-n-as-integer .MyHeader}

Función auxiliar, utilizada por BubblePointTemperature, en la cual se
recorre numéricamente la envolvente T, P de la mezcla multicomponente.

## Public Function DewPointTemperature() {#public-function-dewpointtemperature .MyHeader}

Función que permite el cálculo del punto de rocío de una mezcla
multicomponente, a partir de los datos de presión y composición
(global), insertados en la variable mvarFeedFlow.

## Public Function DewPointPressure() {#public-function-dewpointpressure .MyHeader}

Función que permite el cálculo del punto de rocío de una mezcla
multicomponente, a partir de los datos de temperatura y composición
(global), insertados en la variable mvarFeedFlow.

## Private Function NR_DownloadX(ByRef Xanswer() As Double, ByRef Datos As DatosFlash, ByRef X() As Double, ByRef Y() As Double, n As Integer) {#private-function-nr_downloadxbyref-xanswer-as-double-byref-datos-as-datosflash-byref-x-as-double-byref-y-as-double-n-as-integer .MyHeader}

Función que transmite los contenidos del arreglo Xanswer a la estructura
Datos.

## Private Function NR_UploadX(ByRef Xanswer() As Double, ByRef Datos As DatosFlash, ByRef X() As Double, ByRef Y() As Double, n As Integer) {#private-function-nr_uploadxbyref-xanswer-as-double-byref-datos-as-datosflash-byref-x-as-double-byref-y-as-double-n-as-integer .MyHeader}

Función que transmite los contenidos de la estructura Datos al arreglo
Xanswer.

## Private Function NR_Xnew(ByRef X() As Double, deltaX() As Double) {#private-function-nr_xnewbyref-x-as-double-deltax-as-double .MyHeader}

Función auxiliar de los algoritmos de Newton Raphson, que calcula los
nuevos valores de las variables iteración tras iteración.

## Private Function NR_EvaluateFi(ByRef f() As Double, ByRef X() As Double, ByRef Y() As Double, T As Double, P As Double, n As Integer) {#private-function-nr_evaluatefibyref-f-as-double-byref-x-as-double-byref-y-as-double-t-as-double-p-as-double-n-as-integer .MyHeader}

Función auxiliar de los algoritmos de Newton Raphson, que calcula los
valores de la función objetivo para todos los componentes de la mezcla.

## Private Function PE_Hgf(ByVal n As Integer, T As Double, P As Double) As Double {#private-function-pe_hgfbyval-n-as-integer-t-as-double-p-as-double-as-double .MyHeader}

Función utilizada en el cálculo de propiedades energéticas, utilizando
la función de Clapeyron.

## Private Function NR_UploadLiquidMolarFraction(ByRef X() As Double, n As Integer) {#private-function-nr_uploadliquidmolarfractionbyref-x-as-double-n-as-integer .MyHeader}

Función auxiliar de los algoritmos de Newton Raphson, que traslada los
contenidos de el arreglo X(), a la variable
mvarLiquidSolver.MolarFraction().

## Private Function NR_UploadVaporMolarFraction(ByRef Y() As Double, n As Integer) {#private-function-nr_uploadvapormolarfractionbyref-y-as-double-n-as-integer .MyHeader}

Función auxiliar de los algoritmos de Newton Raphson, que traslada los
contenidos de el arreglo Y(), a la variable
mvarVaporSolver.MolarFraction().

## Private Function NR_Fi(ByRef X() As Double, ByRef Y() As Double, ByVal T As Double, ByVal P As Double, i As Integer) As Double {#private-function-nr_fibyref-x-as-double-byref-y-as-double-byval-t-as-double-byval-p-as-double-i-as-integer-as-double .MyHeader}

Función auxiliar de los algoritmos de Newton Raphson, que calcula los
valores de la función objetivo para el componente i de la mezcla.

## Private Function NR_DewPointPressure(firstP As Double, firstX() As Double, n As Integer) {#private-function-nr_dewpointpressurefirstp-as-double-firstx-as-double-n-as-integer .MyHeader}

Función que calcula el punto de rocío de un sistema multicomponente, a
través del método de Newton Raphson multivariable, conociendo como datos
la temperatura y composición global de el sistema.

## Private Sub DimSolverObjects(ByVal n As Integer) {#private-sub-dimsolverobjectsbyval-n-as-integer .MyHeader}

Procedimiento que especifica el tipo de *clase* que será manejada a
través de las variables LiquidSolver y VaporSolver, transmitiendo a
éstas, las propiedades de los compuestos que son necesaria para su uso.

## Private Sub ClearSolverObjects() {#private-sub-clearsolverobjects .MyHeader}

Procedimiento que destruye los contenidos de las variables LiquidSolver
y VaporSolver.

## Private Sub NR_BubblePointPressure(firstP As Double, firstY() As Double, n As Integer) {#private-sub-nr_bubblepointpressurefirstp-as-double-firsty-as-double-n-as-integer .MyHeader}

Función que calcula el punto de burbuja de un sistema multicomponente, a
través del método de Newton Raphson multivariable, conociendo como datos
la temperatura y composición global de el sistema.

## Private Sub NR_BubblePointTemperature(firstT As Double, firstY() As Double, n As Integer) {#private-sub-nr_bubblepointtemperaturefirstt-as-double-firsty-as-double-n-as-integer .MyHeader}

Función que calcula el punto de burbuja de un sistema multicomponente, a
través del método de Newton Raphson multivariable, conociendo como datos
la presión y composición global del sistema.

## Private Sub NR_DewPointTemperature(firstT As Double, firstX() As Double, n As Integer) {#private-sub-nr_dewpointtemperaturefirstt-as-double-firstx-as-double-n-as-integer .MyHeader}

Función que calcula el punto de rocío de un sistema multicomponente, a
través del método de Newton Raphson multivariable, conociendo como datos
la presión y composición global del sistema.

## Private Sub FillSatPressureSolver(ByVal n As Integer) {#private-sub-fillsatpressuresolverbyval-n-as-integer .MyHeader}

Procedimiento auxiliar, que transmite a la instancia SatPressureCalc,
los datos de la mezcla que son necesarios para su uso.

## Private Sub CalculateKi(ByVal n As Integer, ByVal T As Double, ByVal P As Double, ByRef Ki() As Double) {#private-sub-calculatekibyval-n-as-integer-byval-t-as-double-byval-p-as-double-byref-ki-as-double .MyHeader}

Procedimiento que calcula Ki en los algoritmos de resolución de ELV.

## Public Function AdiabaticFlash(Heat As Double, IntPressure As Double) As Integer {#public-function-adiabaticflashheat-as-double-intpressure-as-double-as-integer .MyHeader}

Función que calcula el flash adiabático, con la presión interna igual a
IntPressure, y calor igual a Heat.

## Public Sub IsoThermicFlash(ByRef Betac As Double) {#public-sub-isothermicflashbyref-betac-as-double .MyHeader}

Función que calcula el flash isotérmico.

## Private Function FranctionsEquals(ByRef x1() As Double, ByRef x2() As Double, ByRef y1() As Double, ByRef y2() As Double) As Boolean {#private-function-franctionsequalsbyref-x1-as-double-byref-x2-as-double-byref-y1-as-double-byref-y2-as-double-as-boolean .MyHeader}

Función que determina si las composiciones en iteraciones consecutivas
son iguales con cierto error.

## Private Function Flash_CalculateXi(Beta As Double, ByRef Ki() As Double, ByRef X() As Double, ByVal n As Integer) As Boolean {#private-function-flash_calculatexibeta-as-double-byref-ki-as-double-byref-x-as-double-byval-n-as-integer-as-boolean .MyHeader}

Función auxiliar utilizada en los cálculos de flash, para obtener los
valores de las composiciones molares de la fase líquida.

## Private Function Flash_CalculateYi(ByRef X() As Double, ByRef Y() As Double, ByRef Ki() As Double, ByVal n As Integer) As Boolean {#private-function-flash_calculateyibyref-x-as-double-byref-y-as-double-byref-ki-as-double-byval-n-as-integer-as-boolean .MyHeader}

Función auxiliar utilizada en los cálculos de flash, para obtener los
valores de las composiciones molares de la fase vapor.

## Private Function Flash_RachfordRice_NR(initialguess As Double, ByRef answer As Double, ByRef Ki() As Double, ByVal n As Integer) As Boolean {#private-function-flash_rachfordrice_nrinitialguess-as-double-byref-answer-as-double-byref-ki-as-double-byval-n-as-integer-as-boolean .MyHeader}

Función que resuelve mediante un algoritmo de Newton Raphson la ecuación
de Rachford Rice.

## Private Function Flash_RachfordRice(Beta As Double, ByRef Ki() As Double, ByVal n As Integer) {#private-function-flash_rachfordricebeta-as-double-byref-ki-as-double-byval-n-as-integer .MyHeader}

Función que evalúa la ecuación de Rachford Rice.

## Private Function Flash_DerivateRachfordRice(Beta As Double, ByRef Ki() As Double, ByVal n As Integer) {#private-function-flash_derivaterachfordricebeta-as-double-byref-ki-as-double-byval-n-as-integer .MyHeader}

Función que evalúa la derivada de la ecuación de Rachford Rice respecto
a la fracción de vapor.

## Private Sub RaoultFirstAprox(TypeOfCalc As TypeCalculation, ByVal n As Integer) {#private-sub-raoultfirstaproxtypeofcalc-as-typecalculation-byval-n-as-integer .MyHeader}

Función que calcula el ELV mediante la ley de Raoult, como primera
aproximación para los demás algoritmos de cálculo.

## Private Function TempIni(TypeOfCalc As TypeCalculation, n As Integer) As Double {#private-function-tempinitypeofcalc-as-typecalculation-n-as-integer-as-double .MyHeader}

Función que calcula el primer estimado de temperatura para los
algoritmos de cálculo de ELV.

## Private Sub FillVaporFraction(vData As TypeCalculation, Optional Val As Double) {#private-sub-fillvaporfractionvdata-as-typecalculation-optional-val-as-double .MyHeader}

Función que transmite el valor de la fracción de vapor, a cada uno de
las instancias de la *clase* clsFlow existentes, según el algoritmo
descrito por el parámetro vData.

## Public Function BubblePointPressure() As Boolean {#public-function-bubblepointpressure-as-boolean .MyHeader}

Función que permite el cálculo del punto de burbuja de una mezcla
multicomponente, a partir de los datos de temperatura y composición
(global), insertados en la variable mvarFeedFlow.

## Private Sub BubblePointPressureHP(ByVal n As Integer) {#private-sub-bubblepointpressurehpbyval-n-as-integer .MyHeader}

Función auxiliar, utilizada por BubblePointPressure, en la cual se
recorre numéricamente la envolvente T, P de la mezcla multicomponente.

## Private Function DewpointPressureHP(ByVal n As Integer) {#private-function-dewpointpressurehpbyval-n-as-integer .MyHeader}

Función auxiliar, utilizada por DewpointPressure, en la cual se recorre
numéricamente la envolvente T, P de la mezcla multicomponente.

## Private Function DewPointTemperatureHP(ByVal n As Integer) {#private-function-dewpointtemperaturehpbyval-n-as-integer .MyHeader}

Función auxiliar, utilizada por DewPointTemperature, en la cual se
recorre numéricamente la envolvente T, P de la mezcla multicomponente.

## Private Function Ekilib_BubblePointPressure(ByVal n As Integer, firstP As Double, firstY() As Double) {#private-function-ekilib_bubblepointpressurebyval-n-as-integer-firstp-as-double-firsty-as-double .MyHeader}

Función que calcula el punto de burbuja de un sistema multicomponente, a
través del método utilizado por el programa Ekilib, conociendo como
datos la temperatura y composición global de el sistema.

## Private Sub Ekilib_LazoK_Y(ByRef sumaY As Double, n As Integer, T As Double, P As Double) {#private-sub-ekilib_lazok_ybyref-sumay-as-double-n-as-integer-t-as-double-p-as-double .MyHeader}

Procedimiento que representa el lazo interno utilizado para calcular las
composiciones molares de vapor en los algoritmos en los cuales éstas
sean desconocidas.

## Private Function Enthalpy_Entropy(ByVal n As Integer, T As Double, P As Double, Beta As Double, ByRef h As Double, ByRef s As Double) {#private-function-enthalpy_entropybyval-n-as-integer-t-as-double-p-as-double-beta-as-double-byref-h-as-double-byref-s-as-double .MyHeader}

Función que calcula la entalpía y la entropía de una mezcla en
equilibrio líquido vapor, a las condiciones de temperatura, presión y
fracción vaporizada determinada a través de los parámetros T,P y Beta
respectivamente.

## Private Function Ekilib_DewPointPressure(ByVal n As Integer, firstP As Double, firstX() As Double) {#private-function-ekilib_dewpointpressurebyval-n-as-integer-firstp-as-double-firstx-as-double .MyHeader}

Función que calcula el punto de rocío de un sistema multicomponente, a
través del método utilizado por el programa Ekilib, conociendo como
datos la temperatura y composición global del sistema.

## Private Function IdealEnthalpy(ByVal n As Integer, ByVal T As Double, ByVal Tref As Double, ByVal Ho As Double, ByRef lH As Double, ByRef vH As Double) {#private-function-idealenthalpybyval-n-as-integer-byval-t-as-double-byval-tref-as-double-byval-ho-as-double-byref-lh-as-double-byref-vh-as-double .MyHeader}

Función que calcula el cambio de entalpía de la mezcla en estado de gas
ideal desde la temperatura de referencia (Tref) hasta la temperatura del
sistema (T).

## Private Function IdealEntropy(ByVal n As Integer, ByVal T As Double, ByVal Tref As Double, ByVal P As Double, Pref As Double, So As Double, ByRef lS As Double, ByRef vS As Double) {#private-function-idealentropybyval-n-as-integer-byval-t-as-double-byval-tref-as-double-byval-p-as-double-pref-as-double-so-as-double-byref-ls-as-double-byref-vs-as-double .MyHeader}

Función que calcula el cambio de entropía de la mezcla en estado de gas
ideal desde la temperatura y presión de referencia (Tref y Pref
respectivamente) hasta la temperatura y presión del sistema (T y P
respectivamente).

## Private Function Ekilib_BubblePointTemperature(ByVal n As Integer, firstT As Double, firstY() As Double) {#private-function-ekilib_bubblepointtemperaturebyval-n-as-integer-firstt-as-double-firsty-as-double .MyHeader}

Función que calcula el punto de burbuja de un sistema multicomponente, a
través del método utilizado por el programa Ekilib, conociendo como
datos la presión y composición global del sistema.

## Private Function Ekilib_DewPointTemperature(ByVal n As Integer, firstT As Double, firstX() As Double) {#private-function-ekilib_dewpointtemperaturebyval-n-as-integer-firstt-as-double-firstx-as-double .MyHeader}

Función que calcula el punto de rocío de un sistema multicomponente, a
través del método utilizado por el programa Ekilib, conociendo como
datos la presión y composición global del sistema.

## McommonFunctions {#mcommonfunctions .MyHeader}

Modulo que contiene diversidad de funciones y métodos numéricos
utilizados por varias de las *clases* que conforman la librería.

## Public Const UnivGasConst = 8.31451 {#public-const-univgasconst-8.31451 .MyHeader}

Constante Universal de los gases

## Public Declare Sub CopyMemory Lib \"kernel32\" Alias \"RtlMoveMemory\" (Dest As Any, Source As Any, ByVal Bytes As Long) {#public-declare-sub-copymemory-lib-kernel32-alias-rtlmovememory-dest-as-any-source-as-any-byval-bytes-as-long .MyHeader}

Función API que permite copiar un arreglo a otro sin necesidad de
utilizar un lazo For\...Next.

## Public Filter As New clsJMLFilter {#public-filter-as-new-clsjmlfilter .MyHeader}

Instancia de la *clase* clsJMLFilter, utilizada para filtrar las
entradas numéricas del programa.

## Function SWAPN(ByRef A1 As Variant, ByRef A2 As Variant) {#function-swapnbyref-a1-as-variant-byref-a2-as-variant .MyHeader}

Función que intercambia el valor de A1 con el de A2.

## Public Function DeterDiag(n As Integer, ByRef A() As Double, ByRef P() As Integer, ByVal d As Integer) As Double {#public-function-deterdiagn-as-integer-byref-a-as-double-byref-p-as-integer-byval-d-as-integer-as-double .MyHeader}

Función que calcula el determinante de una matriz.

## Public Function SumFrac(ByRef X() As Double, ByRef SUM As Double) {#public-function-sumfracbyref-x-as-double-byref-sum-as-double .MyHeader}

Función que calcula la sumatoria de las fracciones molares X().

## Public Function Compara(X() As Double, Y() As Double, E As Double) As Boolean {#public-function-comparax-as-double-y-as-double-e-as-double-as-boolean .MyHeader}

Función que compara los valores de X() y Y() para descartar la
posibilidad de una solución trivial.

## Public Sub IncrementarYRodar(ByRef x1 As Double, ByRef x2 As Double, ByRef x3 As Double, ByVal Incremento As Double, ByRef f1 As Double \_ {#public-sub-incrementaryrodarbyref-x1-as-double-byref-x2-as-double-byref-x3-as-double-byval-incremento-as-double-byref-f1-as-double-_ .MyHeader}

## , ByRef f2 As Double, ByRef f3 As Double) {#byref-f2-as-double-byref-f3-as-double .MyHeader}

Procedimiento que asigna el valor de X2 a X1, el de X3 a X2 y finalmente
incrementa el valor de X3.

## Public Sub Parabola(x1 As Double, x2 As Double, x3 As Double, fx1 As Double, fx2 As Double, fx3 As Double, ByRef Incremento As Double, Lineal As Boolean) {#public-sub-parabolax1-as-double-x2-as-double-x3-as-double-fx1-as-double-fx2-as-double-fx3-as-double-byref-incremento-as-double-lineal-as-boolean .MyHeader}

Procedimiento que predice un nuevo valor dentro de algoritmos
iterativos, mediante una aproximación parabólica.

## Public Function CopySign(A As Double, B As Double) As Double {#public-function-copysigna-as-double-b-as-double-as-double .MyHeader}

Función que cambia el signo de A por el de B.

## Public Function NuevoIncremento(x1 As Double, x2 As Double, x3 As Double, f1 As Double, f2 As Double, f3 As Double, ByVal Iteracion As Integer) As Double {#public-function-nuevoincrementox1-as-double-x2-as-double-x3-as-double-f1-as-double-f2-as-double-f3-as-double-byval-iteracion-as-integer-as-double .MyHeader}

Función que calcula nuevos incrementos en diversos algoritmos
iterativos.

## Public Function Max(ByVal x1 As Double, ByVal x2 As Double) As Double {#public-function-maxbyval-x1-as-double-byval-x2-as-double-as-double .MyHeader}

Función que devuelve el valor más grande entre x1 y x2.

## Public Sub GeneralConstantsEOS(intEOS As TADiPEDC, ByRef K1 As Integer, ByRef K2 As Integer, ByRef K3 As Integer, ByRef OmA As Double, ByRef OmB As Double, ByRef OmC As Double, ByRef h() As Double) {#public-sub-generalconstantseosinteos-as-tadipedc-byref-k1-as-integer-byref-k2-as-integer-byref-k3-as-integer-byref-oma-as-double-byref-omb-as-double-byref-omc-as-double-byref-h-as-double .MyHeader}

Procedimiento que permite según la ecuación de estado cúbica (intEOS),
conocer los valores de los parámetros necesarios para los cálculos a
través de la ecuación de Abbott.

## Public Function CubicSolver(ByVal A0 As Double, ByVal A1 As Double, ByVal A2 As Double, ByVal A3 As Double, ByRef y1 As Double, ByRef y2 As Double, ByRef y3 As Double) As Integer {#public-function-cubicsolverbyval-a0-as-double-byval-a1-as-double-byval-a2-as-double-byval-a3-as-double-byref-y1-as-double-byref-y2-as-double-byref-y3-as-double-as-integer .MyHeader}

Función que resuelve una ecuación cúbica de la forma:
a0+a1\*x+a2\*x2+a3\*x3.

## Private Sub SWAP(i As Double, j As Double) {#private-sub-swapi-as-double-j-as-double .MyHeader}

Función que intercambia el valor de i con el de j.

## Public Sub CopyX(ByRef X() As Double, ByRef XC() As Double) {#public-sub-copyxbyref-x-as-double-byref-xc-as-double .MyHeader}

Función utilizada para copiar los contenidos de X() a XC() a través de
la función CopyMemory.

## Private Function Arccos(alfa As Double) As Double {#private-function-arccosalfa-as-double-as-double .MyHeader}

Función que calcula el arcoseno de alfa.

## Private Function XALAY(X As Double, Y As Double) As Double {#private-function-xalayx-as-double-y-as-double-as-double .MyHeader}

Función que multiplica X , Y veces.

## Public Function LUbksb(ByRef A() As Double, ByRef n As Integer, NP As Integer, ByRef INDX() As Double, ByRef B() As Double) {#public-function-lubksbbyref-a-as-double-byref-n-as-integer-np-as-integer-byref-indx-as-double-byref-b-as-double .MyHeader}

## Public Function LUDecomp(ByVal n As Integer, NP As Integer, ByRef A() As Double, ByRef INDX() As Double, ByRef d As Double) As Boolean {#public-function-ludecompbyval-n-as-integer-np-as-integer-byref-a-as-double-byref-indx-as-double-byref-d-as-double-as-boolean .MyHeader}

Funciones que permiten la descomposición LU de una matriz.

## Public Function SolveAx(n As Integer, A() As Double, B() As Double, ByRef X() As Double) As Boolean {#public-function-solveaxn-as-integer-a-as-double-b-as-double-byref-x-as-double-as-boolean .MyHeader}

Función que resuelve el sistema de ecuaciones representada como:
A()X()=B().

## Public Function CheckFunConv(ByRef f() As Double, ByVal E As Double) As Boolean {#public-function-checkfunconvbyref-f-as-double-byval-e-as-double-as-boolean .MyHeader}

Función que verifica la convergencia de los algoritmos usados en el
método numérico de Newton-Raphson.

## Public Function CheckRootConv(ByRef X() As Double, ByVal E As Double) As Boolean {#public-function-checkrootconvbyref-x-as-double-byval-e-as-double-as-boolean .MyHeader}

Función que verifica la convergencia de los algoritmos usados en el
método numérico de Newton-Raphson.

## Public Function FirstDerivate(fo As Double, f1 As Double, h As Double) As Double {#public-function-firstderivatefo-as-double-f1-as-double-h-as-double-as-double .MyHeader}

Función que calcula la primera derivada numérica de una función dados
f(x) (parámetro fo), f(x+h) (parámetro f1) y el paso h.

## Public Function SecondDerivate(fo As Double, f1 As Double, f_1 As Double, h As Double) As Double {#public-function-secondderivatefo-as-double-f1-as-double-f_1-as-double-h-as-double-as-double .MyHeader}

Función que calcula la primera derivada numérica de una función dados
f(x) (parámetro fo), f(x+h) (parámetro f1), f(x-h) (parámetro f_1) y el
paso h.

## Public Function CubicSolverNR(ByVal A As Double, ByVal B As Double, ByVal C As Double, ByRef x1 As Double, ByRef x2 As Double, ByRef x3 As Double) As Integer {#public-function-cubicsolvernrbyval-a-as-double-byval-b-as-double-byval-c-as-double-byref-x1-as-double-byref-x2-as-double-byref-x3-as-double-as-integer .MyHeader}

Función que resuelve una ecuación cúbica de la forma:
a0+a1\*x+a2\*x2+a3\*x3.
