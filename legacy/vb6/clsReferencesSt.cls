VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsReferencesSt"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarTo As Double
    Private mvarPo As Double
    Private mvarSo As Double
    Private mvarHo As Double
    Private mvarvo As Double
    Private mvarZo As Double
    Private mvarEstado As TADiPRefSt
Public Enum TADiPRefSt
    LiqSat = 1
    VapSat
    GI
End Enum
Public Property Let Estado(ByVal vData As TADiPRefSt)
On Error GoTo Estado_ErrorHandler

    mvarEstado = vData
    Exit Property

Estado_ErrorHandler:
  Err.Raise Err.Number

Exit_Estado:


End Property
Public Property Get Estado() As TADiPRefSt
   Estado = mvarEstado
End Property
Public Property Let Ho(ByVal vData As Double)
On Error GoTo Ho_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarHo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Ho_ErrorHandler:
  Err.Raise Err.Number

Exit_Ho:

End Property
Public Property Get Ho() As Double
On Error GoTo Ho_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Ho = mvarHo
' ---- Code Body Ends ---- (Default)
Exit Property

Ho_ErrorHandler:
  Err.Raise Err.Number

Exit_Ho:

End Property
Public Property Let TTo(ByVal vData As Double)
On Error GoTo TTo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarTo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

TTo_ErrorHandler:
  Err.Raise Err.Number

Exit_TTo:

End Property
Public Property Get TTo() As Double
On Error GoTo TTo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    TTo = mvarTo
' ---- Code Body Ends ---- (Default)
Exit Property

TTo_ErrorHandler:
  Err.Raise Err.Number

Exit_TTo:

End Property
Public Property Let So(ByVal vData As Double)
On Error GoTo So_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarSo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

So_ErrorHandler:
  Err.Raise Err.Number

Exit_So:

End Property
Public Property Get So() As Double
On Error GoTo So_ErrorHandler

' ---- Code Body Starts ---- (Default)
    So = mvarSo
' ---- Code Body Ends ---- (Default)
Exit Property

So_ErrorHandler:
  Err.Raise Err.Number

Exit_So:

End Property
Public Property Let Po(ByVal vData As Double)
On Error GoTo Po_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Po_ErrorHandler:
  Err.Raise Err.Number

Exit_Po:

End Property
Public Property Get Po() As Double
On Error GoTo Po_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Po = mvarPo
' ---- Code Body Ends ---- (Default)
Exit Property

Po_ErrorHandler:
  Err.Raise Err.Number

Exit_Po:

End Property
Private Sub Class_Initialize()
On Error GoTo Class_Initialize_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPo = 101325
    mvarTo = 298.15
    mvarHo = 0
    mvarSo = 0
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Initialize_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Initialize:

End Sub
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
Public Property Let Zo(ByVal vData As Double)
On Error GoTo Zo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarZo = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Zo_ErrorHandler:
  Err.Raise Err.Number

Exit_Zo:

End Property
Public Property Get Zo() As Double
On Error GoTo Zo_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Zo = mvarZo
' ---- Code Body Ends ---- (Default)
Exit Property

Zo_ErrorHandler:
  Err.Raise Err.Number

Exit_Zo:

End Property


