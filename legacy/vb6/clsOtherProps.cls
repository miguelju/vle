VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsOtherProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvardelta As Double
    Private mvarvlModel As TADiPvlModel
    Private mvarwSRK As Double
    Private mvarZRA As Double
    Private mvarVo As Double
    Private mvarTb As Double
    Private mvarDipMoment As Double
    Private mvarm As Double
    Private mvarn As Double
    Private mvarK1 As Double
    Private mvarPsatAtTr As Double
    Private mvarg As Double
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
Public Property Let vlModel(ByVal vData As TADiPvlModel)
On Error GoTo vlModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarvlModel = vData
' ---- Code Body Ends ---- (Default)
Exit Property

vlModel_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatAtTr:

End Property
Public Property Get vlModel() As TADiPvlModel
On Error GoTo vlModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    vlModel = mvarvlModel
' ---- Code Body Ends ---- (Default)
Exit Property

vlModel_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatAtTr:

End Property
Public Property Let K1(ByVal vData As Double)
On Error GoTo K1_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarK1 = K1
' ---- Code Body Ends ---- (Default)
Exit Property

K1_ErrorHandler:
  Err.Raise Err.Number

Exit_K1:

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
Public Property Get K1() As Double
On Error GoTo K1_ErrorHandler

' ---- Code Body Starts ---- (Default)
    K1 = mvarK1
' ---- Code Body Ends ---- (Default)
Exit Property

K1_ErrorHandler:
  Err.Raise Err.Number

Exit_K1:

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
Public Property Let DipMoment(ByVal vData As Double)
On Error GoTo DipMoment_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarDipMoment = vData
' ---- Code Body Ends ---- (Default)
Exit Property

DipMoment_ErrorHandler:
  Err.Raise Err.Number

Exit_DipMoment:

End Property
Public Property Get DipMoment() As Double
On Error GoTo DipMoment_ErrorHandler

' ---- Code Body Starts ---- (Default)
    DipMoment = mvarDipMoment
' ---- Code Body Ends ---- (Default)
Exit Property

DipMoment_ErrorHandler:
  Err.Raise Err.Number

Exit_DipMoment:

End Property
Public Property Let Tb(ByVal vData As Double)
On Error GoTo Tb_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTb = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Tb_ErrorHandler:
  Err.Raise Err.Number

Exit_Tb:

End Property
Public Property Get Tb() As Double
On Error GoTo Tb_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Tb = mvarTb
' ---- Code Body Ends ---- (Default)
Exit Property

Tb_ErrorHandler:
  Err.Raise Err.Number

Exit_Tb:

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
