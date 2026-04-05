VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsVirialMulticomp"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarCriticalProperties As colclsProperties
    Private mvarMolarFraction() As Double
    Private mvarKijParameters As Variant
Public Property Let KijParameters(ByVal vData As Variant)
On Error GoTo KijParameters_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarKijParameters = vData
' ---- Code Body Ends ---- (Default)
Exit Property

KijParameters_ErrorHandler:
  Err.Raise Err.Number

Exit_KijParameters:

End Property
Public Property Set KijParameters(ByVal vData As Object)
On Error GoTo KijParameters_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarKijParameters = vData
' ---- Code Body Ends ---- (Default)
Exit Property

KijParameters_ErrorHandler:
  Err.Raise Err.Number

Exit_KijParameters:

End Property
Public Property Get KijParameters() As Variant
On Error GoTo KijParameters_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If IsObject(mvarKijParameters) Then
        Set KijParameters = mvarKijParameters
    Else
        KijParameters = mvarKijParameters
    End If
' ---- Code Body Ends ---- (Default)
Exit Property

KijParameters_ErrorHandler:
  Err.Raise Err.Number

Exit_KijParameters:

End Property
Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As Double)
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Static sIndex As Boolean
    If sIndex = False Then
        If Index = 1 Then
            sIndex = True
            ReDim Preserve mvarMolarFraction(1 To 1 + Index)
        End If
    End If
    mvarMolarFraction(Index) = MolarFraction
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
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
Public Property Set CriticalProperties(ByVal vData As colclsProperties)
On Error GoTo CriticalProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarCriticalProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

CriticalProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_CriticalProperties:

End Property
Public Property Get CriticalProperties() As colclsProperties
On Error GoTo CriticalProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarCriticalProperties Is Nothing Then
        Set mvarCriticalProperties = New colclsProperties
    End If
    Set CriticalProperties = mvarCriticalProperties
' ---- Code Body Ends ---- (Default)
Exit Property

CriticalProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_CriticalProperties:

End Property
Public Function CalculatePartialFugacity(k As Integer, T As Double, P As Double) As Double
On Error GoTo CalculatePartialFugacity_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, j As Integer, Sum As Double, Aux As Double, Bkk As Double
    Sum = 0
    For i = LBound(mvarMolarFraction) To UBound(mvarMolarFraction)
        For j = LBound(mvarMolarFraction) To UBound(mvarMolarFraction)
            Sum = Sum + mvarMolarFraction(i) * mvarMolarFraction(j) * (2 * Deltaik(i, k, T) - Deltaik(i, j, T))
        Next
    Next
    Aux = (1 / 2) * Sum
    Bkk = calculateBij(k, k, T)
    Aux = (Bkk + Aux) * P / (UnivGasConst * T)
    CalculatePartialFugacity = Exp(Aux)
' ---- Code Body Ends ---- (Default)
Exit Function

CalculatePartialFugacity_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculatePartialFugacity:

End Function
Private Function calculateBij(i As Integer, j As Integer, T As Double) As Double
On Error GoTo calculateBij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tcij As Double, Pcij As Double, wij As Double, Bo As Double, B1 As Double
    Dim Zcij As Double, Vcij As Double, Vci As Double, Vcj As Double
    wij = (mvarCriticalProperties.Item(i).w + mvarCriticalProperties.Item(j).w) / 2
    If VarType(mvarKijParameters) = vbEmpty Then
        Tcij = ((mvarCriticalProperties.Item(i).Tc * mvarCriticalProperties.Item(j).Tc)) ^ (1 / 2)
    Else
        Tcij = ((mvarCriticalProperties.Item(i).Tc * mvarCriticalProperties.Item(j).Tc)) ^ (1 / 2) * (1 - mvarKijParameters(i, j))
    End If
    Bo = 0.083 - 0.422 / (T / Tcij) ^ 1.6
    B1 = 0.139 - 0.172 / (T / Tcij) ^ 4.2
    Zcij = (mvarCriticalProperties.Item(i).Zc + mvarCriticalProperties.Item(j).Zc) / 2
    Vci = mvarCriticalProperties.Item(i).Zc * (UnivGasConst * mvarCriticalProperties.Item(i).Tc) / mvarCriticalProperties.Item(i).Pc
    Vcj = mvarCriticalProperties.Item(j).Zc * (UnivGasConst * mvarCriticalProperties.Item(j).Tc) / mvarCriticalProperties.Item(j).Pc
    Vcij = ((Vci ^ (1 / 3) + Vcj ^ (1 / 3)) / 2) ^ 3
    Pcij = Zcij * UnivGasConst * Tcij / Vcij
    calculateBij = (UnivGasConst * Tcij) / Pcij * (Bo + wij * B1)
' ---- Code Body Ends ---- (Default)
Exit Function

calculateBij_ErrorHandler:
  Err.Raise Err.Number

Exit_calculateBij:

End Function
Public Function HR(P As Double, T As Double) As Double
On Error GoTo HR_ErrorHandler

' ---- Code Body Starts ---- (Default)
    HR = (P / UnivGasConst) * (calculateB(T) / T - calculatedBdT(T))
' ---- Code Body Ends ---- (Default)
Exit Function

HR_ErrorHandler:
  Err.Raise Err.Number

Exit_HR:

End Function
Public Function SR(P As Double, T As Double) As Double
On Error GoTo SR_ErrorHandler

' ---- Code Body Starts ---- (Default)
    SR = -(P / UnivGasConst) * calculatedBdT(T)
' ---- Code Body Ends ---- (Default)
Exit Function

SR_ErrorHandler:
  Err.Raise Err.Number

Exit_SR:

End Function
Private Function Deltaik(i As Integer, j As Integer, T As Double) As Double
On Error GoTo Deltaik_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Deltaik = 2 * calculateBij(i, j, T) - calculateBij(i, i, T) - calculateBij(j, j, T)
' ---- Code Body Ends ---- (Default)
Exit Function

Deltaik_ErrorHandler:
  Err.Raise Err.Number

Exit_Deltaik:

End Function
Public Function CalculateZ(T As Double, P As Double) As Double
On Error GoTo CalculateZ_ErrorHandler

' ---- Code Body Starts ---- (Default)
    CalculateZ = 1 + calculateB(T) * P / (UnivGasConst * T)
' ---- Code Body Ends ---- (Default)
Exit Function

CalculateZ_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculateZ:

End Function
Private Function CalculatedBijdT(i As Integer, j As Integer, T As Double) As Double
On Error GoTo CalculatedBijdT_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tcij As Double, Pcij As Double, wij As Double, dBodT As Double, dB1dT As Double
    Dim Zcij As Double, Vcij As Double, Vci As Double, Vcj As Double
    wij = (mvarCriticalProperties.Item(i).w + mvarCriticalProperties.Item(j).w) / 2
    If VarType(mvarKijParameters) = vbEmpty Then
        Tcij = ((mvarCriticalProperties.Item(i).Tc * mvarCriticalProperties.Item(j).Tc)) ^ (1 / 2)
    Else
        Tcij = ((mvarCriticalProperties.Item(i).Tc * mvarCriticalProperties.Item(j).Tc)) ^ (1 / 2) * (1 - mvarKijParameters(i, j))
    End If
    dBodT = (0.6752 / Tcij) / (T / Tcij) ^ 2.6
    dB1dT = (0.7224 / Tcij) / (T / Tcij) ^ 5.2
    Zcij = (mvarCriticalProperties.Item(i).Zc + mvarCriticalProperties.Item(j).Zc) / 2
    Vci = mvarCriticalProperties.Item(i).Zc * (UnivGasConst * mvarCriticalProperties.Item(i).Tc) / mvarCriticalProperties.Item(i).Pc
    Vcj = mvarCriticalProperties.Item(j).Zc * (UnivGasConst * mvarCriticalProperties.Item(j).Tc) / mvarCriticalProperties.Item(j).Pc
    Vcij = ((Vci ^ (1 / 3) + Vcj ^ (1 / 3)) / 2) ^ 3
    Pcij = Zcij * UnivGasConst * Tcij / Vcij
    CalculatedBijdT = (UnivGasConst * Tcij) / Pcij * (dBodT + wij * dB1dT)
' ---- Code Body Ends ---- (Default)
Exit Function

CalculatedBijdT_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculatedBijdT:

End Function
Private Function calculateB(T As Double) As Double
On Error GoTo calculateB_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Sum As Double, i As Integer, j As Integer
    Sum = 0
    For i = 1 To UBound(mvarMolarFraction)
        For j = 1 To UBound(mvarMolarFraction)
        Sum = Sum + mvarMolarFraction(i) * mvarMolarFraction(j) * calculateBij(i, j, T)
        Next
    Next
    calculateB = Sum
' ---- Code Body Ends ---- (Default)
Exit Function

calculateB_ErrorHandler:
  Err.Raise Err.Number

Exit_calculateB:

End Function
Private Function calculatedBdT(T As Double) As Double
On Error GoTo calculatedBdT_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Sum As Double, i As Integer, j As Integer
    Sum = 0
    For i = 1 To UBound(mvarMolarFraction)
        For j = 1 To UBound(mvarMolarFraction)
        Sum = Sum + mvarMolarFraction(i) * mvarMolarFraction(j) * CalculatedBijdT(i, j, T)
        Next
    Next
    calculatedBdT = Sum
' ---- Code Body Ends ---- (Default)
Exit Function

calculatedBdT_ErrorHandler:
  Err.Raise Err.Number

Exit_calculatedBdT:

End Function
