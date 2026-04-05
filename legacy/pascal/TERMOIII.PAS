Unit TermoIII(6);
{$U-}
{ Paquete termodinámico hecho por: }
{ Francisco Avelino Da Silva. }
{ Luis Alberto Báez Linde.    }
{ Caracas, 89 }

Interface

{$U TermoI}
{$U TermoII}

Uses Sane,TermoI,TermoII;

Var
 TolTemp    :Extended; 
 TolPres    :Extended; 
 TolFracc   :Extended; 
 DZTrivial  :Extended; { Tolerancias, utilizadas por TermoIV y TermoV }
 NMaxItera  :Integer;  { Número máximo de iteraciones, " }

FUNCTION Gamma_i(i,Modelo:Integer;CoefAIJ:MatrizConst):Extended;
 
FUNCTION Gibbs_Exceso(Modelo:integer;CoefAIJ:MatrizConst):Extended;

FUNCTION Entalpia_Exceso(Modelo:integer;CoefAIJ:MatrizConst):Extended;

FUNCTION Entropia_Exceso(Modelo:integer;CoefAIJ:MatrizConst):Extended;  
                                                   
FUNCTION EntalpiadeCondensacion (ModelVaporSat:integer;ModeloPresionVapor:TipoPresVapor)
                                 :extended;

FUNCTION EntalpiaResLiq(ModelVaporSat,ModeloLiq:integer;CoefAIJ:MatrizConst;
                        ModeloPresionVapor:TipoPresVapor;CorrecionVdP:Boolean):extended;  

FUNCTION EntropiaResLiq(ModelVaporSat,ModeloLiq:integer;CoefAIJ:MatrizConst;
                        ModeloPresionVapor:TipoPresVapor):extended; 
                        
FUNCTION CalcParBinWilson(GamInf1,GamInf2:extended;var A12,A21:extended):Error;
                       
PROCEDURE VariarLosAij(Var Aij:MatrizConst;T1,T2:Extended;ModeloLiq:Integer);
                                                                       
{ NOTA: Todas las temperaturas en [K], presiones en [bar], Energías en [kJ/kmol], Entropías [kJ/kmol.K] }

Implementation

var
 Temp,Pres :Extended; { en K y kPa respectivamente}
 N         :Integer;
 Cm        :Propiedades;
 X,Y       :FraccMolar;
 PreVap_i  :FraccMolar; { en bar, solo se usa para entropías residuales }
 
PROCEDURE CopiarVariables;
begin
 N:=ObtenerNyComp(Cm);
 ObtenerConcentraciones(X,Y);
 ObtenerPyT(Pres,Temp);
 Pres:=Pres*100; { a [kPa] }
end;

FUNCTION VolLiqM:extended; {calcula el volumen de mezcla en liquido para ScatcharH}
var
 sumaaux:extended; {variableauxiliar local}
 j:integer;
begin
 sumaaux:=0;
 for j:=1 to N do
  sumaaux:=sumaaux+x[j]*Cm[j].vl;
 VolLiqM:=sumaaux;
end; { VolLiqM }
  
FUNCTION SumaTetaDel(VolM:extended):extended; {es llamada por  ScatchardH}
var
 sumaaux:extended; {variable aux local}
 j:integer;
begin
 sumaaux:=0;
 for j:=1 to N do
  With Cm[j] do
   sumaaux:=sumaaux+x[j]*vl*delta/VolM;
 SumaTetaDel:=sumaaux;
end; { SumaTetaDel }

FUNCTION Gamma_i;
var 
 Aux:extended; {variableAuxiliar}
 Sumaij,Sumaji:extended; {variables auxiliares para las sumas en vanLaar}
 VliqM,SumaTD:extended; {volumen de mezcla liq y sumatoria de la fraccion de volumen}
                        {por el parametro de solubilidad usados como variables auxiliares en ScatcharH}
  
 PROCEDURe Sumas(var deij,deji:extended); {calcula las sumatorias para el cálculo según vanLaar}
  var
   j:integer;
  begin
   deij:=0; deji:=0;
   for j:=1 to N do
    begin
     deij:=deij+x[j]*CoefAIJ[i,j];
     deji:=deji+x[j]*CoefAIJ[j,i];
    end;
  end;  { Sumas }
   
 FUNCTIOn SumasWil(k:integer):extended; {es llamada por Wilson para el calculo}
  Var                                   {de las simples sumatorias requeridas}
   sumaaux:extended;
   j:integer;
  begin
   sumaaux:=0;
    for j:=1 to N do
     sumaaux:=sumaaux+x[j]*CoefAIJ[k,j];
   SumasWil:=sumaaux;
  end;  { SumasWil }
   
 FUNCTIOn SumaAnidada(k:integer):extended; {es llamada por Wilson para realizar}
  Var                                     {la sumatoria con una sumatoria interna}
   j:integer;                             {como denominador}
   sumaaux:extended;
  begin
   sumaaux:=0;
   for j:=1 to N do  { j es el componente j de la mezcla }
    sumaaux:=sumaaux + x[j]*CoefAIJ[j,k]/SumasWil(j);
   SumaAnidada:=sumaaux;
  end;    { SumaAnidada }
    
begin
 CopiarVariables; { se requiere para inicializar las composiciones locales}
 Case Modelo of   { presion, temperatura, N y componentes }
  SolucionIdeal:
              Gamma_i:=1;
  ScatchardH :begin
               VliqM:=VolLiqM;
               with Cm[i] do
                begin
                 aux:=vl*XpwrI(delta-SumaTetaDel(VliqM),2)/(R*0.23898); { R a cal/mol.K }
                 aux:=aux/Temp+ln(vl/VliqM)+1-vl/VliqM;
                end;
                Gamma_i:=exp(aux);
              end;
  Margules:   begin                   {solo para mezclas binarias}
                if i=1 then
                  aux:=sqr(x[2])*(CoefAIJ[1,2]+2*x[1]*(CoefAIJ[2,1]-CoefAIJ[1,2]))
                else
                  aux:=sqr(x[1])*(CoefAIJ[2,1]+2*x[2]*(CoefAIJ[1,2]-CoefAIJ[2,1]));
               Gamma_i:=exp(aux);
              end;
  VanLaar:    begin
              {con CoefAIJ(i,i) = 0}
               Sumas(Sumaij,Sumaji);
               Gamma_i:=Exp(Sumaij*(1-x[i])*XpwrI(Sumaji/(x[i]*Sumaij+(1-x[i])*Sumaji),2))
              end;
  Wilson:     begin
               {con CoefAIJ(i,i) = 1}
               Gamma_i:=Exp(1-Ln(SumasWil(i))-SumaAnidada(i));
              end;
  otherwise Gamma_i:=Nan(255);  {Modelo no especificado}  
 end; { Case Modelo…}
end; { Gamma_i }

FUNCTION Gibbs_Exceso;
var
 i:integer;
 suma:extended;
begin
 CopiarVariables;
 suma:=0;
 for i:=1 to N do
  suma:=suma+X[i]*Ln(Gamma_i(i,Modelo,CoefAIJ));
 Gibbs_Exceso:=suma*R*Temp;
end; { Gibbs_Exceso }
  
FUNCTION Entalpia_Exceso;
var
 Aux,Aux2:extended;
 j,k:integer;
 SumUp,SumDown:extended; 
begin
 CopiarVariables;
 Aux:=0;
 case Modelo of 
   SolucionIdeal:begin
                   Entalpia_Exceso:=0;
                   Exit;
                 end;
   ScatchardH   :begin
                   Aux2:=SumaTetaDel(VolLiqM);
                   for j:=1 to N do
                     with Cm[j] do
                       Aux:=Aux+X[j]*vl*XpwrI(delta-Aux2,2);
                   Aux:=Aux*4.18445; {Lo pasa a de cal/gmol a kJ/kgmol}
                 end;
   Margules     :begin           {usar solo para mezclas binarias!!!}
                   for j:=1 to N do
                    Aux:=Aux+X[j]*ln(Gamma_i(j,Modelo,CoefAIJ));
                   Aux:=R*Temp*Aux;
                 end;
   VanLaar      :begin
                   for j:=1 to N do
                    Aux:=Aux+X[j]*ln(Gamma_i(j,Modelo,CoefAIJ));
                   Aux:=R*Temp*Aux;
                 end;   
   Wilson       :begin
                  for j:= 1 to N do
                   begin
                   SumUp:=0;SumDown:=0;
                    for k:= 1 to N do
                     begin
                      if k <> j then
                       begin
                        SumUp:=SumUp+X[k]*(ln(CoefAij[j,k]*Cm[j].vl/Cm[k].vl))*CoefAij[j,k];
                        SumDown:=SumDown+X[k]*CoefAij[j,k];
                       end;
                     end;
                    SumDown:=SumDown+X[j];
                    Aux:=Aux-X[j]*R*Temp*SumUp/SumDown;
                   end;
                 end;
   otherwise     Aux:=Nan(255);  {Modelo no especificado}  
  end; { Case Modelo…}
  Entalpia_Exceso:=Aux;
end; { Entalpia_Exceso }

FUNCTION Entropia_Exceso{(Modelo:integer;CoefAIJ:MatrizConst):Extended; };
                                                               { kJ/[kmol.K] }                                         
var
 i:integer;
 suma:extended;
begin
 CopiarVariables;
 case Modelo of 
     SolucionIdeal,
     ScatchardH,
     Margules,
     VanLaar :Entropia_Exceso:=0;
     Wilson  :Entropia_Exceso:=(Entalpia_Exceso(Modelo,CoefAIJ)
                            -Gibbs_Exceso(Modelo,CoefAIJ))/Temp;
    otherwise Entropia_Exceso:=Nan(255);  {Modelo no especificado}  
  end; { Case Modelo…}
end;

Const
 Clapeyron=1;
 Ental=2;
 Entro=3;
 
FUNCTION CalcularAlgo(Accion:Integer;ModelVaporSat:integer;
         ModeloPresionVapor:TipoPresVapor):Extended;
 var Suma:Extended; 
    Unitario:FraccMolar;
    PropAux:Propiedades;
    i:Integer;        
    Z1,Z2:Extended;                  

 begin
 If ModelVaporSat<>ChaoSeader then
  begin
   Suma:=0;
   Unitario[1]:=1;
   Concentraciones(Unitario,Unitario);
   For i:=1 to N do
    begin
     PropAux[1]:=Cm[i];
     Componentes(PropAux,1);
     PresionyTemp(PreVap_i[i],Temp);
     CalcularA_B_CdeMezcla(ModelVaporSat,Liquido);
     Case Accion of
      Clapeyron:
        Suma:=x[i]*(FactorCompre(ModelVaporSat,Liquido)-FactorCompre(ModelVaporSat,Gas))
           *DPrVapor_DT(Cm[i],Temp,ModeloPresionVapor)/PreVap_i[i]+Suma;
      Ental:Suma:=Suma+x[i]*EntalpiaRes(ModelVaporSat,Gas,FactorCompre(ModelVaporSat,Gas));
      Entro:Suma:=Suma+x[i]*EntropiaRes(ModelVaporSat,Gas,FactorCompre(ModelVaporSat,Gas));
      end;
    end;
   CalcularAlgo:=Suma;
  end
 else
  CalcularAlgo:=NaN(255);
 Componentes(Cm,N);
 PresionyTemp(Pres/100,Temp);
 Concentraciones(x,y);
end;

FUNCTION EntalpiadeCondensacion {(ModelVaporSat:integer;ModeloPresionVapor:TipoPresVapor};
var Suma:Extended;
    i:Integer;                          
begin
 CopiarVariables;
 For i:=1 to N do
  PreVap_i[i]:=PrVapor(Cm[i],Temp,ModeloPresionVapor);
 EntalpiaDeCondensacion:=CalcularAlgo(Clapeyron,ModelVaporSat,ModeloPresionVapor)
                         *R*Sqr(Temp);
end;

FUNCTION EntalpiaResLiq;
Var Suma,aux:Extended;i:Integer;
begin
 If ModelVaporSat=ChaoSeader then 
  begin
   CopiarVariables;
   Suma:=0;Pres:=Pres/100;
   for i:=1 to N do
    Suma:=Suma-x[i]*DLnFugPuro_DT_i(i,ChaoSeader,Pres,aux);
   Suma:=Suma*R*Sqr(Temp);Pres:=Pres*100;
  end
 else
  begin
   Suma:=EntalpiadeCondensacion(ModelVaporSat,ModeloPresionVapor)
       +CalcularAlgo(Ental,ModelVaporSat,ModeloPresionVapor)
       +Entalpia_Exceso(ModeloLiq,CoefAIJ);
   If CorrecionVdP then
   for i:=1 to N do
    Suma:=Suma+X[i]*Cm[i].vl*(Pres-PreVap_i[i]*100)*0.001;
  end;
 EntalpiaResLiq:=Suma
end;

FUNCTION EntropiaResLiq;
 Var Suma,aux   :Extended;
     Prf        :Extended; { Presión de referencia para las entropías }
     i          :Integer;
begin
 If ModelVaporSat=ChaoSeader then 
  begin
   CopiarVariables;
   Suma:=0;
   for i:=1 to N do
    Suma:=Suma+x[i]*DLnFugPuro_DT_i(i,ChaoSeader,Pres,aux);{ - EntalpiaResidual}
   Suma:=Suma*R*Temp*Nan(001);{ MientrasTanto } { y luego + EntropiaRes gas }
  end
 else
  begin
   ObtenerTyP_Ho_SoReferencia(Aux,Prf,Aux,Aux);
   Suma:=EntalpiadeCondensacion(ModelVaporSat,ModeloPresionVapor)/Temp
       +CalcularAlgo(Entro,ModelVaporSat,ModeloPresionVapor)
       +Entropia_Exceso(ModeloLiq,CoefAIJ)+R*ln(Pres/100/Prf);
   for i:=1 to N do
    Suma:=Suma-X[i]*R*ln(PreVap_i[i]/Prf);
  end;
 EntropiaResLiq:=Suma
end;

FUNCTION CalcParBinWilson;
 var
  listo:boolean;
  k:integer; 
  aux:extended; {variable auxiliar}
  E:Environment;
  
  FUNCTIOn f(A21:extended):extended;
   begin
    f:=Exp(1-A21-GamInf1) + GamInf2 + Ln(A21)-1;
   end;
   
  FUNCTIOn fder(A21:extended):extended;
   begin
    fder:=-Exp(1-A21-GamInf1)+1/A21;
   end;
   
begin
 GetEnvironment(E);
 SetEnvironment(0);
 A21:=0.5; aux:=A21;{se inicializa A21 para su resolucion por Newton}
 GamInf1:=Ln(GamInf1); GamInf2:=Ln(GamInf2);
 k:=0; listo:=false;{contador}
  repeat
   k:=k+1;
   A21:=A21-f(A21)/fder(A21);
   if Abs(A21-Aux) <= TolFracc then
    listo:=true
   else
    Aux:=A21;
  until (listo) or (k>NMaxItera) ;
  A12:=1-Ln(A21)-GamInf2;
  If K>NMaxItera then
   CalcParBinWilson:=NoConverge
  else
   If (ClassExtended(A12)<>NormalNum) or (A12<0) or (A21<0) then
    CalcParBinWilson:=ErrorMatematico
   else
    CalcParBinWilson:=SinError;
  SetEnvironment(E);
end; { CalcParBinWilson }

PROCEDURE VariarLosAij;
Var  Variacion:Extended;
     i        :Integer;
     j        :Integer;
     a1       :Extended;
begin
  CopiarVariables;
  If (ModeloLiq=VanLaar) or (ModeloLiq=Margules) then
   Variacion:=T1/T2; 
  For i:=1 to N do
   begin
    For j:=1 to N do
     if ModeloLiq=Wilson then
      begin
       Variacion:=Cm[j].vl/Cm[i].vl;
       a1:=T1*ln(Aij[i,j]/Variacion);
       Aij[i,j]:=Variacion*Exp(a1/T2);
      end
     else
      Aij[i,j]:=Aij[i,j]*Variacion;
    If ModeloLiq=Wilson then
     Aij[i,i]:=1
    else
     Aij[i,i]:=0;
   end;
end;
 
Begin
 TolFracc:=0.001;
 TolTemp:=0.01;
 TolPres:=0.01;
 DZTrivial:=0.03;
 NMaxItera:=20;
End.