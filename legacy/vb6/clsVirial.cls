VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsVirial"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarAcentricFactor As Double
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
Public Function CalculateZ(Tr As Double, Pr As Double) As Double
On Error GoTo CalculateZ_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Bo As Double, B1 As Double
    Bo = 0.083 - (0.422 / (Tr ^ 1.6))
    B1 = 0.139 - (0.172 / (Tr ^ 4.2))
    CalculateZ = 1 + (Pr / Tr) * (Bo + mvarAcentricFactor * B1)
' ---- Code Body Ends ---- (Default)
Exit Function

CalculateZ_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculateZ:

End Function
Public Function CalculateFugacity(Tr As Double, Pr As Double) As Double
On Error GoTo CalculateFugacity_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Bo As Double, B1 As Double
    Bo = 0.083 - (0.422 / (Tr ^ 1.6))
    B1 = 0.139 - (0.172 / (Tr ^ 4.2))
    CalculateFugacity = Exp((Pr / Tr) * (Bo + mvarAcentricFactor * B1))
' ---- Code Body Ends ---- (Default)
Exit Function

CalculateFugacity_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculateFugacity:

End Function
Public Function HR_SR(Tr As Double, Pr As Double, ByRef HR As Double, ByRef SR As Double) As Boolean
On Error GoTo HR_SR_ErrorHandler

' ---- Code Body Starts ---- (Default)
Dim dBodTr As Double, dB1dTr As Double
Dim Bo As Double, B1 As Double
    Bo = 0.083 - (0.422 / (Tr ^ 1.6))
    B1 = 0.139 - (0.172 / (Tr ^ 4.2))
    dBodTr = 0.675 / (Tr ^ 2.6)
    dB1dTr = 0.722 / (Tr ^ 5.2)
    HR = -Pr * (Bo - Tr * dBodTr + mvarAcentricFactor * (B1 - Tr * dB1dTr))
    SR = Pr * (dBodTr + mvarAcentricFactor * dB1dTr)
' ---- Code Body Ends ---- (Default)
Exit Function

HR_SR_ErrorHandler:
  Err.Raise Err.Number

Exit_HR_SR:

End Function
