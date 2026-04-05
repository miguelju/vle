VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsActivityMulticomp"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Attribute VB_Ext_KEY = "Member0" ,"colclsActivityProps"
Option Explicit
    Public Enum TADiPGammaModel
        IdealSolutiong = 25
        VanLaarg = 21
        Wilsong = 22
        ScatchardHg = 23
        Margulesg = 24
    End Enum
    Public Enum TADiPvlModel
        Rackett = 1
        Thomson = 2
    End Enum
    
    Private mvarProperties As colclsProperties
    Private mvarModel As TADiPGammaModel
    Private x() As Double
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

' ---- Code Body Starts ----
        Set mvarProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_properties:

End Property
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
Public Function CalcGamma(vntIndexKey As Integer, T As Double, ByRef Gamma As Double) As Boolean
On Error GoTo CalcGamma_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Aux As Double, Suma As Double, Suma1 As Double, k As Integer
    Select Case mvarModel
        Case IdealSolutiong
              Gamma = 1
        Case ScatchardHg
            For k = 1 To UBound(x())
                Suma1 = Suma1 + x(k) * vl(k, T)
            Next
            For k = 1 To UBound(x())
                Suma = Suma + x(k) * vl(k, T) * mvarProperties(k).delta / Suma1
            Next
            Aux = (vl(vntIndexKey, T) / (UnivGasConst * T)) * (mvarProperties(vntIndexKey).delta - Suma) ^ 2
            Gamma = Exp(Aux)
        Case Margulesg 'solo para mezclas binarias
            ChangeAij T
            If vntIndexKey = 1 Then
              Aux = (mvarProperties.Aij(1, 2) + 2 * (mvarProperties.Aij(2, 1) _
              - mvarProperties.Aij(1, 2)) * x(1)) * x(2) * x(2)
            Else
              Aux = x(1) * x(1) * (mvarProperties.Aij(2, 1) + 2 * x(2) * _
              (mvarProperties.Aij(1, 2) - mvarProperties.Aij(2, 1)))
            End If
            Gamma = Exp(Aux)
        Case VanLaarg 'con CoefAIJ(i,i) = 0
               Dim j As Integer
               Dim Sumaij As Double, Sumaji As Double
               ChangeAij T
               For j = 1 To UBound(x)
                    Sumaij = Sumaij + x(j) * mvarProperties.Aij(vntIndexKey, j)
                    Sumaji = Sumaji + x(j) * mvarProperties.Aij(j, vntIndexKey)
                Next
               Gamma = Exp(Sumaij * (1 - x(vntIndexKey)) * (Sumaji / (x(vntIndexKey) * Sumaij + _
               (1 - x(vntIndexKey)) * Sumaji)) ^ 2)
        Case Wilsong 'con CoefAIJ(i,i) = 1
               Gamma = Exp(1 - Log(SumasWilson(T, vntIndexKey)) - SumaAnidada(T, vntIndexKey))
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

CalcGamma_ErrorHandler:
  Err.Raise Err.Number

Exit_CalcGamma:

End Function
Private Function WilsonAij(T As Double, i As Integer, j As Integer) As Double
On Error GoTo WilsonAij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If i <> j Then
        WilsonAij = (vl(j, T) / vl(i, T)) * Exp(-mvarProperties.Aij(i, j) / (UnivGasConst * T)) ' 1.987
    Else
        WilsonAij = 1
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

WilsonAij_ErrorHandler:
  Err.Raise Err.Number

Exit_WilsonAij:

End Function
Private Function SumasWilson(T As Double, vntIndexKey As Integer) As Double
On Error GoTo SumasWilson_ErrorHandler

' ---- Code Body Starts ---- (Default)

    Dim sumaaux As Double
    Dim j As Integer
    For j = 1 To UBound(x)
        sumaaux = sumaaux + x(j) * WilsonAij(T, vntIndexKey, j)
    Next
    SumasWilson = sumaaux
' ---- Code Body Ends ---- (Default)
Exit Function

SumasWilson_ErrorHandler:
  Err.Raise Err.Number

Exit_SumasWilson:

End Function
Private Function SumaAnidada(T As Double, vntIndexKey As Integer) As Double
On Error GoTo SumaAnidada_ErrorHandler

' ---- Code Body Starts ---- (Default)

    Dim sumaaux As Double
    Dim j As Integer
    For j = 1 To UBound(x)
        sumaaux = sumaaux + x(j) * WilsonAij(T, j, vntIndexKey) / SumasWilson(T, j)
    Next
    SumaAnidada = sumaaux
' ---- Code Body Ends ---- (Default)
Exit Function

SumaAnidada_ErrorHandler:
  Err.Raise Err.Number

Exit_SumaAnidada:

End Function
Public Property Get MolarFraction(Index As Integer) As Double
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    MolarFraction = x(Index)
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As Double)
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ReDim Preserve x(Index)
    x(Index) = MolarFraction
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
Public Property Let Model(ByVal vData As TADiPGammaModel)
On Error GoTo Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarModel = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Model:

End Property
Public Property Get Model() As TADiPGammaModel
On Error GoTo Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Model = mvarModel
' ---- Code Body Ends ---- (Default)
Exit Property

Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Model:

End Property

Public Function exGibbs(T As Double) As Double
On Error GoTo exGibbs_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Suma As Double, i As Integer, Gamma As Double
    For i = 1 To UBound(x())
        CalcGamma i, T, Gamma
        Suma = Suma + x(i) * Log(Gamma)
    Next
    exGibbs = UnivGasConst * T * Suma
' ---- Code Body Ends ---- (Default)
Exit Function

exGibbs_ErrorHandler:
  Err.Raise Err.Number

Exit_exGibbs:

End Function
Public Function exEnthalpy(T As Double) As Double
On Error GoTo exEnthalpy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim exGibbs_dT As Double, Part1 As Double, Part2 As Double
    Const h = 0.001
    Part2 = exGibbs(T)
    Part1 = exGibbs(T + h)
    exGibbs_dT = (Part1 - Part2) / h
    exEnthalpy = exGibbs(T) - T * exGibbs_dT
' ---- Code Body Ends ---- (Default)
Exit Function

exEnthalpy_ErrorHandler:
  Err.Raise Err.Number

Exit_exEnthalpy:

End Function
Public Function exEntropy(T As Double) As Double
On Error GoTo exEntropy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    exEntropy = (exEnthalpy(T) - exGibbs(T)) / T
' ---- Code Body Ends ---- (Default)
Exit Function

exEntropy_ErrorHandler:
  Err.Raise Err.Number

Exit_exEntropy:

End Function
Public Function vl(i As Integer, T As Double) As Double
On Error GoTo vl_ErrorHandler

' ---- Code Body Starts ---- (Default)
Const a = -1.52816
Const b = 1.43907
Const C = -0.81446
Const d = 0.190454
Const E = -0.296123
Const f = 0.386914
Const g = -0.042527258
Const h = -0.0480645
Dim Vro As Double, Vrs As Double, Tr As Double
Dim Toh As Double
    Select Case mvarProperties(i).vlModel
    Case 1
        Tr = T / mvarProperties(i).Tc
    If Tr <= 0.75 Then
        Toh = 1 + (1 - Tr) ^ (2 / 7)
    Else
        Toh = 1.6 + (0.00693026 / (Tr - 0.655))
    End If
        vl = ((UnivGasConst * mvarProperties(i).Tc) / mvarProperties(i).Pc) * mvarProperties(i).ZRA ^ Toh
    Case 2
        Tr = T / mvarProperties(i).Tc
        Vro = 1 + a * ((1 - Tr) ^ (1 / 3)) + b * ((1 - Tr) ^ (2 / 3)) + C * (1 - Tr) + d * ((1 - Tr) ^ (4 / 3))
        Vrs = (E + f * Tr + g * Tr ^ 2 + h * Tr ^ 3) / (Tr - 1.00001)
        vl = mvarProperties(i).Vo * Vro * (1 - mvarProperties(i).wSRK * Vrs)
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

vl_ErrorHandler:
  Err.Raise Err.Number

Exit_vl:

End Function
Public Function IvdP_h(P1 As Double, P2 As Double, i As Integer, T As Double) As Double
On Error GoTo IvdP_h_ErrorHandler

' ---- Code Body Starts ---- (Default)
Dim deriv As Double, temp As Double
Dim h As Double
    h = 0.0001
    temp = T + h
    h = temp - T
    deriv = (vl(i, temp) - vl(i, T)) / h
    deriv = T * deriv
    IvdP_h = (vl(i, T) - deriv) * (P2 - P1)
' ---- Code Body Ends ---- (Default)
Exit Function

IvdP_h_ErrorHandler:
  Err.Raise Err.Number

Exit_IvdP_h:

End Function
Public Function IvdP_s(P1 As Double, P2 As Double, i As Integer, T As Double) As Double
On Error GoTo IvdP_s_ErrorHandler

' ---- Code Body Starts ---- (Default)

Dim deriv As Double, temp As Double
Dim h As Double
    h = 0.0001
    temp = T + h
    h = temp - T
    deriv = (vl(i, temp) - vl(i, T)) / h
    deriv = -T * deriv
    IvdP_s = deriv * (P2 - P1)
' ---- Code Body Ends ---- (Default)
Exit Function

IvdP_s_ErrorHandler:
  Err.Raise Err.Number

Exit_IvdP_s:

End Function
Private Sub ChangeAij(T As Double)
On Error GoTo ChangeAij_ErrorHandler

' ---- Code Body Starts ---- (Default)

    Dim Change As Double, i As Integer, j As Integer, n As Integer, T1 As Double
    n = UBound(x())
    T1 = mvarProperties.AijTemperature
    If T <> T1 Then
        If mvarModel = Margulesg Or mvarModel = VanLaarg Then Change = T1 / T
        For i = 1 To n
            For j = 1 To n
               If i <> j Then
                   mvarProperties.Aij(i, j) = mvarProperties.Aij(i, j) * Change
               Else
                   mvarProperties.Aij(i, i) = 0
               End If
            Next
        Next
        mvarProperties.AijTemperature = T
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

ChangeAij_ErrorHandler:
  Err.Raise Err.Number

Exit_ChangeAij:
  
End Sub
