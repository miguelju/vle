VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsQbics"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False


Option Explicit
    Public Enum TADiPPhiModel
        IdealSolutionp = 25
        IdealGas = -2
        PR1976_Qbicp = 0
        RK1949_Qbicp = 1
        RKS1972_Qbicp = 2
        VdW1870_Qbicp = 3
        PRL1997_Qbicp = 4
        RKSL1997_Qbicp = 5
        RKSGD1978_Qbicp = 6
        RP1978_Qbicp = 7
        Berth1899_Qbicp = 8
        VdWAda1984_Qbicp = 9
        VdWVald1989_Qbicp = 10
        RKSmn1980_Qbicp = 11
        RKSATmn1995_Qbicp = 12
        PRATmng1997_Qbicp = 13
        PRMmn1989_Qbicp = 14
        PRSV1986_Qbicp = 15
        VdWOL1998_Qbicp = 16
        RKOL1998_Qbicp = 17
        PROL1998_Qbicp = 18
    End Enum

    Public Enum TADiPMR
        WS = 26
        HOV
        HVS
        MHV1
        MHV2
        IVDW
        IIVDW
        Clasica_I
'        Clasica_II
    
    End Enum

    Public Enum TADiPPhaseID
        PhaseIDvapor = 0
        PhaseIDLiquid = 1
    End Enum

    Public Enum TADiPdim
        WithDim = 0
        DimLess = 1
    End Enum


'valores de los parametros que describen las ecuaciones cubicas
    Private gintK1 As Integer, gIntK2 As Integer, gintK3 As Integer
    Private gdblOmA As Double, gdblOmB As Double, gdblOmC As Double
    Private gdblH(0 To 4) As Double

'estructura que contiene los valores de los coeficientes del desarrollo
'cubico en volumen
    Private Type ParametersEOS
        A As Double
        B As Double
        C As Double
    End Type
    Private mvarModel As TADiPPhiModel
    Private mParameter As ParametersEOS
    Private mParameteri() As ParametersEOS
    Private mvarPhaseOfMixture As TADiPPhaseID
    Private mvarMolarFraction() As Double
    Private mvarProperties As colclsProperties
    Private mvarKij As Variant 'local copy
    Private LiquidSolver As Object
    Private mvarMixture As clsAllProps
    Private mvarParent As clsLVE 'local copy
    Private A_ij() As Double
    Private ParametroIIVDW() As Double
    Private ParametroWS() As Double

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
    
    
    
'estructura que contiene los valores de los coeficientes del desarrollo
'cubico en volumen
    Private mvarCubicCoef1 As Double, mvarCubicCoef2 As Double, mvarCubicCoef3 As Double
    
  '  Private mParameter As ParametersEOS
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
    Dim E As Double, k As Integer, Sum As Double, SatP
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
        Sum = 0
        For k = 1 To 10
            Sum = Sum + h(k) * (-Log(mvarPsatAtTr / ReducedTemperature)) ^ k
        Next
        SumHk = ((1 - Log(mvarPsatAtTr / ReducedTemperature)) ^ E) * Sum
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
    Dim temp As Double
    h = 0.00001
    T2 = IniT
    Do 'newton raphson
        T1 = T2
        f = F_de_PryTrVr(T1, ReducedVolume, ReducedPressure)
        temp = T1 + h
        h = temp - T1
        Fh = (F_de_PryTrVr(temp, ReducedVolume, ReducedPressure) - f) / h
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
Public Property Get zc() As Double
On Error GoTo Zc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    zc = mvarZc
' ---- Code Body Ends ---- (Default)
Exit Property

Zc_ErrorHandler:
  Err.Raise Err.Number

Exit_Zc:

End Property
Public Property Let zc(ByVal vData As Double)
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
Private Function Vmolar(Temperature As Double, Volume As Double, ByRef Pressure As Double)
Dim A As Double, B As Double, X As Double, ReducedTemperature As Double
ReducedTemperature = Temperature / mvarTc
'esta cosa da perfecta para el vapor pero para liquido comprimido da espantoso....

    A = (gdblOmA * Alpha(ReducedTemperature)) / (mvarPc * (ReducedTemperature) ^ 2)
    B = gdblOmB / (mvarPc * (ReducedTemperature))
    X = Volume / (UnivGasConst * Temperature)
    Volume = (X ^ 2 + gintK1 * B * X - A * X + gIntK2 * B ^ 2 + gintK3 * A * B)

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







Public Property Set Parent(ByVal vData As clsLVE)
    Set mvarParent = vData
End Property
Public Property Get Parent() As clsLVE
    Set Parent = mvarParent
End Property
Public Property Let Kij(ByVal vData As Variant)
On Error GoTo Kij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarKij = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Kij_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij:

End Property
Public Property Set Kij(ByVal vData As Object)
On Error GoTo Kij_ErrorHandler

' ---- Code Body Starts ---- (Default)
   Set mvarKij = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Kij_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij:

End Property
Public Property Get Kij() As Variant
On Error GoTo Kij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If IsObject(mvarKij) Then
        Set Kij = mvarKij
    Else
        Kij = mvarKij
    End If
' ---- Code Body Ends ---- (Default)
Exit Property

Kij_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij:

End Property
Public Property Get Properties() As colclsProperties
On Error GoTo Properties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarProperties Is Nothing Then
        Set mvarProperties = New colclsProperties
    End If
    Set Properties = mvarProperties
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_properties:

End Property
Public Property Set Properties(vData As colclsProperties)
On Error GoTo Properties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_properties:

End Property
Public Property Let PhaseOfMixture(ByVal vData As TADiPPhaseID)
On Error GoTo PhaseOfMixture_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPhaseOfMixture = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PhaseOfMixture_ErrorHandler:
  Err.Raise Err.Number

Exit_PhaseOfMixture:

End Property
Public Property Get PhaseOfMixture() As TADiPPhaseID
On Error GoTo PhaseOfMixture_ErrorHandler

' ---- Code Body Starts ---- (Default)
    PhaseOfMixture = mvarPhaseOfMixture
' ---- Code Body Ends ---- (Default)
Exit Property

PhaseOfMixture_ErrorHandler:
  Err.Raise Err.Number

Exit_PhaseOfMixture:

End Property
Public Function PartialFugacityCoeficientInSolution(Component As Integer, _
Temperature As Double, Pressure As Double, ByRef PartialFugacityCoeficient _
As Double, Optional Z As Double) As Boolean

On Error GoTo PartialFugacityCoeficientInSolution_ErrorHandler

    Dim A As Double, B As Double, n As Integer, mvarmixingrule As TADiPMR
    Dim b_i As Double, a_i As Double, AuxB As Double, Qi As Double
    Dim AuxA As Double, delta As Double, Iz As Double, g As Double, _
    ri As Double, si As Double, AA As Double, BB As Double, parametro1 As Double
    Dim Funcion1 As Double, Funcion2 As Double, derivada1 As Double, derivada2 As Double
    Dim Q As Double, d As Double, lnGamma As Double, C As Double, Gamma As Double
    Dim q1 As Double, q2 As Double, i As Integer, EG As Double, Aux1 As Double
    Dim Aux2 As Double, Aux3 As Double, j As Integer, X As Double, covol As Double
    Dim rule As Integer
    mvarmixingrule = rule
    Aux1 = 0
    Aux2 = 0
    Aux3 = 0
    parametro1 = 0
    
    n = mvarProperties.Count
    If mvarmixingrule = Clasica_I Then
        ai__bi Component, Temperature, a_i, b_i, n
        MixingRules Temperature, 0, WithDim, rule
        AuxB = b_i / mParameter.B
        ri = gintK1 * AuxB
        si = 2 * gIntK2 * AuxB
        AuxA = a_i / mParameter.A
        delta = (gintK1 ^ 2) - 4 * gIntK2
    Else
        a_i = AAi(Component, Temperature, Pressure, WithDim)
        b_i = BBi(Component, Temperature, Pressure, WithDim)
        MixingRules Temperature, 0, WithDim, rule
        AuxB = b_i / mParameter.B
        AuxA = a_i / mParameter.A
    End If
    MixingRules Temperature, Pressure, DimLess, rule
    A = mParameter.A
    B = mParameter.B
    
    If Z = vbEmpty Then
        Z = ZSolver()
    End If
    
        Iz = I_z(Z, gintK1, gIntK2, B)
    


    C_cal C
    If mvarmixingrule <> Clasica_I Then
        Funcion1 = Log((Z - B))
        Funcion2 = Log(((Z + (1 + 2 ^ 0.5) * B) / (Z + (1 - 2 ^ 0.5) * B)))
    End If
    
    
    
    Select Case mvarmixingrule
'    Estas ecuaciones fueron tomadas del texto Modeling Vapor-Liquid Equilibria de
'    Orbey -Sandler, exepto la clasica que era la original de VLE
        Case Clasica_I
            If delta <> 0 Then
                g = Z * Z + gintK1 * B * Z + gIntK2 * B * B
                Qi = ((AuxB * Z * delta + (si * gintK1 - 2 * ri * gIntK2) * B - (ri _
                * gintK1 - 2 * si) * Z) / _
                (g * delta)) + ((AuxA + 1) * delta - (ri * gintK1 - 2 * si)) * Iz / delta
            ElseIf delta = 0 Then
                Qi = Iz * (AuxA + (AuxB * Z + ri * B / 2) * Iz + 1)
            End If
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Log(Z - B) + A * Qi)
        
        Case WS
            MixingRules Temperature, Pressure, WithDim, rule

            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)
        
            For i = 1 To n
                For j = 1 To n
                    Q = Q + mvarMolarFraction(i) * mvarMolarFraction(j) * ParametroWS(i, j)
                Next
            Next
            For i = 1 To n
                parametro1 = parametro1 + mvarMolarFraction(i) * AAi(i, Temperature, Pressure, WithDim) / _
                (BBi(i, Temperature, Pressure, WithDim) * Temperature * UnivGasConst)
            Next

            EG = mvarParent.ExG(Temperature, n)
            d = parametro1 + EG / (C * Temperature * UnivGasConst)
            For j = 1 To n
                Aux1 = Aux1 + mvarMolarFraction(j) * ParametroWS(Component, j)
            Next
            Aux2 = 2 * Aux1
            Aux3 = (lnGamma / C) + a_i / (b_i * Temperature * UnivGasConst)
            derivada1 = (1 / (1 - d)) * Aux2 - (Q / (1 - d) ^ 2) * (1 - Aux3)
            derivada2 = Temperature * UnivGasConst * d * derivada1 + ((mParameter.B) * _
            Temperature * UnivGasConst) * Aux3
            
            Qi = ((mParameter.A) / (2 * 2 ^ 0.5 * (mParameter.B) * Temperature * UnivGasConst)) * _
            (derivada2 / (mParameter.A) - derivada1 / (mParameter.B))
            PartialFugacityCoeficient = Exp((derivada1 / (mParameter.B)) * (Z - 1) - Funcion1 _
            + Qi * Log(((Z + (1 - 2 ^ 0.5) * B) / (Z + (1 + 2 ^ 0.5) * B))))
        
        Case HOV
            MixingRules Temperature, Pressure, WithDim, rule
            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)
            
            Qi = (1 / (2 * 2 ^ 0.5) * (a_i / (b_i * Temperature * UnivGasConst) + lnGamma / C))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
        
        Case HVS
            MixingRules Temperature, Pressure, WithDim, rule
            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)

            Qi = (1 / (2 * 2 ^ 0.5) * (a_i / (b_i * Temperature * UnivGasConst) + lnGamma / C + _
            Log(mParameter.B / b_i) / C + (1 / C) * (AuxB - 1)))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)

        Case IVDW
            MixingRules Temperature, Pressure, WithDim, rule
            For j = 1 To n
                parametro1 = parametro1 + mvarMolarFraction(j) * A_ij(Component, j)
            Next
            Qi = (A / (2 * 2 ^ 0.5 * B)) * ((2 * parametro1 / (mParameter.A)) - (AuxB))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
            
        Case IIVDW
            MixingRules Temperature, Pressure, WithDim, rule
            For j = 1 To n
                parametro1 = parametro1 + mvarMolarFraction(j) * A_ij(Component, j)
            Next
            
            derivada1 = 2 * (parametro1 + ParametroIIVDW(Component)) - (mParameter.A)
            Qi = (A / (2 * 2 ^ 0.5 * B)) * ((derivada1 / (mParameter.A)) + 1 - (AuxB))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
        
        Case MHV1
            MixingRules Temperature, Pressure, WithDim, rule
        
            q1 = -0.593
            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)

            Qi = (1 / (2 * 2 ^ 0.5) * (a_i / (b_i * Temperature * UnivGasConst) + lnGamma / q1 - Log(AuxB) / q1 + (1 / q1) * (AuxB - 1)))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
        
        Case MHV2
            MixingRules Temperature, Pressure, WithDim, rule

            q1 = -0.478
            q2 = -0.0047

            derivada1 = (q1 * a_i / (b_i * Temperature * UnivGasConst) + q2 * _
            ((a_i / (b_i * Temperature * UnivGasConst)) ^ 2 + ((mParameter.A) / (B * Temperature * UnivGasConst)) ^ 2) _
            + lnGamma + Log(1 / AuxB) + AuxB - 1) / (q1 + 2 * q2 * ((mParameter.A) / (B * Temperature * UnivGasConst)))

            Qi = (1 / (2 * 2 ^ 0.5) * derivada1)
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
    End Select

Exit Function

PartialFugacityCoeficientInSolution_ErrorHandler:
  Err.Raise Err.Number

Exit_PartialFugacityCoeficientInSolution:

End Function


Public Sub MixingRules(ByVal T As Double, P As Double, Dimensions As _
TADiPdim, rule As Integer)
On Error GoTo MixingRules_ErrorHandler


    Dim i As Integer, j As Integer, n As Integer, num As Double, den As Double, mvarmixingrule As TADiPMR
    Dim C As Double, EG As Double, Aux As Double, Aux2 As Double, Aux1 As Double, Aux3 As Double
    Dim q1 As Double, q2 As Double, Par1 As Double, Par2 As Double, Par3 As Double
    Dim Alfamix As Double, x1 As Double, x2 As Double, Dimen As TADiPdim
    
    Dimen = WithDim
    n = mvarProperties.Count
    ReDim Preserve mParameteri(1 To n)
    mParameter.A = 0
    mParameter.B = 0
    ReDim Aij(n, n) As Double
    ReDim bij(n, n) As Double
    ReDim km(n, n) As Double
    ReDim A_ij(n, n) As Double
    ReDim ParametroIIVDW(n) As Double
    ReDim ParametroWS(n, n) As Double
    
    mvarmixingrule = rule
    C_cal C
    
    Select Case mvarmixingrule
    
        Case WS
        Aux = 0
        num = 0
        den = 0
        
            'Esta si funciona, AHORA!!!!
            EG = mvarParent.ExG(T, n)
            For i = 1 To n
                For j = 1 To n
                     bij(i, j) = 0.5 * ((BBi(i, T, P, Dimen) - _
                     AAi(i, T, P, Dimen) / (UnivGasConst * T)) + (BBi(j, T, P, Dimen) - _
                     AAi(j, T, P, Dimen) / (UnivGasConst * T))) * (1 - mvarKij(i, j))
                     ParametroWS(i, j) = bij(i, j)
                Next
            Next
            For i = 1 To n
                For j = 1 To n
                    num = num + mvarMolarFraction(i) * mvarMolarFraction(j) * (bij(i, j))
                Next
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / BBi(i, T, P, Dimen)
            Next
            mParameter.B = (UnivGasConst * T * num) / (UnivGasConst * T - (Aux + EG / C))
            
            mParameter.A = mParameter.B * (EG / (C) + Aux)
            

            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If

        Case IVDW
            'Esta!!!!!
            For i = 1 To n
                For j = 1 To n
                    Aij(i, j) = ((AAi(i, T, P, Dimen) * AAi(j, T, P, Dimen)) ^ 0.5) * (1 - mvarKij(i, j))
                    A_ij(i, j) = Aij(i, j)
               Next
            Next
            For i = 1 To n
                For j = 1 To n
                    mParameter.A = mParameter.A + mvarMolarFraction(i) * mvarMolarFraction(j) * _
                    Aij(i, j)
                Next
            Next
            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next

            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If

        Case IIVDW
            'Esta si funciona
             For i = 1 To n
                For j = 1 To n
                    km(i, j) = mvarKij(i, j) * mvarMolarFraction(i) + mvarKij(j, i) * mvarMolarFraction(j)
                Next
            Next
            For i = 1 To n
                For j = 1 To n
                    Aij(i, j) = ((AAi(i, T, P, Dimen) * AAi(j, T, P, _
                    Dimen)) ^ 0.5) * (1 - km(i, j))
                    A_ij(i, j) = Aij(i, j)
                Next
            Next
            For i = 1 To n
                For j = 1 To n
                    mParameter.A = mParameter.A + mvarMolarFraction(i) * mvarMolarFraction(j) * _
                    Aij(i, j)
                Next
            Next
            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            'Esta parte es para el calculo de la fugacidad
            For i = 1 To n
                For j = 1 To n
                    If j = i Then
                        ParametroIIVDW(i) = ParametroIIVDW(i)
                    Else
                        ParametroIIVDW(i) = ParametroIIVDW(i) + mvarMolarFraction(i) * mvarMolarFraction(j) * _
                        ((AAi(i, T, P, Dimen) * AAi(j, T, P, Dimen)) ^ 0.5) * (mvarMolarFraction(j) * _
                        km(j, i) - (1 - mvarMolarFraction(i)) * km(i, j))
                    End If
                Next
            Next
            
            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If

        Case HOV
            'AHORA SIP!!!
            EG = mvarParent.ExG(T, n)
            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / BBi(i, T, P, Dimen)
            Next
                mParameter.A = mParameter.B * (Aux + EG / C)
            
            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If

        Case MHV1
            'Esta no funciona, le falta el valor de q1
            EG = mvarParent.ExG(T, n)
            q1 = -0.593

            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)
                Aux2 = Aux2 + mvarMolarFraction(i) * Log(mParameter.B / BBi(i, T, P, Dimen))
            Next
                mParameter.A = mParameter.B * UnivGasConst * T * (Aux + (1 / q1) * (EG / (UnivGasConst * T) + Aux2))
            
            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If

        Case Clasica_I
            'Esta si funciona
            For i = 1 To n
                mParameteri(i).A = AAi(i, T, P, Dimensions)
                mParameteri(i).B = BBi(i, T, P, Dimensions)
            Next
            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * _
                mParameteri(i).B
            Next
            For i = 1 To n
                For j = 1 To n
                   mParameter.A = mParameter.A + mvarMolarFraction(i) * _
                    mvarMolarFraction(j) _
                   * AAij(i, j)
                Next
            Next


        Case MHV2
        'Esta ecuacion fue tomada del libro de Orbey y Sandler
            EG = mvarParent.ExG(T, n)
            q1 = -0.478
            q2 = -0.0047

            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)
                Aux1 = Aux1 + mvarMolarFraction(i) * (AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)) ^ 2
                Aux2 = Aux2 + mvarMolarFraction(i) * Log(mParameter.B / BBi(i, T, P, Dimen))
            Next

                Aux3 = -q1 * Aux - q2 * Aux1 - EG / (UnivGasConst * T) - Aux2
            'para clacular el valor de alfamix(epsilon en el libro), se debe resolver
            'un polinomio de segundo orden, el resultado es la mayor de las raices:

            x1 = (-q1 + ((q1) ^ 2 - 4 * q2 * Aux3) ^ 0.5) / (2 * q2)
            x2 = (-q1 - ((q1) ^ 2 - 4 * q2 * Aux3) ^ 0.5) / (2 * q2)

            If x1 >= x2 Then
                Alfamix = x1
            Else
                Alfamix = x2
            End If
                mParameter.A = mParameter.B * UnivGasConst * T * Alfamix
            
            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If


        Case HVS
            'Esta no funciona, ya la aregle falta ver si sirve
            EG = mvarParent.ExG(T, n)
            Aux = 0
            Aux2 = 0

            For i = 1 To n
                mParameter.B = mParameter.B + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)
                Aux2 = Aux2 + mvarMolarFraction(i) * Log(mParameter.B / BBi(i, T, P, Dimen))
            Next
                mParameter.A = mParameter.B * UnivGasConst * T * (Aux + (1 / C) * (EG / (UnivGasConst * T) + Aux2))

            If Dimensions = DimLess Then
                mParameter.B = mParameter.B * P / (T * UnivGasConst)
                mParameter.A = mParameter.A * P / (T * UnivGasConst) ^ 2
            End If
'        Case Clasica_II
'            'Esta si funciona
'            For i = 1 To n
'                For j = 1 To n
'                    Aij(i, j) = ((AAi(i, T, P, Dimen) * AAi(j, T, P, _
'                    Dimen)) ^ 0.5) * (1 - mvarKij(i, j))
'                    bij(i, j) = (BBi(i, T, P, Dimen) + BBi(j, T, P, _
'                    Dimen)) / 2
'               Next
'            Next
'            For i = 1 To n
'                For j = 1 To n
'                    mParameter.A = mParameter.A + mvarMolarFraction(i) * mvarMolarFraction(j) * _
'                    Aij(i, j)
'                    mParameter.B = mParameter.B + mvarMolarFraction(i) * mvarMolarFraction(j) * _
'                    bij(i, j)
'                Next
'            Next
    End Select

Exit Sub

MixingRules_ErrorHandler:
  Err.Raise Err.Number

Exit_MixingRules:

End Sub
Private Function AAi(i As Integer, T As Double, P As Double, Dimensions As _
TADiPdim) As Double
On Error GoTo AAi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tres As Double, Pr As Double, SatP As Double
   
    Tres = T / mvarProperties(i).Tc
    
        Me.AcentricFactor = mvarProperties(i).w
        mvarParent.Parametro_EDEC_Modificadas T, i, SatP
         Me.PsatAtTr = SatP / mvarProperties(i).Pc
'        .PsatAtTr = mvarProperties(i).PsatAtTr
        Me.m = mvarProperties(i).m
        Me.n = mvarProperties(i).n
        Me.g = mvarProperties(i).g
        Me.zc = mvarProperties(i).zc
        Me.PRSVK1 = mvarProperties(i).PRSVK1
        Me.Model = mvarModel
        If Dimensions = DimLess Then
            Pr = P / mvarProperties(i).Pc
            AAi = gdblOmA * Me.Alpha(Tres) * Pr / (Tres * Tres)
        Else
            AAi = (UnivGasConst * UnivGasConst) * gdblOmA * Me.Alpha(Tres) * _
            mvarProperties(i).Tc * mvarProperties(i).Tc / mvarProperties(i).Pc
        End If
    
' ---- Code Body Ends ---- (Default)
Exit Function

AAi_ErrorHandler:
  Err.Raise Err.Number

Exit_AAi:

End Function
Private Function BBi(ByVal i As Integer, ByVal T As Double, ByVal P As _
Double, Dimensions As TADiPdim) As Double
On Error GoTo BBi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If Dimensions = DimLess Then
        BBi = gdblOmB * P * mvarProperties(i).Tc / T / mvarProperties(i).Pc
    Else
        BBi = (UnivGasConst * gdblOmB * mvarProperties(i).Tc) / _
        mvarProperties(i).Pc
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

BBi_ErrorHandler:
  Err.Raise Err.Number

Exit_BBi:

End Function
Public Function ZSolver(Optional Phase As TADiPPhaseID) As Double
On Error GoTo ZSolver_ErrorHandler
Dim PolyCoef(0 To 2) As Double
Dim Discriminante As Double
Dim Sol(1 To 3) As Double

' ---- Code Body Starts ---- (Default)
    Dim CubicCoef1 As Double, CubicCoef2 As Double, CubicCoef3 As Double
    Dim firstAprox As Double, Zg As Double, Zf As Double, temp As _
TADiPPhaseID
   ' If mvarModel <> IdealGas Then
        CubicCoef1 = -(((1 - gintK1) * mParameter.B) + 1)
        CubicCoef2 = (((gIntK2 - gintK1) * mParameter.B ^ 2) - (gintK1 * _
        mParameter.B) + mParameter.A)
        CubicCoef3 = -(gIntK2 * mParameter.B * (1 + mParameter.B) + gintK3 * _
        mParameter.A) * mParameter.B

'        PolyCoef(0) = CubicCoef3
'        PolyCoef(1) = CubicCoef2
'        PolyCoef(2) = CubicCoef1
'        ThirdGradePolySolver PolyCoef(), Discriminante, Sol()
'        GetZfZg Sol(), Discriminante, Zf, Zg
'        If Zf = Imaginary Or Zg = Imaginary Then
'            MsgBox "Raices Imaginarias"
''            End
'        End If
        CubicSolver CubicCoef3, CubicCoef2, CubicCoef1, 1#, Zg, firstAprox, _
        Zf
    'Else
'        Zg = 1
'        Zf = 0
'    End If
    temp = mvarPhaseOfMixture
    If Phase <> vbEmpty Then
        temp = Phase
    End If
    Select Case temp
        Case PhaseIDvapor
            If Zg > 0 Then
                ZSolver = Zg
            ElseIf Zf > 0 Then
                ZSolver = Zf
            End If
        Case PhaseIDLiquid
            If Zf > 0 Then
                ZSolver = Zf
            ElseIf Zg > 0 Then
                ZSolver = Zg
            End If
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

ZSolver_ErrorHandler:
  Err.Raise Err.Number

Exit_ZSolver:

End Function

Private Sub Class_Terminate()
On Error GoTo Class_Terminate_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = Nothing
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Terminate_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Terminate:

End Sub
Public Property Get MolarFraction(Index As Integer) As Double
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    MolarFraction = mvarMolarFraction(Index)
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As _
Double)
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Static sIndex As Boolean
    If sIndex = False Then
        If Index = 1 Then
            sIndex = True
            ReDim Preserve mvarMolarFraction(1 To 1 + Index)
            ReDim Preserve mParameteri(1 To 1 + Index)
        End If
    End If
    If Index > UBound(mParameteri()) Then
        ReDim Preserve mvarMolarFraction(1 To 1 + Index)
        ReDim Preserve mParameteri(1 To 1 + Index)
    End If
    mvarMolarFraction(Index) = MolarFraction
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
Public Function PWithDimensions(ByVal T As Double, ByVal v As Double) As _
Double
Dim rule As Integer

On Error GoTo PWithDimensions_ErrorHandler

' ---- Code Body Starts ---- (Default)
    MixingRules T, 0, WithDim, rule
    PWithDimensions = (UnivGasConst * T / (v - mParameter.B)) - mParameter.A _
    * (v - gintK3 * mParameter.B) / _
    ((v - mParameter.B) * (v ^ 2 + gintK1 * mParameter.B * v + gIntK2 * _
    mParameter.B ^ 2))
' ---- Code Body Ends ---- (Default)
Exit Function

PWithDimensions_ErrorHandler:
  Err.Raise Err.Number

Exit_PWithDimensions:

End Function
Private Sub CalculateJacobianP(ByVal T As Double, ByVal v As Double, ByRef _
Jacob() As Double)
On Error GoTo CalculateJacobianP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim h As Double
    h = 0.0001
    Jacob(1, 1) = FirstDerivate(FirstDerivateP(T, v), FirstDerivateP(T + 1, _
    v), 1)
    Jacob(1, 2) = FirstDerivate(FirstDerivateP(T, v), FirstDerivateP(T, v + _
    h), h)
    Jacob(2, 1) = FirstDerivate(SecondDerivateP(T, v), SecondDerivateP(T + _
    1, v), 1)
    Jacob(2, 2) = FirstDerivate(SecondDerivateP(T, v), SecondDerivateP(T, v _
    + h), h)
' ---- Code Body Ends ---- (Default)
Exit Sub

CalculateJacobianP_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculateJacobianP:

End Sub
Private Function FirstDerivateP(T As Double, v As Double) As Double
On Error GoTo FirstDerivateP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim A As Double, B As Double, R As Double, rule As Integer
    
    MixingRules T, 0, WithDim, rule
    A = mParameter.A
    B = mParameter.B
    R = UnivGasConst
    FirstDerivateP = -R * T * v ^ 4 + (2 * A - 2 * R * T * gintK1 * B) * v ^ _
    3 + (A * gintK1 * B _
    - 3 * A * gintK3 * B - 2 * R * T * gIntK2 * B ^ 2 - A * B - R * T * _
    (gintK1 ^ 2) * B ^ 2) * v ^ 2 _
    + (-2 * A * gintK3 * gintK1 * B ^ 2 - 2 * R * T * gintK1 * gIntK2 * B ^ _
    3 + 2 * A * gintK3 * B ^ 2) _
    * v + A * gIntK2 * B ^ 3 + A * gintK3 * gintK1 * B ^ 3 - R * T * (gIntK2 _
    ^ 2) * B ^ 4 - _
    A * gintK3 * gIntK2 * B ^ 3
' ---- Code Body Ends ---- (Default)
Exit Function

FirstDerivateP_ErrorHandler:
  Err.Raise Err.Number

Exit_FirstDerivateP:

End Function
Private Function SecondDerivateP(T As Double, v As Double) As Double
On Error GoTo SecondDerivateP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim A As Double, B As Double, R As Double, rule As Integer
    MixingRules T, 0, WithDim, rule
    A = mParameter.A
    B = mParameter.B
    R = UnivGasConst
    SecondDerivateP = -2 * R * T * v ^ 6 + (6 * A - 6 * R * T * gintK1 * B) _
    * v ^ 5 + (6 * A * gintK1 * B _
    - 6 * R * T * gIntK2 * B ^ 2 - 12 * A * gintK3 * B - 6 * A * B - 6 * R * _
    T * (B ^ 2) * gintK1 ^ 2) _
    * v ^ 4 + (-16 * A * gintK3 * gintK1 * B ^ 2 + 2 * A * (B ^ 2) * gintK1 _
    ^ 2 - 2 * A * gIntK2 * B ^ 2 _
    - 2 * A * gintK1 * B ^ 2 - 12 * R * T * gintK1 * gIntK2 * B ^ 3 + 16 * A _
    * gintK3 * B ^ 2 - _
    2 * R * T * (B ^ 3) * gintK1 ^ 3 + 2 * A * B ^ 2) * v ^ 3 + (-6 * A * _
    gintK3 * (B ^ 3) * gintK1 ^ 2 _
    + 18 * A * gintK3 * gintK1 * B ^ 3 + 12 * A * gIntK2 * B ^ 3 - 6 * A * _
    gintK3 * B ^ 3 - _
    6 * R * T * gIntK2 * (B ^ 4) * gintK1 ^ 2 - 6 * A * gintK3 * gIntK2 * B _
    ^ 3 - 6 * R * T * (B ^ 4) _
    * gIntK2 ^ 2) * v ^ 2 + (-6 * A * gintK3 * gintK1 * B ^ 4 + 6 * A * _
    gintK1 * gIntK2 * B ^ 4 - _
    6 * A * gIntK2 * B ^ 4 - 6 * R * T * gintK1 * (B ^ 5) * gIntK2 ^ 2 - 6 * _
    A * gintK3 * gintK1 * _
    gIntK2 * B ^ 4 + 6 * A * gintK3 * (B ^ 4) * gintK1 ^ 2) * v _
    - 2 * A * (B ^ 5) * gIntK2 ^ 2 - 2 * A * gintK3 * (B ^ 5) * gintK1 ^ 2 _
    + 2 * A * gintK3 * gintK1 * gIntK2 * B ^ 5 - 2 * A * gintK1 * gIntK2 * B _
    ^ 5 - _
    2 * A * gintK3 * (B ^ 5) * gIntK2 ^ 2 - 2 * R * T * (B ^ 6) * gIntK2 ^ 3 _
    + 2 * A * gintK3 * gIntK2 * B ^ 5
' ---- Code Body Ends ---- (Default)
Exit Function

SecondDerivateP_ErrorHandler:
  Err.Raise Err.Number

Exit_SecondDerivateP:

End Function
Public Sub psCriticalProperties(ByRef tpsc As Double, ByRef vpsc As Double, _
ByRef ppsc As Double)
On Error GoTo psCriticalProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim f(1 To 2) As Double, i As Integer, n As Integer
    Dim j(1 To 2, 1 To 2) As Double, Converged As Boolean, rule As Integer
    
    Dim X(1 To 2) As Double
    Dim delta(1 To 2) As Double
    n = mvarProperties.Count
    X(1) = 0: X(2) = 0
    Converged = False
    MixingRules 300, 0, WithDim, rule
    For i = 1 To n
        X(1) = X(1) + mvarMolarFraction(i) * mvarProperties(i).Tc
        X(2) = mParameter.B / ((2 ^ (1 / 3)) - 1)
    Next
    Do
        DoEvents
        CalculateJacobianP X(1), X(2), j()
        EvaluateF X(), f()
        SolveAx 2, j(), f(), delta()
        Xnew X(), delta(), 2
        EvaluateF X(), f()
        If CheckFunConv(f(), 1E-20) = True Then Converged = True
        If CheckRootConv(delta(), 0.000000000000001) = True Then Converged = _
True
    Loop Until Converged = True
    tpsc = X(1): vpsc = X(2)
    ppsc = PWithDimensions(tpsc, X(2))
' ---- Code Body Ends ---- (Default)
Exit Sub

psCriticalProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_psCriticalProperties:

End Sub
Public Function CriticalProperties(ByRef Tc As Double, ByRef Pc As Double, _
ByRef vc As Double) As Boolean
On Error GoTo CriticalProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim n As Integer, T As Double, Th As Double, v1 As Double, v2 As Double
    Dim Sum As Double, i As Integer, temp As Double
    ReDim Dn(1 To mvarProperties.Count) As Double
    ReDim dnh(1 To mvarProperties.Count) As Double
    Dim f As Double, Fh As Double
    Dim h As Double
    Dim vH As Double
    Dim EPS As Double
    Dim rule As Integer
    
    EPS = 0.01
    n = mvarProperties.Count
    'Calculate Initial guess for T an  v
    For i = 1 To n
        Sum = Sum + mvarMolarFraction(i) * mvarProperties(i).Tc
    Next
    T = 1.5 * Sum: Th = T
    MixingRules T, 0, WithDim, rule
    v2 = 5 * mParameter.B
    Do
        DoEvents
        v1 = v2
        temp = v1
        h = EPS * temp
        vH = v1 + h
        h = vH - temp
        CP_Calculate_T T, v1, n, Dn()
        CP_Calculate_T Th, vH, n, dnh()
        f = CP_Calculate_C(T, v1, Dn(), n)
        Fh = CP_Calculate_C(Th, vH, dnh(), n)
        v2 = v1 - f / (FirstDerivate(f, Fh, h))
    Loop Until Abs(v2 - v1) / v1 < 0.1
    vc = v2
    Tc = T
    Pc = PWithDimensions(Tc, vc)
' ---- Code Body Ends ---- (Default)
Exit Function

CriticalProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_CriticalProperties:

End Function
Private Function CP_Calculate_T(ByRef T As Double, ByVal v As Double, ByVal _
n As Integer, ByRef Dn() As Double) As Boolean
On Error GoTo CP_Calculate_T_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ReDim Q(1 To n, 1 To n) As Double
    ReDim Qt(1 To n, 1 To n) As Double
    ReDim P(1 To n) As Integer
    ReDim Pt(1 To n) As Integer
    Dim Th As Double, h As Double, EPS As Double, temp As Double
    Dim T1 As Double, T2 As Double, f As Double, Fh As Double
    Dim d As Integer, dT As Integer, Sum As Double, MAXIT As Integer
    Dim l As Integer
    EPS = 0.001: MAXIT = 100
'Newton Rhapson to solve T. Reference AICHeJ September 1980 vol 26, No 5,
'pag 772
    l = 0: T2 = T
    Do
        l = l + 1
        T1 = T2: temp = T2
        h = EPS * temp: Th = temp + h: h = Th - temp
        CP_FillQ Q(), T1, v, n
        CP_FillQ Qt(), Th, v, n
        MatrizGauss n, 1, Q(), P(), d
        MatrizGauss n, 1, Qt(), Pt(), dT
        f = DeterDiag(n, Q(), P(), d)
        Fh = DeterDiag(n, Qt(), Pt(), dT)
        T2 = T1 - f / (FirstDerivate(f, Fh, h))
    Loop Until (Abs(T2 - T1) / T2) <= 0.01 Or Abs(f) < 0.001
    BackSub n, Q(), P(), Dn()
    Sum = VectorMod(Dn())
    Norm Dn(), Sum
    T = T2
' ---- Code Body Ends ---- (Default)
Exit Function

CP_Calculate_T_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_Calculate_T:

End Function
Private Function CP_FillQ(ByRef Q() As Double, ByVal T As Double, ByVal v As _
Double, n As Integer) As Boolean
On Error GoTo CP_FillQ_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, j As Integer
    For i = 1 To n
        For j = 1 To n
            Q(i, j) = (T / 100) * (CP_D_lnFugacity_nt(i, j, T, v, n))
        Next
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

CP_FillQ_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_FillQ:

End Function
Private Function AAij(ByVal i As Integer, ByVal j As Integer) As Double
On Error GoTo AAij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If VarType(mvarKij) = vbEmpty Then
        AAij = (Sqr(mParameteri(i).A * mParameteri(j).A))
    Else
        AAij = (1 - mvarKij(i, j)) * (Sqr(mParameteri(i).A * _
        mParameteri(j).A))
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

AAij_ErrorHandler:
  Err.Raise Err.Number

Exit_AAij:

End Function
Private Function CP_D_lnFugacity_nt(ByVal i As Integer, ByVal j As Integer, _
ByVal T As Double, ByVal v As Double, ByVal n As Integer) As Double
On Error GoTo CP_D_lnFugacity_nt_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Part1 As Double, Part2 As Double
    Dim TempMolarFraction() As Double, cont As Integer, P As Double
    Const h = 0.0001
    ReDim TempMolarFraction(n)
    CopyMemory TempMolarFraction(1), mvarMolarFraction(1), n * _
    Len(mvarMolarFraction(1))
    P = PWithDimensions(T, v)
    PartialFugacityCoeficientInSolution i, T, P, Part1
    Part1 = Log(Part1 * P * mvarMolarFraction(i))
    For cont = 1 To n
        If cont <> j Then: mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 + h): Else: mvarMolarFraction(cont) = (TempMolarFraction(cont) + h) / _
        (1 + h)
    Next
    P = PWithDimensions(T, v / (1 + h))
    PartialFugacityCoeficientInSolution i, T, P, Part2, P * (v / (1 + h)) / _
    (T * UnivGasConst)
    Part2 = Log(Part2 * P * mvarMolarFraction(i))
    CP_D_lnFugacity_nt = (Part2 - Part1) / h
    CopyMemory mvarMolarFraction(1), TempMolarFraction(1), n * _
    Len(TempMolarFraction(1))
' ---- Code Body Ends ---- (Default)
Exit Function

CP_D_lnFugacity_nt_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_D_lnFugacity_nt:

End Function
Private Function CP_Calculate_C(T As Double, v As Double, ByRef Dn() As _
Double, n As Integer) As Double
On Error GoTo CP_Calculate_C_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, rule As Integer, j As Integer, k As Integer, Sum As Double, Aux As _
    Double
    For k = 1 To n
        For j = 1 To n
            For i = 1 To n
                Sum = Sum + CP_SD_ln_Fug_nt(i, j, k, T, v, n) * Dn(i) * _
                Dn(j) * Dn(k)
            Next
        Next
    Next
    MixingRules T, 0, WithDim, rule
    With mParameter
        Aux = ((v - .B) / (2 * .B))
        CP_Calculate_C = Sum * Aux * Aux
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

CP_Calculate_C_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_Calculate_C:

End Function
Private Function CP_SD_ln_Fug_nt(ByVal i As Integer, ByVal j As Integer, _
ByVal k As Integer, ByVal T As Double, ByVal v As Double, ByVal n As _
Integer) As Double
On Error GoTo CP_SD_ln_Fug_nt_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Part1 As Double, Part2 As Double, Part3 As Double, Part4 As Double
    Dim TempMolarFraction() As Double, cont As Integer
    Const h = 0.0001
    ReDim TempMolarFraction(n)
    CopyMemory TempMolarFraction(1), mvarMolarFraction(1), n * _
    Len(mvarMolarFraction(1))
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 - 2 * h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) - 2 * _
        h) / (1 - 2 * h)
    Next
    Part1 = CP_D_lnFugacity_nt(i, j, T, v / (1 - 2 * h), n)
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 - h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) - h) / (1 _
        - h)
    Next
    Part2 = CP_D_lnFugacity_nt(i, j, T, v / (1 - h), n)
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 + h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) + h) / (1 _
        + h)
    Next
    Part3 = CP_D_lnFugacity_nt(i, j, T, v / (1 + h), n)
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 + 2 * h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) + 2 * _
        h) / (1 + 2 * h)
    Next
    Part4 = CP_D_lnFugacity_nt(i, j, T, v / (1 + 2 * h), n)
    CP_SD_ln_Fug_nt = (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h)
    CopyMemory mvarMolarFraction(1), TempMolarFraction(1), n * _
    Len(TempMolarFraction(1))
' ---- Code Body Ends ---- (Default)
Exit Function

CP_SD_ln_Fug_nt_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_SD_ln_Fug_nt:

End Function
Private Sub EvaluateF(X() As Double, ByRef f() As Double)
On Error GoTo EvaluateF_ErrorHandler

' ---- Code Body Starts ---- (Default)
    f(1) = -FirstDerivateP(X(1), X(2))
    f(2) = -SecondDerivateP(X(1), X(2))
' ---- Code Body Ends ---- (Default)
Exit Sub

EvaluateF_ErrorHandler:
  Err.Raise Err.Number

Exit_EvaluateF:

End Sub
Private Function Xnew(ByRef X() As Double, deltaX() As Double, ByVal n As _
Integer)
On Error GoTo Xnew_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        X(i) = X(i) + deltaX(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Xnew_ErrorHandler:
  Err.Raise Err.Number

Exit_Xnew:

End Function
Public Function ResidualProperties(ByVal T As Double, ByVal P As Double, _
ByVal Z As Double, ByRef HR As Double, ByRef SR As Double) As Boolean
On Error GoTo ResidualProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Iz As Double, rule As Integer, MixtureA As Double, MixtureB As Double, a__T As Double
    MixingRules T, P, DimLess, rule
    MixtureA = mParameter.A
    MixtureB = mParameter.B
   
    
        Iz = I_z(Z, gintK1, gIntK2, MixtureB)
   
    a__T = a_T(T)
    SR = (Log(Z - MixtureB) + MixtureA * Iz * a__T) * UnivGasConst
    HR = ((Z - 1) + MixtureA * Iz * (a__T + 1)) * UnivGasConst * T
' ---- Code Body Ends ---- (Default)
Exit Function

ResidualProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_ResidualProperties:

End Function
Private Function a_T(T As Double) As Double
On Error GoTo a_T_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim temp As Double, rule As Integer, Part1 As Double, Part2 As Double, Part3 As Double, _
    Part4 As Double
    
    Const h = 0.0001
    MixingRules T, 0, WithDim, rule
    temp = mParameter.A
    MixingRules T - 2 * h, 0, WithDim, rule
    Part1 = mParameter.A
    MixingRules (T - h), 0, WithDim, rule
    Part2 = mParameter.A
    MixingRules (T + h), 0, WithDim, rule
    Part3 = mParameter.A
    MixingRules (T + 2 * h), 0, WithDim, rule
    Part4 = mParameter.A
    a_T = T * (-1) * (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h * temp)
' ---- Code Body Ends ---- (Default)
Exit Function

a_T_ErrorHandler:
  Err.Raise Err.Number

Exit_a_T:

End Function
'Private Function b_T(T As Double) As Double
'    Dim temp As Double, Part1 As Double, Part2 As Double, Part3 As Double,
'Part4 As Double
'    Const h = 0.0001
'    MixingRules T, 0, WithDim
'    temp = mParameter.B
'    MixingRules T - 2 * h, 0, WithDim
'    Part1 = mParameter.B
'    MixingRules (T - h), 0, WithDim
'    Part2 = mParameter.B
'    MixingRules (T + h), 0, WithDim
'    Part3 = mParameter.B
'    MixingRules (T + 2 * h), 0, WithDim
'    Part4 = mParameter.B
'    b_T = T * (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h * temp)
'End Function

Private Function ai__bi(i As Integer, T As Double, a_i As Double, b_i As _
Double, ByVal n As Integer)
On Error GoTo ai__bi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Part1 As Double, Part2 As Double, Part3 As Double, Part4 As Double
    Dim Part5 As Double, Part6 As Double, Part7 As Double, Part8 As Double
    Dim TempMolarFraction() As Double, cont As Integer, rule As Integer
    
    Const h = 0.0001
    ReDim TempMolarFraction(n) As Double
    For cont = 1 To n
        TempMolarFraction(cont) = mvarMolarFraction(cont)
    Next
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 - 2 * h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) - 2 * h) / (1 _
            - 2 * h)
        End If
    Next
    MixingRules T, 0, WithDim, rule
    Part1 = mParameter.B * (1 - 2 * h)
    Part5 = mParameter.A * (1 - 2 * h)
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 - h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) - h) / (1 - _
            h)
        End If
    Next
    MixingRules T, 0, WithDim, rule
    Part2 = mParameter.B * (1 - h)
    Part6 = mParameter.A * (1 - h)
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 + h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) + h) / (1 + _
            h)
        End If
    Next
    MixingRules T, 0, WithDim, rule
    Part3 = mParameter.B * (1 + h)
    Part7 = mParameter.A * (1 + h)
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 + 2 * h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) + 2 * h) / (1 _
            + 2 * h)
        End If
    Next
    MixingRules T, 0, WithDim, rule
    Part4 = mParameter.B * (1 + 2 * h)
    Part8 = mParameter.A * (1 + 2 * h)
    b_i = (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h)
    a_i = (Part5 - 8 * Part6 + 8 * Part7 - Part8) / (12 * h)
    For cont = 1 To n
        mvarMolarFraction(cont) = TempMolarFraction(cont)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

ai__bi_ErrorHandler:
  Err.Raise Err.Number

Exit_ai__bi:

End Function
Public Sub Kij_ReadExperimentalData(ByVal FilePath As String, ByRef Data() _
As Double, ByRef datasets As Integer, ByRef Dato As Double, ByRef isoterm As _
Boolean)
On Error GoTo Kij_ReadExperimentalData_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim PrincipalStr As String, Aux As Integer, ii As Integer
    Dim SecondaryStr As String, i As Integer, Lines As Integer, jj As _
Integer
    Open FilePath For Input As #1
        ii = 0
        jj = 0
        Do While Not EOF(1)
            Line Input #1, PrincipalStr
            ii = ii + 1
            If ii = 2 Then
                If PrincipalStr = "T" Then
                    isoterm = True
                ElseIf PrincipalStr = "P" Then
                    isoterm = False
                End If
            End If
            If ii = 3 Then
                Dato = Filter.InternationalValue(PrincipalStr)
            End If
            If ii = 4 Then
                Lines = CInt(PrincipalStr)
                ReDim Data(1 To Lines, 1 To 3)
            End If
            If ii > 6 And ii <= (Lines + 6) Then
                jj = jj + 1
                For i = 1 To 2
                    PrincipalStr = Trim(PrincipalStr)
                    Aux = InStr(1, PrincipalStr, Chr(32), 1)
                    SecondaryStr = Mid(PrincipalStr, 1, Aux - 1)
                    PrincipalStr = Mid(PrincipalStr, Aux + 1, Len(PrincipalStr))
                    Data(jj, i) = Filter.InternationalValue(SecondaryStr)
                Next
                Data(jj, 1) = Data(jj, 1) * 100000#
                Data(jj, 3) = Filter.InternationalValue(PrincipalStr)
            End If

        Loop
    Close #1
    datasets = Lines
' ---- Code Body Ends ---- (Default)
Exit Sub

Kij_ReadExperimentalData_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_ReadExperimentalData:

End Sub
Public Function Kij_GoldenSearch(ax As Double, bx As Double, cx As Double, _
tol As Double, Xmin As Double, Dato As Double, datasets As Integer, ByRef _
Data() As Double, isoterm As Boolean)
On Error GoTo Kij_GoldenSearch_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Const R = 0.61803399
    Const C = 0.38196602
    Dim x0 As Double, x3 As Double, x1 As Double, x2 As Double, f1 As _
Double, f2 As Double
    x0 = ax
    x3 = cx
    If Abs(cx - bx) > Abs(bx - ax) Then
      x1 = bx
      x2 = bx + C * (cx - bx)
    Else
      x2 = bx
      x1 = bx - C * (bx - ax)
    End If
    f1 = Kij_Qf(x1, Data(), datasets, Dato, isoterm)
    f2 = Kij_Qf(x2, Data(), datasets, Dato, isoterm)
    While Abs(x3 - x0) > tol * (Abs(x1) + Abs(x2))
      DoEvents
      If f2 < f1 Then
        x0 = x1
        x1 = x2
        x2 = R * x1 + C * x3
        f1 = f2
        f2 = Kij_Qf(x2, Data(), datasets, Dato, isoterm)
      Else
        x3 = x2
        x2 = x1
        x1 = R * x2 + C * x0
        f2 = f1
        f1 = Kij_Qf(x1, Data(), datasets, Dato, isoterm)
      End If
    Wend
    If f1 < f2 Then
      Kij_GoldenSearch = f1
      Xmin = x1
    Else
      Kij_GoldenSearch = f2
      Xmin = x2
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

Kij_GoldenSearch_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_GoldenSearch:

End Function
Public Function Kij_BracketMin(ByRef ax As Double, ByRef bx As Double, ByRef _
cx As Double, ByRef fa As Double, ByRef fb As Double, ByRef fc As Double, _
datasets As Integer, Dato As Double, ByRef Data() As Double, isoterm As _
Boolean) As Double
On Error GoTo Kij_BracketMin_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Const GOLD = 1.618034
    Const GLIMIT = 100!
    Const TINY = 1E-20
    Dim dum As Double, DONE As Boolean, R As Double, Q As Double, U As _
    Double, Ulim As Double, FU As Double
    fa = Kij_Qf(ax, Data(), datasets, Dato, isoterm)
    fb = Kij_Qf(bx, Data(), datasets, Dato, isoterm)
    If fb > fa Then
      dum = ax
      ax = bx
      bx = dum
      dum = fb
      fb = fa
      fa = dum
    End If
    cx = bx + GOLD * (bx - ax)
    fc = Kij_Qf(cx, Data(), datasets, Dato, isoterm)
    Do
      DoEvents
      If fb < fc Then Exit Do
      DONE = -1
      R = (bx - ax) * (fb - fc)
      Q = (bx - cx) * (fb - fa)
      dum = Q - R
      If Abs(dum) < TINY Then dum = TINY
      U = bx - ((bx - cx) * Q - (bx - ax) * R) / (2! * dum)
      Ulim = bx + GLIMIT * (cx - bx)
      If (bx - U) * (U - cx) > 0! Then
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
        If FU < fc Then
          ax = bx
          fa = fb
          bx = U
          fb = FU
          Exit Function
        ElseIf FU > fb Then
          cx = U
          fc = FU
          Exit Function
        End If
        U = cx + GOLD * (cx - bx)
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
      ElseIf (cx - U) * (U - Ulim) > 0! Then
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
        If FU < fc Then
          bx = cx
          cx = U
          U = cx + GOLD * (cx - bx)
          fb = fc
          fc = FU
          FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
        End If
      ElseIf (U - Ulim) * (Ulim - cx) >= 0! Then
        U = Ulim
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
      Else
        U = cx + GOLD * (cx - bx)
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
      End If
      If DONE Then
        ax = bx
        bx = cx
        cx = U
        fa = fb
        fb = fc
        fc = FU
      Else
        DONE = 0
      End If
    Loop While Not DONE
' ---- Code Body Ends ---- (Default)
Exit Function

Kij_BracketMin_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_BracketMin:

End Function
Public Function Kij_Qf(mmkij As Double, ByRef Data() As Double, datasets As _
Integer, Dato As Double, isoterm As Boolean) As Double
On Error GoTo Kij_Qf_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Suma As Double, i As Integer, Fig1 As Double, Fil1 As Double, Fig2 _
    As Double, Fil2 As Double
    ReDim mvarMolarFraction(1 To 2) As Double
    ReDim mvarKij(1 To 2, 1 To 2) As Double

        mvarKij(1, 1) = 0
        mvarKij(1, 2) = mmkij
        mvarKij(2, 1) = mmkij
        mvarKij(2, 2) = 0

        Suma = 0
        For i = 1 To datasets
            DoEvents
            'liquido
            mvarMolarFraction(1) = Data(i, 2)
            mvarMolarFraction(2) = 1 - Data(i, 2)
            mvarPhaseOfMixture = PhaseIDLiquid
            If isoterm = True Then
                PartialFugacityCoeficientInSolution 1, Dato, Data(i, 1), Fil1
                PartialFugacityCoeficientInSolution 2, Dato, Data(i, 1), Fil2
            Else
                PartialFugacityCoeficientInSolution 1, Data(i, 1), Dato, Fil1
                PartialFugacityCoeficientInSolution 2, Data(i, 1), Dato, Fil2
            End If
            Fil1 = mvarMolarFraction(1) * Fil1
            Fil2 = mvarMolarFraction(2) * Fil2
            'vapor
            mvarMolarFraction(1) = Data(i, 3)
            mvarMolarFraction(2) = 1 - Data(i, 3)
            mvarPhaseOfMixture = PhaseIDvapor
            If isoterm = True Then
                PartialFugacityCoeficientInSolution 1, Dato, Data(i, 1), Fig1
                PartialFugacityCoeficientInSolution 2, Dato, Data(i, 1), Fig2
            Else
                PartialFugacityCoeficientInSolution 1, Data(i, 1), Dato, Fig1
                PartialFugacityCoeficientInSolution 2, Data(i, 1), Dato, Fig2
            End If
            Fig1 = mvarMolarFraction(1) * Fig1
            Fig2 = mvarMolarFraction(2) * Fig2
            Suma = Suma + (Abs(Fig1 - Fil1) / Fig1) + (Abs(Fig2 - Fil2) / Fil2)
        Next
        Kij_Qf = Suma
' ---- Code Body Ends ---- (Default)
Exit Function

Kij_Qf_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_Qf:

End Function
Private Sub C_cal(ByRef C As Double)
    Select Case mvarModel
        Case PR1976_Qbicp, PRL1997_Qbicp, PRATmng1997_Qbicp, _
        PRMmn1989_Qbicp, PROL1998_Qbicp, RP1978_Qbicp, PRSV1986_Qbicp
        
        C = -0.62322540138
        
        Case RK1949_Qbicp, RKS1972_Qbicp, RKSL1997_Qbicp, _
        RKSGD1978_Qbicp, RKSmn1980_Qbicp, RKSATmn1995_Qbicp, RKOL1998_Qbicp
        
        C = -0.69314718056
        
        Case VdW1870_Qbicp, VdWAda1984_Qbicp, VdWVald1989_Qbicp, _
        VdWOL1998_Qbicp
        
        C = -1
        
        Case Berth1899_Qbicp
    
    End Select

End Sub
Private Sub Parametros(Component As Integer, Temperature As Double, Pressure As Double)

End Sub


