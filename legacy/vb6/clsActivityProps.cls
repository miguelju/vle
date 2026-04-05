VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsActivityProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvardelta As Double
    Private mvarZRA As Double
    Private mvarvlModel As TADiPvlModel
    Private mvarVo As Double
    Private mvarPc As Double
    Private mvarTc As Double
    Private mvarwSRK As Double
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
Public Property Let vlModel(ByVal vData As TADiPvlModel)
On Error GoTo vlModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarvlModel = vData
' ---- Code Body Ends ---- (Default)
Exit Property

vlModel_ErrorHandler:
  Err.Raise Err.Number

Exit_ZRA:

End Property
Public Property Get vlModel() As TADiPvlModel
On Error GoTo vlModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    vlModel = mvarvlModel
' ---- Code Body Ends ---- (Default)
Exit Property

vlModel_ErrorHandler:
  Err.Raise Err.Number

Exit_vlModel:

End Property

Public Property Let Vo(ByVal vData As Double)
On Error GoTo Vo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarVo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Vo_ErrorHandler:
  Err.Raise Err.Number

Exit_Vo:

End Property
Public Property Get Vo() As Double
On Error GoTo Vo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Vo = mvarVo
' ---- Code Body Ends ---- (Default)
Exit Property

Vo_ErrorHandler:
  Err.Raise Err.Number

Exit_Vo:

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
Public Property Let wSRK(ByVal vData As Double)
On Error GoTo wSRK_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarwSRK = vData
' ---- Code Body Ends ---- (Default)
Exit Property

wSRK_ErrorHandler:
  Err.Raise Err.Number

Exit_wSRK:

End Property
Public Property Get wSRK() As Double
On Error GoTo wSRK_ErrorHandler

' ---- Code Body Starts ---- (Default)
    wSRK = mvarwSRK
' ---- Code Body Ends ---- (Default)
Exit Property

wSRK_ErrorHandler:
  Err.Raise Err.Number

Exit_wSRK:

End Property
Public Property Let delta(ByVal vData As Double)
On Error GoTo delta_ErrorHandler

' ---- Code Body Starts ---- (Default)
   mvardelta = vData
' ---- Code Body Ends ---- (Default)
Exit Property

delta_ErrorHandler:
  Err.Raise Err.Number

Exit_delta:

End Property
Public Property Get delta() As Double
On Error GoTo delta_ErrorHandler

' ---- Code Body Starts ---- (Default)
    delta = mvardelta
' ---- Code Body Ends ---- (Default)
Exit Property

delta_ErrorHandler:
  Err.Raise Err.Number

Exit_delta:

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
