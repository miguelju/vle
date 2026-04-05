VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsSatPressure"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Public Enum TADiPSatPressureModel
        Riedel = 1
        Muller = 2
        RPM = 3 'Riedel-Plack-Miller
        DBPsat = 4
        Maxwell = 5
    End Enum
    Private DBO As New clsDataBase
    Private mvarProperties As colclsProperties
    'Private mvarProperties As colclsProperties
    Private mvarPsatConst As colclsPropertyDescription
    Private mvarSatPressureModel As TADiPSatPressureModel
Public Property Let SatPressureModel(ByVal vData As TADiPSatPressureModel)
On Error GoTo SatPressureModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarSatPressureModel = vData
' ---- Code Body Ends ---- (Default)
Exit Property

SatPressureModel_ErrorHandler:
  Err.Raise Err.Number

Exit_SatPressureModel:

End Property
Public Property Get SatPressureModel() As TADiPSatPressureModel
On Error GoTo SatPressureModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    SatPressureModel = mvarSatPressureModel
' ---- Code Body Ends ---- (Default)
Exit Property

SatPressureModel_ErrorHandler:
  Err.Raise Err.Number

Exit_SatPressureModel:

End Property
Public Property Get CriticalConstants() As colclsProperties
On Error GoTo CriticalConstants_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarProperties Is Nothing Then
        Set mvarProperties = New colclsProperties
    End If
    Set CriticalConstants = mvarProperties
' ---- Code Body Ends ---- (Default)
Exit Property

CriticalConstants_ErrorHandler:
  Err.Raise Err.Number

Exit_CriticalConstants:

End Property
Public Property Set CriticalConstants(vData As colclsProperties)
On Error GoTo CriticalConstants_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

CriticalConstants_ErrorHandler:
  Err.Raise Err.Number

Exit_CriticalConstants:

End Property
Public Property Get OtherConstants() As colclsProperties
On Error GoTo OtherConstants_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarProperties Is Nothing Then
        Set mvarProperties = New colclsProperties
    End If
    Set OtherConstants = mvarProperties
' ---- Code Body Ends ---- (Default)
Exit Property

OtherConstants_ErrorHandler:
  Err.Raise Err.Number

Exit_OtherConstants:

End Property
Public Property Set OtherConstants(vData As colclsProperties)
On Error GoTo OtherConstants_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

OtherConstants_ErrorHandler:
  Err.Raise Err.Number

Exit_OtherConstants:

End Property
Public Property Get PsatConst() As colclsPropertyDescription
On Error GoTo PsatConst_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarPsatConst Is Nothing Then
        Set mvarPsatConst = New colclsPropertyDescription
    End If
    Set PsatConst = mvarPsatConst
' ---- Code Body Ends ---- (Default)
Exit Property

PsatConst_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatConst:

End Property
Public Property Set PsatConst(vData As colclsPropertyDescription)
On Error GoTo PsatConst_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarPsatConst = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PsatConst_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatConst:

End Property
Public Function SatPressure(ByVal i As Integer, ByVal Temperature As Double, ByRef SatP As Double) As Boolean
On Error GoTo SatPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tr As Double, j As Integer
    Dim Aux As Double, Aux1 As Double, Prsat As Double
    Select Case mvarSatPressureModel
        Case Riedel
            Tr = mvarProperties.Item(i).Tb / mvarProperties.Item(i).Tc
            Aux = -35 + 36 / Tr + 42 * Log(Tr) - Tr ^ 6
            Aux = (0.315 * Aux + Log(mvarProperties.Item(i).Pc / 101325)) / (0.0838 * Aux - Log(Tr))
            Aux1 = 0.0838 * (3.758 - Aux)
            Tr = Temperature / mvarProperties.Item(i).Tc
            Aux = (-35 + 36 / Tr) * Aux1 + (42 * Aux1 + Aux) * Log(Tr) - Aux1 * Tr ^ 6
            Prsat = Exp(Aux)
            SatP = Prsat * mvarProperties(i).Pc
        Case RPM
            Tr = mvarProperties.Item(i).Tb / mvarProperties.Item(i).Tc
            Aux = Log((mvarProperties.Item(i).Pc) / 101325) * Tr / (1# - Tr)
            Aux1 = 0.4835 + 0.4605 * Aux
            Aux = (Aux / Aux1 - (1# + Tr)) / ((3# + Tr) * (1# - Tr) ^ 2)
            Tr = Temperature / mvarProperties.Item(i).Tc
            Aux = -Aux1 * (Aux * (3# + Tr) * (1# - Tr) ^ 3 - (Tr) ^ 2 + 1#) / Tr
            Prsat = Exp(Aux)
            SatP = Prsat * mvarProperties(i).Pc
        Case Muller
            Tr = mvarProperties.Item(i).Tb / mvarProperties.Item(i).Tc
            Aux = 5.37273 * (1 + mvarProperties.Item(i).w)
            Aux1 = 0.0134 - Log((mvarProperties.Item(i).Pc * 0.00001)) - Aux * (1# - 1# / Tr)
            Aux1 = Aux1 / (Log(Tr) - 0.832223 * (1# - 1# / Tr))
            Aux = Aux - 0.832223 * Aux1
            Tr = Temperature / mvarProperties.Item(i).Tc
            Aux = Aux * (1# - 1# / Tr) + Aux1 * Log(Tr)
            Prsat = Exp(Aux)
            SatP = Prsat * mvarProperties(i).Pc
        Case DBPsat
            ReDim Coef(0 To mvarPsatConst(i).NbCoef) As Double
            For j = 0 To mvarPsatConst(i).NbCoef
                Coef(j) = mvarPsatConst(i).Coef(j)
            Next
            Prsat = DBO.FunctionEvaluation(Temperature, 0, mvarPsatConst(i).NbFormula, Coef())
            SatP = Prsat * mvarProperties(i).Pc
    End Select
    SatPressure = True
' ---- Code Body Ends ---- (Default)
Exit Function

SatPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_SatPressure:

End Function
Public Function DSatPressure_T(ByVal i As Integer, ByVal Temperature As Double) As Double
On Error GoTo DSatPressure_T_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim h As Double, SatP As Double, SatPh As Double
    Dim temp
    h = 0.00001
    SatPressure i, Temperature, SatP
    SatPressure i, Temperature + h, SatPh
    temp = Temperature + h
    h = temp - Temperature
    DSatPressure_T = (SatPh - SatP) / h
' ---- Code Body Ends ---- (Default)
Exit Function

DSatPressure_T_ErrorHandler:
  Err.Raise Err.Number

Exit_DSatPressure_T:

End Function
Public Function SatTemperature(ByVal i As Integer, ByVal Pressure As Double, ByRef SatT As Double) As Boolean
On Error GoTo SatTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim lnPr As Double
    Pressure = Pressure / mvarProperties(i).Pc
    lnPr = Log(Pressure)
    SatT = (1 / (Regula_Falsi_M(lnPr, 1 / 0.2, 1, 0.00001, i)))
    SatT = SatT * mvarProperties(i).Tc
    SatTemperature = True
' ---- Code Body Ends ---- (Default)
Exit Function

SatTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_SatTemperature:

End Function
Private Function Regula_Falsi_M(Result As Double, ByVal x1 As Double, ByVal x2 As Double, Precision As Double, i As Integer)
On Error GoTo Regula_Falsi_M_ErrorHandler

' ---- Code Body Starts ---- (Default)
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
            MsgBox "No existe solución" + Str$(x3) + "la función no tiene cambio de signo"
            Regula_Falsi_M = x3
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
Private Function Funcion(x As Double, i As Integer) As Double
On Error GoTo Funcion_ErrorHandler

' VOY A HACER UN CAMBIO PORQUE ESTABAN CALCULANDO EL LOGARITMO DE PSAT Y NO DE PR
    Dim Prsat As Double, SatP As Double
    SatPressure i, (1 / x) * mvarProperties.Item(i).Tc, SatP
    Prsat = SatP / mvarProperties.Item(i).Pc
    Funcion = Log(Prsat)
' ---- Code Body Ends ---- (Default)
Exit Function

Funcion_ErrorHandler:
  Err.Raise Err.Number

Exit_Funcion:

End Function
Public Function DlnSatPressure_T(ByVal i As Integer, ByVal Temperature As Double) As Double
On Error GoTo DlnSatPressure_T_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim h As Double, SatP As Double, SatPh As Double
    Dim temp
    h = 0.00001
    SatPressure i, Temperature, SatP
    SatPressure i, Temperature + h, SatPh
    temp = Temperature + h
    h = temp - Temperature
    DlnSatPressure_T = (Log(SatPh) - Log(SatP)) / h
' ---- Code Body Ends ---- (Default)
Exit Function

DlnSatPressure_T_ErrorHandler:
  Err.Raise Err.Number

Exit_DlnSatPressure_T:

End Function
