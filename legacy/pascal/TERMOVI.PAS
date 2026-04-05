Unit TermoVI(10);
{$U-}
{$U TermoI}
{$U TermoII}
{$U TermoIII}
{$U TermoV}

{ Paquete termodinámico hecho por: }
{ Francisco Avelino Da Silva. }
{ Luis Alberto Báez Linde.    }
{ Caracas, 89 }

interface

Uses Sane,TermoI,TermoII,MemTypes,QuickDraw,TermoIII,OSIntf,ToolIntf,PasInOut,TermoV;

FUNCTION  KBinarioAureo(ElSistema:Propiedades;var LosDatos:Isotermas;Cuantas:integer;
                      ModeloGas,ModeloLiquido:integer; ConRangoDado:boolean;
                      Ko,Kf:extended;DarErrores:ProcPtr;
                      var KBin:extended):Error;
                      
implementation
                      
const

 noError = 0;
 pivotenulo = -1;
 linealdependiente = -2;

type
 vector = array [1..6] of extended;
 matriz = array [1..6,1..6] of extended;
 puntos = array [1..10] of extended;
 
var
 ErrorMatriz:integer;
 Evento     :EventRecord;
 Fin        :boolean;

PROCEDURE PasarDatos(Error,ErrorIni,ErrorMin:Extended;P:ProcPtr); Inline $205F, $4E90;

FUNCTION NumeroMalo(x:extended):boolean;
var
 tipo:NumClass;
begin
 tipo:=ClassExtended(x);
 if (tipo=NormalNum) or (tipo=ZeroNum) then
  NumeroMalo:=false
 else
  NumeroMalo:=true;
end;
     
FUNCTION SalirYa:boolean;
begin
  SalirYa:=false;
  while GetNextEvent(KeyDownMask,Evento) do
   with Evento do
    if (Chr(BitAnd(message,CharCodeMask)) = ',') and
       (BitAnd(modifiers,CmdKey)<>0) then
        begin
         SalirYa:=true;
         Fin:=true;
        end;
end;

 procedure MaximoPivote(var LaMatriz:matriz;i,dimension:integer);
  var
   j,sitio:integer;
   PivoteGrande,aux:extended;
  begin
   PivoteGrande:=abs(LaMatriz[i,i]);
   sitio:=i;
   for j:=i+1 to dimension do
    begin
      aux:=abs(LaMatriz[j,i]);
      if PivoteGrande < aux then
        begin
          PivoteGrande:=aux;
          sitio:=j;
        end;
    end;   
      if PivoteGrande<=1.0e-7 then 
        ErrorMatriz:=Pivotenulo
      else
       if sitio <> i then 
          for j:= 1 to dimension+1 do   {En lugar de "1" colocar "i" cuando}
            begin                       {se use Gauss OJO!!!, esto optimiza a Gauss}
             aux:=LaMatriz[i,j];
             LaMatriz[i,j]:=LaMatriz[sitio,j];
             LaMatriz[sitio,j]:=aux;
            end;
  end;

procedure Gauss(var LaMatriz:matriz;var ElVector:vector;dimension:integer;
                        Normalizado:boolean);
var
 i,k,j:integer;
 elMayor,m:Extended;
 
  procedurE Normalizacion;
  var
   aux:Extended;
   j:integer;
  begin
   elMayor:=abs(LaMatriz[i,i]);
   for j:=i+1 to dimension do
     begin
      aux:=abs(LaMatriz[i,j]);
       if elMayor < aux then
         elMayor:=aux;
     end;
    if elMayor <=1.0e-7 then
      ErrorMatriz:=LinealDependiente
    else
     for j:=1 to dimension + 1 do
       LaMatriz[i,j]:=LaMatriz[i,j]/elMayor;
  end;
  
  
  procedurE Regresion;
   var
    ii,jj:integer;suma:Extended;
   begin
    for ii:= dimension downto 1 do
     begin
      suma:=0;
      for jj:= ii+1 to dimension do 
       suma:=suma+LaMatriz[ii,jj]*LaMatriz[jj,dimension+1];
     if abs(LaMatriz[ii,ii])<=1.0e-7 then 
       ErrorMatriz:=pivotenulo;
      LaMatriz[ii,dimension+1]:=(LaMatriz[ii,dimension+1]-suma)/LaMatriz[ii,ii];
      ElVector[ii]:=LaMatriz[ii,dimension+1];
     end;
   end;
    
begin
 for i:= 1 to dimension do
   LaMatriz[i,dimension+1]:=ElVector[i];
 i:=0;ErrorMatriz:=NoError;
 repeat
   i:=i+1;
   if (ErrorMatriz=NoError) and (Normalizado) then
      Normalizacion;
   if ErrorMatriz=NoError then
      MaximoPivote(LaMatriz,i,dimension);
   if ErrorMatriz=NoError then
     for j:=i+1 to dimension do
      begin
       m:=LaMatriz[j,i]/LaMatriz[i,i];
       for k:= i+1 to dimension+1 do    {si se pone k:=i pone ceros }
         LaMatriz[j,k]:=LaMatriz[j,k]-m*LaMatriz[i,k];
      end;
 until (i=dimension-1) or (ErrorMatriz<>NoError);
    if ErrorMatriz=NoError then Regresion;
end;  
                        
FUNCTIOn HayKInicial(LasX,LasY:puntos;N,M:integer;var Ko:Extended):Boolean; {N es el orden del polinomio}
var                                                                       {M es el número de puntos}
 aux:extended;
 auxBool:boolean;
 LaMatriz:matriz;
 ElVector:vector;
 dimension:integer;
 i,j,k:integer;
 x1,x2,x3:extended;{las raíces}
 a,b,c:extended;   { coeficientes}
 F1,F2,F3,AuxF:record
           F:extended;
           cual:integer;
          end;
 
 FUNCTIOn Polin(x:extended):extended;
  var 
   aux:extended;
   i:integer;
 begin
  aux:=0;
  for i:=0 to N do
   aux:=aux+ElVector[i+1]*XpwrI(x,i);
  Polin:=aux;
 end;
 
BEGIN  
 for i:=0 to N do
  begin
   aux:=0;
   k:=0;
   repeat
      k:=k+1;
      aux:=aux+XpwrI(LasX[k],i);
   until k = M;
   for j:=1 to i+1 do
     LaMatriz[i+2-j,j]:=aux;
  end;
 for j:=2 to N+1 do
  begin
   aux:=0;
   k:=0;
   repeat
    k:=k+1;
    aux:=aux+XpwrI(LasX[k],N+j-1);
   until k=M;
    for i:=N+1 downto j do
     LaMatriz[i,j+N+1-i]:=aux;
  end;
  for i:=0 to N do
   begin
    aux:=0;
    k:=0;
    repeat
     k:=k+1;
     aux:=aux+LasY[k]*XpwrI(LasX[k],i);
    until k=M;
    ElVector[i+1]:=aux;
   end;
 Gauss(LaMatriz,ElVector,N+1,false);
 a:=Polin(-100);                                {evalúo los extremos que deben ser +}
 b:=Polin(100);
 if Not(NumeroMalo(a) or NumeroMalo(b)) then
  auxBool:=(a>=0) and (b>=0)
 else
  auxBool:=false;
 HayKInicial:=auxBool;
 if auxBool then
  begin
   a:=3*ElVector[4]/(4*ElVector[5]);
   b:=2*ElVector[3]/(4*ElVector[5]);
   c:=ElVector[2]/(4*ElVector[5]);
   Raices(X1,X2,X3,a,b,c); 
   if ClassExtended(X1) = NormalNum then
    begin
     F1.F:=Polin(X1);
     F1.Cual:=1;
    end
   else
     F1.Cual:=-1;
   if ClassExtended(X2) = NormalNum then
    begin
     F2.F:=Polin(X2);
     F2.Cual:=2;
    end
   else
     F2.Cual:=-1;
   if ClassExtended(X3) = NormalNum then
    begin
     F3.F:=Polin(X3);
     F3.Cual:=3;
    end
   else
     F3.Cual:=-1;
  AuxF:=F1;
  if (AuxF.cual<0) or ((AuxF.F>F2.F)and(F2.cual>0)) then
   AuxF:=F2;
  if (AuxF.cual<0) or ((AuxF.F>F3.F)and(F3.cual>0)) then
   AuxF:=F3;
   case AuxF.cual of
    1:Ko:=X1;
    2:Ko:=X2;
    3:Ko:=X3;
   end;
  end; { if AuxBool… }
END;

FUNCTION KBinarioAureo;
                      
 const L = 0.618033989;
 var
 i,j,iter:integer; 
 convergencia,EsK1:boolean;
 X,Y:fraccmolar;
 Kij:MatrizConst;
 ViejoKij:Extended;
 Fo,Ff,F1,F2,K1,K2:extended;
 Del:extended;
 Ambiente:Environment;
 ErrorMat:boolean;
 ErrorInicial:Extended;
 CambioExtDer,CambioExtIzq:boolean;  { vigilan que haya cambiado los extremos }

 
  PROCEDURe CargarXY (LaIsot,ElPunto:integer);
  begin
   with LosDatos do
   begin
    X[1]:=XD[LaIsot,1,ElPunto];
    X[2]:=1-X[1];
    Y[1]:=XD[LaIsot,2,ElPunto];
    Y[2]:=1-Y[1];
   end;
  end;
  
  FUNCTIOn Fs:extended;
  var
     isot,punto,i:integer; {isot:isoterma dada, punto: puntoo en isoterma}
     ZL,ZG:extended;
     CFL,CFG:array[1..2] of extended;
     Aux:extended;
  
  begin
   with LosDatos do
   begin
     Aux:=0;
        for isot:=1 to Cuantas do
         begin
         for punto:=1 to ND[isot] do
          begin
               if Iso = 'T' then
                PresionYTemp(PD[isot,punto],TD[isot])
               else
                PresionYTemp(TD[isot],PD[isot,punto]); {sino TD es presion}
               CargarXY(isot,punto);
               Concentraciones(X,Y);             
               CalcularA_B_CdeMezcla(ModeloLiquido,Liquido);
               ZL:=FactorCompre(ModeloLiquido,Liquido);
               for i:=1 to 2 do
                 CFL[i]:=CoFug_i(i,ModeloLiquido,Liquido,ZL);
               if (ModeloLiquido<>ModeloGas) then
                if Iso = 'T' then
                 PresionYTemp(PD[isot,punto],TD[isot])
                else
                 PresionYTemp(TD[isot],PD[isot,punto]);
               CalcularA_B_CdeMezcla(ModeloGas,Gas);
               ZG:=FactorCompre(ModeloGas,Gas);
               for i:=1 to 2 do
                 CFG[i]:=CoFug_i(i,ModeloGas,Gas,ZG);
               for i:=1 to 2 do
                 Aux:=Aux+Abs(X[i]*CFL[i]-Y[i]*CFG[i])/(Y[i]*CFG[i]);
            end; {for punto…}
          end; {for isot…}
    end;{ del with datos}
    Fs:=Aux;
  end;
  
  PROCEDURe PonerNuevosK(LaK:extended);
  begin
   Kij[1,1]:=0;
   Kij[1,2]:=LaK;Kij[2,1]:=LaK;
   Kij[2,2]:=0;
   IntroducirKij(Kij);
  end;
  
  PROCEDURe RevizarLosF;
  begin
   if F1 > F2 then
    begin
     EsK1:=false;
     Kf:=K1;
     Ff:=F1;
     CambioExtDer:=true;
    end
   else
    begin
     EsK1:=true;
     Ko:=K2;
     Fo:=F2;
     CambioExtIzq:=true;
    end;
  end;
  

FUNCTIOn RangodeCambio(X:extended;var Xo,Xf,Fo,Ff:extended):Integer;
  var
   F2,X2,a,b:extended;
   itera:integer;
   final:boolean;
   PendientePositiva:boolean;
   
   FUNCTIon Pendiente:Extended;
   begin
    Pendiente:=(F2-Ff)/(X2-X);
   end;
   
   PROCEDUre CalcularFs;
   begin
    PonerNuevosK(X);
    Ff:=Fs;
    X2:=X+0.000001;
    PonerNuevosK(X2);
    F2:=Fs;
   end;
   
   PROCEDUre CalcularB;
   begin
    b:=Ff-a*X;
   end;
   
   PROCEDUre CalcularNuevaX;
   var
    aux:extended;
   begin
    Xo:=X;
    Fo:=Ff;
    X:=-b/a;
    if Abs(X-Xo)>Abs(Xo)*0.5 then {*1.25 estaba antes pero creo que es mucho}
     if Abs(Xo)>TolFracc then
      X:=Xo+CopySign((X-Xo),Xo*0.5) {*1.25 estaba antes pero creo que es mucho el incremento}
     else
      X:=Xo+CopySign((X-Xo),0.01);  {Entra aquí cuando Xo=0}
   end;
   
  begin { de RangodeCambio… }
   Xo:=X;
   if SalirYa or Fin then Exit;
   CalcularFs;
   a:=Pendiente;
   if a>0 then PendientePositiva:=true
   else PendientePositiva:=false;
   itera:=0;
   final:=false;
   repeat
    if Not(Final) then
     begin
      itera:=itera+1;
      CalcularB;
      CalcularNuevaX;
      final:=(itera > NMaxItera);
      if Not(Final) then
       begin
        if SalirYa or Fin then Exit;
        CalcularFs;
        a:=Pendiente;
        final:=((a>0) and Not(PendientePositiva)) or ((a<0) and PendientePositiva);
       end;
     end;
   until final;
   Xf:=X;
   if Xo > Xf then
    begin
     X:=Xo;
     F2:=Fo;
     Xo:=Xf;
     Fo:=Ff;
     Xf:=X;
     Ff:=F2;
    end;
   if Itera > NMaxItera then RangodeCambio:=NoseEncontroRango
   else
    RangodeCambio:=0;
 end; { de RangodeCambio… }
 
 PROCEDURe BuscarRango;
  var
   LasX,LasY:puntos;
   i:integer;
   aux:extended;
   j:integer;
   Encontrado:boolean;
  begin
   Ko:=-0.4;
   for i:=1 to 9 do
    begin
     PonerNuevosK(Ko);
     LasX[i]:=Ko;
     if SalirYa or Fin then
      begin
       KBinarioAureo:=Abortado;
       Kij[1,2]:=ViejoKij;
       Kij[2,1]:=ViejoKij;
       IntroducirKij(Kij);
       Exit;
      end;
     LasY[i]:=fs;
     Ko:=Ko+0.1;
    end;
   i:=0;
   repeat
    Encontrado:=HayKInicial(LasX,LasY,4,9-i,Aux);
    i:=i+1;
    if  Not(Encontrado) then
     for j:=1 to (9-i) do
      begin
       LasX[j]:=LasX[j+1];
       LasY[j]:=LasY[j+1];
      end
   until Encontrado or (i=5);
   if Not(Encontrado) or NumeroMalo(Aux) then    { por dar algún comienzo en caso que falle la primera búsqueda }
    Aux:=0
   else
    if (Aux<-0.5) or (Aux>1) then Aux:=0;
   i:=RangodeCambio(aux,Ko,Kf,Fo,Ff);
   if i<>0 then
    begin
     KBinarioAureo:=i; { i = NoseEncontroRango }
     Fin:=true;
    end
   else
    if NumeroMalo(Ko) or NumeroMalo(Kf) then
     begin
      KBinarioAureo:=NoseEncontroRango;
      Fin:=true;
     end
    else
     if (Ko>Kf) or (Ko<-0.5) or (Kf>1) then {si se sale de rango entonces tomar el mayor}
    begin
     Ko:=-0.5;
     Kf:=1;
    end;
 end;
 
begin
 ObtenerKij(Kij);
 ViejoKij:=Kij[1,2];
 Componentes(Elsistema,2);
 Convergencia:=false;iter:=0;
 ErrorMat:=false;
 GetEnvironment(Ambiente);
 SetEnvironment(0);
 if DarErrores<>Nil then
  PasarDatos(-1,-1,TolFracc,DarErrores);
 Fin:=false;
 if SalirYa or fin then
  begin
   KBinarioAureo:=Abortado;
   SetEnvironment(Ambiente);
   Exit;
  end;
 CambioExtDer:=false;
 CambioExtIzq:=false;
 if Not(ConRangoDado) then
  begin
   CambioExtDer:=false;
   CambioExtIzq:=false;
   BuscarRango;
   if Fin then
    begin
     SetEnvironment(Ambiente); {ya KBinarioAureo vale NoseEncontroRango}
     Kij[1,2]:=ViejoKij;
     Kij[2,1]:=ViejoKij;
     IntroducirKij(Kij);
     Exit;
    end;
  end;
 if SalirYa or Fin then
  begin
   KBinarioAureo:=Abortado;
   SetEnvironment(Ambiente);
   Kij[1,2]:=ViejoKij;
   Kij[2,1]:=ViejoKij;
   IntroducirKij(Kij);
   Exit;
  end;
 del:=Kf-Ko;
 ErrorMat:=NumeroMalo(del);
 K1:=Ko+del*L;
 K2:=Kf-del*L;
 if SalirYa or Fin then
  begin
   KBinarioAureo:=Abortado;
   SetEnvironment(Ambiente);
   Kij[1,2]:=ViejoKij;
   Kij[2,1]:=ViejoKij;
   IntroducirKij(Kij);
   Exit;
  end;
 PonerNuevosK(K1);
 F1:=Fs;
 if SalirYa or Fin then
  begin
   KBinarioAureo:=Abortado;
   SetEnvironment(Ambiente);
   Kij[1,2]:=ViejoKij;
   Kij[2,1]:=ViejoKij;
   IntroducirKij(Kij);
   Exit;
  end;
 PonerNuevosK(K2);
 F2:=Fs;
 ErrorInicial:=abs(del);
 if Not(ErrorMat) then
  repeat
    iter:=iter+1;
    if SalirYa or Fin then
     begin
      KBinarioAureo:=Abortado;
      SetEnvironment(Ambiente);
      Kij[1,2]:=ViejoKij;
      Kij[2,1]:=ViejoKij;
      IntroducirKij(Kij);
      Exit;
     end;
    RevizarLosF;
    ErrorMat:=NumeroMalo(Abs(Kf-Ko));
    if Not(ErrorMat) then
     begin
      if DarErrores<>Nil then
       PasarDatos(Abs(Kf-Ko),ErrorInicial,TolFracc,DarErrores);
      if (Abs(Kf-Ko)<TolFracc) then Convergencia:=true
      else
       begin 
        del:=Kf-Ko;
        if EsK1 then
         begin
          K2:=K1;
          F2:=F1;
          K1:=Ko+del*L;
          PonerNuevosK(K1);
          F1:=Fs;
         end
        else
         begin
          K1:=K2;
          F1:=F2;
          K2:=Kf-del*L;
          PonerNuevosK(K2);
          F2:=Fs;
         end;
       end; {if (Abs(K… }
    end; { If Not(Error… }
  until Convergencia or (iter>NMaxItera) or ErrorMat;
  if EsK1 then { le damos el valor a KBin aunque sea malo }
   KBin:=K2
  else
   KBin:=K1;
 Kij[1,2]:=ViejoKij;
 Kij[2,1]:=ViejoKij;
 IntroducirKij(Kij); { Guardando el valor viejo en TermoII }
 if Convergencia and CambioExtDer and CambioExtIzq then KBinarioAureo:=SinError
 else
  if ErrorMat then KBinarioAureo:=ErrorMatematico
  else 
   if (iter>NMaxItera) then KBinarioAureo:=NoConverge
   else 
    if Not(CambioExtIzq) and ConRangoDado then KBinarioAureo:= ExtremoIzquierdo
    else 
     if Not(CambioExtDer) and ConRangoDado then KBinarioAureo:= ExtremoDerecho
     else 
      if Not(ConRangoDado) and (Not(CambioExtIzq) or Not(CambioExtDer)) 
      then KBinarioAureo:=NoseEncontroRango; { Si entra por aquí es que encontró un rango pero no un mínimo }
SetEnvironment(Ambiente);   
end;                                   

BEGIN
 Fin:=false;
END.