VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsPropertyDescription"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarNbFormula As Integer
    Private mvarCoef() As Double
    Private mvarNbCoef As Integer
Public Property Let Coef(Index As Integer, vData As Double)
On Error GoTo Coef_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If Index > UBound(mvarCoef()) Then ReDim Preserve mvarCoef(Index)
    mvarCoef(Index) = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Coef_ErrorHandler:
'  Err.Raise Err.Number
    
Exit_Coef:
    ReDim Preserve mvarCoef(0)
    Resume Next
End Property
Public Property Get Coef(Index As Integer) As Double
On Error GoTo Coef_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Coef = mvarCoef(Index)
' ---- Code Body Ends ---- (Default)
Exit Property

Coef_ErrorHandler:
  Err.Raise Err.Number

Exit_Coef:

End Property
Public Property Get NbFormula() As Integer
On Error GoTo NbFormula_ErrorHandler

' ---- Code Body Starts ---- (Default)
    NbFormula = mvarNbFormula
' ---- Code Body Ends ---- (Default)
Exit Property

NbFormula_ErrorHandler:
  Err.Raise Err.Number

Exit_NbFormula:

End Property
Public Property Let NbFormula(vData As Integer)
On Error GoTo NbFormula_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarNbFormula = vData
' ---- Code Body Ends ---- (Default)
Exit Property

NbFormula_ErrorHandler:
  Err.Raise Err.Number

Exit_NbFormula:

End Property
Public Property Get NbCoef() As Integer
On Error GoTo NbCoef_ErrorHandler

' ---- Code Body Starts ---- (Default)
    NbCoef = UBound(mvarCoef())
    mvarNbCoef = NbCoef
' ---- Code Body Ends ---- (Default)
Exit Property

NbCoef_ErrorHandler:
  Err.Raise Err.Number

Exit_NbCoef:

End Property
Public Property Let NbCoef(vData As Integer)
On Error GoTo NbCoef_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarNbCoef = vData
' ---- Code Body Ends ---- (Default)
Exit Property

NbCoef_ErrorHandler:
  Err.Raise Err.Number

Exit_NbCoef:

End Property
