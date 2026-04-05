VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsLVE"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Attribute VB_Ext_KEY = "Member0" ,"clsAllProps"
Option Explicit
    Private DBO As New clsDataBase
    Public Enum TadipLiquidModels
        idealsolution = 25
        PR1976_Qbicl = 0
        RK1949_Qbicl = 1
        RKS1972_Qbicl = 2
        VdW1870_Qbicl = 3
        PRL1997_Qbicl = 4
        RKSL1997_Qbicl = 5
        RKSGD1978_Qbicl = 6
        RP1978_Qbicl = 7
        Berth1899_Qbicl = 8
        VdWAda1984_Qbicl = 9
        VdWVald1989_Qbicl = 10
        RKSmn1980_Qbicl = 11
        RKSATmn1995_Qbicl = 12
        PRATmng1997_Qbicl = 13
        PRMmn1989_Qbicl = 14
        PRSV1986_Qbicl = 15
        VdWOL1998_Qbicl = 16
        RKOL1998_Qbicl = 17
        PROL1998_Qbicl = 18
        vanLaar = 21
        Wilson = 22
        ScatchardHild = 23
        Margules = 24
    End Enum
    Public Enum TadipVaporModel
        IdealGasv = -2
        PR1976_Qbicv = 0
        RK1949_Qbicv = 1
        RKS1972_Qbicv = 2
        VdW1870_Qbicv = 3
        PRL1997_Qbicv = 4
        RKSL1997_Qbicv = 5
        RKSGD1978_Qbicv = 6
        RP1978_Qbicv = 7
        Berth1899_Qbicv = 8
        VdWAda1984_Qbicv = 9
        VdWVald1989_Qbicv = 10
        RKSmn1980_Qbicv = 11
        RKSATmn1995_Qbicv = 12
        PRATmng1997_Qbicv = 13
        PRMmn1989_Qbicv = 14
        PRSV1986_Qbicv = 15
        VdWOL1998_Qbicv = 16
        RKOL1998_Qbicv = 17
        PROL1998_Qbicv = 18
        virial = -1
    End Enum
    Private Enum TypeCalculation
        DewP = 1
        DewT = 2
        BubP = 3
        BubT = 4
        Flash = 5
    End Enum
    Private Type DatosFlash
        T As Double
        P As Double
        Nl As Double
        Nv As Double
        Nt As Double
        Alpha As Double
        Beta As Double
        Alphac As Double
        Betac As Double
    End Type
    Private mvarLiquidFlow As clsFlow
    Private mvarVaporFlow As clsFlow
    Private mvarFeedFlow As clsFlow
    Private LiquidSolver As Object
    Private VaporSolver As Object
    Private LiquidS As Object
   
    Private SatPressureCalc As New clsSatPressure
    Private mvarMixture As clsAllProps
    Private mvarBoolgammaphi As Boolean
    Private mvarTolerances As New clsTolerances
    Private mvarReferenceState As New clsReferencesSt
Public Property Set ReferenceState(ByVal vData As Object)
On Error GoTo ReferenceState_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarReferenceState = vData
' ---- Code Body Ends ---- (Default)
Exit Property

ReferenceState_ErrorHandler:
  Err.Raise Err.Number

Exit_ReferenceState:

End Property
Public Property Get ReferenceState() As clsReferencesSt
On Error GoTo ReferenceState_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarReferenceState Is Nothing Then
        Set mvarReferenceState = New clsReferencesSt
    End If
    Set ReferenceState = mvarReferenceState
' ---- Code Body Ends ---- (Default)
Exit Property

ReferenceState_ErrorHandler:
  Err.Raise Err.Number

Exit_ReferenceState:

End Property
Public Property Set Tolerances(ByVal vData As Object)
On Error GoTo Tolerances_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarTolerances = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Tolerances_ErrorHandler:
  Err.Raise Err.Number

Exit_Tolerances:

End Property
Public Property Get Tolerances() As clsTolerances
On Error GoTo Tolerances_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarTolerances Is Nothing Then
        Set mvarTolerances = New clsTolerances
    End If
    Set Tolerances = mvarTolerances
' ---- Code Body Ends ---- (Default)
Exit Property

Tolerances_ErrorHandler:
  Err.Raise Err.Number

Exit_Tolerances:

End Property
Private Function NR_CalculateGk(datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, ByRef Ni() As Double, ByRef Gk() As Double, n As Integer)
On Error GoTo NR_CalculateGk_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim k As Integer, sumX As Double, sumY As Double
    Dim i As Integer
    ReDim f(1 To n) As Double
    k = 0
    With datos
        For i = 1 To n
            k = k + 1
            Gk(k) = .Nl * X(i) + .Nv * y(i) - Ni(i)
        Next
        NR_UploadVaporMolarFraction y(), n
        NR_UploadLiquidMolarFraction X(), n
        NR_EvaluateFi f(), X(), y(), .T, .P, n
        For i = 1 To n
            k = k + 1
            Gk(k) = f(i)
        Next
        k = k + 1
        SumFrac X(), sumX
        SumFrac y(), sumY
        Gk(k) = sumX - sumY
        k = k + 1
        Gk(k) = .Nv + .Nl - .Nt
        k = k + 1
        Gk(k) = (.T - .Alphac)
        k = k + 1
        Gk(k) = .Nv - .Betac
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

NR_CalculateGk_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_CalculateGk:

End Function
Private Function NR_CalculateGkBDT(datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, ByRef Ni() As Double, ByRef Gk() As Double, n As Integer)
On Error GoTo NR_CalculateGkBDT_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim k As Integer, sumX As Double, sumY As Double
    Dim i As Integer
    ReDim f(1 To n) As Double
    k = 0
    For i = 1 To n
        k = k + 1
        Gk(k) = datos.Nl * X(i) + datos.Nv * y(i) - Ni(i)
    Next
    NR_UploadVaporMolarFraction y(), n
    NR_UploadLiquidMolarFraction X(), n
    NR_EvaluateFi f(), X(), y(), datos.T, datos.P, n
    For i = 1 To n
        k = k + 1
        Gk(k) = f(i)
    Next
    k = k + 1
    SumFrac X(), sumX
    SumFrac y(), sumY
    Gk(k) = sumX - sumY
    k = k + 1
    Gk(k) = datos.Nv + datos.Nl - datos.Nt
    k = k + 1
    Gk(k) = (datos.P - datos.Alphac)
    k = k + 1
    Gk(k) = datos.Nv - datos.Betac
' ---- Code Body Ends ---- (Default)
Exit Function

NR_CalculateGkBDT_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_CalculateGkBDT:

End Function
Private Function NR_JacobianMatrix(ByVal n As Integer, ByRef Jac() As Double, datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, ByRef Ni() As Double)
On Error GoTo NR_JacobianMatrix_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ReDim tempX(1 To n) As Double
    ReDim tempY(1 To n) As Double
    ReDim Fh(1 To 2 * n + 4) As Double
    ReDim f(1 To 2 * n + 4) As Double
    Dim i As Integer, j As Integer, k As Integer
    Dim h As Double
    Dim tempDat As DatosFlash
    Const EPS = 0.000001
    k = 2 * n + 4
    NR_CalculateGk datos, X(), y(), Ni(), f(), n
    For j = 1 To k
        tempDat = datos
        CopyX X(), tempX()
        CopyX y(), tempY()
        NR_Xkh tempDat, tempX(), tempY(), j, EPS, n, h
        NR_CalculateGk tempDat, tempX(), tempY(), Ni(), Fh(), n
        For i = 1 To k
            Jac(i, j) = (Fh(i) - f(i)) / h
        Next
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_JacobianMatrix_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_JacobianMatrix:

End Function
Private Function NR_JacobianMatrixBDT(ByVal n As Integer, ByRef Jac() As Double, datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, ByRef Ni() As Double)
On Error GoTo NR_JacobianMatrixBDT_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ReDim tempX(1 To n) As Double
    ReDim tempY(1 To n) As Double
    ReDim Fh(1 To 2 * n + 4) As Double
    ReDim f(1 To 2 * n + 4) As Double
    Dim i As Integer, j As Integer, k As Integer
    Dim h As Double
    Dim tempDat As DatosFlash
    Const EPS = 0.000001
    k = 2 * n + 4
    NR_CalculateGkBDT datos, X(), y(), Ni(), f(), n
    For j = 1 To k
        tempDat = datos
        CopyX X(), tempX()
        CopyX y(), tempY()
        NR_Xkh tempDat, tempX(), tempY(), j, EPS, n, h
        NR_CalculateGkBDT tempDat, tempX(), tempY(), Ni(), Fh(), n
        For i = 1 To k
            Jac(i, j) = (Fh(i) - f(i)) / h
        Next
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_JacobianMatrixBDT_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_JacobianMatrixBDT:

End Function
Private Function NR_Xkh(ByRef datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, ByVal k As Integer, ByVal EPS As Double, ByVal n As Integer, ByRef h As Double)
On Error GoTo NR_Xkh_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Temp As Double
    With datos
        Select Case k
            Case 1
                Temp = .T
                h = EPS * Abs(Temp)
                If h = 0 Then
                    h = EPS
                End If
                .T = Temp + h
                h = Abs(.T - Temp)
            Case 2
                Temp = .P
                h = EPS * Abs(Temp)
                If h = 0 Then
                    h = EPS
                End If
                .P = Temp + h
                h = Abs(Temp - .P)
            Case 3
                Temp = .Nl
                h = EPS * Abs(Temp)
                If h = 0 Then
                    h = EPS
                End If
                .Nl = Temp + h
                h = Abs(Temp - .Nl)
            Case 4
                Temp = datos.Nv
                h = EPS * Abs(Temp)
                If h = 0 Then
                    h = EPS
                End If
                .Nv = Abs(Temp + h)
            Case Is > 4
                If k > 4 And k <= n + 4 Then
                    Temp = X(k - 4)
                    h = EPS * Abs(Temp)
                    If h = 0 Then
                        h = EPS
                    End If
                    X(k - 4) = Temp + h
                    h = Abs(X(k - 4) - Temp)
                Else
                    Temp = y(k - 4 - n)
                    h = EPS * Abs(Temp)
                    If h = 0 Then
                        h = EPS
                    End If
                    y(k - 4 - n) = Temp + h
                    h = Abs(y(k - 4 - n) - Temp)
                End If
        End Select
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

NR_Xkh_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_Xkh:

End Function
Private Function CalculatePoynting(ByVal i As Integer, ByVal T As Double, ByVal P As Double)
On Error GoTo CalculatePoynting_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim PureSolver As clsQbics, PrSati As Double, Zg As Double, Zf As Double, _
    Tr As Double, PhiSat As Double, SatP As Double
    Set PureSolver = New clsQbics
    PureSolver.AcentricFactor = mvarMixture.Properties(i).w
    PureSolver.Model = mvarMixture.VaporModel
    Parametro_EDEC_Modificadas T, i, SatP
    PureSolver.PsatAtTr = SatP / mvarMixture.Properties(i).Pc
    Tr = T / mvarMixture.Properties(i).Tc
    PureSolver.Zg_calc Tr, PrSati, Zg, Zf
    PureSolver.FugacityCoeficient Tr, PrSati, Zg, PhiSat
    CalculatePoynting = Exp(Zf * ((P / (PrSati * mvarMixture.Properties(i).Pc)) - 1))
    Set PureSolver = Nothing
' ---- Code Body Ends ---- (Default)
Exit Function

CalculatePoynting_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculatePoynting:

End Function
Private Function CalculatePhiSati(ByVal i As Integer, ByVal T As Double) As Double
On Error GoTo CalculatePhiSati_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim PureSolver As Object
    Dim PrSati As Double, Zg As Double, Zf As Double, Tr As Double, PhiSat As Double
    Dim PSat As Double
    Select Case mvarMixture.VaporModel
        Case Is = -1
            Set PureSolver = New clsVirial
            PureSolver.AcentricFactor = mvarMixture.Properties(i).w
            Tr = T / mvarMixture.Properties(i).Tc
            SatPressureCalc.SatPressure i, T, PSat
            PrSati = PSat / mvarMixture.Properties(i).Pc
            CalculatePhiSati = PureSolver.CalculateFugacity(Tr, PrSati)
        Case Else
            Set PureSolver = New clsQbics
            PureSolver.AcentricFactor = mvarMixture.Properties(i).w
            PureSolver.ZRA = mvarMixture.Properties(i).ZRA
            PureSolver.PsatAtTr = mvarMixture.Properties(i).PsatAtTr
            PureSolver.Pc = mvarMixture.Properties(i).Pc
            PureSolver.Tc = mvarMixture.Properties(i).Tc
            PureSolver.Model = mvarMixture.VaporModel
            Tr = T / mvarMixture.Properties(i).Tc
            SatPressureCalc.SatPressure i, T, PSat
            PrSati = PSat / mvarMixture.Properties(i).Pc
            PureSolver.PsatAtTr = PrSati
            PureSolver.Zg_calc Tr, PrSati, Zg, Zf
            PureSolver.FugacityCoeficient Tr, PrSati, Zg, PhiSat
            CalculatePhiSati = PhiSat
    End Select
    Set PureSolver = Nothing
' ---- Code Body Ends ---- (Default)
Exit Function

CalculatePhiSati_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculatePhiSati:

End Function
Public Property Get Mixture() As clsAllProps
On Error GoTo Mixture_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarMixture Is Nothing Then
        Set mvarMixture = New clsAllProps
    End If
    Set Mixture = mvarMixture
' ---- Code Body Ends ---- (Default)
Exit Property

Mixture_ErrorHandler:
  Err.Raise Err.Number

Exit_Mixture:

End Property
Public Property Set Mixture(vData As clsAllProps)
On Error GoTo Mixture_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarMixture = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Mixture_ErrorHandler:
  Err.Raise Err.Number

Exit_Mixture:

End Property
Public Property Set LiquidFlow(vData As clsFlow)
On Error GoTo LiquidFlow_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarLiquidFlow = vData
' ---- Code Body Ends ---- (Default)
Exit Property

LiquidFlow_ErrorHandler:
  Err.Raise Err.Number

Exit_LiquidFlow:

End Property
Public Property Set FeedFlow(vData As clsFlow)
On Error GoTo FeedFlow_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarFeedFlow = vData
' ---- Code Body Ends ---- (Default)
Exit Property

FeedFlow_ErrorHandler:
  Err.Raise Err.Number

Exit_FeedFlow:

End Property
Public Property Set VaporFlow(vData As clsFlow)
On Error GoTo VaporFlow_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarVaporFlow = vData
' ---- Code Body Ends ---- (Default)
Exit Property

VaporFlow_ErrorHandler:
  Err.Raise Err.Number

Exit_VaporFlow:

End Property
Public Property Get LiquidFlow() As clsFlow
On Error GoTo LiquidFlow_ErrorHandler

' ---- Code Body Starts ---- (Default)
   If mvarLiquidFlow Is Nothing Then
        Set mvarLiquidFlow = New clsFlow
   End If
   Set LiquidFlow = mvarLiquidFlow
' ---- Code Body Ends ---- (Default)
Exit Property

LiquidFlow_ErrorHandler:
  Err.Raise Err.Number

Exit_LiquidFlow:

End Property
Public Property Get FeedFlow() As clsFlow
On Error GoTo FeedFlow_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarFeedFlow Is Nothing Then
        Set mvarFeedFlow = New clsFlow
    End If
    Set FeedFlow = mvarFeedFlow
' ---- Code Body Ends ---- (Default)
Exit Property

FeedFlow_ErrorHandler:
  Err.Raise Err.Number

Exit_FeedFlow:

End Property
Public Property Get VaporFlow() As clsFlow
On Error GoTo VaporFlow_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarVaporFlow Is Nothing Then
        Set mvarVaporFlow = New clsFlow
    End If
    Set VaporFlow = mvarVaporFlow
' ---- Code Body Ends ---- (Default)
Exit Property

VaporFlow_ErrorHandler:
  Err.Raise Err.Number

Exit_VaporFlow:

End Property
Private Sub Class_Terminate()
On Error GoTo Class_Terminate_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarLiquidFlow = Nothing
    Set mvarVaporFlow = Nothing
    Set mvarFeedFlow = Nothing
    Set mvarMixture = Nothing
    Set DBO = Nothing
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Terminate_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Terminate:

End Sub
Public Function BubblePointTemperature()
On Error GoTo BubblePointTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim n As Integer, h As Double, S As Double
    Dim i As Integer
    Dim Tini As Double
    n = mvarMixture.NumberOfComponents
    ReDim y(1 To n) As Double
    DimSolverObjects n
'El algoritmo de HP (del ingles altas presiones ) siempre va a ser utilizado para phi-phi
    If mvarBoolgammaphi Then
        RaoultFirstAprox BubT, n
        Tini = mvarFeedFlow.Temperature
        For i = 1 To n
            y(i) = mvarVaporFlow.MolarFraction(i)
        Next
        Ekilib_BubblePointTemperature n, Tini, y()
'En este punto ya se tiene la temperatura definitiva
    Else
        BubblePointTemperatureHP n
    End If
    FillVaporFraction BubT
    With mvarFeedFlow
        Enthalpy_Entropy n, .Temperature, .Pressure, .VaporFraction, h, S
        .Enthalpy = h
        .Entropy = S
    End With
'    ClearSolverObjects
' ---- Code Body Ends ---- (Default)
Exit Function

BubblePointTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_BubblePointTemperature:

End Function
Private Function BubblePointTemperatureHP(ByVal n As Integer)
On Error GoTo BubblePointTemperatureHP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tc As Double, Pc As Double, vc As Double
    Dim Trivial As Boolean, Tini As Double
    Dim myFinalP As Double, DeltaP As Double
    Dim P1 As Double, P2 As Double, P3 As Double
    Dim T1 As Double, T2 As Double, T3 As Double
    Dim deriv1 As Double, deriv1a As Double, deriv2 As Double
    Dim flagExit As Boolean, i As Integer
    ReDim myy(1 To n) As Double, y(1 To n) As Double, X(1 To n) As Double
    RaoultFirstAprox BubT, n
    Tini = mvarFeedFlow.Temperature
    For i = 1 To n
        y(i) = mvarVaporFlow.MolarFraction(i)
        X(i) = mvarLiquidFlow.MolarFraction(i)
    Next
    Ekilib_BubblePointTemperature n, Tini, y()
    For i = 1 To n
        y(i) = mvarVaporFlow.MolarFraction(i)
    Next
    Trivial = Compara(X(), y(), mvarTolerances.TolMolarFraction)
    If Trivial = True Then
        For i = 1 To n
            LiquidSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            myy(i) = mvarFeedFlow.MolarFraction(i)
        Next
        'liquidSolver.Properties Tc, Pc, vc
        LiquidSolver.psCriticalProperties Tc, vc, Pc
        myFinalP = mvarFeedFlow.Pressure
        P1 = 0.98 * Pc
        DeltaP = (myFinalP - P1) / 100
        mvarFeedFlow.Pressure = P1
        RaoultFirstAprox BubT, n
        T1 = mvarFeedFlow.Temperature
        For i = 1 To n: myy(i) = mvarVaporFlow.MolarFraction(i): Next
        NR_BubblePointTemperature T1, myy(), n
        deriv1 = 0
        T1 = mvarFeedFlow.Temperature
        P2 = P1 + DeltaP
        T2 = T1 + deriv1 * DeltaP
        mvarFeedFlow.Pressure = P2
        For i = 1 To n: myy(i) = mvarVaporFlow.MolarFraction(i): Next
        NR_BubblePointTemperature T2, myy(), n
        T2 = mvarFeedFlow.Temperature
        deriv2 = 0
        Do While Not flagExit
            DoEvents
            P3 = P2 + DeltaP
            T3 = T2 + DeltaP * (deriv1 + deriv2 * DeltaP)
            mvarFeedFlow.Pressure = P3
            For i = 1 To n: myy(i) = mvarVaporFlow.MolarFraction(i): Next
            NR_BubblePointTemperature T3, myy(), n
            T3 = mvarFeedFlow.Temperature
            deriv1a = deriv1
            deriv1 = (T3 - T2) / (P3 - P2)
            deriv2 = (deriv1 - deriv1a) / (P3 - P1)
            P1 = P2
            T1 = T2
            P2 = P3
            T2 = T3
            If Abs(P3 - myFinalP) / myFinalP < 0.0000001 Then flagExit = True
        Loop
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

BubblePointTemperatureHP_ErrorHandler:
  Err.Raise Err.Number

Exit_BubblePointTemperatureHP:

End Function
Public Function DewPointTemperature()
On Error GoTo DewPointTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim n As Integer, h As Double, S As Double
    Dim i As Integer
    Dim Tini As Double
    n = mvarMixture.NumberOfComponents
    ReDim X(1 To n) As Double
    DimSolverObjects n
    If mvarBoolgammaphi Then
        RaoultFirstAprox DewT, n
        Tini = mvarFeedFlow.Temperature
        For i = 1 To n
            X(i) = mvarLiquidFlow.MolarFraction(i)
        Next
        Ekilib_DewPointTemperature n, Tini, X()
    Else
        DewPointTemperatureHP n
    End If
    FillVaporFraction DewT
    With mvarFeedFlow
        Enthalpy_Entropy n, .Temperature, .Pressure, .VaporFraction, h, S
        .Enthalpy = h
        .Entropy = S
    End With
'    ClearSolverObjects
' ---- Code Body Ends ---- (Default)
Exit Function

DewPointTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_DewPointTemperature:

End Function
Public Function DewPointPressure()
On Error GoTo DewPointPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim n As Integer, h As Double, S As Double
    Dim i As Integer
    Dim Pini As Double
    n = mvarMixture.NumberOfComponents
    ReDim X(1 To n) As Double
    DimSolverObjects n
    If mvarBoolgammaphi Then
        ExG mvarFeedFlow.Temperature, n
        RaoultFirstAprox DewP, n
        Pini = mvarFeedFlow.Pressure
        For i = 1 To n
            X(i) = mvarLiquidFlow.MolarFraction(i)
        Next
        Ekilib_DewPointPressure n, Pini, X()
    Else
       DewpointPressureHP n
    End If
    FillVaporFraction DewP
    With mvarFeedFlow
        Enthalpy_Entropy n, .Temperature, .Pressure, .VaporFraction, h, S
        .Enthalpy = h
        .Entropy = S
    End With
'    FillVaporFraction dewP
'    ClearSolverObjects
' ---- Code Body Ends ---- (Default)
Exit Function

DewPointPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_DewPointPressure:

End Function
Private Function NR_DownloadX(ByRef Xanswer() As Double, ByRef datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, n As Integer)
On Error GoTo NR_DownloadX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim k As Integer, i As Integer
    k = 2 * n + 4
    With datos
        For i = 1 To k
            Select Case i
                Case 1
                    .T = Xanswer(i)
                Case 2
                    .P = Xanswer(i)
                Case 3
                    .Nl = Xanswer(i)
                Case 4
                    .Nv = Xanswer(i)
                Case Is > 4
                    If i > 4 And i <= n + 4 Then
                        X(i - 4) = Xanswer(i)
                    Else
                        y(i - 4 - n) = Xanswer(i)
                    End If
            End Select
        Next
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

NR_DownloadX_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_DownloadX:

End Function
Private Function NR_UploadX(ByRef Xanswer() As Double, ByRef datos As DatosFlash, ByRef X() As Double, ByRef y() As Double, n As Integer)
On Error GoTo NR_UploadX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim k As Integer, i As Integer
    k = 2 * n + 4
    With datos
        For i = 1 To k
            Select Case i
                Case 1
                    Xanswer(i) = .T
                Case 2
                    Xanswer(i) = .P
                Case 3
                    Xanswer(i) = .Nl
                Case 4
                    Xanswer(i) = .Nv
                Case Is > 4
                    If i > 4 And i <= n + 4 Then
                        Xanswer(i) = X(i - 4)
                    Else
                        Xanswer(i) = y(i - 4 - n)
                    End If
            End Select
        Next
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

NR_UploadX_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_UploadX:

End Function
Private Function NR_Xnew(ByRef X() As Double, deltaX() As Double)
On Error GoTo NR_Xnew_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To UBound(X())
        X(i) = X(i) + deltaX(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_Xnew_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_Xnew:

End Function
Public Function MonocompTP(ByVal Dato As Double, ByRef Zf As Double, ByRef Zg As Double, i As Integer, tipo As Boolean) As Double
On Error GoTo MonocompTP_ErrorHandler
' ---- Code Body Starts ---- (Default)
'Esta funcion sirve para hallar la Tsat o Psat de un solo componente
'en caso de ec cubica devuelve Zf y Zg
    Dim dummy1 As Double, Solve As Object
    Select Case mvarMixture.VaporModel
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
        RP1978_Qbic, Berth1899_Qbic
            Set Solve = New clsQbics
            With Solve
                .AcentricFactor = mvarMixture.Properties(i).w
                .m = mvarMixture.Properties(i).m
                .n = mvarMixture.Properties(i).n
                .g = mvarMixture.Properties(i).g
                .PRSVK1 = mvarMixture.Properties(i).K1
                .Zc = mvarMixture.Properties(i).Zc
                .PsatAtTr = mvarMixture.Properties(i).PsatAtTr
                .Model = mvarMixture.VaporModel
                If tipo = True Then
                    Dato = Dato / mvarMixture.Properties(i).Tc
                    .MaxwellTestGivenTr Dato, dummy1, Zg, Zf
                    MonocompTP = dummy1 * mvarMixture.Properties(i).Pc
                Else
                    Dato = Dato / mvarMixture.Properties(i).Pc
                    .MaxwellTestGivenPr Dato, dummy1, Zg, Zf
                    MonocompTP = dummy1 * mvarMixture.Properties(i).Tc
                End If
            End With
        Case Else
            FillSatPressureSolver mvarMixture.NumberOfComponents
            If tipo = True Then
                SatPressureCalc.SatPressure i, Dato, dummy1
                MonocompTP = dummy1
            Else
                SatPressureCalc.SatTemperature i, Dato, dummy1
                MonocompTP = dummy1
            End If
        End Select
        'select case para saber que raiz de liquido tomar
' ---- Code Body Ends ---- (Default)
Exit Function
MonocompTP_ErrorHandler:
  Err.Raise Err.Number
Exit_MonocompTP:
End Function
Public Function MonocompHV(ByVal Z As Double, ByVal T As Double, ByVal P As Double, ByVal i As Integer) As Double
On Error GoTo MonocompHV_ErrorHandler

' ---- Code Body Starts ---- (Default)
'Esta función sirve para calcular la entalpia de vapor de un solo componente,datos Z T P
Dim Tr As Double, Pr As Double, HR As Double, Aux As Double
Dim Coef() As Double, j As Integer, SR As Double

    Select Case mvarMixture.VaporModel
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, RP1978_Qbic, Berth1899_Qbic
            Dim Solve As New clsQbics
            With Solve
                    .Pc = mvarMixture.Properties(i).Pc
                    .Tc = mvarMixture.Properties(i).Tc
                    .AcentricFactor = mvarMixture.Properties(i).w
                    .m = mvarMixture.Properties(i).m
                    .n = mvarMixture.Properties(i).n
                    .g = mvarMixture.Properties(i).g
                    .PRSVK1 = mvarMixture.Properties(i).K1
                    .Zc = mvarMixture.Properties(i).Zc
                    .PsatAtTr = mvarMixture.Properties(i).PsatAtTr
                    .Model = mvarMixture.VaporModel
                    Tr = T / mvarMixture.Properties(i).Tc
                    Pr = P / mvarMixture.Properties(i).Pc
                    .ResidualEnthalpy Tr, Pr, Z, HR
                    HR = UnivGasConst * T * HR
            End With
        Case virial
            Dim solver As New clsVirial
            With solver
                .AcentricFactor = mvarMixture.Properties(i).w
                Tr = T / mvarMixture.Properties(i).Tc
                Pr = P / mvarMixture.Properties(i).Pc
                Z = .CalculateZ(Tr, Pr)
                .HR_SR Tr, Pr, HR, SR
                HR = UnivGasConst * T * HR
            End With
        Case Else
            HR = 0
        End Select
        
        ReDim Coef(0 To mvarMixture.CpConsts(i).NbCoef)
        For j = 0 To mvarMixture.CpConsts(i).NbCoef
            Coef(j) = mvarMixture.CpConsts(i).Coef(j)
        Next
        With mvarReferenceState
            MonocompHV = .Ho + HR + Aux + DBO.FunctionIntegration(.TTo, T, 0, mvarMixture.CpConsts(i).NbFormula, Coef())
        End With
' ---- Code Body Ends ---- (Default)
Exit Function
MonocompHV_ErrorHandler:
  Err.Raise Err.Number
Exit_MonocompHV:
End Function
Public Function MonocompHL(ByVal Z As Double, ByVal T As Double, ByVal P As Double, ByVal i As Integer) As Double
On Error GoTo MonocompHL_ErrorHandler
' ---- Code Body Starts ---- (Default)
'Esta función sirve para calcular la entalpia de vapor de un solo componente,datos Z T P
Dim Tr As Double, Pr As Double, HR As Double, Aux As Double
Dim Coef() As Double, j As Integer, SR As Double, Zg As Double
    Select Case mvarMixture.Liquid1Model
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, RP1978_Qbic, Berth1899_Qbic
            Dim Solve As New clsQbics
            With Solve
                    .Pc = mvarMixture.Properties(i).Pc
                    .Tc = mvarMixture.Properties(i).Tc
                    .AcentricFactor = mvarMixture.Properties(i).w
                    .m = mvarMixture.Properties(i).m
                    .n = mvarMixture.Properties(i).n
                    .g = mvarMixture.Properties(i).g
                    .PRSVK1 = mvarMixture.Properties(i).K1
                    .Zc = mvarMixture.Properties(i).Zc
                    .PsatAtTr = mvarMixture.Properties(i).PsatAtTr
                    .Model = mvarMixture.Liquid1Model
                    Tr = T / mvarMixture.Properties(i).Tc
                    Pr = P / mvarMixture.Properties(i).Pc
                    If Z = 0 Then
                        .SolveCubic Tr, Pr, Zg, Z
                    End If
                    .ResidualEnthalpy Tr, Pr, Z, HR
                    
                    HR = UnivGasConst * T * HR
            End With
        Case Else
            DimSolverObjects mvarMixture.NumberOfComponents
            HR = -UnivGasConst * T * T * (SatPressureCalc.DlnSatPressure_T(i, T))
        End Select
        
        ReDim Coef(0 To mvarMixture.CpConsts(i).NbCoef)
        For j = 0 To mvarMixture.CpConsts(i).NbCoef
            Coef(j) = mvarMixture.CpConsts(i).Coef(j)
        Next
        With mvarReferenceState
            MonocompHL = .Ho + HR + Aux + DBO.FunctionIntegration(.TTo, T, 0, mvarMixture.CpConsts(i).NbFormula, Coef())
        End With
' ---- Code Body Ends ---- (Default)
Exit Function
MonocompHL_ErrorHandler:
  Err.Raise Err.Number
Exit_MonocompHL:
End Function
Private Function NR_EvaluateFi(ByRef f() As Double, ByRef X() As Double, ByRef y() As Double, T As Double, P As Double, n As Integer)
On Error GoTo NR_EvaluateFi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        f(i) = NR_Fi(X(), y(), T, P, i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_EvaluateFi_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_EvaluateFi:

End Function
Private Function PE_Hgf(ByVal n As Integer, T As Double, P As Double) As Double
On Error GoTo PE_Hgf_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Sum As Double
    For i = 1 To n
        Sum = Sum + LiquidSolver.MolarFraction(i) * UnivGasConst * T * T * _
        (SatPressureCalc.DlnSatPressure_T(i, T))
    Next
    PE_Hgf = Sum
' ---- Code Body Ends ---- (Default)
Exit Function

PE_Hgf_ErrorHandler:
  Err.Raise Err.Number

Exit_PE_Hgf:

End Function
Private Function NR_UploadLiquidMolarFraction(ByRef X() As Double, n As Integer)
On Error GoTo NR_UploadLiquidMolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        LiquidSolver.MolarFraction(i) = X(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_UploadLiquidMolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_UploadLiquidMolarFraction:

End Function
Private Function NR_UploadVaporMolarFraction(ByRef y() As Double, n As Integer)
On Error GoTo NR_UploadVaporMolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        VaporSolver.MolarFraction(i) = y(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_UploadVaporMolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_UploadVaporMolarFraction:

End Function
Private Function NR_Fi(ByRef X() As Double, ByRef y() As Double, ByVal T As Double, ByVal P As Double, i As Integer) As Double
On Error GoTo NR_Fi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Phil As Double, Phiv As Double
    'liquid part for Ki calculation
    Select Case mvarMixture.Liquid1Model
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
        RP1978_Qbic, Berth1899_Qbic
            LiquidSolver.PartialFugacityCoeficientInSolution i, T, P, Phil
    End Select
    'VAPOR PART TO COMPLETE Ki CALCULATION
    Select Case mvarMixture.VaporModel
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
        RP1978_Qbic, Berth1899_Qbic
            VaporSolver.PartialFugacityCoeficientInSolution i, T, P, Phiv
    End Select
    NR_Fi = y(i) * Phiv - X(i) * Phil
' ---- Code Body Ends ---- (Default)
Exit Function

NR_Fi_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_Fi:

End Function
Private Function NR_DewPointPressure(firstP As Double, firstX() As Double, n As Integer)
On Error GoTo NR_DewPointPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, y() As Double
    Const MAXIT = 200, TOLF = 0.000000000001
    ReDim y(1 To n) As Double
    ReDim f(1 To 2 * n + 4) As Double
    ReDim X(1 To n) As Double
    ReDim P(1 To 2 * n + 4) As Double
    ReDim deltaX(1 To 2 * n + 4) As Double
    ReDim xans(1 To 2 * n + 4) As Double
    ReDim xans1(1 To 2 * n + 4) As Double
    ReDim Ni(1 To n) As Double
    ReDim Jac(1 To 2 * n + 4, 1 To 2 * n + 4) As Double
    Dim datos As DatosFlash
    Dim m As Integer, NbVars As Integer, NbBytes As Long
    Dim gm As Double
    mvarFeedFlow.Pressure = firstP
    CopyMemory X(1), firstX(1), n * Len(firstX(1))
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = X(i)
        VaporSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        Ni(i) = mvarFeedFlow.MolarFraction(i)
        y(i) = mvarFeedFlow.MolarFraction(i)
        LiquidSolver.MolarFraction(i) = X(i)
    Next
    With datos
        .P = mvarFeedFlow.Pressure
        .T = mvarFeedFlow.Temperature
        .Betac = 1
        .Alphac = datos.T
        .Nl = 0
        .Nv = 1
        .Nt = 1
    End With
    NR_UploadX xans(), datos, X(), y(), n
    NR_CalculateGk datos, X(), y(), Ni(), f(), n
    gm = VectorMod(f())
    NR_JacobianMatrix n, Jac(), datos, X(), y(), Ni()
    If gm < TOLF Then
        mvarFeedFlow.Pressure = datos.P
        mvarVaporFlow.Pressure = datos.P
        mvarVaporFlow.Temperature = datos.T
        mvarLiquidFlow.Pressure = datos.P
        mvarLiquidFlow.Temperature = datos.T
        For i = 1 To n
            mvarLiquidFlow.MolarFraction(i) = X(i)
            mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        Next
        Exit Function
    End If
    m = 0
    NbVars = 2 * n + 4
    NbBytes = n * Len(y(1))
    Do 'start of iteration loop
        m = m + 1
        For i = 1 To NbVars: P(i) = -f(i): Next
        SolveAx NbVars, Jac(), P(), deltaX()
        CopyMemory xans1(1), y(1), NbBytes
        NR_Xnew xans(), deltaX()
        NR_DownloadX xans(), datos, X(), y(), n
        NR_CalculateGk datos, X(), y(), Ni(), f(), n
        NR_JacobianMatrix n, Jac(), datos, X(), y(), Ni()
    Loop Until CheckFunConv(f(), TOLF) = True Or CheckFunConv(deltaX(), TOLF) = True Or m > MAXIT
    mvarFeedFlow.Pressure = datos.P
    mvarVaporFlow.Pressure = datos.P
    mvarVaporFlow.Temperature = datos.T
    mvarLiquidFlow.Pressure = datos.P
    mvarLiquidFlow.Temperature = datos.T
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = X(i)
        mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

NR_DewPointPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_DewPointPressure:

End Function
Private Sub DimSolverObjects(ByVal n As Integer)
On Error GoTo DimSolverObjects_ErrorHandler
' ---- Code Body Starts ---- (Default)
    Dim i As Integer, j As Integer
    Set mvarLiquidFlow = New clsFlow
    Set mvarVaporFlow = New clsFlow
    FillSatPressureSolver n
    mvarBoolgammaphi = False
    Select Case mvarMixture.Liquid1Model
        Case Is < 20
            Set LiquidSolver = New clsQbics
            Set LiquidSolver.Parent = Me
            LiquidSolver.PhaseOfMixture = PhaseIDLiquid
            LiquidSolver.Model = mvarMixture.Liquid1Model
            For i = 1 To n
                LiquidSolver.Properties.Add mvarMixture.Properties(i).Tc, _
                mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).w, mvarMixture.Properties(i).Zc, _
                mvarMixture.Properties(i).m, mvarMixture.Properties(i).n, mvarMixture.Properties(i).g, mvarMixture.Properties(i).K1, mvarMixture.Properties(i).PsatAtTr
            Next i
            LiquidSolver.Kij = mvarMixture.Properties.Kij
         Case Is > 20
            If mvarMixture.Liquid1Model = 25 Then
                Set LiquidSolver = New clsActivityMulticomp
                LiquidSolver.Model = mvarMixture.Liquid1Model
                For i = 1 To n
                    LiquidSolver.Properties.Add mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).vlModel
                Next i
                mvarBoolgammaphi = True
            Else
                Set LiquidSolver = New clsActivityMulticomp
                LiquidSolver.Model = mvarMixture.Liquid1Model
                LiquidSolver.Properties.AijTemperature = mvarMixture.Properties.AijTemperature
                For i = 1 To n
                   LiquidSolver.Properties.Add mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).vlModel
                   For j = 1 To n
                        LiquidSolver.Properties.Aij(i, j) = mvarMixture.Properties.Aij(i, j)
                   Next j
                Next i
                mvarBoolgammaphi = True
            End If
    End Select
    Select Case mvarMixture.VaporModel
        Case Is = -1
            Set VaporSolver = New clsVirialMulticomp
            For i = 1 To n
                VaporSolver.CriticalProperties.Add mvarMixture.Properties(i).Tc, _
                mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).w, mvarMixture.Properties(i).Zc
            Next
            mvarBoolgammaphi = True
            VaporSolver.KijParameters = mvarMixture.Properties.Kij
            Exit Sub
        Case Is < 0
            Set VaporSolver = New clsQbics
            Set VaporSolver.Parent = Me
            VaporSolver.Model = mvarMixture.VaporModel
            VaporSolver.PhaseOfMixture = PhaseIDvapor
            mvarBoolgammaphi = True
        Case Else
            Set VaporSolver = New clsQbics
            Set VaporSolver.Parent = Me
            VaporSolver.PhaseOfMixture = PhaseIDvapor
            VaporSolver.Model = mvarMixture.VaporModel
            For i = 1 To n
                VaporSolver.Properties.Add mvarMixture.Properties(i).Tc, _
                mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).w, mvarMixture.Properties(i).Zc, _
                mvarMixture.Properties(i).m, mvarMixture.Properties(i).n, mvarMixture.Properties(i).g, mvarMixture.Properties(i).K1, mvarMixture.Properties(i).PsatAtTr
            Next i
            VaporSolver.Kij = mvarMixture.Properties.Kij
    End Select
' ---- Code Body Ends ---- (Default)
Exit Sub

DimSolverObjects_ErrorHandler:
  Err.Raise Err.Number
Exit_DimSolverObjects:
End Sub
'Private Sub ClearSolverObjects()
'    Set LiquidSolver = Nothing
'    Set VaporSolver = Nothing
'    Set SatPressureCalc = Nothing
'End Sub
Private Sub NR_BubblePointPressure(firstP As Double, firstY() As Double, n As Integer)
On Error GoTo NR_BubblePointPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, y() As Double
    Const MAXIT = 200, TOLF = 0.000000000001
    ReDim y(1 To n) As Double
    ReDim f(1 To 2 * n + 4) As Double
    ReDim X(1 To n) As Double
    ReDim P(1 To 2 * n + 4) As Double
    ReDim deltaX(1 To 2 * n + 4) As Double
    ReDim xans(1 To 2 * n + 4) As Double
    ReDim xans1(1 To 2 * n + 4) As Double
    ReDim Ni(1 To n) As Double
    ReDim Jac(1 To 2 * n + 4, 1 To 2 * n + 4) As Double
    Dim datos As DatosFlash
    Dim m As Integer, NbVars As Integer, NbBytes As Long
    Dim gm As Double
    mvarFeedFlow.Pressure = firstP
    CopyMemory y(1), firstY(1), n * Len(firstY(1))
    With mvarFeedFlow
        For i = 1 To n
            mvarVaporFlow.MolarFraction(i) = y(i)
            LiquidSolver.MolarFraction(i) = .MolarFraction(i)
            Ni(i) = .MolarFraction(i)
            X(i) = .MolarFraction(i)
            VaporSolver.MolarFraction(i) = y(i)
        Next
    End With
    With datos
        .P = mvarFeedFlow.Pressure: .T = mvarFeedFlow.Temperature
        .Betac = 0: .Alphac = .T
        .Nl = 1: .Nv = 0: .Nt = 1
    End With
    NR_UploadX xans(), datos, X(), y(), n
    NR_CalculateGk datos, X(), y(), Ni(), f(), n
    gm = VectorMod(f())
    NR_JacobianMatrix n, Jac(), datos, X(), y(), Ni()
    If gm < TOLF Then
        With datos
            mvarFeedFlow.Pressure = .P
            mvarVaporFlow.Pressure = .P
            mvarVaporFlow.Temperature = .T
            mvarLiquidFlow.Pressure = .P
            mvarLiquidFlow.Temperature = .T
        End With
        For i = 1 To n
            mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            mvarVaporFlow.MolarFraction(i) = y(i)
        Next
        FillVaporFraction BubP, 1
        Exit Sub
    End If
    m = 0
    NbVars = 2 * n + 4
    NbBytes = n * Len(y(1))
    Do 'start of iteration loop
        m = m + 1
        For i = 1 To NbVars: P(i) = -f(i): Next
        SolveAx NbVars, Jac(), P(), deltaX()
        CopyMemory xans1(1), y(1), NbBytes
        NR_Xnew xans(), deltaX()
        NR_DownloadX xans(), datos, X(), y(), n
        NR_CalculateGk datos, X(), y(), Ni(), f(), n
        NR_JacobianMatrix n, Jac(), datos, X(), y(), Ni()
    Loop Until CheckFunConv(f(), TOLF) = True Or CheckFunConv(deltaX(), TOLF) = True Or m > MAXIT
    With datos
        mvarFeedFlow.Pressure = .P
        mvarVaporFlow.Pressure = .P
        mvarVaporFlow.Temperature = .T
        mvarLiquidFlow.Pressure = .P
        mvarLiquidFlow.Temperature = .T
    End With
    With mvarFeedFlow
        For i = 1 To n
            mvarLiquidFlow.MolarFraction(i) = .MolarFraction(i)
            mvarVaporFlow.MolarFraction(i) = y(i)
        Next
    End With
' ---- Code Body Ends ---- (Default)
Exit Sub

NR_BubblePointPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_BubblePointPressure:

End Sub
Private Sub NR_BubblePointTemperature(firstT As Double, firstY() As Double, n As Integer)
On Error GoTo NR_BubblePointTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, y() As Double
    Const MAXIT = 200, TOLF = 0.000000000001
    ReDim y(1 To n) As Double
    ReDim f(1 To 2 * n + 4) As Double
    ReDim X(1 To n) As Double
    ReDim P(1 To 2 * n + 4) As Double
    ReDim deltaX(1 To 2 * n + 4) As Double
    ReDim xans(1 To 2 * n + 4) As Double
    ReDim xans1(1 To 2 * n + 4) As Double
    ReDim Ni(1 To n) As Double
    ReDim Jac(1 To 2 * n + 4, 1 To 2 * n + 4) As Double
    Dim datos As DatosFlash
    Dim m As Integer, NbVars As Integer, NbBytes As Long
    Dim gm As Double
    mvarFeedFlow.Temperature = firstT
    For i = 1 To n
        mvarVaporFlow.MolarFraction(i) = firstY(i)
        LiquidSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        Ni(i) = mvarFeedFlow.MolarFraction(i)
        X(i) = mvarFeedFlow.MolarFraction(i)
        y(i) = firstY(i)
        VaporSolver.MolarFraction(i) = y(i)
        LiquidSolver.MolarFraction(i) = X(i)
    Next
    datos.P = mvarFeedFlow.Pressure
    datos.T = mvarFeedFlow.Temperature
    datos.Betac = 0
    datos.Alphac = datos.P
    datos.Nl = 1
    datos.Nv = 0
    datos.Nt = 1
    NR_UploadX xans(), datos, X(), y(), n
    NR_CalculateGkBDT datos, X(), y(), Ni(), f(), n
    gm = VectorMod(f())
    NR_JacobianMatrixBDT n, Jac(), datos, X(), y(), Ni()
    If gm < TOLF Then
        mvarFeedFlow.Pressure = datos.P
        mvarVaporFlow.Pressure = datos.P
        mvarVaporFlow.Temperature = datos.T
        mvarLiquidFlow.Pressure = datos.P
        mvarLiquidFlow.Temperature = datos.T
        For i = 1 To n
            mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            mvarVaporFlow.MolarFraction(i) = y(i)
        Next
        FillVaporFraction BubP, 1
        Exit Sub
    End If
    m = 0
    NbVars = 2 * n + 4
    NbBytes = n * Len(y(1))
    Do 'start of iteration loop
        m = m + 1
        For i = 1 To NbVars: P(i) = -f(i): Next
        SolveAx NbVars, Jac(), P(), deltaX()
        CopyMemory xans1(1), y(1), NbBytes
        NR_Xnew xans(), deltaX()
        NR_DownloadX xans(), datos, X(), y(), n
        NR_CalculateGkBDT datos, X(), y(), Ni(), f(), n
        NR_JacobianMatrixBDT n, Jac(), datos, X(), y(), Ni()
    Loop Until CheckFunConv(f(), TOLF) = True Or CheckFunConv(deltaX(), TOLF) = True Or m > MAXIT
    mvarFeedFlow.Pressure = datos.P
    mvarFeedFlow.Temperature = datos.T
    mvarVaporFlow.Pressure = datos.P
    mvarVaporFlow.Temperature = datos.T
    mvarLiquidFlow.Pressure = datos.P
    mvarLiquidFlow.Temperature = datos.T
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        mvarVaporFlow.MolarFraction(i) = y(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Sub

NR_BubblePointTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_BubblePointTemperature:

End Sub
Private Sub NR_DewPointTemperature(firstT As Double, firstX() As Double, n As Integer)
On Error GoTo NR_DewPointTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, y() As Double
    Const MAXIT = 200, TOLF = 0.00000001
    ReDim y(1 To n) As Double
    ReDim f(1 To 2 * n + 4) As Double
    ReDim X(1 To n) As Double
    ReDim P(1 To 2 * n + 4) As Double
    ReDim deltaX(1 To 2 * n + 4) As Double
    ReDim xans(1 To 2 * n + 4) As Double
    ReDim xans1(1 To 2 * n + 4) As Double
    ReDim Ni(1 To n) As Double
    ReDim Jac(1 To 2 * n + 4, 1 To 2 * n + 4) As Double
    Dim datos As DatosFlash
    Dim m As Integer, NbVars As Integer, NbBytes As Long
    Dim gm As Double
    mvarFeedFlow.Temperature = firstT
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = firstX(i)
        VaporSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        Ni(i) = mvarFeedFlow.MolarFraction(i)
        X(i) = firstX(i)
        y(i) = mvarFeedFlow.MolarFraction(i)
        LiquidSolver.MolarFraction(i) = X(i)
    Next
    datos.P = mvarFeedFlow.Pressure
    datos.T = mvarFeedFlow.Temperature
    datos.Betac = 1
    datos.Alphac = datos.P
    datos.Nl = 0
    datos.Nv = 1
    datos.Nt = 1
    NR_UploadX xans(), datos, X(), y(), n
    NR_CalculateGkBDT datos, X(), y(), Ni(), f(), n
    gm = VectorMod(f())
    NR_JacobianMatrixBDT n, Jac(), datos, X(), y(), Ni()
    If gm < TOLF Then
        mvarFeedFlow.Pressure = datos.P
        mvarVaporFlow.Pressure = datos.P
        mvarVaporFlow.Temperature = datos.T
        mvarLiquidFlow.Pressure = datos.P
        mvarLiquidFlow.Temperature = datos.T
        For i = 1 To n
            mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            mvarVaporFlow.MolarFraction(i) = y(i)
        Next
        FillVaporFraction BubP, 1
        Exit Sub
    End If
    m = 0
    NbVars = 2 * n + 4
    NbBytes = n * Len(y(1))
    Do 'start of iteration loop
        m = m + 1
        For i = 1 To NbVars: P(i) = -f(i): Next
        SolveAx NbVars, Jac(), P(), deltaX()
        CopyMemory xans1(1), y(1), NbBytes
        NR_Xnew xans(), deltaX()
        NR_DownloadX xans(), datos, X(), y(), n
        NR_CalculateGkBDT datos, X(), y(), Ni(), f(), n
        NR_JacobianMatrixBDT n, Jac(), datos, X(), y(), Ni()
    Loop Until CheckFunConv(f(), TOLF) = True Or CheckFunConv(deltaX(), TOLF) = True Or m > MAXIT
    mvarFeedFlow.Pressure = datos.P
    mvarFeedFlow.Temperature = datos.T
    mvarVaporFlow.Pressure = datos.P
    mvarVaporFlow.Temperature = datos.T
    mvarLiquidFlow.Pressure = datos.P
    mvarLiquidFlow.Temperature = datos.T
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = X(i)
        mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Sub

NR_DewPointTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_NR_DewPointTemperature:

End Sub
Public Sub FillSatPressureSolver(ByVal n As Integer)
On Error GoTo FillSatPressureSolver_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Item
    SatPressureCalc.SatPressureModel = mvarMixture.SatPressureModel
    Select Case mvarMixture.SatPressureModel
        
        Case Riedel, Muller, RPM
            For Each Item In mvarMixture.Properties 'i = 1 To n
                SatPressureCalc.CriticalConstants.Add Item.Tc, _
                Item.Pc, Item.w, 0
            Next
            For i = 1 To n
                SatPressureCalc.OtherConstants.Add mvarMixture.Properties(i).Zc, _
                mvarMixture.Properties(i).w, 0, mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, Rackett
                
                SatPressureCalc.OtherConstants.Add
            Next
        Case DBPsat
            Set SatPressureCalc.PsatConst = mvarMixture.PsatConst
            For Each Item In mvarMixture.Properties 'i = 1 To n
                SatPressureCalc.CriticalConstants.Add Item.Tc, _
                Item.Pc, Item.w, 0
            Next
    End Select
' ---- Code Body Ends ---- (Default)
Exit Sub

FillSatPressureSolver_ErrorHandler:
  Err.Raise Err.Number

Exit_FillSatPressureSolver:

End Sub
Private Sub CalculateKi(ByVal n As Integer, ByVal T As Double, ByVal P As Double, ByRef Ki() As Double)
On Error GoTo CalculateKi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Phil As Double, Phiv As Double, PSat As Double
    Dim Gamma As Double, Poynting As Double, PhiSat As Double, Aux As Double
    For i = 1 To n
        Select Case mvarMixture.Liquid1Model
            Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
            RP1978_Qbic, Berth1899_Qbic, VdWAda1984_Qbic, VdWVald1989_Qbic, RKSmn1980_Qbic, PRATmng1997_Qbic, PRMmn1989_Qbic, PRSV1986_Qbic, VdWOL1998_Qbic, RKOL1998_Qbic, PROL1998_Qbic
                LiquidSolver.PartialFugacityCoeficientInSolution i, T, P, Phil
                Ki(i) = Phil * P
            Case vanLaar, Wilson, ScatchardHild, Margules
                SatPressureCalc.SatPressure i, T, PSat
                With LiquidSolver
                    .CalcGamma i, T, Gamma
                End With
                Ki(i) = PSat * Gamma
            Case idealsolution
                SatPressureCalc.SatPressure i, T, PSat
'                Select Case mvarMixture.VaporModel
'                    Case IdealGas
                        Ki(i) = PSat
'                    Case Else
'                        Ki(i) = 1
'                End Select
        End Select
        Select Case mvarMixture.VaporModel
            Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
            RP1978_Qbic, Berth1899_Qbic, VdWAda1984_Qbic, VdWVald1989_Qbic, RKSATmn1995_Qbic, PRATmng1997_Qbic, PRMmn1989_Qbic, PRSV1986_Qbic, VdWOL1998_Qbic, RKOL1998_Qbic, PROL1998_Qbic
            VaporSolver.PartialFugacityCoeficientInSolution i, T, P, Phiv
                If mvarBoolgammaphi Then
                    PhiSat = CalculatePhiSati(i, T)
                    If mvarMixture.VaporModel <> IdealGasv Then
                        Poynting = CalculatePoynting(i, T, P)
                    Else
                        Poynting = 1
                    End If
                    Aux = PhiSat * Poynting
                    Ki(i) = (Ki(i) * Aux) / (Phiv * P)
                Else
                    Ki(i) = Ki(i) / (Phiv * P)
                End If
            Case virial
                Phiv = VaporSolver.CalculatePartialFugacity(i, T, P)
                If mvarBoolgammaphi Then
                    PhiSat = CalculatePhiSati(i, T)
                    Aux = (Phiv / PhiSat)
                    Ki(i) = Ki(i) / (Aux * P)
                Else
                    Ki(i) = Ki(i) / (Phiv * P)
                End If
            Case IdealGas
            Ki(i) = Ki(i) / P
        End Select
    Next
' ---- Code Body Ends ---- (Default)
Exit Sub

CalculateKi_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculateKi:

End Sub
Public Function AdiabaticFlash(Heat As Double, IntPressure As Double) As Integer
On Error GoTo AdiabaticFlash_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Const EPS = 0.00000003
    Dim Hin As Double, Sin As Double, Betac As Double
    Dim Iter As Integer, Ho As Double, So As Double
    Dim h1 As Double, S1 As Double, n As Integer
    Dim H2 As Double, dum As Double
    Dim a As Double, B As Double, C As Double, d As Double, E As Double, fa As Double
    Dim fb As Double, fc As Double, P As Double, Q As Double, R As Double, S As Double
    Dim tol1 As Double, xm As Double
    Dim FeedPressure, FeedTemperature As Double, Feedbeta As Double
    FeedPressure = mvarFeedFlow.Pressure
    FeedTemperature = mvarFeedFlow.Temperature
    n = mvarMixture.NumberOfComponents
    MaxVar = 0.25
    BubblePointTemperature
    a = mvarFeedFlow.Temperature
    DewPointTemperature
    B = mvarFeedFlow.Temperature
    mvarFeedFlow.Temperature = FeedTemperature
    Select Case FeedTemperature
        Case Is < a
             'liquid state
             Betac = 0
             Feedbeta = 0
        Case Is > a
             If FeedTemperature > B Then
                  'vapor state
                  Betac = 1
                  Feedbeta = 1
             ElseIf FeedTemperature < B Then
                  '2 phases
                  IsoThermicFlash Betac
                  Feedbeta = Betac
             End If
    End Select
    Enthalpy_Entropy n, FeedTemperature, mvarFeedFlow.Pressure, Betac, Hin, Sin
    Hin = Hin + Heat
    mvarFeedFlow.Pressure = IntPressure
    BubblePointTemperature
    a = mvarFeedFlow.Temperature
    DewPointTemperature
    B = mvarFeedFlow.Temperature
    Enthalpy_Entropy n, a, mvarFeedFlow.Pressure, 0, h1, S1
    fa = h1 - Hin
    Enthalpy_Entropy n, B, mvarFeedFlow.Pressure, 1, H2, So
    fb = H2 - Hin
    If (fa > 0 And fb > 0) Or (fa < 0 And fb < 0) Then
        'Salir del procedimiento
          Exit Function
    End If
    'Utilizaremos el metodo de Brent(Numerical Recipes in Fortran 2nd ED pag 352)
    'para conseguir la raiz de temperatura(Tbub<T<Trocio)
    C = B
    fc = fb
    For Iter = 1 To mvarTolerances.MaxIter
        DoEvents
        If ((fb > 0 And fc > 0) Or (fb < 0 And fc < 0)) Then
            C = a
            fc = fa
            d = B - a
            E = d
        End If
        If (Abs(fc) < Abs(fb)) Then
            a = B
            B = C
            C = a
            fa = fb
            fb = fc
            fc = fa
        End If
        tol1 = 2 * EPS * Abs(B) + 0.5 * (mvarTolerances.TolTemperature / 100)
        xm = 0.5 * (C - B)
        If (Abs(xm) < tol1 Or fb = 0) Then
            mvarFeedFlow.Temperature = FeedTemperature
            mvarFeedFlow.Pressure = FeedPressure
            FillVaporFraction Flash, Betac
            Exit For
        End If
        If Abs(E) > tol1 And Abs(fa) > Abs(fb) Then
            S = fb / fa
            If a = C Then
                P = 2 * xm * S
                Q = 1 - S
            Else
                Q = fa / fc
                R = fb / fc
                P = S * (2 * xm * Q * (Q - R) - (B - a) * (R - 1))
                Q = (Q - 1) * (R - 1) * (S - 1)
            End If
            If P > 0 Then Q = -Q
            P = Abs(P)
            dum = 3 * xm * Q - Abs(tol1 * Q)
            If dum > Abs(E * Q) Then dum = E * Q
            If 2 * P < dum Then
                E = d
                d = P / Q
            Else
                d = xm
                E = d
            End If
        Else
            d = xm
            E = d
        End If
        a = B
        fa = fb
        If Abs(d) > tol1 Then
            B = B + d
        Else
            B = B + Abs(tol1) * Sgn(xm)
        End If
        mvarFeedFlow.Temperature = B
        IsoThermicFlash Betac
        If Betac > 1 Or Betac < 0 Then
            Exit Function
        End If
        Enthalpy_Entropy n, B, mvarFeedFlow.Pressure, Betac, Ho, So
        fb = Ho - Hin
    Next Iter
' ---- Code Body Ends ---- (Default)
Exit Function

AdiabaticFlash_ErrorHandler:
  Err.Raise Err.Number

Exit_AdiabaticFlash:

End Function
Public Sub IsoThermicFlash(ByRef Betac As Double)
On Error GoTo IsoThermicFlash_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim PSat As Double, Sum As Double, itera As Integer
    Dim veri As Boolean, h As Double, n As Integer, a As Double, B As Double, m As Double
    n = mvarMixture.NumberOfComponents
    'Kid array to interpolate, Ki and Kif boundary
    ReDim Ki(1 To n) As Double
    ReDim Kif(1 To n) As Double
    ReDim Kid(1 To n) As Double
    ReDim Xi(1 To n) As Double
    ReDim Yi(1 To n) As Double
    ReDim XiK(1 To n) As Double
    ReDim YiK(1 To n) As Double
    Dim P As Double
    Dim i As Integer, S As Double
    veri = True
    P = mvarFeedFlow.Pressure
    DimSolverObjects n
    BubblePointPressure
    For i = 1 To n
        Ki(i) = mvarVaporFlow.MolarFraction(i) / mvarLiquidFlow.MolarFraction(i)
    Next
    a = mvarFeedFlow.Pressure
    DewPointPressure
    For i = 1 To n
        Kif(i) = mvarVaporFlow.MolarFraction(i) / mvarLiquidFlow.MolarFraction(i)
    Next
    B = mvarFeedFlow.Pressure
    Select Case P
        Case Is < B
             'vapor state
             mvarVaporFlow.Temperature = mvarFeedFlow.Temperature
             mvarVaporFlow.Pressure = P
             mvarVaporFlow.VaporFraction = 1
             For i = 1 To n
                mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
             Next
             mvarFeedFlow.Pressure = P
             mvarLiquidFlow.Temperature = mvarFeedFlow.Temperature
             mvarLiquidFlow.Pressure = P
             mvarLiquidFlow.VaporFraction = 0
             For i = 1 To n
                mvarLiquidFlow.MolarFraction(i) = 0
             Next
             mvarFeedFlow.Pressure = P
             Betac = 13113006
             Exit Sub
        Case Is > B
             If P > a Then
                'liquid state
                mvarLiquidFlow.Temperature = mvarFeedFlow.Temperature
                mvarLiquidFlow.Pressure = P
                mvarLiquidFlow.VaporFraction = 0
                For i = 1 To n
                   mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
                Next
                mvarFeedFlow.Pressure = P
                mvarVaporFlow.Temperature = mvarFeedFlow.Temperature
                mvarVaporFlow.Pressure = P
                mvarVaporFlow.VaporFraction = 1
                For i = 1 To n
                   mvarVaporFlow.MolarFraction(i) = 0
                Next
                mvarFeedFlow.Pressure = P
                Betac = -13113006
                Exit Sub
             ElseIf P < a Then
                  '2 phases continue the procedure
                  mvarFeedFlow.Pressure = P
             End If
    End Select
    'lo voy a cambiar por interpolacion lineal de las K entre burbuja y Rocio
    'a P del flash
    For i = 1 To n
        'm = (Ki(i) - Kif(i)) / (A - B)
        veri = SatPressureCalc.SatPressure(i, mvarFeedFlow.Temperature, PSat)
        Ki(i) = PSat / mvarFeedFlow.Pressure ' m * mvarFeedFlow.Pressure + m * B + Kif(i)
    Next
    veri = Flash_RachfordRice_NR(0.5, Betac, Ki(), n)
    Flash_CalculateXi Betac, Ki(), Xi(), n
    Flash_CalculateYi Xi(), Yi(), Ki(), n
    SumFrac Xi(), Sum
    Norm Xi(), Sum
    SumFrac Yi(), Sum
    Norm Yi(), Sum
    CopyMemory XiK(1), Xi(1), n * Len(Xi(1))
    CopyMemory YiK(1), Yi(1), n * Len(Yi(1))
    Do
        DoEvents
        itera = itera + 1
        CopyMemory Xi(1), XiK(1), n * Len(XiK(1))
        With LiquidSolver
            For i = 1 To n: .MolarFraction(i) = Xi(i): Next
        End With
        CopyMemory Yi(1), YiK(1), n * Len(YiK(1))
        With VaporSolver
            For i = 1 To n: .MolarFraction(i) = Yi(i): Next
        End With
        With mvarFeedFlow
            CalculateKi n, .Temperature, .Pressure, Ki()
        End With
        veri = Flash_RachfordRice_NR(0.5, Betac, Ki(), n)
        If veri = False Then Exit Sub
        Flash_CalculateXi Betac, Ki(), XiK(), n
        Flash_CalculateYi XiK(), YiK(), Ki(), n
    Loop Until FranctionsEquals(Xi(), XiK(), Yi(), YiK()) = True Or itera > mvarTolerances.MaxIter
    If Betac > 1 Or Betac < 0 Then
        'MsgBox "Vapor Fraction out of range", vbOKOnly
    End If
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = XiK(i)
        mvarVaporFlow.MolarFraction(i) = YiK(i)
    Next
    If Betac < 0.00000001 Then Betac = 0
    mvarVaporFlow.Pressure = mvarFeedFlow.Pressure
    mvarVaporFlow.Temperature = mvarFeedFlow.Temperature
    mvarLiquidFlow.Pressure = mvarFeedFlow.Pressure
    mvarLiquidFlow.Temperature = mvarFeedFlow.Temperature
    FillVaporFraction Flash, Betac
    With mvarFeedFlow
        Enthalpy_Entropy n, .Temperature, .Pressure, .VaporFraction, h, S
        .Enthalpy = h
        .Entropy = S
    End With
   'ojo
    mvarFeedFlow.Enthalpy = mvarLiquidFlow.Enthalpy + mvarVaporFlow.Enthalpy - h
' ---- Code Body Ends ---- (Default)
Exit Sub

IsoThermicFlash_ErrorHandler:
  Err.Raise Err.Number

Exit_IsoThermicFlash:

End Sub
Private Function FranctionsEquals(ByRef x1() As Double, ByRef x2() As Double, ByRef y1() As Double, ByRef y2() As Double) As Boolean
On Error GoTo FranctionsEquals_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To UBound(x1())
        If (Abs(x1(i) - x2(i)) < mvarTolerances.TolMolarFraction) Then
            If (Abs(y1(i) - y2(i)) < mvarTolerances.TolMolarFraction) Then
                FranctionsEquals = True
                Exit Function
            End If
        End If
    Next
    FranctionsEquals = False
' ---- Code Body Ends ---- (Default)
Exit Function

FranctionsEquals_ErrorHandler:
  Err.Raise Err.Number

Exit_FranctionsEquals:

End Function
Private Function Flash_CalculateXi(Beta As Double, ByRef Ki() As Double, ByRef X() As Double, ByVal n As Integer) As Boolean
On Error GoTo Flash_CalculateXi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        X(i) = mvarFeedFlow.MolarFraction(i) / (1 + Beta * (Ki(i) - 1))
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Flash_CalculateXi_ErrorHandler:
  Err.Raise Err.Number

Exit_Flash_CalculateXi:

End Function
Private Function Flash_CalculateYi(ByRef X() As Double, ByRef y() As Double, ByRef Ki() As Double, ByVal n As Integer) As Boolean
On Error GoTo Flash_CalculateYi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        y(i) = X(i) * Ki(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Flash_CalculateYi_ErrorHandler:
  Err.Raise Err.Number

Exit_Flash_CalculateYi:

End Function
Private Function Flash_RachfordRice_NR(initialguess As Double, ByRef answer As Double, ByRef Ki() As Double, ByVal n As Integer) As Boolean
On Error GoTo Flash_RachfordRice_NR_ErrorHandler
Dim x1 As Double, a As Double, B As Double, C As Double, fa As Double, fb As Double, c0 As Double
' ---- Code Body Starts ---- (Default)
Dim Betak As Double, Beta As Double, i As Integer, f As Double, fd As Double
Dim Error1 As Double, NumerItera As Integer, fc As Double, NumerIter As Integer
    i = 0
    Flash_RachfordRice_NR = False
    Betak = initialguess
    a = 1
    B = 0
    fa = Flash_RachfordRice(a, Ki(), n)
    fb = Flash_RachfordRice(B, Ki(), n)
   ' este código fue cambiado
    If fa * fb < 0 Then
        c0 = (a + B) / 2
        fc = Flash_RachfordRice(c0, Ki(), n)
        If fc * fa < 0 Then
            B = c0
            fb = fc
        Else
            a = c0
            fa = fc
        End If
        Error1 = 11314458
        While Error1 > 0.00001 And NumerIter < 1000
            C = (a + B) / 2
            fc = Flash_RachfordRice(C, Ki(), n)
            Error1 = Abs(C - c0) / C
            NumerIter = NumerIter + 1
            c0 = C
            If fa * fc < 0 Then
                B = C
                fb = fc
            ElseIf fa * fc > 0 Then
                a = C
                fa = fc
            Else
                Error1 = 0
            End If
        Wend
        answer = C
    Else
        Do
        Beta = Betak
        i = i + 1
        f = Flash_RachfordRice(Beta, Ki(), n)
        fd = Flash_DerivateRachfordRice(Beta, Ki(), n)
        Betak = Beta - (f / fd)
        If i > 500 Then
           Flash_RachfordRice_NR = False
           Exit Function
        End If
        Loop Until Abs((Betak - Beta) / Betak) < 0.000001
        answer = Betak
    End If
   
    If Betak > 1 Or Betak < 0 Then
        Flash_RachfordRice_NR = False
    Else
        Flash_RachfordRice_NR = True
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

Flash_RachfordRice_NR_ErrorHandler:
  Err.Raise Err.Number

Exit_Flash_RachfordRice_NR:

End Function
Private Function Flash_RachfordRice(Beta As Double, ByRef Ki() As Double, ByVal n As Integer)
On Error GoTo Flash_RachfordRice_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Sum As Double
    For i = 1 To n
        Sum = Sum + (mvarFeedFlow.MolarFraction(i) * (Ki(i) - 1)) / (1 + Beta * (Ki(i) - 1))
    Next
    Flash_RachfordRice = Sum
' ---- Code Body Ends ---- (Default)
Exit Function

Flash_RachfordRice_ErrorHandler:
  Err.Raise Err.Number

Exit_Flash_RachfordRice:

End Function
Private Function Flash_DerivateRachfordRice(Beta As Double, ByRef Ki() As Double, ByVal n As Integer)
On Error GoTo Flash_DerivateRachfordRice_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Sum As Double
    For i = 1 To n
        Sum = Sum - (mvarFeedFlow.MolarFraction(i) * (Ki(i) - 1) ^ 2) / ((1 + Beta * (Ki(i) - 1)) ^ 2)
    Next
    Flash_DerivateRachfordRice = Sum
' ---- Code Body Ends ---- (Default)
Exit Function

Flash_DerivateRachfordRice_ErrorHandler:
  Err.Raise Err.Number

Exit_Flash_DerivateRachfordRice:

End Function
Private Sub RaoultFirstAprox(TypeOfCalc As TypeCalculation, ByVal n As Integer)
On Error GoTo RaoultFirstAprox_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, veri As Boolean, Prsat As Double, arrPrsat() As Double
    Dim Aux As Double, Aux1 As Double, lclFeedT As Double
    Dim lclFeedP As Double
    ReDim arrPrsat(1 To n) As Double
    Select Case TypeOfCalc
        Case BubP
            lclFeedT = mvarFeedFlow.Temperature
            
            For i = 1 To n
                If mvarMixture.Properties(i).Tc < lclFeedT Then
                    veri = SatPressureCalc.SatPressure(i, mvarMixture.Properties(i).Tc, Prsat)
                    If veri = True Then
                       arrPrsat(i) = Prsat
                    End If
                Else
                    veri = SatPressureCalc.SatPressure(i, lclFeedT, Prsat)
                    If veri = True Then
                       arrPrsat(i) = Prsat
                    End If
                End If
            Next
            For i = 1 To n
                mvarMixture.Properties(i).PsatAtTr = arrPrsat(i)
                Aux = arrPrsat(i) * mvarFeedFlow.MolarFraction(i)
                Aux1 = Aux1 + Aux
            Next
            mvarFeedFlow.Pressure = Aux1
            For i = 1 To n
                mvarVaporFlow.MolarFraction(i) = (arrPrsat(i) / Aux1) * mvarFeedFlow.MolarFraction(i)
                mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            Next
        Case DewP
            lclFeedT = mvarFeedFlow.Temperature
            For i = 1 To n
                If mvarMixture.Properties(i).Tc < lclFeedT Then
                    veri = SatPressureCalc.SatPressure(i, mvarMixture.Properties(i).Tc, Prsat)
                    If veri = True Then
                       arrPrsat(i) = Prsat
                    End If
                Else
                    veri = SatPressureCalc.SatPressure(i, lclFeedT, Prsat)
                    If veri = True Then
                       arrPrsat(i) = Prsat
                    End If
                End If
            Next
            For i = 1 To n
                mvarMixture.Properties(i).PsatAtTr = arrPrsat(i)
                Aux = mvarFeedFlow.MolarFraction(i) / arrPrsat(i)
                Aux1 = Aux1 + Aux
            Next
            Aux = 1 / Aux1
            mvarFeedFlow.Pressure = Aux
            For i = 1 To n
                mvarLiquidFlow.MolarFraction(i) = (Aux / arrPrsat(i)) * mvarFeedFlow.MolarFraction(i)
                mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            Next
        Case BubT
            lclFeedT = TempIni(BubT, n)
            lclFeedP = mvarFeedFlow.Pressure
            mvarFeedFlow.Temperature = lclFeedT
            For i = 1 To n
                veri = SatPressureCalc.SatPressure(i, lclFeedT, Prsat)
                If veri = True Then
                   arrPrsat(i) = Prsat
                End If
            Next
            For i = 1 To n
                mvarMixture.Properties(i).PsatAtTr = arrPrsat(i)
                mvarVaporFlow.MolarFraction(i) = (arrPrsat(i) / lclFeedP) * mvarFeedFlow.MolarFraction(i)
                mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            Next
        Case DewT
            lclFeedT = TempIni(DewT, n)
            lclFeedP = mvarFeedFlow.Pressure
            mvarFeedFlow.Temperature = lclFeedT
            For i = 1 To n
                veri = SatPressureCalc.SatPressure(i, lclFeedT, Prsat)
                If veri = True Then
                   arrPrsat(i) = Prsat
                End If
            Next
            For i = 1 To n
                mvarMixture.Properties(i).PsatAtTr = arrPrsat(i)
                mvarLiquidFlow.MolarFraction(i) = (lclFeedP / arrPrsat(i)) * mvarFeedFlow.MolarFraction(i)
                mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
            Next
    End Select
' ---- Code Body Ends ---- (Default)
Exit Sub

RaoultFirstAprox_ErrorHandler:
  Err.Raise Err.Number

Exit_RaoultFirstAprox:

End Sub
Private Function TempIni(TypeOfCalc As TypeCalculation, n As Integer) As Double
On Error GoTo TempIni_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Aux As Double, Inc As Double, i As Integer, T1 As Double, T2 As Double
    Dim itera As Integer, f1 As Double, f2 As Double, f3 As Double, PSat As Double, lclFeedP As Double
    'El siguiente algoritmo es tomado del programa Ekilib
    MaxVar = 0.25
    itera = 0: Inc = 0
    lclFeedP = mvarFeedFlow.Pressure
    Aux = mvarFeedFlow.MolarFraction(1) * mvarMixture.Properties(1).Tb
    For i = 2 To n
        Aux = Aux + mvarFeedFlow.MolarFraction(i) * mvarMixture.Properties(i).Tb
    Next
    Do
        itera = itera + 1
        If itera >= 2 Then
            IncrementarYRodar T1, T2, Aux, Inc, f1, f2, f3
        End If
        Select Case TypeOfCalc
            Case BubT
                SatPressureCalc.SatPressure 1, Aux, PSat
                f3 = mvarFeedFlow.MolarFraction(1) * PSat
                For i = 2 To n
                    SatPressureCalc.SatPressure i, Aux, PSat
                    f3 = f3 + mvarFeedFlow.MolarFraction(i) * PSat
                Next
                f3 = f3 / lclFeedP
            Case DewT
                SatPressureCalc.SatPressure 1, Aux, PSat
                f3 = mvarFeedFlow.MolarFraction(1) / PSat
                For i = 2 To n
                    SatPressureCalc.SatPressure i, Aux, PSat
                    f3 = f3 + mvarFeedFlow.MolarFraction(i) / PSat
                Next
                f3 = f3 * lclFeedP
        End Select
        f3 = Log(f3)
        Inc = NuevoIncremento(T1, T2, Aux, f1, f2, f3, itera)
    Loop Until Abs(Inc) < 0.1 Or itera > mvarTolerances.MaxIter
    TempIni = Aux
' ---- Code Body Ends ---- (Default)
Exit Function

TempIni_ErrorHandler:
  Err.Raise Err.Number

Exit_TempIni:

End Function
Private Sub FillVaporFraction(vData As TypeCalculation, Optional Val As Double)
On Error GoTo FillVaporFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Select Case vData
        Case BubP, BubT
            mvarFeedFlow.VaporFraction = 0
        Case DewP, DewT
            mvarFeedFlow.VaporFraction = 1
        Case Flash
            mvarFeedFlow.VaporFraction = Val
    End Select
    mvarLiquidFlow.VaporFraction = 0
    mvarVaporFlow.VaporFraction = 1
' ---- Code Body Ends ---- (Default)
Exit Sub

FillVaporFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_FillVaporFraction:

End Sub
Public Function BubblePointPressure() As Boolean
On Error GoTo BubblePointPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, n As Integer, h As Double, S As Double
    Dim Pini As Double
    n = mvarMixture.NumberOfComponents
    ReDim y(1 To n) As Double
    DimSolverObjects n
    If mvarBoolgammaphi Then
        RaoultFirstAprox BubP, n
        Pini = mvarFeedFlow.Pressure
        With mvarVaporFlow
            For i = 1 To n: y(i) = .MolarFraction(i): Next
        End With
        Ekilib_BubblePointPressure n, Pini, y()
    Else
        BubblePointPressureHP n
    End If
    FillVaporFraction BubP
    With mvarFeedFlow
        Enthalpy_Entropy n, .Temperature, .Pressure, .VaporFraction, h, S
        .Enthalpy = h
        .Entropy = S
    End With
    'ClearSolverObjects
' ---- Code Body Ends ---- (Default)
Exit Function

BubblePointPressure_ErrorHandler:

  Err.Raise Err.Number

Exit_BubblePointPressure:

End Function
Private Sub BubblePointPressureHP(ByVal n As Integer)
On Error GoTo BubblePointPressureHP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tc As Double, Pc As Double, vc As Double
    Dim Trivial As Boolean, Pini As Double
    Dim myFinalT As Double, DeltaT As Double
    Dim P1 As Double, P2 As Double, P3 As Double
    Dim T1 As Double, T2 As Double, T3 As Double
    Dim deriv1 As Double, deriv1a As Double, deriv2 As Double
    Dim flagExit As Boolean, i As Integer
    ReDim myy(1 To n) As Double, y(1 To n) As Double, X(1 To n) As Double
    RaoultFirstAprox BubP, n
    Pini = mvarFeedFlow.Pressure
    For i = 1 To n
        y(i) = mvarVaporFlow.MolarFraction(i)
        X(i) = mvarLiquidFlow.MolarFraction(i)
    Next
    Ekilib_BubblePointPressure n, Pini, y()
    With mvarVaporFlow
        For i = 1 To n
            y(i) = .MolarFraction(i)
        Next
    End With
    Trivial = Compara(X(), y(), mvarTolerances.TolMolarFraction) '0.01) '
    If Trivial = True Then
        With mvarFeedFlow
            For i = 1 To n
               LiquidSolver.MolarFraction(i) = .MolarFraction(i)
               myy(i) = .MolarFraction(i)
            Next
            myFinalT = .Temperature
        End With
        LiquidSolver.psCriticalProperties Tc, vc, Pc
        T1 = 0.8 * Tc
        DeltaT = (myFinalT - T1) / 10
        mvarFeedFlow.Temperature = T1
        RaoultFirstAprox BubP, n
        P1 = mvarFeedFlow.Pressure
        With mvarVaporFlow
            For i = 1 To n: myy(i) = .MolarFraction(i): Next
        End With
        NR_BubblePointPressure P1, myy(), n
        deriv1 = 0
        T2 = T1 + DeltaT
        P2 = P1 + deriv1 * DeltaT
        mvarFeedFlow.Temperature = T2
        NR_BubblePointPressure P2, myy(), n
        deriv2 = 0
        Do While Not flagExit
            DoEvents
            T3 = T2 + DeltaT
            P3 = P2 + DeltaT * (deriv1 + deriv2 * DeltaT)
            mvarFeedFlow.Temperature = T3
            With mvarVaporFlow
                For i = 1 To n: myy(i) = .MolarFraction(i): Next
            End With
            NR_BubblePointPressure P3, myy(), n
            P3 = mvarFeedFlow.Pressure
            deriv1a = deriv1
            deriv1 = (P3 - P2) / (T3 - T2)
            deriv2 = (deriv1 - deriv1a) / (T3 - T1)
            P1 = P2
            T1 = T2
            P2 = P3
            T2 = T3
            If Abs(T3 - myFinalT) / myFinalT < 0.0000001 Then flagExit = True
        Loop
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

BubblePointPressureHP_ErrorHandler:
  Err.Raise Err.Number

Exit_BubblePointPressureHP:

End Sub
Private Function DewpointPressureHP(ByVal n As Integer)
On Error GoTo DewpointPressureHP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tc As Double, Pc As Double, vc As Double
    Dim Trivial As Boolean, Pini As Double
    Dim myFinalT As Double, DeltaT As Double
    Dim P1 As Double, P2 As Double, P3 As Double
    Dim T1 As Double, T2 As Double, T3 As Double
    Dim deriv1 As Double, deriv1a As Double, deriv2 As Double
    Dim flagExit As Boolean, i As Integer
    ReDim mXX(1 To n) As Double, y(1 To n) As Double, X(1 To n) As Double
    RaoultFirstAprox DewP, n
    Pini = mvarFeedFlow.Pressure
    For i = 1 To n
        y(i) = mvarVaporFlow.MolarFraction(i)
        X(i) = mvarLiquidFlow.MolarFraction(i)
    Next
    Ekilib_DewPointPressure n, Pini, X()
    For i = 1 To n: X(i) = mvarLiquidFlow.MolarFraction(i): Next
    Trivial = Compara(X(), y(), mvarTolerances.TolMolarFraction)
    If Trivial = True Then
        With mvarFeedFlow
            For i = 1 To n
                VaporSolver.MolarFraction(i) = .MolarFraction(i)
                mXX(i) = .MolarFraction(i)
            Next
            LiquidSolver.psCriticalProperties Tc, vc, Pc
            myFinalT = .Temperature
            T1 = 0.9 * Tc
            DeltaT = (myFinalT - T1) / 10
            .Temperature = T1
            RaoultFirstAprox DewP, n
            P1 = .Pressure
        End With
        For i = 1 To n: mXX(i) = mvarLiquidFlow.MolarFraction(i): Next
        NR_DewPointPressure P1, mXX(), n
        deriv1 = 0
        T2 = T1 + DeltaT
        P2 = P1 + deriv1 * DeltaT
        mvarFeedFlow.Temperature = T2
        NR_DewPointPressure P2, mXX(), n
        deriv2 = 0
        Do While Not flagExit
            DoEvents
            T3 = T2 + DeltaT
            P3 = P2 + DeltaT * (deriv1 + deriv2 * DeltaT)
            mvarFeedFlow.Temperature = T3
            With mvarLiquidFlow
                For i = 1 To n: mXX(i) = .MolarFraction(i): Next
            End With
            NR_DewPointPressure P3, mXX(), n
            P3 = mvarFeedFlow.Pressure
            deriv1a = deriv1
            deriv1 = (P3 - P2) / (T3 - T2)
            deriv2 = (deriv1 - deriv1a) / (T3 - T1)
            P1 = P2
            T1 = T2
            P2 = P3
            T2 = T3
            If Abs(T3 - myFinalT) / myFinalT < 0.0000001 Then flagExit = True
        Loop
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

DewpointPressureHP_ErrorHandler:
  Err.Raise Err.Number

Exit_DewpointPressureHP:

End Function
Private Function DewPointTemperatureHP(ByVal n As Integer)
On Error GoTo DewPointTemperatureHP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tc As Double, Pc As Double, vc As Double
    Dim Trivial As Boolean, Tini As Double
    Dim myFinalP As Double, DeltaP As Double
    Dim P1 As Double, P2 As Double, P3 As Double
    Dim T1 As Double, T2 As Double, T3 As Double
    Dim deriv1 As Double, deriv1a As Double, deriv2 As Double
    Dim flagExit As Boolean, i As Integer
    ReDim myy(1 To n) As Double, y(1 To n) As Double, X(1 To n) As Double
    RaoultFirstAprox DewT, n
    Tini = mvarFeedFlow.Temperature
    For i = 1 To n
        y(i) = mvarVaporFlow.MolarFraction(i)
        X(i) = mvarLiquidFlow.MolarFraction(i)
    Next
    Ekilib_DewPointTemperature n, Tini, X()
    For i = 1 To n: X(i) = mvarLiquidFlow.MolarFraction(i): Next
    Trivial = Compara(X(), y(), mvarTolerances.TolMolarFraction)
    If Trivial = True Then
        With mvarFeedFlow
            For i = 1 To n
                VaporSolver.MolarFraction(i) = .MolarFraction(i)
                myy(i) = .MolarFraction(i)
            Next
            VaporSolver.psCriticalProperties Tc, vc, Pc
            myFinalP = .Pressure
            P1 = 0.9 * Pc
            DeltaP = (myFinalP - P1) / 10
            .Pressure = P1
            RaoultFirstAprox DewT, n
            T1 = .Temperature
            For i = 1 To n: myy(i) = mvarLiquidFlow.MolarFraction(i): Next
            NR_DewPointTemperature T1, myy(), n
            deriv1 = 0
            T1 = .Temperature
            P2 = P1 + DeltaP
            T2 = T1 + deriv1 * DeltaP
            .Pressure = P2
            For i = 1 To n: myy(i) = mvarLiquidFlow.MolarFraction(i): Next
            NR_DewPointTemperature T2, myy(), n
            T2 = .Temperature
            deriv2 = 0
        End With
        Do While Not flagExit
            DoEvents
            P3 = P2 + DeltaP
            T3 = T2 + DeltaP * (deriv1 + deriv2 * DeltaP)
            mvarFeedFlow.Pressure = P3
            With mvarLiquidFlow
                For i = 1 To n: myy(i) = .MolarFraction(i): Next
            End With
            NR_DewPointTemperature T3, myy(), n
            T3 = mvarFeedFlow.Temperature
            deriv1a = deriv1
            deriv1 = (T3 - T2) / (P3 - P2)
            deriv2 = (deriv1 - deriv1a) / (P3 - P1)
            P1 = P2
            T1 = T2
            P2 = P3
            T2 = T3
            If Abs(P3 - myFinalP) / myFinalP < 0.0000001 Then flagExit = True
        Loop
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

DewPointTemperatureHP_ErrorHandler:
  Err.Raise Err.Number

Exit_DewPointTemperatureHP:

End Function
Private Function Ekilib_BubblePointPressure(ByVal n As Integer, firstP As Double, firstY() As Double)
On Error GoTo Ekilib_BubblePointPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim DeltaTP As Double, i As Integer, PresAux As Double
    Dim Iteraciones As Integer, Converged As Boolean
    Dim tolPres As Double, TP1 As Double, TP2 As Double, f1 As Double, f2 As Double, f3 As Double
    Dim VarUtil As Double, lclFeedT As Double
    Converged = False: MaxVar = 0.25
    DeltaTP = 0: PresAux = 0
    tolPres = mvarTolerances.TolPressure
    'DimSolverObjects n
    With mvarFeedFlow
        .Pressure = firstP
        lclFeedT = .Temperature
    End With
    For i = 1 To n
        mvarVaporFlow.MolarFraction(i) = firstY(i)
        LiquidSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        VaporSolver.MolarFraction(i) = mvarVaporFlow.MolarFraction(i)
    Next
    VarUtil = mvarFeedFlow.Pressure
'    VarUtil = 2654481.55787
    Do
        DoEvents
        Iteraciones = Iteraciones + 1
        If Iteraciones >= 2 Then
            IncrementarYRodar TP1, TP2, VarUtil, DeltaTP, f1, f2, f3
        End If
'aqui es donde se cae.........
        Ekilib_LazoK_Y f3, n, lclFeedT, VarUtil
        PresAux = Abs(f3) - 1
 '       f3 = Log(PresAux)
        f3 = Log(f3)
        DeltaTP = NuevoIncremento(TP1, TP2, VarUtil, f1, f2, f3, Iteraciones)
        If (Abs(DeltaTP) < tolPres) Or Iteraciones > mvarTolerances.MaxIter Or Abs(f3) < 0.000001 Then Converged = True
    Loop Until Converged = True
    mvarFeedFlow.Pressure = VarUtil
    With mvarVaporFlow
        .Pressure = VarUtil
        .Temperature = lclFeedT
    End With
    With mvarLiquidFlow
        .Pressure = VarUtil
        .Temperature = lclFeedT
    End With
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        mvarVaporFlow.MolarFraction(i) = VaporSolver.MolarFraction(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Ekilib_BubblePointPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_Ekilib_BubblePointPressure:

End Function
Private Sub Ekilib_LazoK_Y(ByRef sumaY As Double, n As Integer, T As Double, P As Double)
On Error GoTo Ekilib_LazoK_Y_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim itera As Integer, SumaAnt As Double, i As Integer, Converged As Boolean
    ReDim y2(1 To n) As Double
    ReDim Ki(1 To n) As Double
    Converged = False: sumaY = -2000: itera = 0
    With VaporSolver
        For i = 1 To n: y2(i) = .MolarFraction(i): Next
    End With
    Do
        SumaAnt = sumaY
        sumaY = 0
        itera = itera + 1
        With VaporSolver
            For i = 1 To n: .MolarFraction(i) = y2(i): Next
        End With
        CalculateKi n, T, P, Ki()
        With mvarFeedFlow
            For i = 1 To n
                y2(i) = Ki(i) * .MolarFraction(i)
                sumaY = sumaY + y2(i)
            Next
        End With
        Norm y2(), sumaY
        If Abs(sumaY - SumaAnt) < mvarTolerances.TolMolarFraction * n Then Converged = True
    Loop Until Converged
' ---- Code Body Ends ---- (Default)
Exit Sub

Ekilib_LazoK_Y_ErrorHandler:
  Err.Raise Err.Number

Exit_Ekilib_LazoK_Y:

End Sub
Private Sub Ekilib_LazoK_X(ByRef sumaX As Double, n As Integer, T As Double, P As Double)
On Error GoTo Ekilib_LazoK_X_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim itera As Integer, SumaAnt As Double, i As Integer, Converged As Boolean
    ReDim x2(1 To n) As Double
    ReDim Ki(1 To n) As Double
    With LiquidSolver
        For i = 1 To n: x2(i) = .MolarFraction(i): Next
    End With
    Converged = False: sumaX = -2000: itera = 0
    Do
        SumaAnt = sumaX
        sumaX = 0
        itera = itera + 1
        With LiquidSolver
            For i = 1 To n: .MolarFraction(i) = x2(i): Next
        End With
        CalculateKi n, T, P, Ki()
        With mvarFeedFlow
            For i = 1 To n
                x2(i) = .MolarFraction(i) / Ki(i)
                sumaX = sumaX + x2(i)
            Next
        End With
        Norm x2(), sumaX
        If (sumaX - SumaAnt) < mvarTolerances.TolMolarFraction * n Then Converged = True
    Loop Until Converged = True
' ---- Code Body Ends ---- (Default)
Exit Sub

Ekilib_LazoK_X_ErrorHandler:
  Err.Raise Err.Number

Exit_Ekilib_LazoK_X:

End Sub
Private Function Enthalpy_Entropy(ByVal n As Integer, T As Double, P As Double, Beta As Double, ByRef h As Double, ByRef S As Double)
On Error GoTo Enthalpy_Entropy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim HRl As Double, HRv As Double, Srl As Double, Srv As Double
    Dim SatP As Double, Sum As Double, sum2 As Double, i As Integer
    Dim lHid As Double, vHid As Double, lSid As Double, vSid As Double
    Dim Tr As Double, Pr As Double
    Dim Hl As Double, Hv As Double, Sl As Double, Sv As Double, Z As Double
    
    'parte Liquida
    Select Case mvarMixture.Liquid1Model
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
        RP1978_Qbic, Berth1899_Qbic
            With LiquidSolver
                .MixingRules T, P, DimLess, MRule
                Z = .ZSolver(PhaseIDLiquid)
                .ResidualProperties T, P, Z, HRl, Srl
            End With
        Case vanLaar, Wilson, ScatchardHild, Margules, idealsolution
            Sum = 0
            sum2 = 0
            Dim Purenew As Object
                With LiquidSolver
                    For i = 1 To n
                        SatPressureCalc.SatPressure i, T, SatP
                        Tr = T / mvarMixture.Properties(i).Tc
                        Pr = SatP / mvarMixture.Properties(i).Pc
                            Select Case mvarMixture.VaporModel
                                Case Is = -1
                                    Set Purenew = New clsVirial
                                    Purenew.AcentricFactor = mvarMixture.Properties(i).w
                                    Purenew.HR_SR Tr, Pr, HRl, Srl
                                    HRl = HRl * UnivGasConst * mvarMixture.Properties(i).Tc
                                    Srl = Srl * UnivGasConst
                                Case -2
                                    HRv = 0
                                    Srv = 0
                                Case Else
                                    Set Purenew = New clsQbics
                                    Purenew.Model = mvarMixture.VaporModel
                                    Purenew.AcentricFactor = mvarMixture.Properties(i).w
                                    Purenew.Tc = mvarMixture.Properties(i).Tc
                                    Purenew.Pc = mvarMixture.Properties(i).Pc
                                    Purenew.Zc = mvarMixture.Properties(i).Zc
                                    Purenew.m = mvarMixture.Properties(i).m
                                    Purenew.n = mvarMixture.Properties(i).n
                                    Purenew.g = mvarMixture.Properties(i).g
                                    Purenew.PRSVK1 = mvarMixture.Properties(i).K1
                                    Purenew.PsatAtTr = mvarMixture.Properties(i).PsatAtTr
                                    Purenew.ZGivenTrPr Tr, Pr, Z
                                    Purenew.ResidualEnthalpy Tr, Pr, Z, HRl
                                    Purenew.ResidualEntropy Tr, Pr, Z, Srl
                                    HRl = HRl * UnivGasConst * T
                                    Srl = Srl * UnivGasConst
                                End Select
                            Set Purenew = Nothing
                            Sum = Sum + .MolarFraction(i) * (UnivGasConst * Log(SatP / mvarReferenceState.Po) + Srl + .IvdP_s(SatP, P, i, T) + UnivGasConst * Log(.MolarFraction(i)))
                            sum2 = sum2 + .MolarFraction(i) * (HRl + .IvdP_h(SatP, P, i, T))
                    Next
                    If Not .Model = idealsolution Then
                        Srl = Sum - (PE_Hgf(n, T, P) / T) + .exEntropy(T)
                        HRl = sum2 + .exEnthalpy(T) - PE_Hgf(n, T, P)
                    Else
                        Srl = Sum - (PE_Hgf(n, T, P) / T)
                        HRl = sum2 - PE_Hgf(n, T, P)
                    End If
                End With
            End Select
    Select Case mvarMixture.VaporModel
        Case PR1976_Qbic, RK1949_Qbic, RKS1972_Qbic, VdW1870_Qbic, PRL1997_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, _
        RP1978_Qbic, Berth1899_Qbic
            With VaporSolver
                .MixingRules T, P, DimLess, MRule
                Z = .ZSolver(PhaseIDvapor)
                .ResidualProperties T, P, Z, HRv, Srv
            End With
        Case IdealGas
            HRv = 0
            Srv = 0
        Case virial
            With VaporSolver
                HRv = .HR(P, T)
                Srv = .SR(P, T)
            End With
    End Select
    
    With mvarReferenceState
        IdealEnthalpy n, T, .TTo, .Ho, lHid, vHid
        IdealEntropy n, T, .TTo, P, .Po, .So, lSid, vSid
    End With
    
    Hl = lHid + HRl
    Hv = vHid + HRv
    Sl = lSid + Srl
    Sv = vSid + Srv
    mvarVaporFlow.Enthalpy = Hv
    mvarVaporFlow.Entropy = Sv
    mvarLiquidFlow.Entropy = Sl
    mvarLiquidFlow.Enthalpy = Hl
    h = Hl + Beta * (Hv - Hl)
    S = Sl + Beta * (Sv - Sl)
' ---- Code Body Ends ---- (Default)
Exit Function

Enthalpy_Entropy_ErrorHandler:
  Err.Raise Err.Number

Exit_Enthalpy_Entropy:

End Function
Private Function Ekilib_DewPointPressure(ByVal n As Integer, firstP As Double, firstX() As Double)
On Error GoTo Ekilib_DewPointPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim DeltaTP As Double, i As Integer, Aux As Double, PresAux As Double
    Dim Iteraciones As Integer, Converged As Boolean
    Dim tolPres As Double, TP1 As Double, TP2 As Double, f1 As Double, f2 As Double, f3 As Double
    Dim VarUtil As Double, lclFeedT As Double
    DimSolverObjects n
    mvarFeedFlow.Pressure = firstP
    Converged = False
    MaxVar = 0.25
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = firstX(i)
        LiquidSolver.MolarFraction(i) = mvarLiquidFlow.MolarFraction(i)
        VaporSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
    Next
    lclFeedT = mvarFeedFlow.Temperature
    VarUtil = mvarFeedFlow.Pressure
    DeltaTP = 0
    PresAux = 0
    tolPres = mvarTolerances.TolPressure
    Aux = tolPres
    Do
        DoEvents
        Iteraciones = Iteraciones + 1
        If Iteraciones >= 2 Then
            IncrementarYRodar TP1, TP2, VarUtil, DeltaTP, f1, f2, f3
        End If
        Ekilib_LazoK_X f3, n, lclFeedT, VarUtil
        PresAux = Abs(f3) - 1
        f3 = Log(f3)
        DeltaTP = NuevoIncremento(TP1, TP2, VarUtil, f1, f2, f3, Iteraciones)
        If (Abs(DeltaTP) < Aux * VarUtil) Or Iteraciones > mvarTolerances.MaxIter Or Abs(f3) < 0.000001 Then Converged = True
    Loop Until Converged = True
    mvarFeedFlow.Pressure = VarUtil
    mvarVaporFlow.Pressure = VarUtil
    mvarLiquidFlow.Pressure = VarUtil
    mvarVaporFlow.Temperature = lclFeedT
    mvarLiquidFlow.Temperature = lclFeedT
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = LiquidSolver.MolarFraction(i)
        mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
    Next
    FillVaporFraction DewP, 1
' ---- Code Body Ends ---- (Default)
Exit Function

Ekilib_DewPointPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_Ekilib_DewPointPressure:

End Function
Private Function IdealEnthalpy(ByVal n As Integer, ByVal T As Double, ByVal Tref As Double, ByVal Ho As Double, ByRef lH As Double, ByRef vH As Double)
On Error GoTo IdealEnthalpy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Aux As Double, j As Integer
    Dim SumaLiq As Double, SumaVap As Double
    Dim Coef() As Double
    For i = 1 To n
        ReDim Coef(0 To mvarMixture.CpConsts(i).NbCoef)
        For j = 0 To mvarMixture.CpConsts(i).NbCoef
            Coef(j) = mvarMixture.CpConsts(i).Coef(j)
        Next
        Aux = DBO.FunctionIntegration(Tref, T, 0, mvarMixture.CpConsts(i).NbFormula, Coef())
        SumaLiq = SumaLiq + mvarLiquidFlow.MolarFraction(i) * Aux
        SumaVap = SumaVap + mvarVaporFlow.MolarFraction(i) * Aux
    Next
    lH = SumaLiq + Ho
    vH = SumaVap + Ho
' ---- Code Body Ends ---- (Default)
Exit Function

IdealEnthalpy_ErrorHandler:
  Err.Raise Err.Number

Exit_IdealEnthalpy:

End Function
Private Function IdealEntropy(ByVal n As Integer, ByVal T As Double, ByVal Tref As Double, ByVal P As Double, Pref As Double, So As Double, ByRef LS As Double, ByRef VS As Double)
On Error GoTo IdealEntropy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Aux As Double
    Dim SumaLiq As Double, SumaVap As Double
    Dim Coef() As Double, j As Integer
    For i = 1 To n
        ReDim Coef(0 To mvarMixture.CpConsts(i).NbCoef)
        For j = 0 To mvarMixture.CpConsts(i).NbCoef
            Coef(j) = mvarMixture.CpConsts(i).Coef(j)
        Next
        Aux = DBO.FunctionOverXIntegration(Tref, T, 0, mvarMixture.CpConsts(i).NbFormula, Coef())
        SumaLiq = SumaLiq + mvarLiquidFlow.MolarFraction(i) * Aux
        SumaVap = SumaVap + mvarVaporFlow.MolarFraction(i) * Aux
    Next
    For i = 1 To n
        If mvarLiquidFlow.MolarFraction(i) > 1E-60 Then
            SumaLiq = SumaLiq - UnivGasConst * mvarLiquidFlow.MolarFraction(i) * _
            Log(mvarLiquidFlow.MolarFraction(i))
        End If
        If mvarVaporFlow.MolarFraction(i) > 1E-60 Then
            SumaVap = SumaVap - UnivGasConst * mvarVaporFlow.MolarFraction(i) * _
            Log(mvarVaporFlow.MolarFraction(i))
        End If
    Next
    LS = SumaLiq + So - UnivGasConst * Log(P / Pref)
    VS = SumaVap + So - UnivGasConst * Log(P / Pref)
' ---- Code Body Ends ---- (Default)
Exit Function

IdealEntropy_ErrorHandler:
  Err.Raise Err.Number

Exit_IdealEntropy:

End Function
Private Function Ekilib_BubblePointTemperature(ByVal n As Integer, firstT As Double, firstY() As Double)
On Error GoTo Ekilib_BubblePointTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim DeltaTP As Double, i As Integer, Aux As Double, PresAux As Double
    Dim Iteraciones As Integer, Converged As Boolean
    Dim tolTemp As Double, TP1 As Double, TP2 As Double, f1 As Double, f2 As Double, f3 As Double
    Dim Iteracion As Integer, VarUtil As Double, lclFeedP As Double
    DimSolverObjects n
    mvarFeedFlow.Temperature = firstT
    Converged = False
    MaxVar = 0.25
    lclFeedP = mvarFeedFlow.Pressure
    For i = 1 To n
        mvarVaporFlow.MolarFraction(i) = firstY(i)
        LiquidSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        VaporSolver.MolarFraction(i) = mvarVaporFlow.MolarFraction(i)
    Next
    VarUtil = mvarFeedFlow.Temperature
    DeltaTP = 0: PresAux = 0
    tolTemp = mvarTolerances.TolTemperature
    Aux = tolTemp
    Do
        DoEvents
        Iteraciones = Iteraciones + 1
        If Iteraciones >= 2 Then
            IncrementarYRodar TP1, TP2, VarUtil, DeltaTP, f1, f2, f3
        End If
        Ekilib_LazoK_Y f3, n, VarUtil, lclFeedP
        PresAux = Abs(f3) - 1
        f3 = Log(f3)
        DeltaTP = NuevoIncremento(TP1, TP2, VarUtil, f1, f2, f3, Iteraciones)
        If (Abs(DeltaTP) < Aux) Or Iteracion > mvarTolerances.MaxIter Or Abs(f3) < 0.000001 Then Converged = True
    Loop Until Converged = True
    mvarFeedFlow.Temperature = VarUtil
    mvarVaporFlow.Temperature = VarUtil
    mvarLiquidFlow.Temperature = VarUtil
    mvarVaporFlow.Pressure = lclFeedP
    mvarLiquidFlow.Pressure = lclFeedP
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
        mvarVaporFlow.MolarFraction(i) = VaporSolver.MolarFraction(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Ekilib_BubblePointTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_Ekilib_BubblePointTemperature:

End Function
Private Function Ekilib_DewPointTemperature(ByVal n As Integer, firstT As Double, firstX() As Double)
On Error GoTo Ekilib_DewPointTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim DeltaTP As Double, i As Integer, Aux As Double, PresAux As Double
    Dim Iteraciones As Integer, Converged As Boolean
    Dim tolTemp As Double, TP1 As Double, TP2 As Double, f1 As Double, f2 As Double, f3 As Double
    Dim Iteracion As Integer, VarUtil As Double, lclFeedP As Double
    DimSolverObjects n
    mvarFeedFlow.Temperature = firstT
    Converged = False
    MaxVar = 0.25
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = firstX(i)
        LiquidSolver.MolarFraction(i) = mvarLiquidFlow.MolarFraction(i)
        VaporSolver.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
    Next
    lclFeedP = mvarFeedFlow.Pressure
    VarUtil = mvarFeedFlow.Temperature
    DeltaTP = 0
    PresAux = 0
    tolTemp = mvarTolerances.TolTemperature
    Aux = tolTemp
    Do
        DoEvents
        Iteraciones = Iteraciones + 1
        If Iteraciones >= 2 Then
               IncrementarYRodar TP1, TP2, VarUtil, DeltaTP, f1, f2, f3
        End If
        Ekilib_LazoK_X f3, n, VarUtil, lclFeedP
        PresAux = Abs(f3) - 1
        f3 = Log(f3)
        DeltaTP = NuevoIncremento(TP1, TP2, VarUtil, f1, f2, f3, Iteraciones)
        If (Abs(DeltaTP) < Aux * VarUtil) Or Iteracion > mvarTolerances.MaxIter Or Abs(f3) < 0.000001 Then: Converged = True
    Loop Until Converged = True
    mvarFeedFlow.Temperature = VarUtil
    mvarVaporFlow.Temperature = VarUtil
    mvarLiquidFlow.Temperature = VarUtil
    mvarVaporFlow.Pressure = lclFeedP
    mvarLiquidFlow.Pressure = lclFeedP
    For i = 1 To n
        mvarLiquidFlow.MolarFraction(i) = LiquidSolver.MolarFraction(i)
        mvarVaporFlow.MolarFraction(i) = mvarFeedFlow.MolarFraction(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Ekilib_DewPointTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_Ekilib_DewPointTemperature:

End Function
Public Function ExG(ByVal T As Double, ByVal n As Integer) As Double
On Error GoTo ExG_ErrorHandler

    Dim i As Integer, j As Integer, X() As Double, modelo As Integer
    ReDim X(1 To n)
    For i = 1 To n
        X(i) = mvarFeedFlow.MolarFraction(i)
    Next
        If MGamma Then
            modelo = mvarMixture.Liquid2Model
        Else
            modelo = mvarMixture.Liquid1Model
        End If

            If modelo = 25 Then
                Set LiquidS = New clsActivityMulticomp
                LiquidS.Model = modelo
                For i = 1 To n
                    LiquidS.MolarFraction(i) = X(i)
                    LiquidS.Properties.Add mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).vlModel
                Next i
            Else
                Set LiquidS = New clsActivityMulticomp
                LiquidS.Model = modelo
                LiquidS.Properties.AijTemperature = mvarMixture.Properties.AijTemperature
                For i = 1 To n
                    LiquidS.MolarFraction(i) = X(i)
                    LiquidS.Properties.Add mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).vlModel
                   For j = 1 To n
                        LiquidS.Properties.Aij(i, j) = mvarMixture.Properties.Aij(i, j)
                   Next j
                Next i
            End If
    ExG = LiquidS.exGibbs(T)
Exit Function

ExG_ErrorHandler:
  Err.Raise Err.Number
Exit_ExG:
End Function
Public Sub Parametro_EDEC_Modificadas(T As Double, i As Integer, ByRef SatP As Double)
    SatPressureCalc.SatPressure i, T, SatP
End Sub
Public Function ln_Gamma(ByVal T As Double, ByVal n As Integer, a As Integer) As Double
On Error GoTo ln_Gamma_ErrorHandler

    Dim j As Integer, X() As Double, Gamma As Double, i As Integer
    Dim modelo As Integer
    
    ReDim X(1 To n)
    For i = 1 To n
        X(i) = mvarFeedFlow.MolarFraction(i)
    Next
        If MGamma Then
            modelo = mvarMixture.Liquid2Model
        Else
            modelo = mvarMixture.Liquid1Model
        End If
            If modelo = 25 Then
                Set LiquidS = New clsActivityMulticomp
                LiquidS.Model = modelo
                For i = 1 To n
                    LiquidS.MolarFraction(i) = X(i)
                    LiquidS.Properties.Add mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).vlModel
                Next i
            Else
                Set LiquidS = New clsActivityMulticomp
                LiquidS.Model = modelo
                LiquidS.Properties.AijTemperature = mvarMixture.Properties.AijTemperature
                For i = 1 To n
                    LiquidS.MolarFraction(i) = X(i)
                    LiquidS.Properties.Add mvarMixture.Properties(i).delta, mvarMixture.Properties(i).Vo, mvarMixture.Properties(i).ZRA, mvarMixture.Properties(i).wSRK, mvarMixture.Properties(i).Tc, mvarMixture.Properties(i).Pc, mvarMixture.Properties(i).vlModel
                   For j = 1 To n
                        LiquidS.Properties.Aij(i, j) = mvarMixture.Properties.Aij(i, j)
                   Next j
                Next i
            End If
            LiquidS.CalcGamma a, T, Gamma
            
    ln_Gamma = Log(Gamma)
Exit Function

ln_Gamma_ErrorHandler:
  Err.Raise Err.Number
Exit_ln_Gamma:
End Function
