VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsProperties"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False

Option Explicit
    Private mvardelta As Double
    Private mvarvlModel As TADiPvlModel
    Private mvarwSRK As Double
    Private mvarZRA As Double
    Private mvarvo As Double
    Private mvarTb As Double
    Private mvarDipMoment As Double
    Private mvarm As Double
    Private mvarn As Double
    Private mvarK1 As Double
    Private mvarPsatAtTr As Double
    Private mvarg As Double
    Private mCol As Collection
    Private mvarTc As Double
    Private mvarPc As Double
    Private mvarW As Double
    Private mvarZc As Double
    Private mvarPRSVK1 As Double

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



Public Property Let zc(ByVal vData As Double)
On Error GoTo Zc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarZc = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Zc_ErrorHandler:
  Err.Raise Err.Number

Exit_Zc:

End Property
Public Property Get zc() As Double
On Error GoTo Zc_ErrorHandler

' ---- Code Body Starts ---- (Default)
    zc = mvarZc
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





Public Function Add(delta As Double, Tb As Double, DipMoment As Double, m As Double, n As Double, g As Double, K1 As Double, PsatAtTr As Double, ZRA As Double, Vo As Double, wSRK As Double, vlModel As TADiPvlModel, Optional sKey As String) As clsProperties
On Error GoTo Add_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim objNewMember As clsProperties
    Set objNewMember = New clsProperties
    Set objNewMember = New clsProperties
    objNewMember.delta = delta
    objNewMember.Tb = Tb
    objNewMember.DipMoment = DipMoment
    objNewMember.m = m
    objNewMember.n = n
    objNewMember.g = g
    objNewMember.K1 = K1
    objNewMember.PsatAtTr = PsatAtTr
    objNewMember.Vo = Vo
    objNewMember.wSRK = wSRK
    objNewMember.ZRA = ZRA
    objNewMember.vlModel = vlModel
'    If Len(sKey) = 0 Then
'        mCol.Add objNewMember
'    Else
'        mCol.Add objNewMember, sKey
'    End If
'    Set Add = objNewMember
'    Set objNewMember = Nothing
' ---- Code Body Ends ---- (Default)
Exit Function

Add_ErrorHandler:
  Err.Raise Err.Number

Exit_Add:

End Function

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
On Error GoTo vo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarvo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

vo_ErrorHandler:
  Err.Raise Err.Number

Exit_vo:

End Property
Public Property Get Vo() As Double
On Error GoTo vo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Vo = mvarvo
' ---- Code Body Ends ---- (Default)
Exit Property

vo_ErrorHandler:
  Err.Raise Err.Number

Exit_vo:

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



Public Property Get Item(vntIndexKey As Variant) As clsProperties
On Error GoTo Item_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set Item = mCol(vntIndexKey)
' ---- Code Body Ends ---- (Default)
Exit Property

Item_ErrorHandler:
  Err.Raise Err.Number

Exit_Item:

End Property
Public Property Get Count() As Long
On Error GoTo Count_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Count = mCol.Count
' ---- Code Body Ends ---- (Default)
Exit Property

Count_ErrorHandler:
  Err.Raise Err.Number

Exit_Count:

End Property
Public Sub Remove(vntIndexKey As Variant)
On Error GoTo Remove_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mCol.Remove vntIndexKey
' ---- Code Body Ends ---- (Default)
Exit Sub

Remove_ErrorHandler:
  Err.Raise Err.Number

Exit_Remove:

End Sub
Public Property Get NewEnum() As IUnknown
On Error GoTo NewEnum_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set NewEnum = mCol.[_NewEnum]
' ---- Code Body Ends ---- (Default)
Exit Property

NewEnum_ErrorHandler:
  Err.Raise Err.Number

Exit_NewEnum:

End Property
