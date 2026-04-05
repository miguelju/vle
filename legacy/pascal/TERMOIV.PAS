Unit TermoIV(7);
{$U-}
{ Paquete termodinámico hecho por: }
{ Luis Alberto Báez Linde.    }
{ Francisco Avelino Da Silva. }
{ Caracas, 89 }

{$U TermoI}
{$U TermoII}
{$U TermoIII}

Interface

Uses Sane,TermoI,TermoII,memTypes,Quickdraw,TermoIII,
     OsIntF,ToolIntF; { Estos últimos solo para hacer que }
                      { funciones la tecla 'Control,' para parar los cálculos }
 Const
  
    RocioTemp     = 1; { Identificadores de calculo, para QueHacer }
    RocioPres     = 2;
    BurbujaTemp   = 3;
    BurbujaPres   = 4;
    FIsotermico   = 5;
    FAdiabatico   = 6; { Nota: puede hacer balances de energía no adiabáticos }
                       { y también de una sola fase (P Ej. expansión en válvulas)}
                       { También de monocomponentes }

 Type
 
  Inicio     = (PorUsuario,PorAntoine,PorRiedel,PorRPM,PorMuller);
  
  DatosFlash = record
    X,Y        :FraccMolar; { Comp. líquido y vapor }
    Z          :FraccMolar; { Comp. alimentación    }
    Presion    :Extended;   { Presion [bar] }
    Temperatura:Extended;   { Tempertura [K] }
    TempAux    :Extended;   { Solo para T Alimentación FlashAdiabático [K], ver Nota }
    PresAux    :Extended;   { Solo para P Alimentación FlashAdiabático [bar] }
    Calor      :Extended;   { Para FlashAdiabático Calor = 0. }
    HayInicial :boolean;    { Para considerar valores de Temperatura o Presion como iniciales }
    CoefVap    :Extended;   { Coeficiente de vaporización }
    IniciarCV  :Boolean;    { Inicialización para el CoefVap? }
    Amortiguado:Boolean;    { Cálculo con factor de Amortiguamiento? }
    FacAmort   :Extended;   { Factor de Amortiguamiento }
    ModeloGas  :Integer;    { Modelo para la fase gaseosa }
    ModeloLiq  :Integer;    { Modelo para la fase líquida }
    Poynting   :Boolean;    { Calcular correción de Poynting? }
    CoefAij    :MatrizConst;{ Los Aij de la ecuacion para líquido }
    TemperAij  :Extended;   { Temperatura a la que están dados estos coeficientes }
    CompFlash  :Propiedades;{ Las propiedades de cada componente }
    K          :FraccMolar; { Vector de los K, pueden también ser los iniciales }
    Usarlos    :Inicio;     { Si usa los K como iniciales entonces Usarlos=Usuario }
    TipPVapor  :TipoPresVapor;{ La ecuación para la presion de vapor }
    Numero     :Integer;    { Número de componentes }
    FugLPuroporC_S:Boolean; { Calcula la fugacidad de líquido puro por Chao_Seader? }
    Iteraciones:Integer;    { Iteraciones efectuadas }
    ErrorIni   :Extended;   { Error inicial. <0 si todavia no esta disponible }
    ErrorAct   :Extended;   { Error en cada iteracion. Solo tiene sentido se ErrorIni>0}
    ErrorMin   :Extended;   { Lo mínimo a que se debe llegar }
   end;               { Nota: TempAux=Estimado Inicial ideal que usó Burbuja o Rocio (Salida)}
       
FUNCTION Entalpia(DatFluido:DatosFlash;Var H:Extended):Error; { H en kJ/kmol }

FUNCTION Entropia(DatFluido:DatosFlash;Var S:Extended):Error; { S en kJ/kmol.K }

FUNCTION CalcularKi(Var Dat:DatosFlash):Error;
                    
FUNCTION Equilibrio(Var Dat:DatosFlash; QueHacer:Integer; DarDat:ProcPtr):Error;         
         
Implementation

 Const 
       MaxZ      = 0.07;  { Para pasar a AltaPres }

 Type FraccMolarPtr=^FraccMolar;
 
      ExtendedPtr=^Extended;
  
Type InformacionBurbRoc=Record
      Listo      :Boolean;
      AltaPres   :Boolean;
      MonoComp   :Boolean;
      DeBurbuja  :Boolean;
      DeTemp     :Boolean;
      F1,F2,F3   :Extended;
      TP1,TP2    :Extended;
      DeltaTP    :Extended;
      ErrorIni   :Extended;
      CompXoY    :FraccMolar;    { Las compocisiones anteriores x o y }
      CompXoYPtr :FraccMolarPtr; { Las compocisiones a calcular, x o y }
      CompK      :FraccMolar;    { Las compocisiones iniciales }
      Errores    :Integer;
      Max        :Extended;
      Min        :Extended;
      ZCriticoG  :Extended;
      ZCriticoL  :Extended;
      VarEnUso   :ExtendedPtr;
      MinPtr     :ExtendedPtr;
      MaxPtr     :ExtendedPtr;
     end;       

 Var ZKiG,ZKiL:Extended;
     MaxVar   :Extended; { Incremento<MaxVar*Variable }
  
PROCEDURE PasarDatos(Var D:DatosFlash;DeQue:Integer;Pr:ProcPtr); Inline $205F, $4E90;

FUNCTION RevisarSeaNAN(E:Extended):Boolean;
 Var Que:NumClass;
 begin
  Que:=ClassExtended(E);
  If (Que=ZeroNum) or (Que=NormalNum) then RevisarSeaNAN:=False
  else RevisarSeaNAN:=True;
 end;
 
FUNCTION RevisarVectorNAN(V1,V2:FraccMolar;N:Integer):Error;
 var ii:Integer;
 begin
  For ii:=1 to N do
   V1[1]:=V1[1]+V1[ii]+V2[ii];
  if RevisarSeaNAN(V1[1]) then RevisarVectorNAN:=ErrorMatematico
  else RevisarVectorNAN:=SinError;
 end;
 
FUNCTION Abortar:Boolean;
 Var Evento:EventRecord;
     Resultado:Boolean;
 begin
  Resultado:=False;
  if GetNextEvent(KeyDownMask+AutoKeyMask,Evento) then
   If (Evento.Modifiers and CmdKey)<>0 then
    if Chr(Evento.Message and CharCodeMask)=',' then
     Resultado:=True;
  Abortar:=Resultado;
 end;
 
FUNCTION ZCriticoMezcla(Var ZCrit,TCrit,PCrit:Extended;Var Compuestos:Propiedades;Z:FraccMolar;
                          Numero,Modelo:Integer):Error;
 Var
     Itera    :Integer;
     Ac,Bc    :Extended;
     IncrEnT  :Extended;
     IncrEnP  :Extended;
     PCritIni :Extended;
     AA,BB,CC,EE,FF:Extended;
     AAi,BBi,CCi:FraccMolar;
     Env      :Environment;
     
 PROCEDURe AcBc;
  begin
   Case Modelo of
     VWAdachi:begin
               ZCrit:=3/8;
              end;
     RKS..RKSPolar:
              begin
               Zcrit:=0.33345
              end;
     PengR   :begin
               Zcrit:=0.30738;
              end;
     SchmidtW:begin
               FF:=0.25988221-0.02142913*CC+0.00337143*sqr(CC); { FF = BetaC }
               Zcrit:=1/3/(1+FF*CC);
               Ac:=XpwrI(1-Zcrit*(1-FF),3);
               Bc:=FF*ZCrit;
              end;
     PatelT..PatelTUSB:
              begin
               ZCrit:=(1-CC)/3;
               FF:=(0.076799-sqrt(0.0847788*ZCrit-0.02199685172))/0.0423894; { FF = w }
               Bc:=0.08517138-0.02640592*FF+0.00788769*Sqr(FF);
               Ac:=3*Sqr(ZCrit)+3*(1-2*ZCrit)*Bc+Sqr(Bc)+1-3*Zcrit;
              end;
   end; { Case }
  end;{ AcBc }

 begin
  PCrit:=0;
  PCritIni:=0;
  For Itera:=1 to Numero do
   begin
    TCrit:=TCrit+Z[itera]*Compuestos[itera].Tc; 
    PCritIni:=PCritIni+Z[Itera]*Compuestos[itera].Pc; { Estimados ideales }
   end;
  Pcrit:=PCritIni;
  ZCrit:=Nan(255);
  Itera:=0;
  If Modelo<SchmidtW then
   AcBc;
  If (Modelo<=UltimoModeloGas) And (Modelo>=SchmidtW) then
   begin
    If Modelo>GasIdeal then
     begin
      Componentes(Compuestos,Numero);
      Concentraciones(Z,Z);
      Repeat
       PresionYTemp(PCrit,TCrit);
       CalcularA_B_CdeMezcla(Modelo,Gas); { ó Líquido, es igual }
       ObtenerConstantes(AA,BB,CC,EE,FF,AAi,BBi,CCi);
       If (Itera=0) or (Modelo>=SchmidtW) then
        AcBc;
       IncrEnP:=PCrit*(Bc/BB-Ac/AA/3-0.666666666)*1.5;
       IncrEnT:=-TCrit*(Bc/BB-1-IncrEnP/PCrit);
       TCrit:=TCrit+IncrEnT;
       PCrit:=PCrit+IncrEnP;
       Itera:=Itera+1;
      Until (Itera>=20) or ((Abs(IncrEnP)<0.05) and (Abs(IncrEnT)<0.5));
     end;
   end;
  If (Itera>=NMaxItera) or (Modelo>UltimoModeloGas) then 
   begin
    ZCriticoMezcla:=NoConverge;
    PCrit:=PCritIni;
    ZCrit:=0.32;
   end
  else ZCriticoMezcla:=SinError;
 end;

PROCEDURE Estimados(Var K:FraccMolar;Presion,Temp:extended;
                    Compo:Propiedades;Usarlos:Inicio;N:Integer);
                    { La Presion en [Bar] y Temp en [K]}
 Var i:integer;
 begin
  For i:=1 to N do K[i]:=PrVapor(Compo[i],Temp,Pred(Usarlos))/Presion;
 end;
 
FUNCTION CalcularKi;
 var      
  Errores  : Error;
  i        : Integer;
  PVap     : Extended;   { Presión de vapor para cada componente }
  MonoComp : Propiedades;{ Solo para meter un componente de la mezcla }
  Fi_Fi    : Boolean;    { True sí modelo Fi_Fi de iguales ecuaciones de Edo. }
  XYUnit   : FraccMolar; { Composición unitaria }
  CofAux   : Extended;
  Zcomp    : Extended;
  Env      : Environment;
begin
 GetEnvironment(Env);
 SetEnvironment(0);
 XYUnit[1]:=1;Errores:=SinError;
 with Dat do 
  begin
   If (ModeloGas=ModeloLiq) or (ModeloLiq>UltimoModeloGas) then Fi_Fi:=true 
   else Fi_Fi:=False;
   Concentraciones(x,y);
   if not(Fi_Fi) then   { Esto sirve para ahorra el cálculo de los Ai,Bi,Ci si }
    PresionYTemp(Presion,Temperatura);{ la técnica es Fi_Fi ó si no varió T }
   Componentes(CompFlash,Numero);
   if (ModeloLiq<=UltimoModeloGas) and 
      (ModeloLiq<>SolucionIdeal) then { Ecuaciones para Gas aplicadas }
    begin                             { a Líquido, ( fi-fi) }
     CalcularA_B_CdeMezcla(ModeloLiq,Liquido);
     Zcomp:=FactorCompre(ModeloLiq,Liquido);
     ZKiL:=ZComp;
     If RevisarSeaNAN(Zcomp) then Errores:=NoHayDosFases;
     If Errores = SinError then
      for i:=1 to Numero do
       K[i]:=CoFug_i(i,ModeloLiq,Liquido,Zcomp)*Presion;
    end
   else { Ahora son solo ecuaciones exclusivas para liquido }
    begin
     for i:=1 to Numero do
      begin
       Pvap:=PrVapor(CompFlash[i],Temperatura,TipPVapor);
       if FugLPuroporC_S then
        CofAux:=CoFug_i(i,ChaoSeader,Liquido,ZComp)*Presion/Pvap 
       else                         { no hace falta Liquido ni ZComp, }
        begin                       { son rellenos en este caso }       
         PresionYTemp(Pvap,Temperatura);
         Monocomp[1]:=CompFlash[i];
         Componentes(MonoComp,1);
         Concentraciones(XYUnit,XYUnit);
         CalcularA_B_CdeMezcla(ModeloGas,Gas);
         ZComp:=FactorCompre(ModeloGas,Gas);
         CofAux:=CoFug_i(1,ModeloGas,Gas,ZComp);
         Concentraciones(x,y);
         Componentes(CompFlash,Numero); { engaño }
         PresionYTemp(Presion,Temperatura);
        end;
       K[i]:=Gamma_i(i,ModeloLiq,CoefAIJ)*Pvap*CofAux;
       If Poynting Then K[i]:=K[i]*CorrectPoynting(Presion,Pvap,Temperatura,CompFlash[i].vl);
      end; { For… }
     Componentes(CompFlash,Numero);
     ZkiL:=0;
    end; { else… }
   If Errores = SinError then
    begin
     If not(Fi_Fi) then
      PresionYTemp(Presion,Temperatura);
     CalcularA_B_CdeMezcla(ModeloGas,Gas);
     Zcomp:=FactorCompre(ModeloGas,Gas);
     ZKiG:=ZComp;
     for i:=1 to Numero do
      K[i]:=K[i]/CoFug_i(i,ModeloGas,Gas,Zcomp)/Presion;
    end;
  end;{ with dat… }
 If Errores=SinError then
  Errores:=RevisarVectorNAN(Dat.K,Dat.K,Dat.Numero);
 SetEnvironment(Env);
 CalcularKi:=Errores;
end; { CalcularKi… }

PROCEDURE CambiarLosAij(Var Aij:MatrizConst;Var T1:Extended;T2:Extended;Num:Integer;
                         ModeloLiq:Integer);
 begin
  If T1>0 then
   begin
    VariarLosAij(Aij,T1,T2,ModeloLiq);
    T1:=T2;
   end;
 end;

 PROCEDURE Parabola(x1,x2,x3,fx1,fx2,fx3:Extended;Var Incremento:Extended;Lineal:boolean);
 var { MOSCA el X3 debe ser el último calculado y así en sucesivo }
  aa,bb,cc,h0,h1:Extended; { Coeficientes cuadráticos }
 begin
  if ClassExtended((X3-X1)/X3)=ZeroNum Then Lineal:=True;
  if Not(Lineal) then
   begin
    h0:=x1-x2;
    h1:=x2-x3;
    cc:=(fx1-fx2-(fx2-fx3)*h0/h1)/(Sqr(x1)-Sqr(x2)-(Sqr(x2)-Sqr(x3))*h0/h1);
    bb:=(fx1-fx2-cc*(Sqr(x1)-Sqr(x2)))/h0;
    aa:=fx1-bb*x1-cc*Sqr(x1);
    h1:=Sqr(bb)-4*aa*cc; { discriminante }
   end
  else
   h1:=-1; {para que pase por lineal}
  if h1>0 then
   begin
     h0:=(-bb-sqrt(h1))/(2*cc); { las dos raíces }
     h1:=(-bb+sqrt(h1))/(2*cc);
     if abs(h0-x3)<abs(h1-x3) then
      Incremento:=h0-x3
     else
      Incremento:=h1-x3;
   end
  else
   begin
    h1:=(fx2-fx3)/(x2-x3);
    Incremento:=-fx3/h1;
   end;
 end;
 
 FUNCTION NuevoIncremento(x1,x2,x3,f1,f2,f3:Extended;Iteracion:Integer):Extended;
 Var Aux:Extended;
 begin
  If Iteracion>2 then
   begin
    Parabola(x1,x2,x3,f1,f2,f3,Aux,False);
   end
  else
   If Iteracion=1 then
    Aux:=-x3*0.01
   else
    Parabola(x1,x2,x3,f1,f2,f3,Aux,True);
  If Abs(Aux)>MaxVar*x3 then
   Aux:=CopySign(Aux,MaxVar*x3);
  NuevoIncremento:=Aux;
 end;  

 PROCEDURE IncrementarYRodar(Var x1,x2,x3,Incremento,f1,f2,f3:Extended);
 begin
  x1:=x2;x2:=x3;x3:=x3+Incremento;
  f1:=f2;f2:=f3; 
 end;

PROCEDURE Normalice(var XY:FraccMolar;SumaT:Extended;N:Integer); 
 var
  i:Integer;
 begin
  for i:=1 to N do
   XY[i]:=XY[i]/SumaT;
 end;
 
FUNCTION CalcularCV(var Dat:DatosFlash;Var CV:extended):Error; { Calcula el coeficiente de vaporización }
 var
  f,fp,aux,CVAnt:extended;
  i,itera:integer;
 begin
  itera:=0;
  with Dat do
   repeat
    f:=0;
    fp:=0;
    itera:=itera+1;
    CVAnt:=CV;
    for i:=1 to Numero do
     begin
      aux:=(1-K[i])/(1+CV*(K[i]-1));
      f:=f+Z[i]*aux;
      fp:=fp+Z[i]*sqr(aux);
     end;
    CV:=CV-f/fp;
   until (Abs(CV-CVAnt)<TolFracc*Abs(CVAnt)) or (itera>NMaxItera);
  if itera>NMaxItera then
     CalcularCV:=NoConvergeCV
  else 
   If (Cv>1) or (Cv<0) then CalcularCv:=FueraDeRangoCV
   else CalcularCV:=SinError;
 end; { CalcularCV… }

FUNCTION FlashIsotermico(Var Dat:DatosFlash; 
                         DesdeBurbRocio:Boolean;DarDat:ProcPtr):Error;
 var
  Errores  : Error;
  XAnt     : FraccMolar;
  YAnt     : FraccMolar;
  Env      : Environment;
   
 PROCEDURe CalcularXyY;
 var 
  i:integer;
 begin
  With Dat do
   for i:=1 to Numero do
    begin
     X[i]:=Z[i]/(1+CoefVap*(K[i]-1));
     Y[i]:=X[i]*K[i];
    end;
 end;  
 
 FUNCTIOn Salida:Boolean;
  Var i:integer;Mayor,aux:extended;
  begin
   Mayor:=0;
   With Dat do
    for i:=1 to Numero do
     begin
      aux:=abs(Xant[i]-X[i]);
      if aux>Mayor*Xant[i] then Mayor:=aux;
      aux:=abs(Yant[i]-Y[i]);
      if aux>Mayor*YAnt[i] then Mayor:=aux;
     end;
    If DarDat<>Nil then
     begin
      If Dat.Iteraciones=1 then Dat.ErrorIni:=Mayor;
      Dat.ErrorAct:=Mayor;
     end;
   if Mayor<TolFracc then Salida:=true
   else Salida:=False;
  end;
  
  FUNCTIOn EstimarCV:Extended;
  begin
   EstimarCV:=0.5;
  end;
  
 PROCEDURe NormalizarXyY(Var X,Y:FraccMolar);
  var i:Integer; Suma:Extended;
  begin 
   Suma:=0;
   for i:=1 to Dat.Numero do
    Suma:=Suma+X[i];
   Normalice(X,Suma,Dat.Numero);
   Suma:=0;
   for i:=1 to Dat.Numero do
    Suma:=Suma+Y[i];
   Normalice(Y,Suma,Dat.Numero);
  end;
  
 begin { FlashIsotermico… }
  GetEnvironment(Env);
  SetEnvironment(0);
  Dat.Iteraciones:=0;
  If DarDat<>Nil then
   begin
    Dat.ErrorIni:=-1;
    Dat.ErrorMin:=TolFracc;
   end;
  Errores:=SinError;
  With Dat do 
   begin
    If DarDat<>Nil then 
     PasarDatos(Dat,FIsotermico,DarDat);
    if Usarlos <> PorUsuario then
     Estimados(K,Presion,Temperatura,CompFlash,Usarlos,Numero);
    If Not(Amortiguado) then
     FacAmort:=1;
    if IniciarCV then
     CoefVap:=EstimarCV
    else
     CoefVap:=0.5;
    Errores:=CalcularCV(Dat,CoefVap);
    CalcularXyY;
    NormalizarXyY(X,Y);
    If Errores<>SinError then
     begin
      Errores:=SinError;
      If CoefVap>1 then CoefVap:=0.98
      else CoefVap:=0.02;
     end;
    PresionYTemp(Presion,Temperatura);
    If ModeloLiq>ScatchardH then
     CambiarLosAij(CoefAij,TemperAij,Temperatura,Numero,ModeloLiq);
    repeat
     Iteraciones:=Iteraciones+1;
     If Abortar Then Errores:=Abortado
     else
      Errores:=CalcularKi(Dat);
     if Errores=SinError then  
      begin
       Errores:=CalcularCV(Dat,CoefVap);
       If Errores=FueraDeRangoCV then
        If (CoefVap>-0.02) and (CoefVap<1.02) then 
         Errores:=SinError;
       XAnt:=X;
       YAnt:=Y;
       If DarDat<>Nil then 
        PasarDatos(Dat,FIsotermico,DarDat);
       CalcularXyY;
       If (Iteraciones<2) or DesdeBurbRocio then
        begin
         If (CoefVap>-2) or (CoefVap<3) then
          Errores:=SinError;
         NormalizarXyY(X,Y);
        end;
      end;
    until Salida or (Errores<>SinError) or (Iteraciones > NMaxItera);
   end; { With Dat… }
  If DarDat<>Nil then 
   PasarDatos(Dat,FIsotermico,DarDat);
  If (Dat.CoefVap>1) or (Dat.CoefVap<0) then 
   begin
    Errores:=FueraDeRangoCV;
    If DesdeBurbRocio and (Dat.ModeloLiq>UltimoModeloGas) then
     If (ZkiG<0.33) then Dat.CoefVap:=-5;
   end { Esto evita un error: a altas P y bajas T a veces los Ki hacen que }
  else { CV>0, pero la EOS estaba en zona de raiz de líquidos }
   If Dat.Iteraciones>NMaxItera then Errores:=NoConverge;
  SetEnvironment(Env);
  FlashIsotermico:=Errores;
 end; { FlashIsotermico… }
                         
 FUNCTION TempInicial(Var Sustancias:Propiedades;Presion:Extended;X:FraccMolar;
                      Tipo:TipoPresVapor;N:Integer;Burbuja:Boolean):Extended;
 var  { Presion en [bar]. Estima la temperatura de ebullición para iniciar }
   Aux     :Extended;         { la temperatura de otros algoritmos }
   F1,F2,F3:Extended;
   T1,T2   :Extended;
   Inc     :Extended;
   i       :Integer;
   Iteracion:Integer;
 begin
  MaxVar:=0.25;
  Aux:=X[1]*TEbullicion(Sustancias[1],Presion,Tipo);
  for i:=2 to N do
   Aux:=Aux+X[i]*TEbullicion(Sustancias[i],Presion,Tipo);
  Iteracion:=0;
  Inc:=0;
  repeat
   Iteracion:=Iteracion+1;
   IncrementarYRodar(T1,T2,Aux,Inc,f1,f2,f3);
   if Burbuja then                          { Temp. ideal de burbuja }
    begin
     F3:=X[1]*PrVapor(Sustancias[1],Aux,Tipo);
     For i:=2 to N do
      F3:=F3+X[i]*PrVapor(Sustancias[i],Aux,Tipo);
     F3:=F3/Presion;
    end
   else                                     { Temp. ideal de rocío }
    begin
     F3:=X[1]/PrVapor(Sustancias[1],Aux,Tipo);
     For i:=2 to N do
      F3:=F3+X[i]/PrVapor(Sustancias[i],Aux,Tipo);
     F3:=F3*Presion;
    end;
   F3:=Ln(F3);
   Inc:=NuevoIncremento(T1,T2,Aux,F1,F2,F3,Iteracion);
  until ((Abs(Inc)<0.1) or (Iteracion>30)) and (Iteracion>1);
  TempInicial:=Aux;
 end; { TempInicial… }
 
 FUNCTION EntalpiaEntropia(Que:Boolean;Var DatFluido:DatosFlash;Var Valor:Extended):Error;
 Var Fi_Fi   :Boolean;    { True si el modelo es igual para ambas fases }
     HSLiq   :Extended;   { Entalpía o entropía del líquido }
     HSGas   :Extended;   { Entalpía o entropía del gas real}
     HSIdeal :Extended;   { Entalpía o entropía del gas ideal }
     ZComp   :Extended;   { Factor de compresibilidad }
     Calidad :Extended;   { X,Coeficiente de Vaporización }
     Env     :Environment;
 begin
  GetEnvironment(Env);
  SetEnvironment(0);
  with DatFluido do
   begin
    Fi_Fi:=False;
    Componentes(CompFlash,Numero);
    Concentraciones(X,Y);
    Calidad:=CoefVap;
    PresionYTemp(Presion,Temperatura);
    If CoefVap>0 then { Cálculo de la entalpía, entropía del gas ó el vapor }
     begin
      CalcularA_B_CdeMezcla(ModeloGas,Gas);
      Zcomp:=FactorCompre(ModeloGas,Gas);
      if que then { Entalpías }
       begin
        HSIdeal:=EntalpiaIdeal(Gas);
        HSGas:=EntalpiaRes(ModeloGas,Gas,Zcomp);
       end
      else        { Entropías }
       begin
        HSIdeal:=EntropiaIdeal(Gas);
        HSGas:=EntropiaRes(ModeloGas,Gas,Zcomp);
       end;
      HSGas:=HSIdeal+HSGas;
      if ModeloGas=ModeloLiq then Fi_Fi:=True
     end
    else
     begin
      HSGas:=0;
      Calidad:=0; { Todo es líquido }
     end;
    If CoefVap<1 then 
     begin 
      If Not(Fi_Fi) then
       PresionYTemp(Presion,Temperatura);
      if Que then
       HSIdeal:=EntalpiaIdeal(Liquido)
      else
       HSIdeal:=EntropiaIdeal(Liquido);
      if (ModeloLiq <= UltimoModeloGas) And (ModeloLiq <> SolucionIdeal) then
       begin
        CalcularA_B_CdeMezcla(ModeloLiq,Liquido);
        Zcomp:=FactorCompre(ModeloLiq,Liquido);
        if que then
         HSLiq:=EntalpiaRes(ModeloLiq,Liquido,Zcomp)
        else
         HSLiq:=EntropiaRes(ModeloLiq,Liquido,Zcomp)
       end
      else { modelo de líquido exclusivamente }
       begin
        If ModeloLiq>ScatchardH then
         CambiarLosAij(CoefAij,TemperAij,Temperatura,Numero,ModeloLiq);
        if FugLPuroporC_S then
         begin
          If que then
           HSLiq:=EntalpiaResLiq(ChaoSeader,ModeloLiq,CoefAIJ,TipPVapor,Poynting)
          else
           HSLiq:=EntropiaResLiq(ChaoSeader,ModeloLiq,CoefAIJ,TipPVapor);
         end
        else
         begin
          if Que Then
           HSLiq:=EntalpiaResLiq(ModeloGas,ModeloLiq,CoefAIJ,TipPVapor,Poynting)
          else 
           HSLiq:=EntropiaResLiq(ModeloGas,ModeloLiq,CoefAIJ,TipPVapor);
         end;
       end;
      HSliq:=HSIdeal+HSLiq;           
     end
    else
     begin
      HSliq:=0;
      Calidad:=1; { Todo es gas }
     end;
   end; { With DatFluido }
  Valor:=HSLiq+Calidad*(HSGas-HSliq);
  If RevisarSeaNAN(Valor) then EntalpiaEntropia:=ErrorMatematico 
  else EntalpiaEntropia:=SinError;
  SetEnvironment(Env);
 end;
 
 FUNCTION Entalpia;
  begin
   Entalpia:=EntalpiaEntropia(True,DatFluido,H);
  end;
  
 FUNCTION Entropia;
  begin
   Entropia:=EntalpiaEntropia(False,DatFluido,S);
  end;
  
 PROCEDURE Acotar(XMin,XMax,X:Extended;Var Incremento:Extended;Var IncAcotado:Boolean); 
 Var XCasiNueva:Extended;
 begin
  XCasiNueva:=X+Incremento;
  If (XCasiNueva>XMax) or (XCasiNueva<XMin) then
   begin
    Incremento:=(XMax+XMin)/2-X;
    IncAcotado:=True;
    If Abs(Incremento)>0.25*X then
     Incremento:=CopySign(Incremento,0.25*X);
   end
  else
   IncAcotado:=False;
 end;

 PROCEDURE ActualizarMinYMax(Var XMin,XMax:Extended;X,F:Extended);
 begin
  if F>0 then     { NOTA: debe hacerce de forma que F sea creciente con X }
   begin 
    if (X<XMax) then 
     XMax:=X;
   end
  else 
   if (X>XMin) then 
    XMin:=X;
 end;

 FUNCTION FlashAdiabatico(Var Dat:DatosFlash; DarDat:ProcPtr):Error;
 var
   IteracionesAdia:Integer;
   Condicion    :Error;
   HAlimentacion:Extended;  { entalpía de la alimentación }
   HSalida      :Extended;  { entalpía de la salida }     
   T1,T2        :Extended;    { temperatura inicial }      
   FT1,FT2,FT3  :Extended;
   CpVirtual    :Extended;
   TempMonoComp :Extended;
   HGasMono     :Extended;
   HLiqMono     :Extended;
   TMin         :Extended; { Para ir acotando la solución, solo se usa en }
   TMax         :Extended; { casos de difícil convergencia }
   TCasiNueva   :Extended; { Solo para verificar se encuentre en el rango }
   DeltaT       :Extended;
   ErrorIni     :Extended;
   MonoComp     :Boolean;
   Salir        :Boolean;
   YaListoMono  :Boolean;
   Cortando     :Boolean; { Solo es un auxiliar, no se usa }
   Env          :Environment;
   DatAlim      :DatosFlash;  { auxiliar }   
 begin
  GetEnvironment(Env);
  SetEnvironment(0);
  DatAlim:=Dat;
  DatAlim.Temperatura:=Dat.TempAux; { La de la alimentación }
  DatAlim.Presion:=Dat.PresAux;     { La de la alimentación }
  TMax:=1e5;
  TMin:=0;   { Valores máximos y mínimos iniciales }
  Dat.Iteraciones:=0;
  Dat.ErrorIni:=-1;
  If DarDat<>Nil then 
   PasarDatos(Dat,FAdiabatico,DarDat);
  YaListoMono:=False; { Para que entre al lazo en T }
  If Dat.Numero>1 then
   begin
    Condicion:=FlashIsotermico(DatAlim,true,Nil); { Evaluación de la alimentación, multicomponente }
    MonoComp:=False;
   end
  else
   begin
    With DatAlim do
     If PrVapor(CompFlash[1],Temperatura,TipPVapor)>Presion then   { Un solo componente }
      CoefVap:=2
     else
      CoefVap:=-1;
    With Dat do
     begin
      TempMonoComp:=TEbullicion(CompFlash[1],Presion,TipPVapor); { T ebullición }
      X[1]:=1; 
      Y[1]:=1;
      K[1]:=1;
     end;
    Condicion:=NoConverge; { Para que más abajo crea que solo hay una fase }
    MonoComp:=True;
   end;
  If Condicion<>ErrorMatematico then
   begin
    If Condicion>=NoConverge then
     begin
      DatAlim.X:=Dat.Z; { Solo hay una fase, Entalpia() se dará cuenta si es un }
      DatAlim.Y:=Dat.Z; { liquido o un gas, por eso X=Y=Z }
     end;
    Condicion:=Entalpia(DatAlim,HAlimentacion);
    HAlimentacion:=HAlimentacion + Dat.Calor;
    If (Condicion=SinError) and MonoComp then
     begin
      DatAlim.Temperatura:=TempMonoComp; { Ahora DatAlim es una variable auxiliar }
      DatAlim.Presion:=Dat.Presion;      { para calcular las entalpías de líquido }
      DatAlim.CoefVap:=2;                { y vapor del MonoComponente a Presión.  }
      Condicion:=Entalpia(DatAlim,HGasMono);
      If Condicion=SinError then
       begin
        DatAlim.CoefVap:=-1;
        Condicion:=Entalpia(DatAlim,HLiqMono);
       end;
      If (Halimentacion>HLiqMono) and (Halimentacion<HGasMono) then
       begin
        Dat.CoefVap:=(HAlimentacion-HLiqMono)/(HGasMono-HLiqMono);
        Dat.Temperatura:=TempMonoComp;
        YaListoMono:=True;
        If Condicion<>ErrorMatematico then Condicion:=SinError;
        IteracionesAdia:=0;
       end
      else 
       begin
        If HAlimentacion<HLiqMono then 
         begin
          TMax:=TempMonoComp;
          Dat.CoefVap:=-1;
         end
        else 
         begin
          TMin:=TempMonoComp;
          Dat.CoefVap:=2;
         end;
       end;
     end; { If Condicion=SinError and MonoComp… }
   end;  { If Condicion<>ErrorMatematico… }
  If (Condicion=SinError) and Not(YaListoMono) then
   begin
    with Dat do
     begin
      If Not(Amortiguado) then FacAmort:=1;
      IteracionesAdia:=0;
      if Not(HayInicial) then
       if (DatAlim.CoefVap<1.0) and Not(MonoComp) then
        Temperatura:=TempInicial(CompFlash,Presion,Z,TipPVapor,Numero,True)
       else 
        Temperatura:=TempAux;
      If Temperatura>Tmax then { Esto funciona solo para mono componente }
       Temperatura:=0.98*TMax; { pues multicomponente es difícil que entre. }
      If Temperatura<TMin then { Esto es para que la temperatura inicial }
       Temperatura:=TMin*1.02; { no salga del rango de temperaturas. }
      CpVirtual:=0;
      DeltaT:=0;
      MaxVar:=0.25;
      repeat
       IteracionesAdia:=IteracionesAdia+1;
       IncrementarYRodar(T1,T2,Temperatura,DeltaT,Ft1,Ft2,Ft3);
       If DarDat<>Nil then 
        begin
         Dat.Iteraciones:=IteracionesAdia;
         PasarDatos(Dat,FAdiabatico,DarDat);
        end;
       If Not(MonoComp) then
        Condicion:=FlashIsotermico(Dat,True,Nil);
       if Condicion<>ErrorMatematico then
        begin
         If Not((Dat.CoefVap>0) and (Dat.CoefVap<1)) then
          begin
           X:=Z;
           Y:=Z;
          end;
         Condicion:=Entalpia(Dat,HSalida);
        end;
       Ft3:=HSalida-Halimentacion;
       If RevisarSeaNAN(FT3) then Condicion:=ErrorMatematico
       else
        begin
         ActualizarMinYMax(TMin,TMax,Temperatura,FT3);
         DeltaT:=FacAmort*NuevoIncremento(T1,T2,Temperatura,Ft1,Ft2,Ft3,IteracionesAdia);
         If IteracionesAdia>1 then
          begin
           CpVirtual:=Abs((Ft3-Ft2)/(Temperatura-T2));
           Acotar(TMin,TMax,Temperatura,DeltaT,Cortando);
           if IteracionesAdia=2 then
            ErrorIni:=Abs(Ft3);
           ErrorAct:=Abs(Ft3);
           ErrorMin:=Abs(TolTemp*CpVirtual*Temperatura)
          end;
        end;
       If Abortar then Condicion:=Abortado
       else Salir:=(IteracionesAdia>=2) and (Abs(FT3)<TolTemp*CpVirtual*Temperatura);
      until  Salir or (IteracionesAdia>NMaxItera) or (Condicion<>SinError);
      Iteraciones:=IteracionesAdia;
      If DarDat<>Nil then 
       PasarDatos(Dat,FAdiabatico,DarDat);
     end; { with… Dat }
   end; { If Condicion… }
  SetEnvironment(Env);
  if Dat.Iteraciones>NMaxItera then
   FlashAdiabatico:=NoConverge
  else
   FlashAdiabatico:=Condicion;
 end; { FlashAdiabatico… }
  
FUNCTION LazoK_XoY(Var Dat:DatosFlash;Var SumaXoY:Extended;Burbuja,
                   ListoXoY:Boolean):Error;
 var Errores:Integer;
     IteraK_XY :Integer;
     SumaAnt   :Extended;
     i         :Integer;
     RevisarZ  :Boolean;
 begin
  With Dat do
   begin
    Errores:=SinError;
    RevisarZ:=(ModeloGas>GasIdeal) and (ModeloLiq>SolucionIdeal) and
                 (ModeloLiq<=UltimoModeloGas);
    PresionYTemp(Presion,Temperatura);
    IteraK_XY:=0;
    SumaXoY:=-2000;
    repeat                               { repeat del lazo de las composiciones }
     SumaAnt:=SumaXoY;
     SumaXoY:=0;
     IteraK_XY:=IteraK_XY+1; 
     If Abortar then Errores:=Abortado
     else 
      If Not(ListoXoY) then
       begin
        If ModeloLiq>ScatchardH then
         CambiarLosAij(CoefAij,TemperAij,Temperatura,Numero,ModeloLiq);
        Errores:=CalcularKi(Dat);
       end; 
     If Errores=SinError then
      begin     
       if Burbuja then                     { Cálculo de las composiciones }
        for i:=1 to Numero do
         begin
          Y[i]:=K[i]*X[i];
          SumaXoY:=SumaXoY+Y[i];
         end
       else
        for i:=1 to Numero do
         begin
          X[i]:=Y[i]/K[i];
          SumaXoY:=SumaXoY+X[i];
         end;            
       If Burbuja then
        Normalice(Y,SumaXoY,Numero)
       else
        Normalice(X,SumaXoY,Numero);
       ListoXoY:=(Abs(SumaXoY-SumaAnt)<=TolFracc*Numero) or (Abs(SumaXoY-1)<TolFracc*Numero)
                 or ListoXoY;
       If RevisarZ then If Abs(Zkil-Zkig)<MaxZ then Errores:=SolucionTrivial;
      end;
    until(listoXoY) or (Errores<>SinError) or (IteraK_XY>NMaxItera);
   end;
  if IteraK_XY>NMaxItera then Errores:=NoConvergeK_XY;
  LazoK_XoY:=Errores;
 end;
      
PROCEDURE AltaPresion(Var Variables:InformacionBurbRoc;Var Dat:DatosFlash;
                    DarDat:ProcPtr);
 Const  MaxVaria=15;
 Var i,j       :Integer;
     IteraInt  :Integer;
     CuentaTrivial:Integer;
     Aux       :Extended;
     Tol       :Extended;
     Salida    :Boolean;
     AcotarVar :Boolean;
 begin
  With Variables,Dat do
   begin
    Iteraciones:=0;
    Dat.ErrorIni:=-1;
    If DarDat<>Nil then 
     PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
    IteraInt:=0;
    AcotarVar:=True;
    If DeTemp then Tol:=TolTemp else Tol:=TolPres;
    If MonoComp then
     begin
      Salida:=False; { True si se esta cortando }
      MaxVar:=0.25;
      Repeat
       Iteraciones:=Iteraciones+1;
       IteraInt:=IteraInt+1;
       If (Abs(DeltaTP)<VarEnUso^*Tol) and (IteraInt>2) and Not(Salida) then
        Listo:=True;
       IncrementarYRodar(TP1,TP2,VarEnUso^,DeltaTP,F1,F2,F3);
       If DarDat<>Nil then 
        PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
       PresionYTemp(Presion,Temperatura);
       Errores:=CalcularKi(Dat);
       If Abortar then Errores:=Abortado;
       If Errores=SinError then
        begin
         If DeBurbuja then
          F3:=Ln(K[1])
         else
          F3:=Ln(1/K[1]);
         if RevisarSeaNAN(F3) then Errores:=ErrorMatematico;
         PresAux:=F3;
        end;
       If Errores=SinError then
        begin
         If (Zkil>ZCriticoL) or (Zkig<ZCriticoG) then
          begin 
           If Zkil>ZCriticoL then
            MaxPtr^:=VarEnUso^
           else 
            MinPtr^:=VarEnUso^;
           DeltaTP:=1e12;
           IteraInt:=0;
          end
         else
          begin
           DeltaTP:=FacAmort*NuevoIncremento(TP1,TP2,VarEnUso^,F1,F2,F3,IteraInt);
           If (IteraInt=1) And ((DeltaTP+VarEnUso^)>Max) or ((DeltaTP+VarEnUso^)<Min) then
            begin
             DeltaTP:=(Max-VarEnUso^)/3;
            end;
           end;
         Acotar(Min,Max,VarEnUso^,DeltaTP,Salida);
         If Iteraciones=2 then ErrorIni:=Abs(DeltaTP/FacAmort);
         ErrorAct:=Abs(DeltaTP/FacAmort);
         ErrorMin:=Abs(Aux*VarEnUso^);
        end; 
      Until (Iteraciones>NMaxItera) or Listo or (Errores<>SinError);
      If DarDat<>Nil then 
       PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
      If Errores=SinError then
       If (Zkil>ZCriticoL) or (Zkig<ZCriticoG) then 
        Errores:=SolucionTrivial
       else
        If (Abs(Max-Min)<Aux*varEnUso^) or (Iteraciones>NMaxItera) then Errores:=noConverge;
     end
    else
     begin
      Monocomp:=False; { Para hacer dos coefvap }
      MaxVar:=0.15;
      ErrorMin:=TolFracc*Numero;
      Repeat
       CompXoY:=CompXoYPtr^; { Compocisiones viejas }
       Iteraciones:=Iteraciones+1;
       Salida:=False;
       Repeat
        IteraInt:=IteraInt+1;
        If (Abs(DeltaTP)<VarEnUso^*Tol) and (IteraInt>2) then
         Salida:=True;
        IncrementarYRodar(TP1,TP2,VarEnUso^,DeltaTP,F1,F2,F3);
        PresionYTemp(Presion,Temperatura);
        If AcotarVar then
         begin
          If IteraInt=3 then Max:=DeltaTP;
          If (IteraInt<=2) or Not(Abs(DeltaTP)<MaxVaria-0.01) then
            CompXoYPtr^:=CompK
          else
           AcotarVar:=False;
          MaxVar:=MaxVaria/VarEnUso^;
          If MaxVar>0.15 then MaxVar:=0.15;
          If (IteraInt>3) and (Abs(Max+DeltaTP)<MaxVaria) then 
           AcotarVar:=False;
         end;
        If DarDat<>Nil then 
         PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
        F3:=0.5; { Inicializacion para Coef. vap. }
        j:=0;
        Repeat
         j:=j+1;
         Errores:=CalcularKi(Dat);
         If Abortar then Errores:=Abortado;
         If Errores=SinError then 
          begin
           Errores:=CalcularCV(Dat,F3);
           Errores:=SinError;
           For i:=1 to Numero do
            begin
             Aux:=Z[i]/(1+F3*(K[i]-1));
             If DeBurbuja then
              Y[i]:=Aux*K[i]
             else
              X[i]:=Aux;
            end;
           Aux:=0;
           For i:=1 to Numero do
            Aux:=Aux+CompXoYPtr^[i];
           Normalice(CompXoYPtr^,Aux,Numero);
          end;
        until (j>=2) or MonoComp or (Errores<>SinError);
        If Errores=SinError then
         begin
          AltaPres:=Abs(Zkil-Zkig)<DZTrivial; { AltaPres como auxiliar }
          CuentaTrivial:=(CuentaTrivial+1)*Ord(AltaPres);
          If CuentaTrivial>5 then Errores:=SolucionTrivial;
          F3:=F3-Ord(Not(DeBurbuja));
          DeltaTP:=FacAmort*NuevoIncremento(TP1,TP2,VarEnUso^,F1,F2,F3,IteraInt);
          If RevisarSeaNaN(DeltaTP) then Errores:=ErrorMatematico;
         end;
        If IteraInt>NMaxItera+3 then Errores:=NoConverge;
       Until Salida or (Errores<>SinError);
       IteraInt:=2;
       MonoComp:=True;
       If Abs(Zkil-Zkig)<DZTrivial then Errores:=SolucionTrivial;
       Aux:=0;
       For i:=1 to Numero do
        begin
         If DeBurbuja then
          Y[i]:=K[i]*x[i]
         else
          X[i]:=Y[i]/K[i];
         Aux:=aux+Abs(CompXoY[i]-CompXoYPtr^[i]); { El error }
        end;
       PresAux:=Aux;
       If Iteraciones=1 then ErrorIni:=Aux;
       ErrorAct:=Aux;
       ErrorMin:=TolFracc*Numero;
       Listo:=(Aux<TolFracc*Numero);
      Until (Listo) or (Iteraciones>NMaxItera) or (Errores<>SinError);
      If DarDat<>Nil then 
       PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
      If Errores=SinError then
       If Iteraciones>NMaxItera then Errores:=NoConvergeK_XY;
     end;
   end;
 end;

PROCEDURE BajaPresion(Var Variables:InformacionBurbRoc;Var Dat:DatosFlash;
                      DarDat:ProcPtr);
 Var i         :Integer;
     Aux       :Extended;
 begin
  MaxVar:=0.25;
  With Variables,Dat do
   begin
    PresAux:=0;
    If DeTemp then Aux:=TolTemp else Aux:=TolPres;
    Repeat
     Iteraciones:=Iteraciones+1;
     If Iteraciones>2 then
      Listo:=(Abs(DeltaTP/FacAmort)<Aux*VarEnUso^) and (Iteraciones>2);
     IncrementarYRodar(TP1,TP2,VarEnUso^,DeltaTP,F1,F2,F3);
     If DarDat<>Nil then 
      PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
     Errores:=LazoK_XoY(Dat,F3,DeBurbuja,False); { Lazo de composiciones }
     If Errores=SinError then
      begin
       PresAux:=Abs(F3)-1;
       If DeBurbuja then
        F3:=Ln(F3)
       else
        F3:=Ln(1/F3);
       if RevisarSeaNAN(F3) then Errores:=ErrorMatematico;
      end;
     If Errores=SinError then
      begin
       DeltaTP:=FacAmort*NuevoIncremento(TP1,TP2,VarEnUso^,F1,F2,F3,Iteraciones);
       If Iteraciones=2 then ErrorIni:=Abs(DeltaTP/FacAmort);
       ErrorAct:=Abs(DeltaTP/FacAmort);
       ErrorMin:=Aux*VarEnUso^;
      end;
     If Errores=SolucionTrivial then
      begin
       AltaPres:=True;
       CompXoYPtr^:=CompK;
       VarEnUso^:=TempAux;
       DeltaTP:=0;
      end;
    Until Listo or (Errores<>SinError) or (Iteraciones>NMaxItera);
   end;
 end;
 
PROCEDURE InicioBurbRoc(Var Variables:InformacionBurbRoc;Var Dat:DatosFlash;DarDat:ProcPtr);
 Var i:Integer;
 begin
  With Variables,Dat do
   begin
    PresAux:=0;
    MonoComp:= Numero = 1;
    Listo:=Abortar; { solo para limpiar los eventos }
    Listo:=False;
    Iteraciones:=0;
    ErrorIni:=-1;
    If DarDat<>Nil then
     PasarDatos(Dat,RocioTemp+Ord(Not(DeTemp))+2*Ord(DeBurbuja),DarDat);
    if Not (HayInicial) then
     begin
      If DeTemp then
       Temperatura:=TempInicial(CompFlash,Presion,Z,TipPVapor,Numero,DeBurbuja)
      else
       begin
        If (ModeloLiq=SolucionIdeal) and (ModeloGas=GasIdeal)
        and Not(Poynting) and Not(FugLPuroporC_S) then
         listo:=True;
        for i:=1 to Numero do             { Cálculo de la Presion }
         CompXoY[i]:=PrVapor(CompFlash[i],Temperatura,Pred(Usarlos));
        Presion:=0;
        if DeBurbuja then 
         for i:=1 to Numero do              { Cálculo de la Presion }
          Presion:=Presion+Z[i]*CompXoY[i]
        else
         begin
          for i:=1 to Numero do
           Presion:=Presion+Z[i]/CompXoY[i];
          Presion:=1/Presion;
         end;
       end;
     end;
    if Usarlos<>PorUsuario then
     Estimados(K,Presion,Temperatura,CompFlash,Usarlos,Numero);
    If Not(Amortiguado) then FacAmort:=1;
    AltaPres:=False; { alta presión }
    if DeBurbuja then
     begin
      X:=Z;
      CompXoYPtr:=@Y; { Para guardar los estimados iniciales }
     end
    else
     begin
      Y:=Z;
      CompXoYPtr:=@X; { Para guardar los estimados iniciales }
     end;
    If MonoComp then
     begin
      CompXoYPtr^[1]:=1;
      If (ModeloGas>GasIdeal) and (ModeloLiq>SolucionIdeal) and
         (ModeloLiq<=UltimoModeloGas) then
       begin
        Errores:=ZCriticoMezcla(ZCriticoG,Max,Min,CompFlash,y,1,ModeloGas);
        Errores:=ZCriticoMezcla(ZCriticoL,Max,Min,CompFlash,y,1,ModeloLiq);
        Max:=VarEnUso^*1.4;
        Min:=VarEnUso^*0.6;
        If DeTemp then
         begin
          If (Max>CompFlash[1].Tc) then Max:=CompFlash[1].Tc;
         end
        else
         If (Max>CompFlash[1].Pc) then Max:=CompFlash[1].Pc;
       end;
     end;
    If ModeloLiq>ScatchardH then
     CambiarLosAij(CoefAij,TemperAij,Temperatura,Numero,ModeloLiq);
    Errores:=LazoK_XoY(Dat,F3,DeBurbuja,True); { Estimados iniciales }
    Errores:=SinError;
    CompK:=CompXoYPtr^;   { Para guardar las X o Y iniciales }
    TempAux:=VarEnUso^;   { y la P o T inicial }
    DeltaTP:=0;
   end;
 end;
 
FUNCTION PresionBR(Var Dat:DatosFlash;Burbuja:Boolean;DarDat:ProcPtr):Error;
var
 Variables  :InformacionBurbRoc;
 Env        :Environment;
 i          :Integer;
begin
 GetEnvironment(Env);
 SetEnvironment(0);
 With Variables,Dat do
  begin
   DeTemp:=False;
   VarEnUso:=@Presion;
   MaxPtr:=@Min;
   MinPtr:=@Max;
   DeBurbuja:=Burbuja;
   InicioBurbRoc(Variables,Dat,DarDat);
   BajaPresion(Variables,Dat,DarDat);
   If Errores=SolucionTrivial then
    AltaPresion(Variables,Dat,DarDat);
   If Iteraciones>NMaxItera then Errores:=NoConverge;
   PresionBR:=Errores;
   If Burbuja then CoefVap:=0 else CoefVap:=1;
  end;
 SetEnvironment(Env);
end; { BurbujaRocioTemp… }

     
FUNCTION TemperaturaBR(Var Dat:DatosFlash;Burbuja:Boolean;DarDat:ProcPtr):Error;
var
 Variables  :InformacionBurbRoc;
 Env        :Environment;
 i          :Integer;
begin
 GetEnvironment(Env);
 SetEnvironment(0);
 With Variables,Dat do
  begin
   DeTemp:=True;
   VarEnUso:=@Temperatura;
   MaxPtr:=@Max;
   MinPtr:=@Min;
   DeBurbuja:=Burbuja;
   InicioBurbRoc(Variables,Dat,DarDat);
   BajaPresion(Variables,Dat,DarDat);
    If Errores=SolucionTrivial then
   AltaPresion(Variables,Dat,DarDat);
   If Iteraciones>NMaxItera then Errores:=NoConverge;
   TemperaturaBR:=Errores;
   If Burbuja then CoefVap:=0 else CoefVap:=1;
  end;
 SetEnvironment(Env);
end; { BurbujaRocioTemp… }


FUNCTION Equilibrio;
Var Resultado:Integer;
begin
  Case QueHacer of
   RocioTemp   : Resultado:=TemperaturaBR(Dat,false,DarDat);
   RocioPres   : Resultado:=PresionBR(Dat,false,DarDat);
   BurbujaTemp : Resultado:=TemperaturaBR(Dat,true,DarDat);
   BurbujaPres : Resultado:=PresionBR(Dat,True,DarDat);
   FIsotermico : Resultado:=FlashIsotermico(Dat,False,DarDat);
   FAdiabatico : Resultado:=FlashAdiabatico(Dat,DarDat);
   otherwise Resultado:=Abortado;
  end;
  Equilibrio:=Resultado;
end; 

begin
End.