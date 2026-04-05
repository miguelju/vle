VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "colclsOtherProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"No"
Attribute VB_Ext_KEY = "Collection" ,"clsOtherProps"
Attribute VB_Ext_KEY = "Member0" ,"clsOtherProps"
Option Explicit
    Private mvarAij(1 To 100, 1 To 100)  As Double
    Private mCol As Collection
    Private mvarKij As Variant
    Private mvarAijTemperature As Double
Public Property Let Kij(ByVal vData As Variant)
On Error GoTo Kij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarKij = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Kij_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij:

End Property
Public Property Set Kij(ByVal vData As Object)
On Error GoTo Kij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarKij = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Kij_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij:

End Property
Public Property Get Kij() As Variant
On Error GoTo Kij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If IsObject(mvarKij) Then
        Set Kij = mvarKij
    Else
        Kij = mvarKij
    End If
' ---- Code Body Ends ---- (Default)
Exit Property

Kij_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij:

End Property
Public Property Let Aij(i As Integer, j As Integer, ByVal vData As Double)
On Error GoTo Aij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarAij(i, j) = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Aij_ErrorHandler:
  Err.Raise Err.Number

Exit_Aij:

End Property
Public Property Get Aij(i As Integer, j As Integer) As Double
On Error GoTo Aij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Aij = mvarAij(i, j)
' ---- Code Body Ends ---- (Default)
Exit Property

Aij_ErrorHandler:
  Err.Raise Err.Number

Exit_Aij:

End Property
Public Function Add(delta As Double, Tb As Double, DipMoment As Double, m As Double, n As Double, g As Double, K1 As Double, PsatAtTr As Double, ZRA As Double, Vo As Double, wSRK As Double, vlModel As TADiPvlModel, Optional sKey As String) As clsOtherProps
On Error GoTo Add_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim objNewMember As clsOtherProps
    Set objNewMember = New clsOtherProps
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
    If Len(sKey) = 0 Then
        mCol.Add objNewMember
    Else
        mCol.Add objNewMember, sKey
    End If
    Set Add = objNewMember
    Set objNewMember = Nothing
' ---- Code Body Ends ---- (Default)
Exit Function

Add_ErrorHandler:
  Err.Raise Err.Number

Exit_Add:

End Function
Public Property Get Item(vntIndexKey As Variant) As clsOtherProps
Attribute Item.VB_UserMemId = 0
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
Attribute NewEnum.VB_UserMemId = -4
Attribute NewEnum.VB_MemberFlags = "40"
On Error GoTo NewEnum_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set NewEnum = mCol.[_NewEnum]
' ---- Code Body Ends ---- (Default)
Exit Property

NewEnum_ErrorHandler:
  Err.Raise Err.Number

Exit_NewEnum:

End Property
Private Sub Class_Initialize()
On Error GoTo Class_Initialize_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mCol = New Collection
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Initialize_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Initialize:

End Sub
Private Sub Class_Terminate()
On Error GoTo Class_Terminate_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mCol = Nothing
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Terminate_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Terminate:

End Sub
Public Property Let AijTemperature(ByVal vData As Double)
On Error GoTo AijTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarAijTemperature = vData
' ---- Code Body Ends ---- (Default)
Exit Property

AijTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_AijTemperature:

End Property
Public Property Get AijTemperature() As Double
On Error GoTo AijTemperature_ErrorHandler

' ---- Code Body Starts ---- (Default)
    AijTemperature = mvarAijTemperature
' ---- Code Body Ends ---- (Default)
Exit Property

AijTemperature_ErrorHandler:
  Err.Raise Err.Number

Exit_AijTemperature:

End Property
