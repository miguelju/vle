VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsTolerances"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarTolPressure As Double
    Private mvarTolTemperature As Double
    Private mvarMaxIter As Integer
    Private mvarTolMolarFraction As Double
Public Property Let TolMolarFraction(ByVal vData As Double)
On Error GoTo TolMolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTolMolarFraction = vData
' ---- Code Body Ends ---- (Default)
Exit Property

TolMolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_TolMolarFraction:

End Property
Public Property Get TolMolarFraction() As Double
On Error GoTo TolMolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    TolMolarFraction = mvarTolMolarFraction
' ---- Code Body Ends ---- (Default)
Exit Property

TolMolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_TolMolarFraction:

End Property
Public Property Let MaxIter(ByVal vData As Integer)
On Error GoTo MaxIter_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarMaxIter = vData
' ---- Code Body Ends ---- (Default)
Exit Property

MaxIter_ErrorHandler:
  Err.Raise Err.Number

Exit_MaxIter:

End Property
Public Property Get MaxIter() As Integer
On Error GoTo MaxIter_ErrorHandler

' ---- Code Body Starts ---- (Default)
    MaxIter = mvarMaxIter
' ---- Code Body Ends ---- (Default)
Exit Property

MaxIter_ErrorHandler:
  Err.Raise Err.Number

Exit_MaxIter:

End Property
Public Property Let TolTemperature(ByVal vData As Double)
On Error GoTo TolTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTolTemperature = vData
' ---- Code Body Ends ---- (Default)
Exit Property

TolTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_TolTemperature:

End Property
Public Property Get TolTemperature() As Double
Attribute TolTemperature.VB_UserMemId = 0
On Error GoTo TolTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    TolTemperature = mvarTolTemperature
' ---- Code Body Ends ---- (Default)
Exit Property

TolTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_TolTemperature:

End Property
Public Property Let TolPressure(ByVal vData As Double)
On Error GoTo TolPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTolPressure = vData
' ---- Code Body Ends ---- (Default)
Exit Property

TolPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_TolPressure:

End Property
Public Property Get TolPressure() As Double
On Error GoTo TolPressure_ErrorHandler

' ---- Code Body Starts ---- (Default)
    TolPressure = mvarTolPressure
' ---- Code Body Ends ---- (Default)
Exit Property

TolPressure_ErrorHandler:
  Err.Raise Err.Number

Exit_TolPressure:

End Property
Private Sub Class_Initialize()
On Error GoTo Class_Initialize_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTolTemperature = 0.0000001
    mvarTolPressure = 0.00000001
    mvarTolMolarFraction = 0.00001
    mvarMaxIter = 500
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Initialize_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Initialize:

End Sub
