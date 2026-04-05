VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsFlow"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarMolarFraction() As Double
    Private mvarTemperature As Double
    Private mvarPressure As Double
    Private mvarVolume As Double
    Private mvarEnthalpy As Double
    Private mvarEntropy As Double
    Private mvarVaporFraction As Double
    Private mvarLiquid1Fraction As Double
    Private mvarLiquid2Fraction As Double
    Private mvarQuality As Double
Public Property Let Liquid2Fraction(ByVal vData As Double)
On Error GoTo Liquid2Fraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarLiquid2Fraction = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid2Fraction_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid2Fraction:

End Property
Public Property Get Liquid2Fraction() As Double
On Error GoTo Liquid2Fraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Liquid2Fraction = mvarLiquid2Fraction
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid2Fraction_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid2Fraction:

End Property
Public Property Let Liquid1Fraction(ByVal vData As Double)
On Error GoTo Liquid1Fraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarLiquid1Fraction = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid1Fraction_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid1Fraction:

End Property
Public Property Get Liquid1Fraction() As Double
On Error GoTo Liquid1Fraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Liquid1Fraction = mvarLiquid1Fraction
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid1Fraction_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid1Fraction:

End Property
Public Property Let VaporFraction(ByVal vData As Double)
On Error GoTo VaporFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarVaporFraction = vData
' ---- Code Body Ends ---- (Default)
Exit Property

VaporFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_VaporFraction:

End Property
Public Property Get VaporFraction() As Double
On Error GoTo VaporFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    VaporFraction = mvarVaporFraction
' ---- Code Body Ends ---- (Default)
Exit Property

VaporFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_VaporFraction:

End Property
Public Property Get Temperature() As Double
On Error GoTo Temperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Temperature = mvarTemperature
' ---- Code Body Ends ---- (Default)
Exit Property

Temperature_ErrorHandler:
  Err.Raise Err.Number

Exit_Temperature:

End Property
Public Property Let Temperature(T As Double)
On Error GoTo Temperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTemperature = T
' ---- Code Body Ends ---- (Default)
Exit Property

Temperature_ErrorHandler:
  Err.Raise Err.Number

Exit_Temperature:

End Property
Public Property Get Pressure() As Double
On Error GoTo Pressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Pressure = mvarPressure
' ---- Code Body Ends ---- (Default)
Exit Property

Pressure_ErrorHandler:
  Err.Raise Err.Number

Exit_Pressure:

End Property
Public Property Let Pressure(P As Double)
On Error GoTo Pressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPressure = P
' ---- Code Body Ends ---- (Default)
Exit Property

Pressure_ErrorHandler:
  Err.Raise Err.Number

Exit_Pressure:

End Property
Public Property Get Enthalpy() As Double
On Error GoTo Enthalpy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Enthalpy = mvarEnthalpy
' ---- Code Body Ends ---- (Default)
Exit Property

Enthalpy_ErrorHandler:
  Err.Raise Err.Number

Exit_Enthalpy:

End Property
Public Property Let Enthalpy(h As Double)
On Error GoTo Enthalpy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarEnthalpy = h
' ---- Code Body Ends ---- (Default)
Exit Property

Enthalpy_ErrorHandler:
  Err.Raise Err.Number

Exit_Enthalpy:

End Property
Public Property Get Entropy() As Double
On Error GoTo Entropy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Entropy = mvarEntropy
' ---- Code Body Ends ---- (Default)
Exit Property

Entropy_ErrorHandler:
  Err.Raise Err.Number

Exit_Entropy:

End Property
Public Property Let Entropy(S As Double)
On Error GoTo Entropy_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarEntropy = S
' ---- Code Body Ends ---- (Default)
Exit Property

Entropy_ErrorHandler:
  Err.Raise Err.Number

Exit_Entropy:

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
Public Property Let MolarFraction(Index As Integer, ByVal NewValue As Double)
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If Index > UBound(mvarMolarFraction()) Then ReDim Preserve mvarMolarFraction(Index)
    mvarMolarFraction(Index) = NewValue
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
   'Err.Raise Err.Number
    ReDim Preserve mvarMolarFraction(0)
    Resume Next
Exit_MolarFraction:

End Property
Private Sub Class_Initialize()
On Error GoTo Class_Initialize_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarLiquid2Fraction = 0
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Initialize_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Initialize:

End Sub
