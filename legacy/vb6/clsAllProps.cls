VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsAllProps"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Member0" ,"colclsCriticalProps"
Attribute VB_Ext_KEY = "Member1" ,"colclsOtherProps"
Attribute VB_Ext_KEY = "Top_Level" ,"No"
Option Explicit
    Private mvarProperties As colclsProperties
    Private mvarLiquid1Model As TadipLiquidModels
    Private mvarVaporModel As TadipVaporModel
    Private mvarLiquid2Model As TadipLiquidModels
    Private mvarCpConsts As colclsPropertyDescription
    Private mvarPsatConst As colclsPropertyDescription
    Private mvarNumberOfComponents As Integer
    Private mvarSatPressureModel As TADiPSatPressureModel
    Private mvarUniversalR As Double
Public Property Get PsatConst() As colclsPropertyDescription
On Error GoTo PsatConst_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarPsatConst Is Nothing Then
        Set mvarPsatConst = New colclsPropertyDescription
    End If
    Set PsatConst = mvarPsatConst
' ---- Code Body Ends ---- (Default)
Exit Property

PsatConst_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatConst:

End Property
Public Property Set PsatConst(vData As colclsPropertyDescription)
On Error GoTo PsatConst_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarPsatConst = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PsatConst_ErrorHandler:
  Err.Raise Err.Number

Exit_PsatConst:

End Property
Public Property Let UniversalR(ByVal vData As Double)
On Error GoTo UniversalR_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarUniversalR = vData
' ---- Code Body Ends ---- (Default)
Exit Property

UniversalR_ErrorHandler:
  Err.Raise Err.Number

Exit_UniversalR:

End Property
Public Property Get UniversalR() As Double
On Error GoTo UniversalR_ErrorHandler

' ---- Code Body Starts ---- (Default)
    UniversalR = mvarUniversalR
' ---- Code Body Ends ---- (Default)
Exit Property

UniversalR_ErrorHandler:
  Err.Raise Err.Number

Exit_UniversalR:

End Property
Public Property Let SatPressureModel(ByVal vData As TADiPSatPressureModel)
On Error GoTo SatPressureModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarSatPressureModel = vData
' ---- Code Body Ends ---- (Default)
Exit Property

SatPressureModel_ErrorHandler:
  Err.Raise Err.Number

Exit_SatPressureModel:

End Property
Public Property Get SatPressureModel() As TADiPSatPressureModel
On Error GoTo SatPressureModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    SatPressureModel = mvarSatPressureModel
' ---- Code Body Ends ---- (Default)
Exit Property

SatPressureModel_ErrorHandler:
  Err.Raise Err.Number

Exit_SatPressureModel:

End Property
Public Property Let NumberOfComponents(ByVal vData As Integer)
On Error GoTo NumberOfComponents_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarNumberOfComponents = vData
' ---- Code Body Ends ---- (Default)
Exit Property

NumberOfComponents_ErrorHandler:
  Err.Raise Err.Number

Exit_NumberOfComponents:

End Property
Public Property Get NumberOfComponents() As Integer
On Error GoTo NumberOfComponents_ErrorHandler

' ---- Code Body Starts ---- (Default)
    NumberOfComponents = mvarNumberOfComponents
' ---- Code Body Ends ---- (Default)
Exit Property

NumberOfComponents_ErrorHandler:
  Err.Raise Err.Number

Exit_NumberOfComponents:

End Property
Public Property Get Properties() As colclsProperties
On Error GoTo Properties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarProperties Is Nothing Then
        Set mvarProperties = New colclsProperties
    End If
    Set Properties = mvarProperties
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_properties:

End Property
Public Property Set Properties(vData As colclsProperties)
On Error GoTo Properties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_properties:

End Property
Public Property Get CpConsts() As colclsPropertyDescription
On Error GoTo CpConsts_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarCpConsts Is Nothing Then
        Set mvarCpConsts = New colclsPropertyDescription
    End If
    Set CpConsts = mvarCpConsts
' ---- Code Body Ends ---- (Default)
Exit Property

CpConsts_ErrorHandler:
  Err.Raise Err.Number

Exit_CpConsts:

End Property
Public Property Set CpConsts(vData As colclsPropertyDescription)
On Error GoTo CpConsts_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarCpConsts = vData
' ---- Code Body Ends ---- (Default)
Exit Property

CpConsts_ErrorHandler:
  Err.Raise Err.Number

Exit_CpConsts:

End Property
Public Property Let Liquid2Model(ByVal vData As TadipLiquidModels)
On Error GoTo Liquid2Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarLiquid2Model = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid2Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid2Model:

End Property
Public Property Get Liquid2Model() As TadipLiquidModels
On Error GoTo Liquid2Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Liquid2Model = mvarLiquid2Model
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid2Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid2Model:

End Property
Public Property Let VaporModel(ByVal vData As TadipVaporModel)
On Error GoTo VaporModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarVaporModel = vData
' ---- Code Body Ends ---- (Default)
Exit Property

VaporModel_ErrorHandler:
  Err.Raise Err.Number

Exit_VaporModel:

End Property
Public Property Get VaporModel() As TadipVaporModel
On Error GoTo VaporModel_ErrorHandler

' ---- Code Body Starts ---- (Default)
    VaporModel = mvarVaporModel
' ---- Code Body Ends ---- (Default)
Exit Property

VaporModel_ErrorHandler:
  Err.Raise Err.Number

Exit_VaporModel:

End Property
Public Property Let Liquid1Model(ByVal vData As TadipLiquidModels)
On Error GoTo Liquid1Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarLiquid1Model = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid1Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid1Model:

End Property
Public Property Get Liquid1Model() As TadipLiquidModels
On Error GoTo Liquid1Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Liquid1Model = mvarLiquid1Model
' ---- Code Body Ends ---- (Default)
Exit Property

Liquid1Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Liquid1Model:

End Property

Private Sub Class_Terminate()
On Error GoTo Class_Terminate_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarCpConsts = Nothing
   
    Set mvarProperties = Nothing
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Terminate_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Terminate:

End Sub
