VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsQbicsProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarCriticalTemperature As Double
    Private mvarCriticalPressure As Double
    Private mvarAcentricFactor As Double
    Private mvarPRSVK1 As Double
    Private mvarPsatAtTr As Double
    Private mvarn As Double
    Private mvarm As Double
    Private mvarg As Double
    Private mvarZc As Double
Public Property Let PsatAtTr(ByVal vData As Double)
On Error GoTo PsatAtTr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPsatAtTr = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PsatAtTr_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatAtTr:

End Property
Public Property Get PsatAtTr() As Double
On Error GoTo PsatAtTr_ErrorHandler

' ---- Code Body Starts ---- (Default)
    PsatAtTr = mvarPsatAtTr
' ---- Code Body Ends ---- (Default)
Exit Property

PsatAtTr_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatAtTr:

End Property
Public Property Let PRSVK1(ByVal vData As Double)
On Error GoTo PRSVK1_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPRSVK1 = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PRSVK1_ErrorHandler:
  Err.Raise Err.Number

Exit_PRSVK1:

End Property
Public Property Get PRSVK1() As Double
On Error GoTo PRSVK1_ErrorHandler

' ---- Code Body Starts ---- (Default)
    PRSVK1 = mvarPRSVK1
' ---- Code Body Ends ---- (Default)
Exit Property

PRSVK1_ErrorHandler:
  Err.Raise Err.Number

Exit_PRSVK1:

End Property
Public Property Let m(ByVal vData As Double)
On Error GoTo m_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarm = vData
' ---- Code Body Ends ---- (Default)
Exit Property

m_ErrorHandler:
  Err.Raise Err.Number

Exit_m:

End Property
Public Property Get m() As Double
On Error GoTo m_ErrorHandler

' ---- Code Body Starts ---- (Default)
    m = mvarm
' ---- Code Body Ends ---- (Default)
Exit Property

m_ErrorHandler:
  Err.Raise Err.Number

Exit_m:

End Property
Public Property Let n(ByVal vData As Double)
On Error GoTo n_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarn = vData
' ---- Code Body Ends ---- (Default)
Exit Property

n_ErrorHandler:
  Err.Raise Err.Number

Exit_n:

End Property
Public Property Get n() As Double
On Error GoTo n_ErrorHandler

' ---- Code Body Starts ---- (Default)
    n = mvarn
' ---- Code Body Ends ---- (Default)
Exit Property

n_ErrorHandler:
  Err.Raise Err.Number

Exit_n:

End Property
Public Property Let g(ByVal vData As Double)
On Error GoTo g_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarg = vData
' ---- Code Body Ends ---- (Default)
Exit Property

g_ErrorHandler:
  Err.Raise Err.Number

Exit_g:

End Property
Public Property Get g() As Double
On Error GoTo g_ErrorHandler

' ---- Code Body Starts ---- (Default)
    g = mvarg
' ---- Code Body Ends ---- (Default)
Exit Property

g_ErrorHandler:
  Err.Raise Err.Number

Exit_g:

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
    mvarAcentricFactor = vData
' ---- Code Body Ends ---- (Default)
Exit Property

w_ErrorHandler:
  Err.Raise Err.Number

Exit_w:

End Property
Public Property Get w() As Double
On Error GoTo w_ErrorHandler

' ---- Code Body Starts ---- (Default)
    w = mvarAcentricFactor
' ---- Code Body Ends ---- (Default)
Exit Property

w_ErrorHandler:
  Err.Raise Err.Number

Exit_w:

End Property
Public Property Let Pc(ByVal vData As Double)
On Error GoTo Pc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarCriticalPressure = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Pc_ErrorHandler:
  Err.Raise Err.Number

Exit_Pc:

End Property
Public Property Get Pc() As Double
On Error GoTo Pc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Pc = mvarCriticalPressure
' ---- Code Body Ends ---- (Default)
Exit Property

Pc_ErrorHandler:
  Err.Raise Err.Number

Exit_Pc:

End Property
Public Property Let Tc(ByVal vData As Double)
On Error GoTo Tc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarCriticalTemperature = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Tc_ErrorHandler:
  Err.Raise Err.Number

Exit_Tc:

End Property
Public Property Get Tc() As Double
On Error GoTo Tc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Tc = mvarCriticalTemperature
' ---- Code Body Ends ---- (Default)
Exit Property

Tc_ErrorHandler:
  Err.Raise Err.Number

Exit_Tc:

End Property
