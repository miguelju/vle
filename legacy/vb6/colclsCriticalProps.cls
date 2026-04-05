VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "colclsCriticalProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"No"
Attribute VB_Ext_KEY = "Collection" ,"clsCriticalProps"
Attribute VB_Ext_KEY = "Member0" ,"clsCriticalProps"
Option Explicit
    Private mCol As Collection
Public Function Add(Tc As Double, Pc As Double, w As Double, Zc As Double, Optional sKey As String) As clsProperties
On Error GoTo Add_ErrorHandler

' ---- Code Body Starts ---- (Default)
    'Dim objNewMember As clsCriticalProps
    Set objNewMember = New clsCriticalProps
    objNewMember.Tc = Tc
    objNewMember.Pc = Pc
    objNewMember.w = w
    objNewMember.Zc = Zc
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
Public Property Get Item(vntIndexKey As Variant) As clsProperties
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
