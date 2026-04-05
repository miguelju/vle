VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsCriticalProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Description = "Sets the Properties of a Compound"
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarTc As Double
    Private mvarPc As Double
    Private mvarW As Double
    Private mvarZc As Double
'local variable(s) to hold property value(s)
Private mvarvc As Double 'local copy
Public Property Let vc(ByVal vData As Double)
'used when assigning a value to the property, on the left side of an assignment.
'Syntax: X.vc = 5
    mvarvc = vData
End Property


Public Property Get vc() As Double
'used when retrieving value of a property, on the right side of an assignment.
'Syntax: Debug.Print X.vc
    vc = mvarvc
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
Public Property Let w(ByVal vData As Double)
On Error GoTo w_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarW = vData
' ---- Code Body Ends ---- (Default)
Exit Property

w_ErrorHandler:
  Err.Raise Err.Number

Exit_w:

End Property
Public Property Get w() As Double
On Error GoTo w_ErrorHandler

' ---- Code Body Starts ---- (Default)
    w = mvarW
' ---- Code Body Ends ---- (Default)
Exit Property

w_ErrorHandler:
  Err.Raise Err.Number

Exit_w:

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
