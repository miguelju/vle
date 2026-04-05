VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsQbicsPure"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
'Enumeración que indica las ecuciones de estado cúbicas diponibles
    Public Enum TADiPEDC
        PR1976_Qbic = 0
        RK1949_Qbic = 1
        RKS1972_Qbic = 2
        VdW1870_Qbic = 3
        PRL1997_Qbic = 4
        RKSL1997_Qbic = 5
        RKSGD1978_Qbic = 6
        RP1978_Qbic = 7
        Berth1899_Qbic = 8
        VdWAda1984_Qbic = 9
        VdWVald1989_Qbic = 10
        RKSmn1980_Qbic = 11
        RKSATmn1995_Qbic = 12
        PRATmng1997_Qbic = 13
        PRMmn1989_Qbic = 14
        PRSV1986_Qbic = 15
        VdWOL1998_Qbic = 16
        RKOL1998_Qbic = 17
        PROL1998_Qbic = 18
    End Enum
'valores de los parametros que describen las ecuaciones cubicas
    Private gintK1 As Integer, gIntK2 As Integer, gintK3 As Integer
    Private gdblOmA As Double, gdblOmB As Double, gdblOmC As Double
    Private gdblH(0 To 4) As Double
'estructura que contiene los valores de los coeficientes del desarrollo
'cubico en volumen
    Private mvarCubicCoef1 As Double, mvarCubicCoef2 As Double, mvarCubicCoef3 As Double
    Private Type ParametersEOS
        A As Double
        B As Double
        C As Double
    End Type
    Private mParameter As ParametersEOS
    Private mvarZc As Double
    Private mvarm As Double
    Private mvarn As Double
    Private mvarg As Double
    Private mvarZRA As Double
    Private Quality As Double
    Private mvarPRSVK1 As Double
'    Private SatPressureCalc As New clsSatPressure
    Private mvarPsatAtTr As Double
    Private mvarPc As Double
    Private mvarTc As Double
    Private mvarQbicsWinEQofState As TADiPEDC
    Private mvarAcentricFactor As Double
    Private mvarLiquidFlow As clsFlow
    Private mvarVaporFlow As clsFlow
    Private mvarFeedFlow As clsFlow
    Private mvarReferenceState As New clsReferencesSt
    Private mvarSistem As clsAllProps
Public Property Get m() As Double
On Error GoTo m_ErrorHandler

' ---- Code Body Starts ---- (Default)
    m = mvarm
' ---- Code Body Ends ---- (Default)
Exit Property

m_ErrorHandler:
  Err.Raise Err.Number

Exit_m:

End Property
Public Property Let m(ByVal vData As Double)
On Error GoTo m_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarm = m
' ---- Code Body Ends ---- (Default)
Exit Property

m_ErrorHandler:
  Err.Raise Err.Number

Exit_m:

End Property
Public Property Get g() As Double
On Error GoTo g_ErrorHandler

' ---- Code Body Starts ---- (Default)
    g = mvarg
' ---- Code Body Ends ---- (Default)
Exit Property

g_ErrorHandler:
    Err.Raise Err.Number

Exit_g:

End Property
Public Property Let g(ByVal vData As Double)
On Error GoTo g_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarg = g
' ---- Code Body Ends ---- (Default)
Exit Property

g_ErrorHandler:
  Err.Raise Err.Number

Exit_g:

End Property
Public Property Get n() As Double
On Error GoTo n_ErrorHandler

' ---- Code Body Starts ---- (Default)
    n = mvarn
' ---- Code Body Ends ---- (Default)
Exit Property

n_ErrorHandler:
  Err.Raise Err.Number

Exit_n:

End Property
Public Property Let n(ByVal vData As Double)
On Error GoTo n_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarn = n
' ---- Code Body Ends ---- (Default)
Exit Property

n_ErrorHandler:
  Err.Raise Err.Number

Exit_n:

End Property
Public Function PrXGivenTrvr(ReducedTemperature As Double, ReducedVolume As Double, ByRef ReducedPressure As Double, ByRef Quality As Double) As Boolean
On Error GoTo PrXGivenTrvr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim comp As Double
    Dim compone As Double
    Dim vrf As Double, vrg As Double, Pr As Double
    On Error GoTo ErrProcedure
    
    If ReducedTemperature = 1 Then
        PrXGivenTrvr = False
        Exit Function
    End If
    
    If ReducedTemperature > 1 Then
        ReducedPressure = Prvr(ReducedTemperature, ReducedVolume, Alpha(ReducedTemperature))
        If ReducedPressure < 1 Then
            PrXGivenTrvr = True
            Quality = -1
        End If
        If ReducedPressure > 1 Then
            PrXGivenTrvr = True
            Quality = -1
        End If
        Exit Function
    End If
    If ReducedTemperature < 1 Then
        MaxwellTestGivenTr ReducedTemperature, Pr, vrg, vrf
        Quality = (ReducedVolume - vrf) / (vrg - vrf)
        comp = Abs(1 - Quality)
        compone = Abs(Quality)
        If 0.000000001 > comp Then
            ReducedPressure = Pr
            Quality = 1
            PrXGivenTrvr = True
            Exit Function
        End If
        If 0.000000001 >= compone Then
            ReducedPressure = Pr
            Quality = 0
            PrXGivenTrvr = True
            Exit Function
        End If
        Select Case ReducedVolume
            Case Is > vrg
                 ReducedPressure = Prvr(ReducedTemperature, ReducedVolume, 1)
                 Quality = -1
            Case Is < vrf
                 ReducedPressure = Prvr(ReducedTemperature, ReducedVolume, 1)
                 Quality = -1
            Case Else
                ReducedPressure = Pr
                Quality = (ReducedVolume - vrf) / (vrg - vrf)
        End Select
    End If
ErrProcedure:
    PrXGivenTrvr = False
' ---- Code Body Ends ---- (Default)
Exit Function

PrXGivenTrvr_ErrorHandler:
  Err.Raise Err.Number

Exit_PrXGivenTrvr:

End Function
Private Function Prvr(Tr As Double, vr As Double, Alphac As Double) As Double
On Error GoTo Prvr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Aux As Double, Aux1 As Double
    Aux = (vr - gdblOmB)
    Aux1 = (vr * vr + gintK1 * gdblOmB * vr + gIntK2 * gdblOmB * gdblOmB)
    Prvr = (Tr / Aux) - (gdblOmA * Alphac * (vr - gintK3 * gdblOmB) / Aux1)
' ---- Code Body Ends ---- (Default)
Exit Function

Prvr_ErrorHandler:
  Err.Raise Err.Number

Exit_Prvr:

End Function
Public Function FugacityCoeficient(ReducedTemperature As Double, ReducedPressure As Double, CompresibilityFactor As Double, ByRef FugacityCoef As Double) As Boolean
On Error GoTo FugacityCoeficient_ErrorHandler

' ---- Code Body Starts ---- (Default)
    On Error GoTo ErrProcedure
    Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
    FugacityCoef = Exp(Fugacity(CompresibilityFactor))
    FugacityCoeficient = True
    Exit Function
ErrProcedure:
    FugacityCoeficient = False
' ---- Code Body Ends ---- (Default)
Exit Function

FugacityCoeficient_ErrorHandler:
  Err.Raise Err.Number

Exit_FugacityCoeficient:

End Function
Public Property Let Model(ByVal vData As TADiPEDC)
On Error GoTo Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarQbicsWinEQofState = vData
    GeneralConstantsEOS mvarQbicsWinEQofState, gintK1, gIntK2, gintK3, gdblOmA, gdblOmB, gdblOmC, gdblH()
' ---- Code Body Ends ---- (Default)
Exit Property

Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Model:

End Property
Private Function SumHk(ReducedTemperature As Double) As Double
On Error GoTo SumHk_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ReDim h(1 To 10) As Double
    Dim E As Double, k As Integer, SUM As Double, SatP
    Select Case mvarQbicsWinEQofState
        Case VdWOL1998_Qbic
           h(1) = 0.33333333
           h(2) = 0.35112597
           h(3) = 0.011287433
           h(4) = -0.0038485685
           h(5) = 0.00064261934
           h(6) = -0.000067252383
           h(7) = 0.0000045962725
           h(8) = -0.00000019990875
           h(9) = 5.0318465E-09
           h(10) = -5.5827084E-11
           E = -1.02

        Case RKOL1998_Qbic
            h(1) = 0.32748
            h(2) = 0.34376954
            h(3) = 0.010596403
            h(4) = -0.0037538497
            h(5) = 0.00063257197
            h(6) = -0.000066481
            h(7) = 0.0000045517956
            h(8) = -0.00000019796921
            h(9) = 4.9748592E-09
            h(10) = -5.5024228E-11
            E = -1.014
            
        Case PROL1998_Qbic
            h(1) = 0.29803582
            h(2) = 0.015003698
            h(3) = -0.0047527103
            h(4) = 0.0008036716
            h(5) = -0.000089548695
            h(6) = 0.0000068691611
            h(7) = -0.00000036067317
            h(8) = 0.000000012409205
            h(9) = -2.5222671E-10
            h(10) = 2.2955503E-12
            E = -0.00041
        End Select
        SUM = 0
        For k = 1 To 10
            SUM = SUM + h(k) * (-Log(mvarPsatAtTr / ReducedTemperature)) ^ k
        Next
        SumHk = ((1 - Log(mvarPsatAtTr / ReducedTemperature)) ^ E) * SUM
' ---- Code Body Ends ---- (Default)
Exit Function

SumHk_ErrorHandler:
  Err.Raise Err.Number

Exit_SumHk:

End Function
Public Function ResidualEnthalpy(ReducedTemperature As Double, ReducedPressure As Double, CompresibilityFactor As Double, ByRef HR As Double) As Boolean
On Error GoTo ResidualEnthalpy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim d_a As Double, Iz As Double, T As Double, a1 As Double, a2 As Double
    Dim a3 As Double, a4 As Double, a5 As Double
    Const h = 0.0001
    On Error GoTo ErrProcedure
    T = ReducedTemperature * mvarTc
    Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
    Iz = I_z(CompresibilityFactor, gintK1, gIntK2, mParameter.B) '* mParameter.B
    a1 = (gdblOmA * Alpha(ReducedTemperature) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a2 = (gdblOmA * Alpha((T - 2 * h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a3 = (gdblOmA * Alpha((T - h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a4 = (gdblOmA * Alpha((T + h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a5 = (gdblOmA * Alpha((T + 2 * h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    d_a = (-1) * T * (a2 - 8 * a3 + 8 * a4 - a5) / (12 * h * a1)
    HR = CompresibilityFactor - 1 + mParameter.A * Iz * (d_a + 1)
    ResidualEnthalpy = True
    Exit Function
ErrProcedure:
    ResidualEnthalpy = False
' ---- Code Body Ends ---- (Default)
Exit Function

ResidualEnthalpy_ErrorHandler:
  Err.Raise Err.Number

Exit_ResidualEnthalpy:

End Function
Public Function ResidualEntropy(ReducedTemperature As Double, ReducedPressure As Double, CompresibilityFactor As Double, ByRef SR As Double) As Boolean
On Error GoTo ResidualEntropy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Z As Double, a1 As Double, a2 As Double, a3 As Double, a4 As Double, a5 As Double
    Dim d_a As Double, Iz As Double, T As Double
    Const h = 0.0001
    On Error GoTo ErrProcedure
    T = ReducedTemperature * mvarTc
    Z = CompresibilityFactor
    Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
    Iz = I_z(CompresibilityFactor, gintK1, gIntK2, mParameter.B)
    a1 = (gdblOmA * Alpha(ReducedTemperature) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a2 = (gdblOmA * Alpha((T - 2 * h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a3 = (gdblOmA * Alpha((T - h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a4 = (gdblOmA * Alpha((T + h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    a5 = (gdblOmA * Alpha((T + 2 * h) / mvarTc) * mvarTc * mvarTc * UnivGasConst * UnivGasConst) / mvarPc
    d_a = (-1) * T * (a2 - 8 * a3 + 8 * a4 - a5) / (12 * h * a1)
    SR = Log(Z - mParameter.B) + mParameter.A * Iz * d_a
    ResidualEntropy = True
    Exit Function
ErrProcedure:
    ResidualEntropy = False
' ---- Code Body Ends ---- (Default)
Exit Function

ResidualEntropy_ErrorHandler:
  Err.Raise Err.Number

Exit_ResidualEntropy:

End Function
Public Function TrXGivenPrZ(ReducedPressure As Double, CompresibilityFactor As Double, ByRef ReducedTemperature As Double, ByRef Quality As Double) As Boolean
On Error GoTo TrXGivenPrZ_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim comp As Double
    On Error GoTo ErrProcedure
    Dim compone As Double
    Dim Zf As Double, Zg As Double, Tr As Double
    
    If ReducedPressure = 1 Then
        TrXGivenPrZ = False
        Exit Function
    End If
    
    If ReducedPressure > 1 Then
        ReducedTemperature = T_despejada(ReducedPressure, CompresibilityFactor, 1)
        Quality = -1
        TrXGivenPrZ = True
        Exit Function
    End If
    If ReducedPressure < 1 Then
        MaxwellTestGivenPr ReducedPressure, Tr, Zg, Zf
        Quality = (CompresibilityFactor - Zf) / (Zg - Zf)
        comp = Abs(1 - Quality)
        compone = Abs(Quality)
        If 0.000000001 > comp Then
            Quality = 1
            ReducedTemperature = Tr
            TrXGivenPrZ = True
            Exit Function
        End If
        If 0.000000001 > compone Then
            Quality = 0
            ReducedTemperature = Tr
            TrXGivenPrZ = True
            Exit Function
        End If
        Select Case CompresibilityFactor
            Case Is > Zg
                ReducedTemperature = T_despejada(ReducedPressure, CompresibilityFactor, 1)
                Quality = -1
            Case Is < Zf
                ReducedTemperature = T_despejada(ReducedPressure, CompresibilityFactor, 1)
                Quality = -1
            Case Else
                ReducedTemperature = Tr
                Quality = (CompresibilityFactor - Zf) / (Zg - Zf)
          End Select
    End If
    TrXGivenPrZ = True
    Exit Function
ErrProcedure:
    TrXGivenPrZ = False
' ---- Code Body Ends ---- (Default)
Exit Function

TrXGivenPrZ_ErrorHandler:
  Err.Raise Err.Number

Exit_TrXGivenPrZ:

End Function
Public Property Get Model() As TADiPEDC
On Error GoTo Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Model = mvarQbicsWinEQofState
' ---- Code Body Ends ---- (Default)
Exit Property

Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Model:

End Property
Public Function TrZGivenPrX(ReducedPressure As Double, Quality As Double, ByRef ReducedTemperature As Double, ByRef CompresibilityFactor As Double) As Boolean
On Error GoTo TrZGivenPrX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double
    Dim Zf As Double, Zg As Double, Tr As Double
    On Error GoTo ErrProcedure
    MaxwellTestGivenPr ReducedPressure, Tr, Zg, Zf
    Res = (1 - Quality) * Zf + Quality * Zg
    ReducedTemperature = Tr
    CompresibilityFactor = Res
    TrZGivenPrX = True
    Exit Function
ErrProcedure:
    TrZGivenPrX = False
' ---- Code Body Ends ---- (Default)
Exit Function

TrZGivenPrX_ErrorHandler:
  Err.Raise Err.Number

Exit_TrZGivenPrX:

End Function
Public Function TrvrGivenPrX(ReducedPressure As Double, Quality As Double, ByRef ReducedTemperature As Double, ByRef ReducedVolume As Double) As Boolean
On Error GoTo TrvrGivenPrX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double
    Dim vrf As Double, vrg As Double, Tr As Double
    On Error GoTo ErrProcedure
    MaxwellTestGivenPr_Vr ReducedPressure, Tr, vrg, vrf
    Res = (1 - Quality) * vrf + Quality * vrg
    ReducedTemperature = Tr
    ReducedVolume = Res
    TrvrGivenPrX = True
    Exit Function
ErrProcedure:
    TrvrGivenPrX = False
' ---- Code Body Ends ---- (Default)
Exit Function

TrvrGivenPrX_ErrorHandler:
  Err.Raise Err.Number

Exit_TrvrGivenPrX:

End Function
Public Function TrPrGivenvrX(RduceVolume As Double, ByVal Quality As Double, ByRef ReducedTemperature As Double, ByRef ReducedPressure As Double) As Boolean
On Error GoTo TrPrGivenvrX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim resul As Double
    Dim Pr As Double
    On Error GoTo ErrProcedure
    Quality = Quality
    resul = Regula_Falsi_M(RduceVolume, 0.4, 0.9994, 0.00000000001, 4)
    ReducedTemperature = resul
    ReducedPressure = Pr
    TrPrGivenvrX = True
    Exit Function
ErrProcedure:
    TrPrGivenvrX = False
' ---- Code Body Ends ---- (Default)
Exit Function

TrPrGivenvrX_ErrorHandler:
  Err.Raise Err.Number

Exit_TrPrGivenvrX:

End Function
Public Function PrZGivenTrX(ReducedTemperature As Double, Quality As Double, ByRef ReducedPressure As Double, ByRef CompresibilityFactor As Double) As Boolean
On Error GoTo PrZGivenTrX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double
    Dim Zf As Double, Zg As Double, Pr As Double
    On Error GoTo ErrProcedure
    MaxwellTestGivenTr ReducedTemperature, Pr, Zg, Zf
    Res = (1 - Quality) * Zf + Quality * Zg
    CompresibilityFactor = Res
    ReducedPressure = Pr
    PrZGivenTrX = True
    Exit Function
ErrProcedure:
    PrZGivenTrX = False
' ---- Code Body Ends ---- (Default)
Exit Function

PrZGivenTrX_ErrorHandler:
  Err.Raise Err.Number

Exit_PrZGivenTrX:

End Function
Public Function PrvrGivenTrX(ReducedTemperature As Double, Quality As Double, ByRef ReducedPressure As Double, ByRef ReducedVolume As Double) As Boolean
On Error GoTo PrvrGivenTrX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double
    Dim vrf As Double, vrg As Double, Pr As Double
    On Error GoTo ErrProcedure
    MaxwellTestGivenTr_vr ReducedTemperature, Pr, vrg, vrf
    Res = (1 - Quality) * vrf + Quality * vrg
    ReducedVolume = Res
    ReducedPressure = Pr
    PrvrGivenTrX = True
    Exit Function
ErrProcedure:
    PrvrGivenTrX = False
' ---- Code Body Ends ---- (Default)
Exit Function

PrvrGivenTrX_ErrorHandler:
  Err.Raise Err.Number

Exit_PrvrGivenTrX:

End Function
Public Function MaxwellTestGivenPr(ReducedPressure As Double, ByRef ReducedTemperature As Double, ByRef VaporCompresibilityFactor As Double, ByRef LiquidCompresibilityFactor As Double) As Boolean
On Error GoTo MaxwellTestGivenPr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Zf As Double, Zg As Double, Tr As Double
    Dim lnPr As Double, firstAprox As Double
    On Error GoTo ErrProcedure
    lnPr = Log(ReducedPressure)
    Tr = (1 / (Regula_Falsi_M(lnPr, 1 / 0.43, 1 / 0.9994, 0.00000000001, 1)))
    Coef_Cubicas ReducedPressure, Tr, Alpha(Tr)
    CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, Zg, firstAprox, Zf
    ReducedTemperature = Tr
    VaporCompresibilityFactor = Zg
    LiquidCompresibilityFactor = Zf
    MaxwellTestGivenPr = True
    Exit Function
ErrProcedure:
    MaxwellTestGivenPr = False
' ---- Code Body Ends ---- (Default)
Exit Function

MaxwellTestGivenPr_ErrorHandler:
  Err.Raise Err.Number

Exit_MaxwellTestGivenPr:

End Function
Friend Function SolveCubic(Tr As Double, Pr As Double, ByRef Zg As Double, ByRef Zf As Double) As Boolean
On Error GoTo SolveCubic_ErrorHandler
Dim Dummy As Double
Coef_Cubicas Pr, Tr, Alpha(Tr)
CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
Zg, Dummy, Zf
ErrProcedure:
    SolveCubic = False
' ---- Code Body Ends ---- (Default)
Exit Function

SolveCubic_ErrorHandler:
  Err.Raise Err.Number

Exit_SolveCubic:

End Function
Public Function MaxwellTestGivenTr(ReducedTemperature As Double, ByRef ReducedPressure As Double, ByRef VaporCompresibilityFactor As Double, ByRef LiquidCompresibilityFactor As Double) As Boolean
On Error GoTo MaxwellTestGivenTr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Alfa As Double, ans As Integer
    Dim gdblLn_fug As Double, firstAprox As Double
    Dim Pr As Double, prsat1 As Double, prsat2 As Double, vl As Double
    On Error GoTo ErrProcedure
    prsat2 = Prsat(ReducedTemperature)
    Alfa = Alpha(ReducedTemperature)
    Do
        prsat1 = prsat2
        Coef_Cubicas prsat1, ReducedTemperature, Alfa
        ans = CubicSolver(mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        VaporCompresibilityFactor, firstAprox, LiquidCompresibilityFactor)
        If ans = 2 Then
            gdblLn_fug = Fugacity(VaporCompresibilityFactor) - _
            Fugacity(LiquidCompresibilityFactor)
            prsat2 = prsat1 * (1 - (gdblLn_fug) / (VaporCompresibilityFactor - _
            LiquidCompresibilityFactor))
        Else
            MsgBox "No se pudieron encontrar raices reales suficientes para la resolucion del problema", 0 + 48, "Alerta"
            Exit Function
        End If
    Loop Until Abs((prsat2 - prsat1) / prsat2) < 0.000001
    Pr = prsat2
    ReducedPressure = Pr
    MaxwellTestGivenTr = True
    Exit Function
ErrProcedure:
    MaxwellTestGivenTr = False
' ---- Code Body Ends ---- (Default)
Exit Function

MaxwellTestGivenTr_ErrorHandler:
  Err.Raise Err.Number

Exit_MaxwellTestGivenTr:

End Function
Private Function Maxwell_RM(ByVal alphat As Double, ByVal vg As Double, ByVal vf As Double) As Double
On Error GoTo Maxwell_RM_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim raizpos As Double
    Select Case mvarQbicsWinEQofState
        Case VdW1870_Qbic, Berth1899_Qbic, VdWAda1984_Qbic, VdWOL1998_Qbic
           Maxwell_RM = (Log((vg - gdblOmB) / (vf - gdblOmB)) + alphat * gdblOmA * (1 / vg - 1 / vf)) / (vg - vf)
        Case Else
           raizpos = Sqr(gintK1 * gintK1 - 4 * gIntK2)
           Maxwell_RM = (Log((vg - gdblOmB) / (vf - gdblOmB)) - alphat * gdblOmA / gdblOmB / raizpos * (Log((vg - 0.5 * gdblOmB * (-gintK1 + raizpos)) / (vf - 0.5 * gdblOmB * (-gintK1 + raizpos))) - Log((vg - 0.5 * gdblOmB * (-gintK1 - raizpos)) / (vf - 0.5 * gdblOmB * (-gintK1 - raizpos))))) / (vg - vf)
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

Maxwell_RM_ErrorHandler:
  Err.Raise Err.Number

Exit_Maxwell_RM:

End Function
Public Function MaxwellTestGivenTr_vr(ReducedTemperature As Double, ByRef ReducedPressure As Double, ByRef vrg As Double, ByRef vrf As Double) As Boolean
On Error GoTo MaxwellTestGivenTr_vr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    Dim P1 As Double, P0 As Double, nan As Double
    Dim ans As Integer, fP0 As Double, Errore As Double
    Dim alphat As Double
    Const itermax = 1000
    Const epsilon = 0.000000000001
    On Error GoTo ErrProcedure
    alphat = Alpha(ReducedTemperature)
    alphat = alphat / ReducedTemperature
    Select Case mvarQbicsWinEQofState
        Case VdW1870_Qbic, Berth1899_Qbic, VdWAda1984_Qbic, VdWOL1998_Qbic
            P0 = VDW_lnPrTr_RM(alphat)
        Case RKS1972_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, RK1949_Qbic, VdWVald1989_Qbic, RKSATmn1995_Qbic, RKOL1998_Qbic, RKSmn1980_Qbic
            P0 = RK_lnPrTr_RM(alphat)
        Case PR1976_Qbic, RP1978_Qbic, PRL1997_Qbic, PRATmng1997_Qbic, PRSV1986_Qbic, PROL1998_Qbic, PRMmn1989_Qbic
            P0 = PR_lnPrTr_RM(alphat)
    End Select
    i = 0
    Do
        i = i + 1
        P1 = P0
        Coef_Cubicasvr P0, alphat
        ans = CubicSolverNR(mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, nan, vrf)
        If ans = 3 Then
            fP0 = Maxwell_RM(alphat, vrg, vrf)
            P0 = fP0
            Errore = Abs(P1 - fP0) / P1
        Else
            MsgBox "No se pudieron encontrar raices reales suficientes para la resolucion del problema", 0 + 48, "Alerta"
            MaxwellTestGivenTr_vr = False
            Exit Function
        End If
    Loop Until Errore < epsilon Or i > itermax
    If i > itermax Then
        MaxwellTestGivenTr_vr = False
    End If
    ReducedPressure = P0 * ReducedTemperature
    MaxwellTestGivenTr_vr = True
ErrProcedure:
    MaxwellTestGivenTr_vr = False
' ---- Code Body Ends ---- (Default)
Exit Function

MaxwellTestGivenTr_vr_ErrorHandler:
  Err.Raise Err.Number

Exit_MaxwellTestGivenTr_vr:

End Function
Private Function VDW_lnPrTr_RM(alphat As Double) As Double
On Error GoTo VDW_lnPrTr_RM_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim X As Double, rx As Double
    X = alphat - 1
    rx = 1 / alphat / alphat
    VDW_lnPrTr_RM = Exp(rx * (-3 * X - 5.699423028 * X ^ 2 - 2.7368021445 * X ^ 3 - 0.10569091076 * X ^ 4 - 0.043694260628 * X ^ 5 + 0.034523665621 * X ^ 6 - 0.010375591483 * X ^ 7 + 0.0016957716778 * X ^ 8 - 0.0001487944021 * X ^ 9 + 5.5123166929E-06 * X ^ 10))
' ---- Code Body Ends ---- (Default)
Exit Function

VDW_lnPrTr_RM_ErrorHandler:
  Err.Raise Err.Number

Exit_VDW_lnPrTr_RM:

End Function
Private Function RK_lnPrTr_RM(alphat As Double) As Double
On Error GoTo RK_lnPrTr_RM_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim X As Double, rx As Double
    X = alphat - 1
    rx = 1 / alphat / alphat
    RK_lnPrTr_RM = Exp(rx * (-3.05362157 * X - 5.7734658603 * X ^ 2 - 2.7742499752 * X ^ 3 - 0.1114471922 * X ^ 4 - 0.043826462377 * X ^ 5 + 0.035920648696 * X ^ 6 - 0.010992288022 * X ^ 7 + 0.0018230077639 * X ^ 8 - 0.00016204060211 * X ^ 9 + 0.000006074119332 * X ^ 10))
' ---- Code Body Ends ---- (Default)
Exit Function

RK_lnPrTr_RM_ErrorHandler:
  Err.Raise Err.Number

Exit_RK_lnPrTr_RM:

End Function
Private Function PR_lnPrTr_RM(alphat As Double) As Double
On Error GoTo PR_lnPrTr_RM_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim X As Double, rx As Double
    X = alphat - 1
    rx = 1 / alphat / alphat
    PR_lnPrTr_RM = Exp(rx * (-3.355301397 * X - 6.1492081935 * X ^ 2 - 3.0095877677 * X ^ 3 - 0.11849721416 * X ^ 4 - 0.058686979142 * X ^ 5 + 0.049811255181 * X ^ 6 - 0.016237323922 * X ^ 7 + 0.0028648545767 * X ^ 8 - 0.00026978074936 * X ^ 9 + 0.000010663890906 * X ^ 10))
' ---- Code Body Ends ---- (Default)
Exit Function

PR_lnPrTr_RM_ErrorHandler:
  Err.Raise Err.Number

Exit_PR_lnPrTr_RM:

End Function
Public Function PrXGivenTrZ(ReducedTemperature As Double, CompresibilityFactor As Double, ByRef ReducedPressure As Double, ByRef Quality As Double) As Boolean
On Error GoTo PrXGivenTrZ_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim comp As Double
    Dim compone As Double
    Dim Zf As Double, Zg As Double, Pr As Double
    On Error GoTo ErrProcedure
        If ReducedTemperature = 1 Then
        PrXGivenTrZ = False
        Exit Function
    End If
    If ReducedTemperature > 1 Then
        ReducedPressure = P_despejada(ReducedTemperature, CompresibilityFactor, 1)
        If ReducedPressure < 1 Then
            PrXGivenTrZ = True
            Quality = -1
        End If
        If ReducedPressure > 1 Then
            PrXGivenTrZ = True
            Quality = -1
        End If
        Exit Function
    End If
    
    If ReducedTemperature < 1 Then
        MaxwellTestGivenTr ReducedTemperature, Pr, Zg, Zf
        Quality = (CompresibilityFactor - Zf) / (Zg - Zf)
        comp = Abs(1 - Quality)
        compone = Abs(Quality)
        If 0.000000001 > comp Then
            ReducedPressure = Pr
            Quality = 1
            PrXGivenTrZ = True
            Exit Function
        End If
        If 0.000000001 >= compone Then
            ReducedPressure = Pr
            Quality = 0
            PrXGivenTrZ = True
            Exit Function
        End If
        Select Case CompresibilityFactor
            Case Is > Zg
                 ReducedPressure = P_despejada(ReducedTemperature, CompresibilityFactor, 1)
                 Quality = -1
            Case Is < Zf
                 ReducedPressure = P_despejada(ReducedTemperature, CompresibilityFactor, 1)
                 Quality = -1
            Case Else
                ReducedPressure = Pr
                Quality = (CompresibilityFactor - Zf) / (Zg - Zf)
        End Select
    End If
    PrXGivenTrZ = True
    Exit Function
ErrProcedure:
    PrXGivenTrZ = False
' ---- Code Body Ends ---- (Default)
Exit Function

PrXGivenTrZ_ErrorHandler:
  Err.Raise Err.Number

Exit_PrXGivenTrZ:

End Function
Public Function TrXGivenPrVr(ReducedPressure As Double, ReducedVolume As Double, ByRef ReducedTemperature As Double, ByRef Quality As Double) As Boolean
On Error GoTo TrXGivenPrVr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim comp As Double
On Error GoTo ErrProcedure
    Dim compone As Double
    Dim vrf As Double, vrg As Double, Tr As Double
    
    If ReducedPressure = 1 Then
        TrXGivenPrVr = False
        Exit Function
    End If
    
    If ReducedPressure > 1 Then
        ReducedTemperature = T_despejada(ReducedPressure, ReducedVolume, 1)
        Quality = -1
        TrXGivenPrVr = True
        Exit Function
    End If
    
    If ReducedPressure < 1 Then
        MaxwellTestGivenPr_Vr ReducedPressure, Tr, vrg, vrf
        Quality = (ReducedVolume - vrf) / (vrg - vrf)
        comp = Abs(1 - Quality)
        compone = Abs(Quality)
        If 0.000000001 > comp Then
            Quality = 1
            ReducedTemperature = Tr
            TrXGivenPrVr = True
            Exit Function
        End If
        If 0.000000001 > compone Then
            Quality = 0
            ReducedTemperature = Tr
            TrXGivenPrVr = True
            Exit Function
        End If
        Select Case ReducedVolume
            Case Is > vrg
                ReducedTemperature = Tr_despejada(ReducedPressure, ReducedVolume, 1)
                Quality = -1
            Case Is < vrf
                ReducedTemperature = Tr_despejada(ReducedPressure, ReducedVolume, 1)
                Quality = -1
            Case Else
                ReducedTemperature = Tr
                Quality = (ReducedVolume - vrf) / (vrg - vrf)
          End Select
    End If
    
ErrProcedure:
    TrXGivenPrVr = False
' ---- Code Body Ends ---- (Default)
Exit Function

TrXGivenPrVr_ErrorHandler:
  Err.Raise Err.Number

Exit_TrXGivenPrVr:

End Function
Public Function ZGivenTrPr(ReducedTemperature As Double, ReducedPressure As Double, ByRef CompresibilityFactor As Double) As Boolean
On Error GoTo ZGivenTrPr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double, comp As Double, firstAprox As Double
    Dim Zf As Double, Zg As Double, Pr As Double

On Error GoTo ErrProcedure
    
    If ReducedTemperature = 1 And ReducedPressure > 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zg
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature = 1 And ReducedPressure < 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zf
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature > 1 And ReducedPressure = 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zf
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature < 1 And ReducedPressure = 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zg
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature > 1 And ReducedPressure > 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zg
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature > 1 And ReducedPressure < 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zg
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature < 1 And ReducedPressure > 1 Then
        Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
        CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        Zg, firstAprox, Zf
        Res = Zf
        CompresibilityFactor = Res
        ZGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature < 1 And ReducedPressure < 1 Then
        MaxwellTestGivenTr ReducedTemperature, Pr, Zg, Zf
        comp = (Abs(ReducedPressure - Pr)) / Pr
        If comp < 0.001 Then
            MsgBox "Estado indeterminado. Debe especificar dos propiedades independientes.(Saturación Líquido-Vapor)"
            ZGivenTrPr = False
            Exit Function
        Else
            Select Case ReducedPressure
                Case Is > Pr
                    Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
                    CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
                    Zg, firstAprox, Zf
                    Res = Zf
                Case Is < Pr
                    Coef_Cubicas ReducedPressure, ReducedTemperature, Alpha(ReducedTemperature)
                    CubicSolver mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
                    Zg, firstAprox, Zf
                    Res = Zg
            End Select
            CompresibilityFactor = Res
            ZGivenTrPr = True
        End If
    End If
    Exit Function
ErrProcedure:
    ZGivenTrPr = False
' ---- Code Body Ends ---- (Default)
Exit Function

ZGivenTrPr_ErrorHandler:
  Err.Raise Err.Number

Exit_ZGivenTrPr:

End Function
Public Function vrGivenTrPr(ReducedTemperature As Double, ReducedPressure As Double, ByRef ReducedVolume As Double) As Boolean
On Error GoTo vrGivenTrPr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double, comp As Double, firstAprox As Double
    Dim vrf As Double, vrg As Double, Pr As Double
    If ReducedTemperature = 1 And ReducedPressure > 1 Then
        Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrg
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature = 1 And ReducedPressure < 1 Then
        Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrf
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature > 1 And ReducedPressure = 1 Then
Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrf
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature < 1 And ReducedPressure = 1 Then
Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrg
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature > 1 And ReducedPressure > 1 Then
        Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrg
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature > 1 And ReducedPressure < 1 Then
Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrg
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
    
    If ReducedTemperature < 1 And ReducedPressure > 1 Then
        Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
        CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
        Res = vrf
        ReducedVolume = Res
        vrGivenTrPr = True
        Exit Function
    End If
      If ReducedTemperature < 1 And ReducedPressure < 1 Then
        MaxwellTestGivenTr_vr ReducedTemperature, Pr, vrg, vrf
        comp = (Abs(ReducedPressure - Pr)) / Pr
        If comp < 0.001 Then
            MsgBox "Estado indeterminado. Debe especificar dos propiedades independientes.(Saturación Líquido-Vapor)"
            vrGivenTrPr = False
            Exit Function
        Else
            Select Case ReducedPressure
                Case Is > Pr
                    Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
                    CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
                    Res = vrg
                Case Is < Pr
                    Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
                    CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
                    Res = vrg
            End Select
            ReducedVolume = Res
            vrGivenTrPr = True
        End If
    End If
    Exit Function
ErrProcedure:
    vrGivenTrPr = False
    
' ---- Code Body Ends ---- (Default)
Exit Function

vrGivenTrPr_ErrorHandler:
  Err.Raise Err.Number

Exit_vrGivenTrPr:

End Function
Public Function MaxwellTestGivenPr_Vr(ReducedPressure As Double, ByRef ReducedTemperature As Double, ByRef vrg As Double, ByRef vrf As Double) As Boolean
On Error GoTo MaxwellTestGivenPr_Vr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tr As Double, nan As Double
    Dim lnPr As Double
    On Error GoTo ErrProcedure
    lnPr = Log(ReducedPressure)
    Tr = (1 / (Regula_Falsi_M(lnPr, 1 / 0.33, 1 / 0.999994, 0.00000000001, 3)))
    ReducedTemperature = Tr
    CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, nan, vrf
    MaxwellTestGivenPr_Vr = True
    Exit Function
ErrProcedure:
    MaxwellTestGivenPr_Vr = False
' ---- Code Body Ends ---- (Default)
Exit Function

MaxwellTestGivenPr_Vr_ErrorHandler:
  Err.Raise Err.Number

Exit_MaxwellTestGivenPr_Vr:

End Function
Public Function TrPrGivenZX(CompresibilityFactor As Double, Quality As Double, ByRef ReducedTemperature As Double, ByRef ReducedPressure As Double) As Boolean
On Error GoTo TrPrGivenZX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim resul As Double
    Dim Zg As Double, Zf As Double, Pr As Double
    On Error GoTo ErrProcedure
    MaxwellTestGivenTr 0.4, Pr, Zg, Zf
    
    If CompresibilityFactor > Zg Then
        MsgBox "Estado no puede ser definido Trsat<0.4"
        TrPrGivenZX = False
        Exit Function
    End If
    
    If CompresibilityFactor < Zf Then
        MsgBox "Estado no puede ser definido Trsat<0.4"
        TrPrGivenZX = False
        Exit Function
    End If
    Quality = Quality
    resul = Regula_Falsi_M(CompresibilityFactor, 0.4, 0.9994, 0.00000000001, 2)
    ReducedTemperature = resul
    ReducedPressure = Pr
    TrPrGivenZX = True
    Exit Function
ErrProcedure:
    TrPrGivenZX = False
' ---- Code Body Ends ---- (Default)
Exit Function

TrPrGivenZX_ErrorHandler:
  Err.Raise Err.Number

Exit_TrPrGivenZX:

End Function
Private Function Regula_Falsi_M(Result As Double, ByVal x1 As Double, ByVal x2 As Double, Precision As Double, i As Integer)
On Error GoTo Regula_Falsi_M_ErrorHandler

' ---- Code Body Starts ---- (Default)
'Este algoritmo fue donado generosamente por el Prof.Jean-Marie Ledanois USB
'   Result: Valor a obtener de la función
'   x1: Punto de partida para las iteraciones
'   x2: Punto de partida para las iteraciones
'   Precision: valor del error
    Dim y1 As Double
    Dim y2 As Double
    Dim y3 As Double
    Dim x3 As Double
    Dim Xx As Double
    Dim Flag As Integer
    Dim Twice1 As Integer
    Dim Twice2 As Integer
    Dim Count As Integer
    y1 = Funcion(x1, i) - Result
    y2 = Funcion(x2, i) - Result
    If y1 * y2 > 0 Then
        x3 = x1 - y1 * (x2 - x1) / (y2 - y1)
        y3 = Funcion(x3, i) - Result
        If y1 * y3 > 0 And y1 * y2 > 0 Then
            Regula_Falsi_M = 0
            Exit Function
        End If
        If y1 * y3 < 0 Then
            x1 = x3
            y1 = y3
        Else
            x1 = x2
            y1 = y2
        End If
    End If
    Do While Flag = False
        x3 = x1 - y1 * (x2 - x1) / (y2 - y1)
        y3 = Funcion(x3, i) - Result
        If y3 * y1 > 0 Then
            Twice2 = Twice2 + 1
            y1 = y3
            x1 = x3
        Else
            Twice1 = Twice1 + 1
            y2 = y3
            x2 = x3
        End If
        If Twice1 = 2 Then
            Twice1 = 0
            y1 = y1 / 2
        End If
        If Twice2 = 2 Then
            Twice2 = 0
            y2 = y2 / 2
        End If
        Count = Count + 1
        Xx = ZZZ_Greater_Abs(x3, (x1 + x2) / 2)
        If Xx = 0 Then
            Flag = True
        ElseIf (Abs(x3 - ((x1 + x2) / 2)) / Xx) < Precision Then
            Flag = True
        End If
        If Count > 100 Then Flag = True
    Loop
    Regula_Falsi_M = x3
' ---- Code Body Ends ---- (Default)
Exit Function

Regula_Falsi_M_ErrorHandler:
  Err.Raise Err.Number

Exit_Regula_Falsi_M:

End Function
Private Function ZZZ_Greater_Abs(x1 As Double, x2 As Double) As Double
On Error GoTo ZZZ_Greater_Abs_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If Abs(x1) > Abs(x2) Then
        ZZZ_Greater_Abs = x1
    Else
        ZZZ_Greater_Abs = x2
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

ZZZ_Greater_Abs_ErrorHandler:
  Err.Raise Err.Number

Exit_ZZZ_Greater_Abs:

End Function
Private Function Funcion(X As Double, i As Integer) As Double
On Error GoTo Funcion_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Res As Double
    Dim Zg As Double, Zf As Double, Pr As Double
    Dim vrg As Double, vrf As Double
    Select Case i
        Case 1
            MaxwellTestGivenTr (1 / (X)), Pr, Zg, Zf
            Funcion = Log(Pr)
        Case 2
            MaxwellTestGivenTr X, Pr, Zg, Zf
            Res = (1 - Quality) * Zf + Quality * Zg
            Funcion = Res
        Case 3
            MaxwellTestGivenTr_vr (1 / (X)), Pr, vrg, vrf
            Funcion = Log(Pr)
        Case 4
            MaxwellTestGivenTr_vr X, Pr, vrg, vrf
            Res = (1 - Quality) * vrf + Quality * vrg
            Funcion = Res
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

Funcion_ErrorHandler:
  Err.Raise Err.Number

Exit_Funcion:

End Function
Private Function T_despejada(ReducedPressure As Double, CompresibilityFactor As Double, initTemperature As Double)
On Error GoTo T_despejada_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim T1 As Double, T2 As Double
    Dim f As Double, Fh As Double, h As Double
    h = 0.00001
    T2 = initTemperature
    Do 'newton raphson
        T1 = T2
        f = F_de_PyT(T1, CompresibilityFactor, ReducedPressure)
        Fh = (F_de_PyT(T1 + h, CompresibilityFactor, ReducedPressure) - f) / h
        T2 = T1 - f / Fh
    Loop Until Abs(T2 - T1) < 0.0000000001
    T_despejada = T2
' ---- Code Body Ends ---- (Default)
Exit Function

T_despejada_ErrorHandler:
  Err.Raise Err.Number

Exit_T_despejada:

End Function
Private Function Tr_despejada(ReducedPressure As Double, ReducedVolume As Double, IniT As Double)
On Error GoTo Tr_despejada_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim T1 As Double, T2 As Double
    Dim f As Double, Fh As Double, h As Double
    Dim Temp As Double
    h = 0.00001
    T2 = IniT
    Do 'newton raphson
        T1 = T2
        f = F_de_PryTrVr(T1, ReducedVolume, ReducedPressure)
        Temp = T1 + h
        h = Temp - T1
        Fh = (F_de_PryTrVr(Temp, ReducedVolume, ReducedPressure) - f) / h
        T2 = T1 - f / Fh
    Loop Until Abs(T2 - T1) < 0.0000000001
    Tr_despejada = T2
' ---- Code Body Ends ---- (Default)
Exit Function

Tr_despejada_ErrorHandler:
  Err.Raise Err.Number

Exit_Tr_despejada:

End Function
Private Function F_de_PyT(ReducedTemperature As Double, CompresibilityFactor As Double, Pressure As Double) As Double
On Error GoTo F_de_PyT_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim A As Double, B As Double, CubicCoef1 As Double
    Dim CubicCoef2 As Double, CubicCoef3 As Double
    A = gdblOmA * Alpha(ReducedTemperature) * (Pressure / (ReducedTemperature) ^ 2)
    B = gdblOmB * (Pressure / (ReducedTemperature))
    CubicCoef1 = -(((1 - gintK1) * B) + 1)
    CubicCoef2 = (((gIntK2 - gintK1) * B ^ 2) - (gintK1 * B) + A)
    CubicCoef3 = -(gIntK2 * B * (1 + B) + gintK3 * A) * B
    F_de_PyT = CompresibilityFactor ^ 3 + CubicCoef1 * CompresibilityFactor ^ 2 + CubicCoef2 * CompresibilityFactor + CubicCoef3
' ---- Code Body Ends ---- (Default)
Exit Function

F_de_PyT_ErrorHandler:
  Err.Raise Err.Number

Exit_F_de_PyT:

End Function
Private Function F_de_PryTrVr(Tr As Double, ReducedVolume As Double, Pr As Double) As Double
On Error GoTo F_de_PryTrVr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim P0 As Double
    Dim CubicCoef1 As Double
    Dim CubicCoef2 As Double, CubicCoef3 As Double, alphat As Double
    alphat = Alpha(Tr)
    alphat = alphat / Tr
    P0 = Pr / Tr
    CubicCoef1 = -(gIntK2 * (gdblOmB ^ 3) * P0 + gIntK2 * (gdblOmB ^ 2) + gdblOmA * gdblOmB * alphat) / P0
    CubicCoef2 = (P0 * gdblOmB * gdblOmB * (gIntK2 - gintK1) - gintK1 * gdblOmB + gdblOmA * alphat) / P0
    CubicCoef3 = (P0 * gdblOmB * (gintK1 - 1) - 1) / P0
    F_de_PryTrVr = ReducedVolume ^ 3 + CubicCoef3 * ReducedVolume ^ 2 + CubicCoef2 * ReducedVolume + CubicCoef1
' ---- Code Body Ends ---- (Default)
Exit Function

F_de_PryTrVr_ErrorHandler:
  Err.Raise Err.Number

Exit_F_de_PryTrVr:

End Function
Private Function P_despejada(ReducedTemperature As Double, CompresibilityFactor As Double, initPressure As Double) As Double
On Error GoTo P_despejada_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim P1 As Double, P2 As Double
    Dim f As Double, Fh As Double, h As Double
    h = 0.00001
    P2 = initPressure
    Do 'newton raphson
        P1 = P2
        f = F_de_PyT(ReducedTemperature, CompresibilityFactor, P1)
        Fh = (F_de_PyT(ReducedTemperature, CompresibilityFactor, P1 + h) - f) / h
        P2 = P1 - f / Fh
    Loop Until Abs(P2 - P1) < 0.0000000001
    P_despejada = P2
' ---- Code Body Ends ---- (Default)
Exit Function

P_despejada_ErrorHandler:
  Err.Raise Err.Number

Exit_P_despejada:

End Function
Private Function Fugacity(Z As Double) As Double
On Error GoTo Fugacity_ErrorHandler

' ---- Code Body Starts ---- (Default)
'funcion para el calculo de coeficiente de fugacidad (4.30 pag 104), pasando como parametro un valor de z
    Dim dblParte_ag As Double, dblparte_bg As Double, dblparte_cg As Double
    Dim dblParte_dg As Double, dblizg  As Double
    Dim dblparte_eg As Double
    dblizg = I_z(Z, gintK1, gIntK2, mParameter.B)
    dblParte_ag = Z - 1 - Log(Abs(Z - mParameter.B)) + mParameter.A * dblizg
    dblparte_bg = mParameter.C / ((1 + gintK1 + gIntK2) * mParameter.B ^ 2)
    dblparte_cg = (1 + gintK1 / 2) * dblizg
    dblparte_eg = (Abs((Z ^ 2 + gintK1 * mParameter.B * Z + gIntK2 * mParameter.B ^ 2)) ^ 0.5)
    dblParte_dg = -Log(Abs((Z - mParameter.B) / dblparte_eg))
    Fugacity = dblParte_ag + dblparte_bg * (dblparte_cg + dblParte_dg)
' ---- Code Body Ends ---- (Default)
Exit Function

Fugacity_ErrorHandler:
  Err.Raise Err.Number

Exit_Fugacity:

End Function
Private Sub Coef_Cubicas(Pr As Double, Tr As Double, alfac As Double)
On Error GoTo Coef_Cubicas_ErrorHandler

' ---- Code Body Starts ---- (Default)
'Calculos de los parametros A(2.43),B(2.44),C(2.84b) y los coeficientes de la ecuacion
'cubica general pasando como parametros los valores de la temperatura y la presion
'el case sirve para saber si se trata de una mezcla o sust.pura
    mParameter.A = gdblOmA * alfac * (Pr / (Tr) ^ 2)
    mParameter.B = gdblOmB * (Pr / (Tr))
    mvarCubicCoef1 = -(((1 - gintK1) * mParameter.B) + 1)
    mvarCubicCoef2 = (((gIntK2 - gintK1) * mParameter.B ^ 2) - (gintK1 * mParameter.B) + mParameter.A)
    mvarCubicCoef3 = -(gIntK2 * mParameter.B * (1 + mParameter.B) + gintK3 * mParameter.A) * mParameter.B
' ---- Code Body Ends ---- (Default)
Exit Sub

Coef_Cubicas_ErrorHandler:
  Err.Raise Err.Number

Exit_Coef_Cubicas:

End Sub
Private Sub Coef_Cubicasvr(P0 As Double, alphat As Double)
On Error GoTo Coef_Cubicasvr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarCubicCoef1 = -(gIntK2 * (gdblOmB ^ 3) * P0 + gIntK2 * (gdblOmB ^ 2) + gdblOmA * gdblOmB * alphat) / P0
    mvarCubicCoef2 = (P0 * gdblOmB * gdblOmB * (gIntK2 - gintK1) - gintK1 * gdblOmB + gdblOmA * alphat) / P0
    mvarCubicCoef3 = (P0 * gdblOmB * (gintK1 - 1) - 1) / P0
' ---- Code Body Ends ---- (Default)
Exit Sub

Coef_Cubicasvr_ErrorHandler:
  Err.Raise Err.Number

Exit_Coef_Cubicasvr:

End Sub
Friend Function Prsat(Tr As Double)
On Error GoTo Prsat_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim alphaT_1 As Double
    alphaT_1 = Alpha(Tr) / Tr - 1#
    Prsat = Tr * Exp(((((gdblH(4) * alphaT_1 + gdblH(3)) * alphaT_1 + gdblH(2)) * alphaT_1 + gdblH(1)) * alphaT_1 + gdblH(0)) * alphaT_1)
' ---- Code Body Ends ---- (Default)
Exit Function

Prsat_ErrorHandler:
  Err.Raise Err.Number

Exit_Prsat:

End Function
Public Property Let PRSVK1(ByVal vData As Double)
On Error GoTo PRSVK1_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPRSVK1 = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PRSVK1_ErrorHandler:
  Err.Raise Err.Number

Exit_PRSVK1:

End Property
Public Property Get PRSVK1() As Double
On Error GoTo PRSVK1_ErrorHandler

' ---- Code Body Starts ---- (Default)
    PRSVK1 = mvarPRSVK1
' ---- Code Body Ends ---- (Default)
Exit Property

PRSVK1_ErrorHandler:
  Err.Raise Err.Number

Exit_PRSVK1:

End Property
Public Property Let PsatAtTr(ByVal vData As Double)
On Error GoTo PsatAtTr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPsatAtTr = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PsatAtTr_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatAtTr:

End Property
Public Property Get PsatAtTr() As Double
On Error GoTo PsatAtTr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    PsatAtTr = mvarPsatAtTr
' ---- Code Body Ends ---- (Default)
Exit Property

PsatAtTr_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatAtTr:

End Property
Public Property Let AcentricFactor(ByVal vData As Double)
On Error GoTo AcentricFactor_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarAcentricFactor = vData
' ---- Code Body Ends ---- (Default)
Exit Property

AcentricFactor_ErrorHandler:
  Err.Raise Err.Number

Exit_AcentricFactor:

End Property
Public Property Get AcentricFactor() As Double
On Error GoTo AcentricFactor_ErrorHandler

' ---- Code Body Starts ---- (Default)
    AcentricFactor = mvarAcentricFactor
' ---- Code Body Ends ---- (Default)
Exit Property

AcentricFactor_ErrorHandler:
  Err.Raise Err.Number

Exit_AcentricFactor:

End Property
Public Property Get Zc() As Double
On Error GoTo Zc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Zc = mvarZc
' ---- Code Body Ends ---- (Default)
Exit Property

Zc_ErrorHandler:
  Err.Raise Err.Number

Exit_Zc:

End Property
Public Property Let Zc(ByVal vData As Double)
On Error GoTo Zc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarZc = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Zc_ErrorHandler:
  Err.Raise Err.Number

Exit_Zc:

End Property
Public Property Get Tc() As Double
On Error GoTo Tc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Tc = mvarTc
' ---- Code Body Ends ---- (Default)
Exit Property

Tc_ErrorHandler:
  Err.Raise Err.Number

Exit_Tc:

End Property
Public Property Let Tc(ByVal vData As Double)
On Error GoTo Tc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTc = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Tc_ErrorHandler:
  Err.Raise Err.Number

Exit_Tc:

End Property
Public Property Get Pc() As Double
On Error GoTo Pc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Pc = mvarPc
' ---- Code Body Ends ---- (Default)
Exit Property

Pc_ErrorHandler:
  Err.Raise Err.Number

Exit_Pc:

End Property
Public Property Let Pc(ByVal vData As Double)
On Error GoTo Pc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPc = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Pc_ErrorHandler:
  Err.Raise Err.Number

Exit_Pc:

End Property
Friend Function I_z(Z As Double, K1 As Integer, K2 As Integer, B As Double) As Double
On Error GoTo I_z_ErrorHandler

' ---- Code Body Starts ---- (Default)
'funcion que calcula el valor de el parametro Iz (4.31a pag 104) necesario para el calculo de
'ln(fugacidad) (4.30 pag 104) psando como parametro un valor de z
    Dim dbldis_iz  As Double
    Dim dblparte_a As Double, dblparte_bg As Double, dblparte_cg As Double
    dbldis_iz = (K1 ^ 2) - 4 * K2
    Select Case dbldis_iz
        Case Is > 0
            dblparte_a = 1 / (B * (dbldis_iz) ^ 0.5)
            dblparte_bg = (2 * Z + B * (K1 - (dbldis_iz) ^ 0.5))
            dblparte_cg = (2 * Z + B * (K1 + (dbldis_iz) ^ 0.5))
            I_z = dblparte_a * Log(Abs((dblparte_bg) / (dblparte_cg)))
        Case Is = 0
            I_z = -2 / (2 * Z + K1 * B)
     End Select
' ---- Code Body Ends ---- (Default)
Exit Function

I_z_ErrorHandler:
  Err.Raise Err.Number

Exit_I_z:

End Function
Friend Function Alpha(ReducedTemperature As Double) As Double
On Error GoTo Alpha_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim m As Double, n As Double, Omegac As Double
    
    Select Case mvarQbicsWinEQofState
    
        Case VdW1870_Qbic
            Alpha = 1
        Case Berth1899_Qbic
            Alpha = 1 / ReducedTemperature
        Case VdWAda1984_Qbic
            m = 0.228165 + 0.791981 * mvarAcentricFactor - 0.648552 * mvarAcentricFactor * mvarAcentricFactor + 0.654505 * mvarAcentricFactor
            Alpha = 10 ^ (m * (1 - ReducedTemperature))
        Case RK1949_Qbic
            Alpha = 1 / ((ReducedTemperature) ^ 0.5)
        Case RKS1972_Qbic
            Alpha = (1 + (0.48 + 1.574 * mvarAcentricFactor - 0.176 * mvarAcentricFactor ^ 2) * (1 - (ReducedTemperature) ^ 0.5)) ^ 2
        Case RKSGD1978_Qbic
            Alpha = (1 + (0.48508 + 1.55171 * mvarAcentricFactor - 0.15613 * mvarAcentricFactor ^ 2) * (1 - (ReducedTemperature) ^ 0.5)) ^ 2
        Case RKSL1997_Qbic
            Alpha = (1 + (0.478972559 + 1.576809191 * mvarAcentricFactor - 0.187219516 * mvarAcentricFactor ^ 2 + 0.020424946 * mvarAcentricFactor ^ 3) * (1 - (ReducedTemperature) ^ 0.5)) ^ 2
        Case PR1976_Qbic
            Alpha = (1 + (0.37464 + 1.54226 * mvarAcentricFactor - 0.26992 * mvarAcentricFactor ^ 2) * (1 - (ReducedTemperature) ^ 0.5)) ^ 2
        Case RP1978_Qbic
            Alpha = (1 + (0.379642 + 1.48503 * mvarAcentricFactor - 0.164423 * mvarAcentricFactor ^ 2 + 0.016666 * mvarAcentricFactor ^ 3) * (1 - (ReducedTemperature) ^ 0.5)) ^ 2
        Case PRL1997_Qbic
            Alpha = (1 + (0.378710697 + 1.487972964 * mvarAcentricFactor - 0.166754831 * mvarAcentricFactor ^ 2 + 0.017169486 * mvarAcentricFactor ^ 3) * (1 - (ReducedTemperature) ^ 0.5)) ^ 2
        Case VdWVald1989_Qbic
            Omegac = mvarAcentricFactor * mvarZc
            m = 0.4745 + (2.7349 + 6.0984 * Omegac) * Omegac
            n = 0.0674 + (2.1031 + 3.9512 * Omegac) * Omegac
            Alpha = 1 + (1 - ReducedTemperature) * (m + n / ReducedTemperature)
        Case RKSmn1980_Qbic
            Alpha = 1 + (1 - ReducedTemperature) * (mvarm + mvarn / ReducedTemperature)
        Case RKSATmn1995_Qbic
            Alpha = Exp((1 - ReducedTemperature) * mvarm * (Abs(1 - ReducedTemperature) ^ (mvarg - 1)) + mvarn * ((1 / ReducedTemperature) - 1))
        Case PRATmng1997_Qbic
            Alpha = Exp((1 - ReducedTemperature) * mvarm * (Abs(1 - ReducedTemperature) ^ (mvarg - 1)) + mvarn * ((1 / ReducedTemperature) - 1))
        Case PRMmn1989_Qbic
            m = 1 - Sqr(ReducedTemperature)
            Alpha = Exp((1 - ReducedTemperature) * mvarm + mvarn * m * m)
        Case PRSV1986_Qbic
            m = Sqr(ReducedTemperature)
            n = 0.378893 + 1.4897153 * mvarAcentricFactor - 0.17131848 * mvarAcentricFactor * mvarAcentricFactor + 0.0196554 * mvarAcentricFactor * mvarAcentricFactor * mvarAcentricFactor
            Alpha = 1 + (n + mvarPRSVK1 * (1 + m) * (0.7 - ReducedTemperature)) * (1 - m)
            Alpha = Alpha * Alpha
        Case VdWOL1998_Qbic, RKOL1998_Qbic, PROL1998_Qbic
            Alpha = ReducedTemperature * (1 + SumHk(ReducedTemperature))
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

Alpha_ErrorHandler:
  Err.Raise Err.Number

Exit_Alpha:

End Function
Public Function q1_calc(Temperature As Double)
Dim ReducedPressure As Double, ReducedTemperature As Double, vrg As Double, vrf As Double, firstAprox As Double
 ReducedPressure = 0
 ReducedTemperature = Temperature / mvarTc
    Coef_Cubicasvr ReducedPressure / ReducedTemperature, Alpha(ReducedTemperature) / ReducedTemperature
    CubicSolverNR mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, vrg, firstAprox, vrf
    q1_calc = vrg * mvarTc * UnivGasConst / mvarPc
End Function
Private Function Vmolar(Temperature As Double, volume As Double, ByRef Pressure As Double)
Dim A As Double, B As Double, X As Double, ReducedTemperature As Double
ReducedTemperature = Temperature / mvarTc
'esta cosa da perfecta para el vapor pero para liquido comprimido da espantoso....

    A = (gdblOmA * Alpha(ReducedTemperature)) / (mvarPc * (ReducedTemperature) ^ 2)
    B = gdblOmB / (mvarPc * (ReducedTemperature))
    X = volume / (UnivGasConst * Temperature)
    volume = (X ^ 2 + gintK1 * B * X - A * X + gIntK2 * B ^ 2 + gintK3 * A * B)

End Function

Private Function Rackett(T As Double) As Double
Dim Tr As Double
Dim Tao As Double
Tr = T / mvarTc
    If Tr <= 0.75 Then
        Tao = 1 + (1 - Tr) ^ (2 / 7)
    Else
        Tao = 1.6 + (0.00693026 / (Tr - 0.655))
    End If
    Rackett = ((UnivGasConst * mvarTc) / mvarPc) * mvarZRA ^ Tao
End Function

Public Property Get ZRA() As Double
On Error GoTo ZRA_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ZRA = mvarZRA
' ---- Code Body Ends ---- (Default)
Exit Property

ZRA_ErrorHandler:
  Err.Raise Err.Number

Exit_ZRA:

End Property
Public Property Let ZRA(ByVal vData As Double)
On Error GoTo ZRA_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarZRA = vData
' ---- Code Body Ends ---- (Default)
Exit Property

ZRA_ErrorHandler:
  Err.Raise Err.Number

Exit_ZRA:

End Property

Public Function Zg_calc(ReducedTemperature As Double, ByRef ReducedPressure As Double, ByRef VaporCompresibilityFactor As Double, ByRef LiquidCompresibilityFactor As Double) As Boolean
On Error GoTo Zg_calc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Alfa As Double, ans As Integer
    Dim firstAprox As Double
    Dim Pr As Double, prsat1 As Double
    On Error GoTo ErrProcedure
    prsat1 = mvarPsatAtTr

    Alfa = Alpha(ReducedTemperature)
        Coef_Cubicas prsat1, ReducedTemperature, Alfa
        ans = CubicSolver(mvarCubicCoef3, mvarCubicCoef2, mvarCubicCoef1, 1#, _
        VaporCompresibilityFactor, firstAprox, LiquidCompresibilityFactor)
    Pr = prsat1
    ReducedPressure = Pr
    Zg_calc = True
    Exit Function
ErrProcedure:
    Zg_calc = False

Exit Function

Zg_calc_ErrorHandler:
  Err.Raise Err.Number

Exit_Zg_calc:

End Function


