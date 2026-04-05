Unit TermoI(4); 
{$U-}

{ Paquete termodinámico hecho por: }
{ Francisco Avelino Da Silva. }
{ Luis Alberto Báez Linde.    }
{ Caracas, 89 }

Interface

Uses Sane;

Const R=8.3144;{ kJ/kmol.K }

      NumeroMaxComponentes=10;

    {Para la selección del Modelo}
    SolucionIdeal = 1; { Solo liquido }
    GasIdeal      = 1;
    VWAdachi      = 2;
    RKS           = 3;
    RKSPolar      = 4;
    PengR         = 5;
    SchmidtW      = 6;
    PatelT        = 7;
    PatelTUSB     = 8;
    ScatchardH    = 9; { Solo líquido }
    Margules      = 10;{ Solo líquido }
    VanLaar       = 11;{ Solo líquido }
    Wilson        = 12;{ Solo líquido }
    
    ChaoSeader    = 20;{ NOTA: COMPLEMENTARIO para modelos de liquido, en especial }
                       { con Scatchard-Hildebrandt o soluciones regulares }
                       { ! NUNCA USARLO COMO MODELO DE LIQUIDO SOLO ¡ }
  
    Gas           = 0;
    Liquido       = 1;
    UltimoModeloGas  = PatelTUSB; { Se necesita conocer la última ec. cúbica }
 
    Abortado         = -1;
    SinError         =  0;    { OK }
    NoConverge       =  1;    { pasa el numero de interaciones permitidas el Flash}
    NoConvergeCV     =  2;    { pasa el numero de interaciones permitidas el Cv   }
    FueraDeRangoCV   =  3;    { CV>1 ó CV<0 }
    NoHayDosFases    =  4;    { Obvio }
    NoConvergeK_XY   =  5;
    SolucionTrivial  =  6;
    ErrorMatematico  =  7;    { Operación Matemática no realizable como 0/0 ó Ln(-1) }
    ExtremoDerecho   =  8;    { Converge en ese extremo, para calculos KBinarioAureo() }
    ExtremoIzquierdo =  9;    { Converge en ese extremo, para calculos KBinarioAureo() }
    NoseEncontroRango= 10;    { Error de inicialización que puede suceder en KBinarioAureo() }

   type
   
   TipoPresVapor = (Antoine,Riedel,RPM,Muller);
  
   Registro = record       { De las propiedades de cada compuesto }
    Tc,Pc,Zc:real;         { Propiedades Críticas }
    a1,a2,a3:real;         { Antoine, ln(P/Pc)=a1-a2/(a3+T)  }
    w,delta,vl:Real;       { F. Acéntrico,Calor de solubilidad [(Cal/cm^3)^0.5], Vol líquido [cm^3/gmol] }
    m,n:real;              { Para la ecuacion RKS polar }
    momentoDip:real;       { [Debyes] }
    cp:array[0..4] of real;{ Cp en [kJ/kmol.K] Cp:=cp[0]+cp[1]*T+cp[2]*T^2+cp[3]*T^3+cp[4]*T^4 }
    Pm,Tb:real;            { Peso molecular, Temperatura de ebullición normal }
   end;
   
   Propiedades = array [1..NumeroMaxComponentes] of registro;
   
   FraccMolar  = array [1..NumeroMaxComponentes] of extended;
   
   MatrizConst = array [1..NumeroMaxComponentes,1..NumeroMaxComponentes] of extended;

   Error       =Integer;    { Los errores ocurridos en algún cálculo }

PROCEDURE Raices(var r1,r2,r3:extended;a,b,c:extended); 
 { Para ecuaciones cubicas del tipo x^3 + ax^2 + bx + c = 0 }
 
FUNCTION PrVapor(Sust:Registro;T:extended;Tipo:TipoPresVapor):Extended;

FUNCTION DPrVapor_DT(Sust:Registro;T:extended;Tipo:TipoPresVapor):Extended;

FUNCTION TEbullicion(Sust:Registro;Presion:Extended;Tipo:TipoPresVapor):Extended; 
                          
FUNCTION CorrectPoynting(Presion,PresionSat,Temperatura,Vliquid:Extended):Extended;
    
{ NOTA: Todas las temperaturas en [K], presiones en [bar], Energías en [kJ/kmol], Entropías [kJ/kmol.K] }
    
Implementation

FUNCTION ArcCos(x:extended):extended;
 begin
  if abs(x)<1e-1000 then ArcCos:=Pi/2  { Esto es suficiente preciso }
  else 
   if x>0 then ArcCos:=ArcTan(Sqrt(1-Sqr(x))/x)
   else ArcCos:=ArcTan(Sqrt(1-Sqr(x))/x)+Pi;
 end;

PROCEDURE Raices;
   var
    M,N,Det,Teta:extended;

 PROCEDURe IntercambioMayor(var x1,x2:extended); { Se quiere X1 como el mayor}
  begin
   if x2>x1 then 
    begin 
     Det:=x1;      { En este caso se usa Det como una variable auxiliar }
     x1:=x2;
     x2:=Det;
    end;
  end;
  
 begin
  M:=b-Sqr(a)/3; 
  N:=c-a*b/3+2*XpwrI(a,3)/27;
  Det:=Sqr(N)/4+XpwrI(M,3)/27;
  if Det>0 then
   begin
    Teta:=-N/2+Sqrt(Det);         { Teta usada como una variable auxiliar}
    r1:=CopySign(Teta,1)*XpwrY(abs(Teta),1/3);
    Teta:=-N/2-Sqrt(Det);
    r1:=r1+CopySign(Teta,1)*XpwrY(abs(Teta),1/3)-a/3;
    r2:=NaN(001);  r3:=NaN(001);  { Son numeros imaginarios, NaN=Not a Number}
   end
  else
   if Det=0 then
    begin
     r1:=-CopySign(N,1)*2*Sqrt(-M/3)-a/3;
     r2:=CopySign(N,1)*Sqrt(-M/3)-a/3;  
     r3:=r2;
     if r2>r1 then
      begin
       r3:=r1;
       r1:=r2;
      end;
    end
   else
    begin 
     Teta:=ArcCos(-N/2*Sqrt(-27/XpwrI(M,3)));
     Det:=2*Sqrt(-M/3);
     r1:= Det*Cos(Teta/3)-a/3;
     r2:=-Det*Cos((Pi-Teta)/3)-a/3;
     r3:=-Det*Cos((Pi+Teta)/3)-a/3;
     IntercambioMayor(r1,r2);
     IntercambioMayor(r2,r3);
     IntercambioMayor(r1,r2);  { Así r1>r2>r3 }
    end;
 end; { Raices }

FUNCTION CorrectPoynting;
 begin                 
  CorrectPoynting:=exp((Presion-PresionSat)*Vliquid/R/Temperatura/10);
 end;
  
FUNCTION PrVapor;
 var Aux,Tr,Aux1:Extended;
 begin
  With Sust do
   begin
   Case Tipo Of
    Antoine:Aux:=a1-a2/(a3+T);
    Riedel :begin
             Tr:=Tb/Tc;
             Aux:=-35+36/Tr+42*ln(Tr)-XpwrI(Tr,6);
             Aux:=(0.315*Aux+ln(Pc/1.0135))/(0.0838*Aux-ln(Tr));
             Aux1:=0.0838*(3.758-Aux);
             Tr:=T/Tc;
             Aux:=(-35+36/Tr)*Aux1+(42*Aux1+Aux)*ln(Tr)-Aux1*XpwrI(Tr,6);
            end;
    RPM    :begin
             Tr:=Tb/Tc;
             Aux:=ln(Pc/1.0135)*Tr/(1-Tr);
             Aux1:=0.4835+0.4605*Aux;
             Aux:=(Aux/Aux1-(1+Tr))/((3+Tr)*sqr(1-Tr));
             Tr:=T/Tc;
             Aux:=-Aux1*(Aux*(3+Tr)*XpwrI(1-Tr,3)-sqr(Tr)+1)/Tr;
            end;
    Muller :begin
             Tr:=Tb/Tc;
             Aux:=5.37273*(1+w);
             Aux1:=0.0134-ln(Pc)-Aux*(1-1/Tr);
             Aux1:=Aux1/(ln(Tr)-0.832223*(1-1/Tr));
             Aux:=Aux-0.832223*Aux1;
             Tr:=T/Tc;
             Aux:=Aux*(1-1/Tr)+Aux1*ln(Tr);
            end;
    end; { Case… }
   PrVapor:=Exp(Aux)*Pc;
   end; { With… }
 end; { PrVapor }

PROCEDURE PseudoAntoine(Sust:Registro;Trefe,Rango:extended;Tipo:TipoPresVapor;
                        var Auno,Ados,Atres:Real);  { da las constantes de Antoine }
 Var                                                    { equivalentes }
  T1,T3,X1,X2,X3,Teta,Gama:extended;
 begin
  T1:=Trefe-Rango;
  T3:=Trefe+Rango;
  X1:=ln(PrVapor(Sust,T1,Tipo)/Sust.Pc);
  X2:=ln(PrVapor(Sust,Trefe,Tipo)/Sust.Pc);
  X3:=ln(PrVapor(Sust,T3,Tipo)/Sust.Pc);
  Teta:=(T1-Trefe)/(Trefe-T3);
  Gama:=(X1-X2)/(X2-X3);
  Atres:=-(Teta*T3-Gama*T1)/(Teta-Gama);
  Ados:=(X2-X3)/(1/(T3+Atres)-1/(Trefe+Atres));
  Auno:=X2+Ados/(Trefe+Atres);
 end; {PseudoAntoine}

FUNCTION Tebullicion;
 Var Tem     :Extended;
     Pres    :Extended;
     Rango   :Extended;
     Auno,Ados,Atres:Real;
     Incremento:Extended;
 begin
  with Sust do
   begin
    Case Tipo of
     Antoine: Tem:=a2/(a1-ln(Presion/Pc))-a3;
     Riedel..Muller:
      begin
       PseudoAntoine(Sust,273,100,Tipo,AUno,ADos,ATres);
       Tem:=ADos/(AUno-ln(Presion/Pc))-ATres;
       Rango:=3;
       Repeat
        PseudoAntoine(Sust,Tem,Rango,Tipo,AUno,ADos,ATres);
        Incremento:=ADos/(AUno-ln(Presion/Pc))-ATres-Tem;
        Tem:=Tem+Incremento;
        Rango:=Rango/5;
       Until (Abs(Incremento)<0.01); { Precisión = 0.01 [K] }
      end;
     end;
   TEbullicion:=Tem;
  end; {del with}
 end;

FUNCTION DPrVapor_DT; {(Sust:Registro;T:extended;Tipo:TipoPresVapor):Extended;}
 var
  aux1,aux2,h:extended;
begin
 with Sust do
  Case Tipo of
    Antoine:aux1:=PrVapor(Sust,T,Tipo)*a2/sqr(T+a3);
    Otherwise
     begin
      aux1:=PrVapor(Sust,T-0.0001,Tipo);
      aux1:=(PrVapor(Sust,T+0.0001,Tipo)-aux1)/0.0002;
     end;
  end;
 DPrVapor_DT:=aux1;
end;
           
Begin
End.