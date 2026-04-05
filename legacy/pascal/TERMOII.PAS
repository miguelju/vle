Unit TermoII(5);
{$U-}

{ Paquete termodin‡mico hecho por: }
{ Luis Alberto B‡ez Linde.    }
{ Francisco Avelino Da Silva. }
{ Caracas, 89 }

Interface

{$U TermoI}

Uses Sane,TermoI;

PROCEDURE Componentes(Compuestos:Propiedades;Cuantos:integer);

PROCEDURE PresionYTemp(Presion,Temperatura:extended);

PROCEDURE Concentraciones(Xliq,Yvap:FraccMolar);
 
PROCEDURE CalcularA_B_CdeMezcla(Modelo,Fase:integer);
 
FUNCTION  FactorCompre(Modelo,Fase:Integer):Extended;

FUNCTION  CoFug_i(i,Modelo,Fase:integer;Z:extended):Extended;

FUNCTION  EntalpiaRes(Modelo,fase:Integer;Z:extended):Extended;

FUNCTION  EntropiaRes(Modelo,fase:Integer;Z:extended):Extended; 

PROCEDURE AsignarTyP_Ho_SoReferencia(Treferencia,Preferencia,HoId,SoId:Extended); 
                          {HoId=Entalpia Ideal, SoId=Entrop’a Ideal;  @ Tref,Pref. }

FUNCTION  EntalpiaIdeal(Fase:integer):Extended;

FUNCTION  EntropiaIdeal(Fase:Integer):Extended;

FUNCTION  DLnFugPuro_DT_i(i,Modelo:integer;Presion:Extended;
				      var DlnFugPuro_DP_i:Extended):Extended;
                       { DlnFugPuro_DP_i en [1/bar]. DLnFugPuro_DT_i en [1/K] }
                           
PROCEDURE H2_Metano(ElH2estaEn,ElMetanoestaEn:integer);
               { Para introducir la posici—n de estos componentes en Propiedades }
               
PROCEDURE IntroducirKij(KKij:MatrizConst); { Par‡metros interacci—n binarios }

PROCEDURE HacerCeroKij;

PROCEDURE ObtenerConstantes(var AA,BB,CC,EE,FF:extended;var AAi,BBi,CCi:FraccMolar);

FUNCTION  ObtenerNyComp(var Compuestos:Propiedades):integer;

PROCEDURE ObtenerPyT(var P,T:extended);

PROCEDURE ObtenerConcentraciones(var Xliq,Yvap:FraccMolar);

PROCEDURE ObtenerTyP_Ho_SoReferencia(Var Treferencia,Preferencia,
					     HoId,SoId:Extended);

PROCEDURE ObtenerKij(var KKij:MatrizConst);

{ NOTA: Todas las temperaturas en [K], presiones en [bar], Energ’as en [kJ/kmol],}
{ Entrop’as [kJ/kmol.K] }

                  
Implementation

Type Vector=FraccMolar;

Const Entalpia = true;
      Entropia = false;

Var A,B,C         :Extended;  { Para usar en cualquier ecuacion de estado para gas }
    E,F           :Extended;  { Auxiliares para EOS de estado que lo requieran }
    Pres,Temp     :Extended;  { Presion[Bar] y Temperatura[K] }
    Tref          :Extended;  { Ref., para el c‡lculo de entalp’as y entrop’as }
    Pref          :Extended;  { Referencia para el c‡lculo de entrop’as }
    Ho            :Extended;  { Entalp’a ideal a las condiciones de referencia }
    So            :Extended;  { Entrop’a ideal a las condiciones de referencia }
    FactorPT      :Extended;  { Necesario para Patel-Teja en DLnFugPuro_DT_i }
    Cm            :Propiedades;{ Las propiedades de los componentes de la mezcla }
    N             :integer;   { El nœmero de componentes }
    x,y           :FraccMolar;{ Fracciones molares en el l’quido y en el vapor }
    Bi,Ai,Ci      :Vector;    { Ai, Bi, Ci }
    Numi          :Vector;{ Para guardar par‡metros especiales de alguna ecuaci—n }
    Num2i         :Vector;{ Para los Patel-Teja; los Omega_Bi. Para SchmidtW Beta }
    ZZc           :Vector;    { Para Patel-Teja; los Zc }
    CambioEnToP   :Boolean;   { Para saber si se ha cambiado Presi—n o Temperatura }
    H2En,
    MetanoEn      :Integer;   { Si hay H2 o Metano en la Mezcla en que posici—n }
                              { de la mezcla, si son 0 entonces no hay }
    Kij           :MatrizConst;               { Parametros de interaccion binarios }
    ANor,                      { variables auxiliares para Chao_Seader }
    AH2,                       { variables auxiliares para Chao_Seader }
    AMe,                       { variables auxiliares para Chao_Seader }
    Aes           :array [0..9] of Extended;{ variables auxiliares para C_S }
    Aele          :array [10..14] of Extended;{ variables auxiliares para C_S }

PROCEDURE Componentes;
 Var i:integer;
 begin
  Cm:=Compuestos;
  N:=cuantos;
  for i:=1 to N do
   Cm[i].Pc:=Cm[i].Pc*100; { a kPa }
 end;

FUNCTION ObtenerNyComp;
 Var i:integer;
 begin
  ObtenerNyComp:=N;
  Compuestos:=Cm;
  for i:=1 to N do         { a Bar }
   Compuestos[i].Pc:=Cm[i].Pc/100;
 end;  

PROCEDURE PresionYTemp;
 begin
  Pres:=Presion*100;{ a [Kpa]}
  Temp:=Temperatura;
  CambioEnToP:=True;
 end;
 
PROCEDURE ObtenerPyT;
 begin
  P:=Pres/100; { a [bar] }
  T:=Temp;
 end;
 
PROCEDURE Concentraciones;
 begin
  x:=Xliq;
  y:=Yvap;
 end;
 
PROCEDURE ObtenerConcentraciones;
 begin
  Xliq:=X;
  Yvap:=Y;
 end;

FUNCTION AAi(i:integer;Modelo:integer):Extended;
 var Tres,Pr,m,Aux:extended;
 begin
  with Cm[i] do
   begin
    Tres:=Temp/Tc;
    Pr:=Pres/Pc;
    Case Modelo of
     VWAdachi:begin
               Numi[i]:=0.228165+0.791981*w+(-0.648552+0.654505*w)*sqr(w);
               Aux:=0.421875*XpwrY(10,Numi[i]*(1-Tres));
              end;
     RKS     :begin
               Numi[i]:=0.480+1.574*w-0.176*sqr(w);
               Aux:=0.42747*sqr(1+Numi[i]*(1-sqrt(Tres)));
              end;
     RKSPolar:begin
               Numi[i]:=1+(1-Tres)*(m+n/Tres);
               Aux:=0.42747*Numi[i];
              end;
     PengR   :begin
               Numi[i]:=0.37464+1.54226*w-0.26992*Sqr(w);
               Aux:=0.45727*sqr(1+Numi[i]*(1-sqrt(Tres)));
              end;
     SchmidtW:begin
               Num2i[i]:=0.25988221-0.02142913*w+0.00337143*sqr(w); { Num2i = Beta }
               Aux:=0.465+1.347*w-0.528*Sqr(w); { mo }
               If Tres<=1 then m:=Aux+Sqr(5*Tres-3*Aux-1)/70
               else m:=Aux+Sqr(4-3*Aux)/70;
               Aux:=XpwrI(1-(1-Num2i[i])/(3*(1+Num2i[i]*w)),3)
			            *Sqr(1+m*(1-sqrt(Tres)));
              end;
     PatelT..PatelTUSB:
              begin
               Numi[i]:=0.452413+1.30982*w-0.295937*sqr(w);
               ZZc[i]:=0.329032-0.076799*w+0.0211947*sqr(w);
               Num2i[i]:=0.08517138-0.02640592*w+0.00788769*sqr(w);
               FactorPT:=3*sqr(ZZc[i])+3*(1-2*ZZc[i])*Num2i[i]+
			            sqr(Num2i[i])+1-3*ZZc[i];
               aux:=sqr(1+Numi[i]*(1-sqrt(Tres)))*FactorPT;
              end;
     end; { Case ModeloÉ }
    AAi:=Aux*Pr/Sqr(Tres);
   end; { With Cm[i]É }
 end; { AiÉ }
 
FUNCTION BBi(i:integer;Modelo:integer):Extended;
 Var Aux:extended;
 begin
  Case Modelo of
   VWAdachi     : Aux:=1/8;
   RKS..RKSPolar: Aux:=0.08664;
   PengR        : Aux:=0.0778;
   SchmidtW     : Aux:=Num2i[i]/(3*(1+Num2i[i]*Cm[i].w));
   PatelT..PatelTUSB:Aux:=Num2i[i];
  end; { Case ModeloÉ}
  BBi:=Aux*Pres*Cm[i].Tc/Temp/Cm[i].Pc;
 end; { BiÉ }
 
FUNCTION CCi(i:integer;Modelo:integer):Extended;
 Var Tres,Pr:extended;
 begin
  Tres:=Temp/Cm[i].Tc;
  Pr:=Pres/Cm[i].Pc;  { Para SchmidtW se usa directamente que Cm[i].w = Ci[i] }
  Case Modelo of
   PatelT..PatelTUSB: CCi:=(1-3*ZZc[i])*Pr/Tres;
  end; { Case ModeloÉ }
 end; { CiÉ }
 
PROCEDURE CalcularA_B_CdeMezcla;
 Var xx:FraccMolar;i,j:integer;aux:extended;
 begin
  If Fase=Liquido then xx:=x
  else xx:=y;
  A:=0;
  B:=0;
  C:=0;
  E:=0;
  F:=0;
  for i:=1 to N do
   begin
    If CambioEnToP then
     begin
      Ai[i]:=AAi(i,Modelo);
      Bi[i]:=BBi(i,Modelo);
     end;
    B:=B+xx[i]*Bi[i];
   end;
  for i:=1 to N do
   for j:=1 to N do
    A:=A+xx[i]*xx[j]*(1-Kij[i,j])*sqrt(Ai[i]*Ai[j]);
  Case Modelo of 
   SchmidtW:begin
             for i:=1 to N do
              begin
               aux:=xx[i]*sqrt(Ai[i]);
               F:=F+aux*Cm[i].w; { Nota: en SchmidtW Ci[i]=Cm[i].w }
               E:=E+aux;         {  y se usa directamente Cm }
              end;
             C:=F/E;
            end;
    PatelT :begin
            for i:=1 to N do
             begin
              If CambioEnToP then
               Ci[i]:=CCi(i,Modelo); 
              C:=C+xx[i]*Ci[i];
             end;
            end;
    PatelTUSB
           :begin
             for i:=1 to N do
              begin
               If CambioEnToP then
                Ci[i]:=CCi(i,Modelo);
               aux:=xx[i]*sqrt(Bi[i]);
               E:=E+aux;
               C:=C+aux*Ci[i];
              end;
             C:=C/E;
            end;
   end; { Case modeloÉ }
  If CambioEnToP then CambioEnToP:=False;
 end; { Calcular A_B_CdeMezcla }
 
PROCEDURE ObtenerConstantes;
begin
   AA:=A;BB:=B;CC:=C;EE:=E;FF:=F;
   AAi:=Ai;BBi:=Bi;CCi:=Ci;
end;
 
PROCEDURE ObtenerKij;
begin
  KKij:=Kij;
end;

PROCEDURE IntroducirKij;
begin
  Kij:=KKij;
end;
 
FUNCTION FactorCompre;
 Var RaizGas,RaizLiquido,Medio:Extended;
 begin
  Case Modelo of
   GasIdeal     :begin 
                  RaizGas:=1;
                  RaizLiquido:=0;
                 end;
   VWAdachi     :Raices(RaizGas,Medio,RaizLiquido,-(B+1),A,-A*B);
   RKS..RKSPolar:Raices(RaizGas,Medio,RaizLiquido,-1,A-B-sqr(B),-A*B);
   PengR        :Raices(RaizGas,Medio,RaizLiquido,B-1,A-B*(3*B+2),B*(B+sqr(B)-A));
   SchmidtW     :Raices(RaizGas,Medio,RaizLiquido,3*C*B-1,A-(6*C+1)*Sqr(B)-(1+3*C)*B
                       ,3*C*(B*(Sqr(B)+B))-A*B);
   PatelT..PatelTUSB
                :Raices(RaizGas,Medio,RaizLiquido,C-1,A-C-B*(1+2*C+B)
                        ,B*(C*(1+B)-A));
   Otherwise    begin 
                 RaizGas:=NaN(255); 
                 RaizLiquido:=NaN(255); 
                end;
  end;
  If Fase=Gas then FactorCompre:=RaizGas
  Else
   if Not((ClassExtended(RaizLiquido)=NormalNum) or
          (ClassExtended(RaizLiquido)=ZeroNum)) then
    FactorCompre:=RaizGas
   else
    begin
     If (RaizLiquido<B) and (Modelo>GasIdeal) and (Modelo<=UltimoModeloGas) then 
      FactorCompre:=RaizGas
     else FactorCompre:=RaizLiquido;
    end;
 end;

FUNCTION CoFug_i;
 Var xx:FraccMolar; 
     Sumaj,Q,g,delt,ri,si,cul:Extended; { variables auxiliares }
     j:integer;
     II,uu,ww:extended;
     Tr,Pr :extended; { variables auxiliares para ChaoSeader }
    
 begin
  If Fase=Liquido then xx:=x
  else xx:=y;
  If Modelo=GasIdeal then CoFug_i:=1
  else
   begin
    if Modelo<=UltimoModeloGas then
     begin
      Sumaj:=0;
      for j:=1 to N do
       Sumaj:=Sumaj+xx[j]*(1-Kij[i,j])*sqrt(Ai[j]);
     end;
    Case Modelo of
     VWAdachi     :Q:=(Bi[i]/B-2*Sumaj*sqrt(Ai[i])/A)/Z;
     RKS..RKSPolar:Q:=(2*Sumaj*sqrt(Ai[i])/A-Bi[i]/B)*ln(Z/(Z+B))/B;
     PengR        :Q:=(2*Sumaj*Sqrt(Ai[i])/A-Bi[i]/B)
			*ln((Z-0.4142*B)/(Z+2.4142*B))/B/2.82842;
     SchmidtW     :begin
                    uu:=1+3*Cm[i].w;
                    ww:=-3*Cm[i].w;
                    delt:=1+18*C+9*sqr(C);
                    g:=Z*(Z+(1+3*C)*B)-3*C*Sqr(B);
                    cul:=3*C*sqrt(Ai[i])*(Cm[i].w/F-1/E);
                    ri:=cul+(1+3*C)*Bi[i]/B;
                    si:=-cul-6*C*Bi[i]/B;
                    Q:=(Bi[i]*Z*delt/B+(si*(1+3*C)+6*ri*C)
			*B-(ri*(1+3*C)-2*si)*Z)/g/delt;
                    cul:=2*Sumaj*Sqrt(Ai[i])*delt/A-ri*(1+3*C)+2*si;
                    if delt> 0.000 then
                     II:=ln((2*Z+B*(1+3*C-sqrt(delt)))/
				(2*Z+B*(1+3*C+sqrt(delt))))/sqrt(delt)/B
                    else
                     II:=2*(ArcTan((2*Z+uu*B)/(B*Sqrt(-delt)))
				-1.570796327)/(B*Sqrt(-delt));
                    cul:=cul*II/delt;
                    Q:=Q+cul;
                   end;
      PatelT..PatelTUSB:      
                   begin
                    uu:=1+C/B;
                    ww:=1-uu;
                    delt:=1+6*C/B+sqr(C/B);
                    g:=Z*(Z+B+C)-C*B;
                    if Modelo=PatelT then
                     ri:=1+C/B-Bi[i]*C/sqr(B)+Ci[i]/B
                    else { PatelTUSB }
                     ri:=1+2*C/B-Bi[i]*C/sqr(B)+sqrt(Bi[i])*(Ci[i]-C)/E/B;
                    si:=1-ri;
                    ri:=ri-(1+C/B)*(1-Bi[i]/B);
                    si:=si+C/B*(1-2*Bi[i]/B);
                    Q:=(Bi[i]*Z*Delt/B+si*(1+C/B)*B+2*ri*C-
				(ri*(1+C/B)-2*si)*Z)/g/delt;
                    cul:=2*Sumaj*Sqrt(Ai[i])*delt/A-ri*(1+C/B)+2*si;
                    if delt > 0.000 then
                      II:=ln((2*Z+B*(1+C/B-sqrt(delt)))/(2*Z+B*
				(1+C/B+sqrt(delt))))/sqrt(delt)/B
                    else
                      II:=2*(ArcTan((2*Z+uu*B)/(B*Sqrt(-delt)))
				-1.570796327)/(B*Sqrt(-delt));
                    cul:=cul*II/delt;
                    Q:=Q+Cul;
                   end;
       ChaoSeader:Begin
                  Tr:=Temp/Cm[i].Tc; Pr:=Pres/Cm[i].Pc;
                    if (i=H2En) then 
                     Aes:=AH2
                    else
                    if (i=MetanoEn) then
                      Aes:=AMe
                    else
                     Aes:=ANor;
                     cul:=Aes[0]+Aes[1]/Tr+Aes[2]*Tr+
				Aes[3]*Sqr(Tr)+Aes[4]*XpwrI(Tr,3);
                     cul:=cul+(Aes[5]+Aes[6]*Tr+Aes[7]*Sqr(Tr))
				*Pr+(Aes[8]+Aes[9]*Tr)*Sqr(Pr);
                     cul:=cul-ln(Pr)/2.302585093;
                     Q:=Aele[10]+Aele[11]*Tr+Aele[12]/Tr+Aele[13]
				*XpwrI(Tr,3)+Aele[14]*(Pr-0.6);
                     cul:=cul+Cm[i].w*Q;
                     CoFug_i:=XPwrY(10,cul);
                     Exit;
                  end;
      end; { Case ModeloÉ }
    CoFug_i:=exp((Z-1)*Bi[i]/B-ln(Z-B)+A*Q);
   end { else }
 end; { CoFug_i }
 
FUNCTION Aa_T(Modelo,Fase:integer;Z:extended):Extended; { Calcula A*a_T }
 var 
  xx                   :FraccMolar; 
  aux,sum1,sum2        :Extended; 
  i,j                  :integer;
 
  FUNCTIOn cdKij_dt(i,j:integer):extended;  { Calcula T/(1-Kij)*dKij/dt }
  begin
   cdKij_dt:=0; {Por ahora, falta la expresion}
  end;
  
  FUNCTIOn Sumando(m:Integer):Extended;
   Var Tred:Extended;
   begin
    Tred:=Temp/Cm[m].Tc;
    Case Modelo of
     VWAdachi:
      Sumando:=1.151292547*Tred*Numi[m];
     RKS:
      begin
       Tred:=Sqrt(Tred);
       Sumando:=Numi[m]*Tred/2/(1+Numi[m]*(1-Tred));  { mi*Tr^0.5/(2*alfa^0.5}
      end;
     RKSPolar:
      Sumando:=(Cm[m].m*Tred+Cm[m].n/Tred)/2/Numi[m]; { Numi = alfa }
     PengR:
      begin
       Tred:=Sqrt(Tred);
       Sumando:=Numi[m]*Tred/2/(1+Numi[m]*(1-Tred));  { Numi[i] = mi }
      end;
      
   { No es posible obtener una valida para Schmidt-Wenzel debido a la definicion }
   { de el factor 'm' en los art’culos investigados. El valor de a_T tiene una   }
   { discontinuidad pues m es continua pero de derivada no continua. }

     PatelT..PatelTUSB:
      begin
       Tred:=Sqrt(Tred);
       Sumando:=Numi[m]*Tred/(1+Numi[m]*(1-Tred))/2; 
      end;
     end; { CaseÉ }
   end;
   
 begin
  If Fase=Liquido then xx:=x
  else xx:=y;
  Aux:=0;
  for i:=1 to N do
   begin
    Sum1:=Sumando(i);
    for j:=1 to N do
     begin
      Sum2:=Sumando(j);
      Aux:=Aux+xx[i]*xx[j]*(1-Kij[i,j])*Sqrt(Ai[i]*Ai[j])*
		(-sum1-Sum2-cdKij_dt(i,j));
     end;
   end;
  Aa_T:=Aux;
 end;{ Aa_T }
  
FUNCTION Ental_Entro(CalculoDe:Boolean;Modelo,Fase:integer;Z:extended):Extended;
 { Calcula el factor AI(a_t-1) para entalp’a residual, — el AIa_T para entrop’a }
 { residual dependiendo de que se ordene en la variable CalculoDe }
 var aux,II:Extended;
 begin
  If CalculoDe=Entalpia then  Aux:=A
  Else { CalculoDe=Entropia } Aux:=0;
  Case Modelo of
   VWAdachi     : II:=-1/Z;
   RKS..RKSPolar: II:=-ln((Z+B)/Z)/B;
   PengR        : II:=-ln((Z+2.4142*B)/(Z-0.4142*B))/2.82842/B;
   SchmidtW     : aux:=NaN(255);
   PatelT..PatelTUSB:
     begin
       II:=C/B;                {en II c‡lculo de delta:= 1+6*C/B+ Sqr(C/B);}
       II:=Sqrt(1+(6+II)*II);  {en II c‡lculo de la raiz de I}
       II:=ln((2*Z+B*(1+C/B-II))/(2*Z+B*(1+C/B+II)))/(B*II);
     end;
   end; { Case ModeloÉ} 
   Case Modelo of
    GasIdeal: Ental_Entro:=0;
    SchmidtW: Ental_Entro:=NaN(255);
    otherwise Ental_Entro:=II*(Aa_T(Modelo,Fase,Z)-Aux);
   end; { Case ModeloÉ}
 end; { Ental_Entro }
 
FUNCTION EntalpiaRes;
 begin
  Case Modelo of
   GasIdeal: EntalpiaRes:=0;
   SchmidtW: EntalpiaRes:=NaN(255); { Problemas de discontinuidad. }
   otherwise EntalpiaRes:=R*Temp*(Z-1-Ental_Entro(Entalpia,Modelo,Fase,Z));
  end;
 end; { EntalpiaRes }
 
FUNCTION EntropiaRes;
 begin
  Case Modelo of
   GasIdeal: EntropiaRes:=0;
   SchmidtW: EntropiaRes:=NaN(255); { Problemas de discontinuidad. }
   otherwise EntropiaRes:=R*(-ln(Z/(Z-B))+ln(Z)-Ental_Entro(Entropia,Modelo,Fase,Z));
  end;
 end; { EntropiaRes }
   
PROCEDURE AsignarTyP_Ho_SoReferencia;
 begin
  Tref:=Treferencia;
  Pref:=Preferencia*100; { a kPa }
  So:=SoId;
  Ho:=HoId;
 end;
 
PROCEDURE ObtenerTyP_Ho_SoReferencia;
 begin
  TReferencia:=Tref;
  PReferencia:=Pref/100; { a bar }
  SoId:=So;
  HoId:=Ho;
 end;

FUNCTION EntalpiaIdeal; { kJ/kmol.K }
 var
  i,j:integer;
  Aux:extended; { almacena sumatoria temporal }
  AuxPotdeT:extended; { almacena temporalmente las potencias de T }
  xx:fraccmolar;
 begin
  if fase=Liquido then 
   xx:=X
  else
   xx:=Y;
  Aux:=0;
  for j:=1 to 5 do
   begin
    AuxPotdeT:=(XpwrI(Temp,j)-XpwrI(Tref,j))/j;
    for i:=1 to N do 
     Aux:=Aux+xx[i]*Cm[i].Cp[j-1]*AuxPotdeT;
   end;
  EntalpiaIdeal:=Aux+Ho;
 end; { Entalpia Ideal }
 
FUNCTION EntropiaIdeal;
 var
  i,j:integer;
  Aux:extended; { almacena sumatoria temporal }
  AuxPotdeT:extended; { almacena temporalmente las potencias de T }
  xx:fraccmolar;
 begin
  if fase=Liquido then 
   xx:=X
  else
   xx:=Y;
  Aux:=0;
  for j:=1 to 5 do
   begin
    If j=1 then
     AuxPotdeT:=ln(Temp/Tref)
    else
     AuxPotdeT:=(XpwrI(Temp,j-1)-XpwrI(Tref,j-1))/(j-1);
    for i:=1 to N do 
     Aux:=Aux+xx[i]*(Cm[i].Cp[j-1]*AuxPotdeT);
   end;
  for j:=1 to N do
   If Abs(xx[i])>1e-60 then { Para evitar ln(0), lim xln(x)=0 cuando x->0 }
    Aux:=Aux-R*xx[j]*ln(xx[j]); { Entrop’a de mezclado ideal }
  EntropiaIdeal:=Aux-R*ln(Pres/Pref)+So;
 end; { Entalpia Ideal }
 
FUNCTION DLnFugPuro_DT_i;
 Var AAux,BAux,CAux,PAux,Aux:Extended;
 begin
  PAux:=Pres;
  Pres:=Presion*100;
  Case Modelo of
   GasIdeal:begin
             DLnFugpuro_DT_i:=0;
             DLnFugpuro_DP_i:=1/Presion;
            end;
   VWAdachi..PatelTUSB:
            begin
             AAux:=A;
             BAux:=B;
             CAux:=C;
             A:=AAi(i,Modelo);   B:=BBi(i,Modelo);   C:=CCi(i,Modelo);
             If Modelo=SchmidtW then C:=Cm[i].w;
             Aux:=FactorCompre(Modelo,Gas);
             DLnFugpuro_DP_i:=Aux/Presion;
             DLnFugPuro_DT_i:=EntalpiaRes(Modelo,gas,Aux)/R/Sqr(Temp);
             A:=Aaux;
             B:=BAux;
             C:=CAux;
            end;
   ChaoSeader:with Cm[i] do
               begin
                AAux:=Temp/Tc;
                BAux:=Pres/Pc;
                if (i=H2En) then 
                 Aes:=AH2
                else
                 if (i=Metanoen) then
                  Aes:=AMe
                 else
                  Aes:=ANor;
                aux:=-Aes[1]/Sqr(AAux)+Aes[2]+AAux*(2*Aes[3]+3*Aes[4]*AAux);
                aux:=aux+BAux*(Aes[6]+2*Aes[7]*AAux)+Aes[9]*Sqr(BAux);
                aux:=aux+w*(Aele[11]-Aele[12]/Sqr(AAux)+3*Aele[13]*Sqr(AAux));
                DLnFugPuro_DT_i:=aux*2.30258/Tc;
                DLnFugPuro_DP_i:=NaN(255);
               end;
  end;
  Pres:=PAux;
 end;
 
PROCEDURE H2_Metano;
 begin
  H2En:=ElH2estaEn;
  MetanoEn:=ElMetanoestaEn;
 end;

PROCEDURE HacerCeroKij;
 Var ii,jj:Integer;
 begin
  for ii:=1 to NUmeroMaxComponentes do
   for jj:=1 to NUmeroMaxComponentes do
    Kij[ii,jj]:=0;
 end;
 
Begin
 CambioEnToP:=True;
 Tref:=273.15;  { K }
 PRef:=101.325; { kPa }
 Ho:=0;         { kJ/kmol }
 So:=0;         { kJ/kmol.K }
 H2En:=0;
 MetanoEn:=0;
 ANor[0]:=2.05135;
 ANor[1]:=-2.10899;
 ANor[2]:=0;
 ANor[3]:=-0.19396;
 ANor[4]:=0.02282;
 ANor[5]:=0.08852;
 ANor[6]:=0;
 ANor[7]:=-0.00872;
 ANor[8]:=-0.00353;
 ANor[9]:=0.00203;
 Aele[10]:=-4.23893;
 Aele[11]:=8.65808;
 Aele[12]:=-1.22060;
 Aele[13]:=-3.15224;
 Aele[14]:=-0.025;
 AH2[0]:=1.50709;
 AH2[1]:=2.74283;
 AH2[2]:=-0.02110;
 AH2[3]:=0.00011;
 AH2[4]:=0;
 AH2[5]:=0.008585;
 AH2[6]:=0;
 AH2[7]:=0;
 AH2[8]:=0;
 AH2[9]:=0;
 AMe[0]:=1.36822;
 AMe[1]:=-1.54831;
 AMe[2]:=0;
 AMe[3]:=0.02889;
 AMe[4]:=-0.01076;
 AMe[5]:=0.10486;
 AMe[6]:=-0.02529;
 AMe[7]:=0;
 AMe[8]:=0;
 AMe[9]:=0;
 HacerCeroKij;
End.
