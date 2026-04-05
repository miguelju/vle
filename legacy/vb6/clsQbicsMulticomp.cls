VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsQbicsMulticomp"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Attribute VB_Ext_KEY = "Member0" ,"colclsQbicsProps"
Option Explicit
    Public Enum TADiPPhiModel
        IdealSolutionp = 25
        IdealGas = -2
        PR1976_Qbicp = 0
        RK1949_Qbicp = 1
        RKS1972_Qbicp = 2
        VdW1870_Qbicp = 3
        PRL1997_Qbicp = 4
        RKSL1997_Qbicp = 5
        RKSGD1978_Qbicp = 6
        RP1978_Qbicp = 7
        Berth1899_Qbicp = 8
        VdWAda1984_Qbicp = 9
        VdWVald1989_Qbicp = 10
        RKSmn1980_Qbicp = 11
        RKSATmn1995_Qbicp = 12
        PRATmng1997_Qbicp = 13
        PRMmn1989_Qbicp = 14
        PRSV1986_Qbicp = 15
        VdWOL1998_Qbicp = 16
        RKOL1998_Qbicp = 17
        PROL1998_Qbicp = 18
    End Enum

    Public Enum TADiPMR
        WS = 26
        HOV
        HVS
        MHV1
        MHV2
        IVDW
        IIVDW
        Clasica_I
'        Clasica_II
    
    End Enum

    Public Enum TADiPPhaseID
        PhaseIDvapor = 0
        PhaseIDLiquid = 1
    End Enum

    Public Enum TADiPdim
        WithDim = 0
        DimLess = 1
    End Enum


'valores de los parametros que describen las ecuaciones cubicas
    Private gintK1 As Integer, gIntK2 As Integer, gintK3 As Integer
    Private gdblOmA As Double, gdblOmB As Double, gdblOmC As Double
    Private gdblH(0 To 4) As Double

'estructura que contiene los valores de los coeficientes del desarrollo
'cubico en volumen
    Private Type ParametersEOS
        a As Double
        b As Double
        c As Double
    End Type
    Private mvarModel As TADiPPhiModel
    Private mParameter As ParametersEOS
    Private mParameteri() As ParametersEOS
    Private mvarPhaseOfMixture As TADiPPhaseID
    Private mvarMolarFraction() As Double
    Private mvarProperties As colclsQbicsProps
    Private mvarKij As Variant 'local copy
    Private LiquidSolver As Object
    Private mvarMixture As clsAllProps
    Private mvarParent As clsLVE 'local copy
    Private A_ij() As Double
    Private ParametroIIVDW() As Double
    Private ParametroWS() As Double
Public Property Set Parent(ByVal vData As clsLVE)
    Set mvarParent = vData
End Property
Public Property Get Parent() As clsLVE
    Set Parent = mvarParent
End Property
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
Public Property Get Properties() As colclsQbicsProps
On Error GoTo Properties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarProperties Is Nothing Then
        Set mvarProperties = New colclsQbicsProps
    End If
    Set Properties = mvarProperties
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_Properties:

End Property
Public Property Set Properties(vData As colclsQbicsProps)
On Error GoTo Properties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = vData
' ---- Code Body Ends ---- (Default)
Exit Property

Properties_ErrorHandler:
  Err.Raise Err.Number

Exit_Properties:

End Property
Public Property Let PhaseOfMixture(ByVal vData As TADiPPhaseID)
On Error GoTo PhaseOfMixture_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarPhaseOfMixture = vData
' ---- Code Body Ends ---- (Default)
Exit Property

PhaseOfMixture_ErrorHandler:
  Err.Raise Err.Number

Exit_PhaseOfMixture:

End Property
Public Property Get PhaseOfMixture() As TADiPPhaseID
On Error GoTo PhaseOfMixture_ErrorHandler

' ---- Code Body Starts ---- (Default)
    PhaseOfMixture = mvarPhaseOfMixture
' ---- Code Body Ends ---- (Default)
Exit Property

PhaseOfMixture_ErrorHandler:
  Err.Raise Err.Number

Exit_PhaseOfMixture:

End Property
Public Function PartialFugacityCoeficientInSolution(Component As Integer, _
Temperature As Double, Pressure As Double, ByRef PartialFugacityCoeficient _
As Double, Optional Z As Double) As Boolean

On Error GoTo PartialFugacityCoeficientInSolution_ErrorHandler

    Dim a As Double, b As Double, n As Integer, mvarmixingrule As TADiPMR
    Dim b_i As Double, a_i As Double, AuxB As Double, Qi As Double
    Dim AuxA As Double, delta As Double, Iz As Double, g As Double, _
    ri As Double, si As Double, AA As Double, BB As Double, parametro1 As Double
    Dim Funcion1 As Double, Funcion2 As Double, derivada1 As Double, derivada2 As Double
    Dim Q As Double, d As Double, lnGamma As Double, c As Double, Gamma As Double
    Dim q1 As Double, q2 As Double, i As Integer, EG As Double, Aux1 As Double
    Dim Aux2 As Double, Aux3 As Double, j As Integer, X As Double, covol As Double
    mvarmixingrule = MRule
    Aux1 = 0
    Aux2 = 0
    Aux3 = 0
    parametro1 = 0
    
    n = mvarProperties.Count
    If mvarmixingrule = Clasica_I Then
        ai__bi Component, Temperature, a_i, b_i, n
        MixingRules Temperature, 0, WithDim, MRule
        AuxB = b_i / mParameter.b
        ri = gintK1 * AuxB
        si = 2 * gIntK2 * AuxB
        AuxA = a_i / mParameter.a
        delta = (gintK1 ^ 2) - 4 * gIntK2
    Else
        a_i = AAi(Component, Temperature, Pressure, WithDim)
        b_i = BBi(Component, Temperature, Pressure, WithDim)
        MixingRules Temperature, 0, WithDim, MRule
        AuxB = b_i / mParameter.b
        AuxA = a_i / mParameter.a
    End If
    MixingRules Temperature, Pressure, DimLess, MRule
    a = mParameter.a
    b = mParameter.b
    
    If Z = vbEmpty Then
        Z = ZSolver()
    End If
    Dim PureSolver As New clsQbicsPure
        Iz = PureSolver.I_z(Z, gintK1, gIntK2, b)
    Set PureSolver = Nothing


    C_cal c
    If mvarmixingrule <> Clasica_I Then
        Funcion1 = Log((Z - b))
        Funcion2 = Log(((Z + (1 + 2 ^ 0.5) * b) / (Z + (1 - 2 ^ 0.5) * b)))
    End If
    
    
    
    Select Case mvarmixingrule
'    Estas ecuaciones fueron tomadas del texto Modeling Vapor-Liquid Equilibria de
'    Orbey -Sandler, exepto la clasica que era la original de VLE
        Case Clasica_I
            If delta <> 0 Then
                g = Z * Z + gintK1 * b * Z + gIntK2 * b * b
                Qi = ((AuxB * Z * delta + (si * gintK1 - 2 * ri * gIntK2) * b - (ri _
                * gintK1 - 2 * si) * Z) / _
                (g * delta)) + ((AuxA + 1) * delta - (ri * gintK1 - 2 * si)) * Iz / delta
            ElseIf delta = 0 Then
                Qi = Iz * (AuxA + (AuxB * Z + ri * b / 2) * Iz + 1)
            End If
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Log(Z - b) + a * Qi)
        
        Case WS
            MixingRules Temperature, Pressure, WithDim, MRule

            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)
        
            For i = 1 To n
                For j = 1 To n
                    Q = Q + mvarMolarFraction(i) * mvarMolarFraction(j) * ParametroWS(i, j)
                Next
            Next
            For i = 1 To n
                parametro1 = parametro1 + mvarMolarFraction(i) * AAi(i, Temperature, Pressure, WithDim) / _
                (BBi(i, Temperature, Pressure, WithDim) * Temperature * UnivGasConst)
            Next

            EG = mvarParent.ExG(Temperature, n)
            d = parametro1 + EG / (c * Temperature * UnivGasConst)
            For j = 1 To n
                Aux1 = Aux1 + mvarMolarFraction(j) * ParametroWS(Component, j)
            Next
            Aux2 = 2 * Aux1
            Aux3 = (lnGamma / c) + a_i / (b_i * Temperature * UnivGasConst)
            derivada1 = (1 / (1 - d)) * Aux2 - (Q / (1 - d) ^ 2) * (1 - Aux3)
            derivada2 = Temperature * UnivGasConst * d * derivada1 + ((mParameter.b) * _
            Temperature * UnivGasConst) * Aux3
            
            Qi = ((mParameter.a) / (2 * 2 ^ 0.5 * (mParameter.b) * Temperature * UnivGasConst)) * _
            (derivada2 / (mParameter.a) - derivada1 / (mParameter.b))
            PartialFugacityCoeficient = Exp((derivada1 / (mParameter.b)) * (Z - 1) - Funcion1 _
            + Qi * Log(((Z + (1 - 2 ^ 0.5) * b) / (Z + (1 + 2 ^ 0.5) * b))))
        
        Case HOV
            MixingRules Temperature, Pressure, WithDim, MRule
            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)
            
            Qi = (1 / (2 * 2 ^ 0.5) * (a_i / (b_i * Temperature * UnivGasConst) + lnGamma / c))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
        
        Case HVS
            MixingRules Temperature, Pressure, WithDim, MRule
            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)

            Qi = (1 / (2 * 2 ^ 0.5) * (a_i / (b_i * Temperature * UnivGasConst) + lnGamma / c + _
            Log(mParameter.b / b_i) / c + (1 / c) * (AuxB - 1)))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)

        Case IVDW
            MixingRules Temperature, Pressure, WithDim, MRule
            For j = 1 To n
                parametro1 = parametro1 + mvarMolarFraction(j) * A_ij(Component, j)
            Next
            Qi = (a / (2 * 2 ^ 0.5 * b)) * ((2 * parametro1 / (mParameter.a)) - (AuxB))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
            
        Case IIVDW
            MixingRules Temperature, Pressure, WithDim, MRule
            For j = 1 To n
                parametro1 = parametro1 + mvarMolarFraction(j) * A_ij(Component, j)
            Next
            
            derivada1 = 2 * (parametro1 + ParametroIIVDW(Component)) - (mParameter.a)
            Qi = (a / (2 * 2 ^ 0.5 * b)) * ((derivada1 / (mParameter.a)) + 1 - (AuxB))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
        
        Case MHV1
            MixingRules Temperature, Pressure, WithDim, MRule
        
            q1 = -0.593
            lnGamma = mvarParent.ln_Gamma(Temperature, n, Component)

            Qi = (1 / (2 * 2 ^ 0.5) * (a_i / (b_i * Temperature * UnivGasConst) + lnGamma / q1 - Log(AuxB) / q1 + (1 / q1) * (AuxB - 1)))
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
        
        Case MHV2
            MixingRules Temperature, Pressure, WithDim, MRule

            q1 = -0.478
            q2 = -0.0047

            derivada1 = (q1 * a_i / (b_i * Temperature * UnivGasConst) + q2 * _
            ((a_i / (b_i * Temperature * UnivGasConst)) ^ 2 + ((mParameter.a) / (b * Temperature * UnivGasConst)) ^ 2) _
            + lnGamma + Log(1 / AuxB) + AuxB - 1) / (q1 + 2 * q2 * ((mParameter.a) / (b * Temperature * UnivGasConst)))

            Qi = (1 / (2 * 2 ^ 0.5) * derivada1)
            PartialFugacityCoeficient = Exp(AuxB * (Z - 1) - Funcion1 - Qi * Funcion2)
    End Select

Exit Function

PartialFugacityCoeficientInSolution_ErrorHandler:
  Err.Raise Err.Number

Exit_PartialFugacityCoeficientInSolution:

End Function
Public Property Get Model() As TADiPPhiModel
On Error GoTo Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Model = mvarModel
' ---- Code Body Ends ---- (Default)
Exit Property

Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Model:

End Property
Public Property Let Model(ByVal vData As TADiPPhiModel)
On Error GoTo Model_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarModel = vData
    If mvarModel <> IdealGas Then
        GeneralConstantsEOS mvarModel, gintK1, gIntK2, gintK3, gdblOmA, _
        gdblOmB, gdblOmC, gdblH()
    End If
' ---- Code Body Ends ---- (Default)
Exit Property

Model_ErrorHandler:
  Err.Raise Err.Number

Exit_Model:

End Property
Public Sub MixingRules(ByVal T As Double, P As Double, Dimensions As _
TADiPdim, Rule As Integer)
On Error GoTo MixingRules_ErrorHandler


    Dim i As Integer, j As Integer, n As Integer, num As Double, den As Double, mvarmixingrule As TADiPMR
    Dim c As Double, EG As Double, Aux As Double, Aux2 As Double, Aux1 As Double, Aux3 As Double
    Dim q1 As Double, q2 As Double, Par1 As Double, Par2 As Double, Par3 As Double
    Dim Alfamix As Double, x1 As Double, x2 As Double, Dimen As TADiPdim
    
    Dimen = WithDim
    n = mvarProperties.Count
    ReDim Preserve mParameteri(1 To n)
    mParameter.a = 0
    mParameter.b = 0
    ReDim Aij(n, n) As Double
    ReDim bij(n, n) As Double
    ReDim km(n, n) As Double
    ReDim A_ij(n, n) As Double
    ReDim ParametroIIVDW(n) As Double
    ReDim ParametroWS(n, n) As Double
    
    mvarmixingrule = Rule
    C_cal c
    
    Select Case mvarmixingrule
    
        Case WS
        Aux = 0
        num = 0
        den = 0
        
            'Esta si funciona, AHORA!!!!
            EG = mvarParent.ExG(T, n)
            For i = 1 To n
                For j = 1 To n
                     bij(i, j) = 0.5 * ((BBi(i, T, P, Dimen) - _
                     AAi(i, T, P, Dimen) / (UnivGasConst * T)) + (BBi(j, T, P, Dimen) - _
                     AAi(j, T, P, Dimen) / (UnivGasConst * T))) * (1 - mvarKij(i, j))
                     ParametroWS(i, j) = bij(i, j)
                Next
            Next
            For i = 1 To n
                For j = 1 To n
                    num = num + mvarMolarFraction(i) * mvarMolarFraction(j) * (bij(i, j))
                Next
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / BBi(i, T, P, Dimen)
            Next
            mParameter.b = (UnivGasConst * T * num) / (UnivGasConst * T - (Aux + EG / c))
            
            mParameter.a = mParameter.b * (EG / (c) + Aux)
            

            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If

        Case IVDW
            'Esta!!!!!
            For i = 1 To n
                For j = 1 To n
                    Aij(i, j) = ((AAi(i, T, P, Dimen) * AAi(j, T, P, Dimen)) ^ 0.5) * (1 - mvarKij(i, j))
                    A_ij(i, j) = Aij(i, j)
               Next
            Next
            For i = 1 To n
                For j = 1 To n
                    mParameter.a = mParameter.a + mvarMolarFraction(i) * mvarMolarFraction(j) * _
                    Aij(i, j)
                Next
            Next
            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next

            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If

        Case IIVDW
            'Esta si funciona
             For i = 1 To n
                For j = 1 To n
                    km(i, j) = mvarKij(i, j) * mvarMolarFraction(i) + mvarKij(j, i) * mvarMolarFraction(j)
                Next
            Next
            For i = 1 To n
                For j = 1 To n
                    Aij(i, j) = ((AAi(i, T, P, Dimen) * AAi(j, T, P, _
                    Dimen)) ^ 0.5) * (1 - km(i, j))
                    A_ij(i, j) = Aij(i, j)
                Next
            Next
            For i = 1 To n
                For j = 1 To n
                    mParameter.a = mParameter.a + mvarMolarFraction(i) * mvarMolarFraction(j) * _
                    Aij(i, j)
                Next
            Next
            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            'Esta parte es para el calculo de la fugacidad
            For i = 1 To n
                For j = 1 To n
                    If j = i Then
                        ParametroIIVDW(i) = ParametroIIVDW(i)
                    Else
                        ParametroIIVDW(i) = ParametroIIVDW(i) + mvarMolarFraction(i) * mvarMolarFraction(j) * _
                        ((AAi(i, T, P, Dimen) * AAi(j, T, P, Dimen)) ^ 0.5) * (mvarMolarFraction(j) * _
                        km(j, i) - (1 - mvarMolarFraction(i)) * km(i, j))
                    End If
                Next
            Next
            
            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If

        Case HOV
            'AHORA SIP!!!
            EG = mvarParent.ExG(T, n)
            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / BBi(i, T, P, Dimen)
            Next
                mParameter.a = mParameter.b * (Aux + EG / c)
            
            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If

        Case MHV1
            'Esta no funciona, le falta el valor de q1
            EG = mvarParent.ExG(T, n)
            q1 = -0.593

            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)
                Aux2 = Aux2 + mvarMolarFraction(i) * Log(mParameter.b / BBi(i, T, P, Dimen))
            Next
                mParameter.a = mParameter.b * UnivGasConst * T * (Aux + (1 / q1) * (EG / (UnivGasConst * T) + Aux2))
            
            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If

        Case Clasica_I
            'Esta si funciona
            For i = 1 To n
                mParameteri(i).a = AAi(i, T, P, Dimensions)
                mParameteri(i).b = BBi(i, T, P, Dimensions)
            Next
            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * _
                mParameteri(i).b
            Next
            For i = 1 To n
                For j = 1 To n
                   mParameter.a = mParameter.a + mvarMolarFraction(i) * _
                    mvarMolarFraction(j) _
                   * AAij(i, j)
                Next
            Next


        Case MHV2
        'Esta ecuacion fue tomada del libro de Orbey y Sandler
            EG = mvarParent.ExG(T, n)
            q1 = -0.478
            q2 = -0.0047

            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)
                Aux1 = Aux1 + mvarMolarFraction(i) * (AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)) ^ 2
                Aux2 = Aux2 + mvarMolarFraction(i) * Log(mParameter.b / BBi(i, T, P, Dimen))
            Next

                Aux3 = -q1 * Aux - q2 * Aux1 - EG / (UnivGasConst * T) - Aux2
            'para clacular el valor de alfamix(epsilon en el libro), se debe resolver
            'un polinomio de segundo orden, el resultado es la mayor de las raices:

            x1 = (-q1 + ((q1) ^ 2 - 4 * q2 * Aux3) ^ 0.5) / (2 * q2)
            x2 = (-q1 - ((q1) ^ 2 - 4 * q2 * Aux3) ^ 0.5) / (2 * q2)

            If x1 >= x2 Then
                Alfamix = x1
            Else
                Alfamix = x2
            End If
                mParameter.a = mParameter.b * UnivGasConst * T * Alfamix
            
            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If


        Case HVS
            'Esta no funciona, ya la aregle falta ver si sirve
            EG = mvarParent.ExG(T, n)
            Aux = 0
            Aux2 = 0

            For i = 1 To n
                mParameter.b = mParameter.b + mvarMolarFraction(i) * BBi(i, T, P, Dimen)
            Next
            For i = 1 To n
                Aux = Aux + mvarMolarFraction(i) * AAi(i, T, P, Dimen) / (BBi(i, T, P, Dimen) * UnivGasConst * T)
                Aux2 = Aux2 + mvarMolarFraction(i) * Log(mParameter.b / BBi(i, T, P, Dimen))
            Next
                mParameter.a = mParameter.b * UnivGasConst * T * (Aux + (1 / c) * (EG / (UnivGasConst * T) + Aux2))

            If Dimensions = DimLess Then
                mParameter.b = mParameter.b * P / (T * UnivGasConst)
                mParameter.a = mParameter.a * P / (T * UnivGasConst) ^ 2
            End If
'        Case Clasica_II
'            'Esta si funciona
'            For i = 1 To n
'                For j = 1 To n
'                    Aij(i, j) = ((AAi(i, T, P, Dimen) * AAi(j, T, P, _
'                    Dimen)) ^ 0.5) * (1 - mvarKij(i, j))
'                    bij(i, j) = (BBi(i, T, P, Dimen) + BBi(j, T, P, _
'                    Dimen)) / 2
'               Next
'            Next
'            For i = 1 To n
'                For j = 1 To n
'                    mParameter.A = mParameter.A + mvarMolarFraction(i) * mvarMolarFraction(j) * _
'                    Aij(i, j)
'                    mParameter.B = mParameter.B + mvarMolarFraction(i) * mvarMolarFraction(j) * _
'                    bij(i, j)
'                Next
'            Next
    End Select

Exit Sub

MixingRules_ErrorHandler:
  Err.Raise Err.Number

Exit_MixingRules:

End Sub
Private Function AAi(i As Integer, T As Double, P As Double, Dimensions As _
TADiPdim) As Double
On Error GoTo AAi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Tres As Double, Pr As Double, SatP As Double
    Dim QBic As clsQbicsPure
    Tres = T / mvarProperties(i).Tc
    Set QBic = New clsQbicsPure
    With QBic
        .AcentricFactor = mvarProperties(i).w
        mvarParent.Parametro_EDEC_Modificadas T, i, SatP
         .PsatAtTr = SatP / mvarProperties(i).Pc
'        .PsatAtTr = mvarProperties(i).PsatAtTr
        .m = mvarProperties(i).m
        .n = mvarProperties(i).n
        .g = mvarProperties(i).g
        .Zc = mvarProperties(i).Zc
        .PRSVK1 = mvarProperties(i).PRSVK1
        .Model = mvarModel
        If Dimensions = DimLess Then
            Pr = P / mvarProperties(i).Pc
            AAi = gdblOmA * .Alpha(Tres) * Pr / (Tres * Tres)
        Else
            AAi = (UnivGasConst * UnivGasConst) * gdblOmA * .Alpha(Tres) * _
            mvarProperties(i).Tc * mvarProperties(i).Tc / mvarProperties(i).Pc
        End If
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

AAi_ErrorHandler:
  Err.Raise Err.Number

Exit_AAi:

End Function
Private Function BBi(ByVal i As Integer, ByVal T As Double, ByVal P As _
Double, Dimensions As TADiPdim) As Double
On Error GoTo BBi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If Dimensions = DimLess Then
        BBi = gdblOmB * P * mvarProperties(i).Tc / T / mvarProperties(i).Pc
    Else
        BBi = (UnivGasConst * gdblOmB * mvarProperties(i).Tc) / _
        mvarProperties(i).Pc
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

BBi_ErrorHandler:
  Err.Raise Err.Number

Exit_BBi:

End Function
Public Function ZSolver(Optional Phase As TADiPPhaseID) As Double
On Error GoTo ZSolver_ErrorHandler
Dim PolyCoef(0 To 2) As Double
Dim Discriminante As Double
Dim Sol(1 To 3) As Double

' ---- Code Body Starts ---- (Default)
    Dim CubicCoef1 As Double, CubicCoef2 As Double, CubicCoef3 As Double
    Dim firstAprox As Double, Zg As Double, Zf As Double, Temp As _
TADiPPhaseID
   ' If mvarModel <> IdealGas Then
        CubicCoef1 = -(((1 - gintK1) * mParameter.b) + 1)
        CubicCoef2 = (((gIntK2 - gintK1) * mParameter.b ^ 2) - (gintK1 * _
        mParameter.b) + mParameter.a)
        CubicCoef3 = -(gIntK2 * mParameter.b * (1 + mParameter.b) + gintK3 * _
        mParameter.a) * mParameter.b

'        PolyCoef(0) = CubicCoef3
'        PolyCoef(1) = CubicCoef2
'        PolyCoef(2) = CubicCoef1
'        ThirdGradePolySolver PolyCoef(), Discriminante, Sol()
'        GetZfZg Sol(), Discriminante, Zf, Zg
'        If Zf = Imaginary Or Zg = Imaginary Then
'            MsgBox "Raices Imaginarias"
''            End
'        End If
        CubicSolver CubicCoef3, CubicCoef2, CubicCoef1, 1#, Zg, firstAprox, _
        Zf
    'Else
'        Zg = 1
'        Zf = 0
'    End If
    Temp = mvarPhaseOfMixture
    If Phase <> vbEmpty Then
        Temp = Phase
    End If
    Select Case Temp
        Case PhaseIDvapor
            If Zg > 0 Then
                ZSolver = Zg
            ElseIf Zf > 0 Then
                ZSolver = Zf
            End If
        Case PhaseIDLiquid
            If Zf > 0 Then
                ZSolver = Zf
            ElseIf Zg > 0 Then
                ZSolver = Zg
            End If
    End Select
' ---- Code Body Ends ---- (Default)
Exit Function

ZSolver_ErrorHandler:
  Err.Raise Err.Number

Exit_ZSolver:

End Function
Private Sub Class_Terminate()
On Error GoTo Class_Terminate_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Set mvarProperties = Nothing
' ---- Code Body Ends ---- (Default)
Exit Sub

Class_Terminate_ErrorHandler:
  Err.Raise Err.Number

Exit_Class_Terminate:

End Sub
Public Property Get MolarFraction(Index As Integer) As Double
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    MolarFraction = mvarMolarFraction(Index)
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
Public Property Let MolarFraction(Index As Integer, ByVal MolarFraction As _
Double)
On Error GoTo MolarFraction_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Static sIndex As Boolean
    If sIndex = False Then
        If Index = 1 Then
            sIndex = True
            ReDim Preserve mvarMolarFraction(1 To 1 + Index)
            ReDim Preserve mParameteri(1 To 1 + Index)
        End If
    End If
    If Index > UBound(mParameteri()) Then
        ReDim Preserve mvarMolarFraction(1 To 1 + Index)
        ReDim Preserve mParameteri(1 To 1 + Index)
    End If
    mvarMolarFraction(Index) = MolarFraction
' ---- Code Body Ends ---- (Default)
Exit Property

MolarFraction_ErrorHandler:
  Err.Raise Err.Number

Exit_MolarFraction:

End Property
Public Function PWithDimensions(ByVal T As Double, ByVal v As Double) As _
Double
On Error GoTo PWithDimensions_ErrorHandler

' ---- Code Body Starts ---- (Default)
    MixingRules T, 0, WithDim, MRule
    PWithDimensions = (UnivGasConst * T / (v - mParameter.b)) - mParameter.a _
    * (v - gintK3 * mParameter.b) / _
    ((v - mParameter.b) * (v ^ 2 + gintK1 * mParameter.b * v + gIntK2 * _
    mParameter.b ^ 2))
' ---- Code Body Ends ---- (Default)
Exit Function

PWithDimensions_ErrorHandler:
  Err.Raise Err.Number

Exit_PWithDimensions:

End Function
Private Sub CalculateJacobianP(ByVal T As Double, ByVal v As Double, ByRef _
Jacob() As Double)
On Error GoTo CalculateJacobianP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim h As Double
    h = 0.0001
    Jacob(1, 1) = FirstDerivate(FirstDerivateP(T, v), FirstDerivateP(T + 1, _
    v), 1)
    Jacob(1, 2) = FirstDerivate(FirstDerivateP(T, v), FirstDerivateP(T, v + _
    h), h)
    Jacob(2, 1) = FirstDerivate(SecondDerivateP(T, v), SecondDerivateP(T + _
    1, v), 1)
    Jacob(2, 2) = FirstDerivate(SecondDerivateP(T, v), SecondDerivateP(T, v _
    + h), h)
' ---- Code Body Ends ---- (Default)
Exit Sub

CalculateJacobianP_ErrorHandler:
  Err.Raise Err.Number

Exit_CalculateJacobianP:

End Sub
Private Function FirstDerivateP(T As Double, v As Double) As Double
On Error GoTo FirstDerivateP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim a As Double, b As Double, R As Double
    MixingRules T, 0, WithDim, MRule
    a = mParameter.a
    b = mParameter.b
    R = UnivGasConst
    FirstDerivateP = -R * T * v ^ 4 + (2 * a - 2 * R * T * gintK1 * b) * v ^ _
    3 + (a * gintK1 * b _
    - 3 * a * gintK3 * b - 2 * R * T * gIntK2 * b ^ 2 - a * b - R * T * _
    (gintK1 ^ 2) * b ^ 2) * v ^ 2 _
    + (-2 * a * gintK3 * gintK1 * b ^ 2 - 2 * R * T * gintK1 * gIntK2 * b ^ _
    3 + 2 * a * gintK3 * b ^ 2) _
    * v + a * gIntK2 * b ^ 3 + a * gintK3 * gintK1 * b ^ 3 - R * T * (gIntK2 _
    ^ 2) * b ^ 4 - _
    a * gintK3 * gIntK2 * b ^ 3
' ---- Code Body Ends ---- (Default)
Exit Function

FirstDerivateP_ErrorHandler:
  Err.Raise Err.Number

Exit_FirstDerivateP:

End Function
Private Function SecondDerivateP(T As Double, v As Double) As Double
On Error GoTo SecondDerivateP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim a As Double, b As Double, R As Double
    MixingRules T, 0, WithDim, MRule
    a = mParameter.a
    b = mParameter.b
    R = UnivGasConst
    SecondDerivateP = -2 * R * T * v ^ 6 + (6 * a - 6 * R * T * gintK1 * b) _
    * v ^ 5 + (6 * a * gintK1 * b _
    - 6 * R * T * gIntK2 * b ^ 2 - 12 * a * gintK3 * b - 6 * a * b - 6 * R * _
    T * (b ^ 2) * gintK1 ^ 2) _
    * v ^ 4 + (-16 * a * gintK3 * gintK1 * b ^ 2 + 2 * a * (b ^ 2) * gintK1 _
    ^ 2 - 2 * a * gIntK2 * b ^ 2 _
    - 2 * a * gintK1 * b ^ 2 - 12 * R * T * gintK1 * gIntK2 * b ^ 3 + 16 * a _
    * gintK3 * b ^ 2 - _
    2 * R * T * (b ^ 3) * gintK1 ^ 3 + 2 * a * b ^ 2) * v ^ 3 + (-6 * a * _
    gintK3 * (b ^ 3) * gintK1 ^ 2 _
    + 18 * a * gintK3 * gintK1 * b ^ 3 + 12 * a * gIntK2 * b ^ 3 - 6 * a * _
    gintK3 * b ^ 3 - _
    6 * R * T * gIntK2 * (b ^ 4) * gintK1 ^ 2 - 6 * a * gintK3 * gIntK2 * b _
    ^ 3 - 6 * R * T * (b ^ 4) _
    * gIntK2 ^ 2) * v ^ 2 + (-6 * a * gintK3 * gintK1 * b ^ 4 + 6 * a * _
    gintK1 * gIntK2 * b ^ 4 - _
    6 * a * gIntK2 * b ^ 4 - 6 * R * T * gintK1 * (b ^ 5) * gIntK2 ^ 2 - 6 * _
    a * gintK3 * gintK1 * _
    gIntK2 * b ^ 4 + 6 * a * gintK3 * (b ^ 4) * gintK1 ^ 2) * v _
    - 2 * a * (b ^ 5) * gIntK2 ^ 2 - 2 * a * gintK3 * (b ^ 5) * gintK1 ^ 2 _
    + 2 * a * gintK3 * gintK1 * gIntK2 * b ^ 5 - 2 * a * gintK1 * gIntK2 * b _
    ^ 5 - _
    2 * a * gintK3 * (b ^ 5) * gIntK2 ^ 2 - 2 * R * T * (b ^ 6) * gIntK2 ^ 3 _
    + 2 * a * gintK3 * gIntK2 * b ^ 5
' ---- Code Body Ends ---- (Default)
Exit Function

SecondDerivateP_ErrorHandler:
  Err.Raise Err.Number

Exit_SecondDerivateP:

End Function
Public Sub psCriticalProperties(ByRef tpsc As Double, ByRef vpsc As Double, _
ByRef ppsc As Double)
On Error GoTo psCriticalProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim f(1 To 2) As Double, i As Integer, n As Integer
    Dim j(1 To 2, 1 To 2) As Double, Converged As Boolean
    Dim X(1 To 2) As Double
    Dim delta(1 To 2) As Double
    n = mvarProperties.Count
    X(1) = 0: X(2) = 0
    Converged = False
    MixingRules 300, 0, WithDim, MRule
    For i = 1 To n
        X(1) = X(1) + mvarMolarFraction(i) * mvarProperties(i).Tc
        X(2) = mParameter.b / ((2 ^ (1 / 3)) - 1)
    Next
    Do
        DoEvents
        CalculateJacobianP X(1), X(2), j()
        EvaluateF X(), f()
        SolveAx 2, j(), f(), delta()
        Xnew X(), delta(), 2
        EvaluateF X(), f()
        If CheckFunConv(f(), 1E-20) = True Then Converged = True
        If CheckRootConv(delta(), 0.000000000000001) = True Then Converged = _
True
    Loop Until Converged = True
    tpsc = X(1): vpsc = X(2)
    ppsc = PWithDimensions(tpsc, X(2))
' ---- Code Body Ends ---- (Default)
Exit Sub

psCriticalProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_psCriticalProperties:

End Sub
Public Function CriticalProperties(ByRef Tc As Double, ByRef Pc As Double, _
ByRef vc As Double) As Boolean
On Error GoTo CriticalProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim n As Integer, T As Double, Th As Double, v1 As Double, v2 As Double
    Dim Sum As Double, i As Integer, Temp As Double
    ReDim Dn(1 To mvarProperties.Count) As Double
    ReDim dnh(1 To mvarProperties.Count) As Double
    Dim f As Double, Fh As Double
    Dim h As Double
    Dim vH As Double
    Dim EPS As Double
    EPS = 0.01
    n = mvarProperties.Count
    'Calculate Initial guess for T an  v
    For i = 1 To n
        Sum = Sum + mvarMolarFraction(i) * mvarProperties(i).Tc
    Next
    T = 1.5 * Sum: Th = T
    MixingRules T, 0, WithDim, MRule
    v2 = 5 * mParameter.b
    Do
        DoEvents
        v1 = v2
        Temp = v1
        h = EPS * Temp
        vH = v1 + h
        h = vH - Temp
        CP_Calculate_T T, v1, n, Dn()
        CP_Calculate_T Th, vH, n, dnh()
        f = CP_Calculate_C(T, v1, Dn(), n)
        Fh = CP_Calculate_C(Th, vH, dnh(), n)
        v2 = v1 - f / (FirstDerivate(f, Fh, h))
    Loop Until Abs(v2 - v1) / v1 < 0.1
    vc = v2
    Tc = T
    Pc = PWithDimensions(Tc, vc)
' ---- Code Body Ends ---- (Default)
Exit Function

CriticalProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_CriticalProperties:

End Function
Private Function CP_Calculate_T(ByRef T As Double, ByVal v As Double, ByVal _
n As Integer, ByRef Dn() As Double) As Boolean
On Error GoTo CP_Calculate_T_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ReDim Q(1 To n, 1 To n) As Double
    ReDim Qt(1 To n, 1 To n) As Double
    ReDim P(1 To n) As Integer
    ReDim Pt(1 To n) As Integer
    Dim Th As Double, h As Double, EPS As Double, Temp As Double
    Dim T1 As Double, T2 As Double, f As Double, Fh As Double
    Dim d As Integer, dT As Integer, Sum As Double, MAXIT As Integer
    Dim l As Integer
    EPS = 0.001: MAXIT = 100
'Newton Rhapson to solve T. Reference AICHeJ September 1980 vol 26, No 5,
'pag 772
    l = 0: T2 = T
    Do
        l = l + 1
        T1 = T2: Temp = T2
        h = EPS * Temp: Th = Temp + h: h = Th - Temp
        CP_FillQ Q(), T1, v, n
        CP_FillQ Qt(), Th, v, n
        MatrizGauss n, 1, Q(), P(), d
        MatrizGauss n, 1, Qt(), Pt(), dT
        f = DeterDiag(n, Q(), P(), d)
        Fh = DeterDiag(n, Qt(), Pt(), dT)
        T2 = T1 - f / (FirstDerivate(f, Fh, h))
    Loop Until (Abs(T2 - T1) / T2) <= 0.01 Or Abs(f) < 0.001
    BackSub n, Q(), P(), Dn()
    Sum = VectorMod(Dn())
    Norm Dn(), Sum
    T = T2
' ---- Code Body Ends ---- (Default)
Exit Function

CP_Calculate_T_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_Calculate_T:

End Function
Private Function CP_FillQ(ByRef Q() As Double, ByVal T As Double, ByVal v As _
Double, n As Integer) As Boolean
On Error GoTo CP_FillQ_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, j As Integer
    For i = 1 To n
        For j = 1 To n
            Q(i, j) = (T / 100) * (CP_D_lnFugacity_nt(i, j, T, v, n))
        Next
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

CP_FillQ_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_FillQ:

End Function
Private Function AAij(ByVal i As Integer, ByVal j As Integer) As Double
On Error GoTo AAij_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If VarType(mvarKij) = vbEmpty Then
        AAij = (Sqr(mParameteri(i).a * mParameteri(j).a))
    Else
        AAij = (1 - mvarKij(i, j)) * (Sqr(mParameteri(i).a * _
        mParameteri(j).a))
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

AAij_ErrorHandler:
  Err.Raise Err.Number

Exit_AAij:

End Function
Private Function CP_D_lnFugacity_nt(ByVal i As Integer, ByVal j As Integer, _
ByVal T As Double, ByVal v As Double, ByVal n As Integer) As Double
On Error GoTo CP_D_lnFugacity_nt_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Part1 As Double, Part2 As Double
    Dim TempMolarFraction() As Double, cont As Integer, P As Double
    Const h = 0.0001
    ReDim TempMolarFraction(n)
    CopyMemory TempMolarFraction(1), mvarMolarFraction(1), n * _
    Len(mvarMolarFraction(1))
    P = PWithDimensions(T, v)
    PartialFugacityCoeficientInSolution i, T, P, Part1
    Part1 = Log(Part1 * P * mvarMolarFraction(i))
    For cont = 1 To n
        If cont <> j Then: mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 + h): Else: mvarMolarFraction(cont) = (TempMolarFraction(cont) + h) / _
        (1 + h)
    Next
    P = PWithDimensions(T, v / (1 + h))
    PartialFugacityCoeficientInSolution i, T, P, Part2, P * (v / (1 + h)) / _
    (T * UnivGasConst)
    Part2 = Log(Part2 * P * mvarMolarFraction(i))
    CP_D_lnFugacity_nt = (Part2 - Part1) / h
    CopyMemory mvarMolarFraction(1), TempMolarFraction(1), n * _
    Len(TempMolarFraction(1))
' ---- Code Body Ends ---- (Default)
Exit Function

CP_D_lnFugacity_nt_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_D_lnFugacity_nt:

End Function
Private Function CP_Calculate_C(T As Double, v As Double, ByRef Dn() As _
Double, n As Integer) As Double
On Error GoTo CP_Calculate_C_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, j As Integer, k As Integer, Sum As Double, Aux As _
    Double
    For k = 1 To n
        For j = 1 To n
            For i = 1 To n
                Sum = Sum + CP_SD_ln_Fug_nt(i, j, k, T, v, n) * Dn(i) * _
                Dn(j) * Dn(k)
            Next
        Next
    Next
    MixingRules T, 0, WithDim, MRule
    With mParameter
        Aux = ((v - .b) / (2 * .b))
        CP_Calculate_C = Sum * Aux * Aux
    End With
' ---- Code Body Ends ---- (Default)
Exit Function

CP_Calculate_C_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_Calculate_C:

End Function
Private Function CP_SD_ln_Fug_nt(ByVal i As Integer, ByVal j As Integer, _
ByVal k As Integer, ByVal T As Double, ByVal v As Double, ByVal n As _
Integer) As Double
On Error GoTo CP_SD_ln_Fug_nt_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Part1 As Double, Part2 As Double, Part3 As Double, Part4 As Double
    Dim TempMolarFraction() As Double, cont As Integer
    Const h = 0.0001
    ReDim TempMolarFraction(n)
    CopyMemory TempMolarFraction(1), mvarMolarFraction(1), n * _
    Len(mvarMolarFraction(1))
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 - 2 * h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) - 2 * _
        h) / (1 - 2 * h)
    Next
    Part1 = CP_D_lnFugacity_nt(i, j, T, v / (1 - 2 * h), n)
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 - h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) - h) / (1 _
        - h)
    Next
    Part2 = CP_D_lnFugacity_nt(i, j, T, v / (1 - h), n)
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 + h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) + h) / (1 _
        + h)
    Next
    Part3 = CP_D_lnFugacity_nt(i, j, T, v / (1 + h), n)
    For cont = 1 To n
        If cont <> k Then mvarMolarFraction(cont) = TempMolarFraction(cont) _
        / (1 + 2 * h) Else mvarMolarFraction(cont) = (TempMolarFraction(cont) + 2 * _
        h) / (1 + 2 * h)
    Next
    Part4 = CP_D_lnFugacity_nt(i, j, T, v / (1 + 2 * h), n)
    CP_SD_ln_Fug_nt = (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h)
    CopyMemory mvarMolarFraction(1), TempMolarFraction(1), n * _
    Len(TempMolarFraction(1))
' ---- Code Body Ends ---- (Default)
Exit Function

CP_SD_ln_Fug_nt_ErrorHandler:
  Err.Raise Err.Number

Exit_CP_SD_ln_Fug_nt:

End Function
Private Sub EvaluateF(X() As Double, ByRef f() As Double)
On Error GoTo EvaluateF_ErrorHandler

' ---- Code Body Starts ---- (Default)
    f(1) = -FirstDerivateP(X(1), X(2))
    f(2) = -SecondDerivateP(X(1), X(2))
' ---- Code Body Ends ---- (Default)
Exit Sub

EvaluateF_ErrorHandler:
  Err.Raise Err.Number

Exit_EvaluateF:

End Sub
Private Function Xnew(ByRef X() As Double, deltaX() As Double, ByVal n As _
Integer)
On Error GoTo Xnew_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To n
        X(i) = X(i) + deltaX(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Xnew_ErrorHandler:
  Err.Raise Err.Number

Exit_Xnew:

End Function
Public Function ResidualProperties(ByVal T As Double, ByVal P As Double, _
ByVal Z As Double, ByRef HR As Double, ByRef SR As Double) As Boolean
On Error GoTo ResidualProperties_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Iz As Double, MixtureA As Double, MixtureB As Double, a__T As Double
    MixingRules T, P, DimLess, MRule
    MixtureA = mParameter.a
    MixtureB = mParameter.b
    Dim PureSolver As clsQbicsPure
    Set PureSolver = New clsQbicsPure
        Iz = PureSolver.I_z(Z, gintK1, gIntK2, MixtureB)
    Set PureSolver = Nothing
    a__T = a_T(T)
    SR = (Log(Z - MixtureB) + MixtureA * Iz * a__T) * UnivGasConst
    HR = ((Z - 1) + MixtureA * Iz * (a__T + 1)) * UnivGasConst * T
' ---- Code Body Ends ---- (Default)
Exit Function

ResidualProperties_ErrorHandler:
  Err.Raise Err.Number

Exit_ResidualProperties:

End Function
Private Function a_T(T As Double) As Double
On Error GoTo a_T_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Temp As Double, Part1 As Double, Part2 As Double, Part3 As Double, _
    Part4 As Double
    
    Const h = 0.0001
    MixingRules T, 0, WithDim, MRule
    Temp = mParameter.a
    MixingRules T - 2 * h, 0, WithDim, MRule
    Part1 = mParameter.a
    MixingRules (T - h), 0, WithDim, MRule
    Part2 = mParameter.a
    MixingRules (T + h), 0, WithDim, MRule
    Part3 = mParameter.a
    MixingRules (T + 2 * h), 0, WithDim, MRule
    Part4 = mParameter.a
    a_T = T * (-1) * (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h * Temp)
' ---- Code Body Ends ---- (Default)
Exit Function

a_T_ErrorHandler:
  Err.Raise Err.Number

Exit_a_T:

End Function
'Private Function b_T(T As Double) As Double
'    Dim temp As Double, Part1 As Double, Part2 As Double, Part3 As Double,
'Part4 As Double
'    Const h = 0.0001
'    MixingRules T, 0, WithDim
'    temp = mParameter.B
'    MixingRules T - 2 * h, 0, WithDim
'    Part1 = mParameter.B
'    MixingRules (T - h), 0, WithDim
'    Part2 = mParameter.B
'    MixingRules (T + h), 0, WithDim
'    Part3 = mParameter.B
'    MixingRules (T + 2 * h), 0, WithDim
'    Part4 = mParameter.B
'    b_T = T * (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h * temp)
'End Function

Private Function ai__bi(i As Integer, T As Double, a_i As Double, b_i As _
Double, ByVal n As Integer)
On Error GoTo ai__bi_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Part1 As Double, Part2 As Double, Part3 As Double, Part4 As Double
    Dim Part5 As Double, Part6 As Double, Part7 As Double, Part8 As Double
    Dim TempMolarFraction() As Double, cont As Integer
    Const h = 0.0001
    ReDim TempMolarFraction(n) As Double
    For cont = 1 To n
        TempMolarFraction(cont) = mvarMolarFraction(cont)
    Next
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 - 2 * h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) - 2 * h) / (1 _
            - 2 * h)
        End If
    Next
    MixingRules T, 0, WithDim, MRule
    Part1 = mParameter.b * (1 - 2 * h)
    Part5 = mParameter.a * (1 - 2 * h)
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 - h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) - h) / (1 - _
            h)
        End If
    Next
    MixingRules T, 0, WithDim, MRule
    Part2 = mParameter.b * (1 - h)
    Part6 = mParameter.a * (1 - h)
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 + h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) + h) / (1 + _
            h)
        End If
    Next
    MixingRules T, 0, WithDim, MRule
    Part3 = mParameter.b * (1 + h)
    Part7 = mParameter.a * (1 + h)
    For cont = 1 To n
        If cont <> i Then
            mvarMolarFraction(cont) = TempMolarFraction(cont) / (1 + 2 * h)
        Else
            mvarMolarFraction(cont) = (TempMolarFraction(cont) + 2 * h) / (1 _
            + 2 * h)
        End If
    Next
    MixingRules T, 0, WithDim, MRule
    Part4 = mParameter.b * (1 + 2 * h)
    Part8 = mParameter.a * (1 + 2 * h)
    b_i = (Part1 - 8 * Part2 + 8 * Part3 - Part4) / (12 * h)
    a_i = (Part5 - 8 * Part6 + 8 * Part7 - Part8) / (12 * h)
    For cont = 1 To n
        mvarMolarFraction(cont) = TempMolarFraction(cont)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

ai__bi_ErrorHandler:
  Err.Raise Err.Number

Exit_ai__bi:

End Function
Public Sub Kij_ReadExperimentalData(ByVal FilePath As String, ByRef Data() _
As Double, ByRef datasets As Integer, ByRef Dato As Double, ByRef isoterm As _
Boolean)
On Error GoTo Kij_ReadExperimentalData_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim PrincipalStr As String, Aux As Integer, ii As Integer
    Dim SecondaryStr As String, i As Integer, Lines As Integer, jj As _
Integer
    Open FilePath For Input As #1
        ii = 0
        jj = 0
        Do While Not EOF(1)
            Line Input #1, PrincipalStr
            ii = ii + 1
            If ii = 2 Then
                If PrincipalStr = "T" Then
                    isoterm = True
                ElseIf PrincipalStr = "P" Then
                    isoterm = False
                End If
            End If
            If ii = 3 Then
                Dato = Filter.InternationalValue(PrincipalStr)
            End If
            If ii = 4 Then
                Lines = CInt(PrincipalStr)
                ReDim Data(1 To Lines, 1 To 3)
            End If
            If ii > 6 And ii <= (Lines + 6) Then
                jj = jj + 1
                For i = 1 To 2
                    PrincipalStr = Trim(PrincipalStr)
                    Aux = InStr(1, PrincipalStr, Chr(32), 1)
                    SecondaryStr = Mid(PrincipalStr, 1, Aux - 1)
                    PrincipalStr = Mid(PrincipalStr, Aux + 1, Len(PrincipalStr))
                    Data(jj, i) = Filter.InternationalValue(SecondaryStr)
                Next
                Data(jj, 1) = Data(jj, 1) * 100000#
                Data(jj, 3) = Filter.InternationalValue(PrincipalStr)
            End If

        Loop
    Close #1
    datasets = Lines
' ---- Code Body Ends ---- (Default)
Exit Sub

Kij_ReadExperimentalData_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_ReadExperimentalData:

End Sub
Public Function Kij_GoldenSearch(ax As Double, bx As Double, cx As Double, _
tol As Double, Xmin As Double, Dato As Double, datasets As Integer, ByRef _
Data() As Double, isoterm As Boolean)
On Error GoTo Kij_GoldenSearch_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Const R = 0.61803399
    Const c = 0.38196602
    Dim x0 As Double, x3 As Double, x1 As Double, x2 As Double, f1 As _
Double, f2 As Double
    x0 = ax
    x3 = cx
    If Abs(cx - bx) > Abs(bx - ax) Then
      x1 = bx
      x2 = bx + c * (cx - bx)
    Else
      x2 = bx
      x1 = bx - c * (bx - ax)
    End If
    f1 = Kij_Qf(x1, Data(), datasets, Dato, isoterm)
    f2 = Kij_Qf(x2, Data(), datasets, Dato, isoterm)
    While Abs(x3 - x0) > tol * (Abs(x1) + Abs(x2))
      DoEvents
      If f2 < f1 Then
        x0 = x1
        x1 = x2
        x2 = R * x1 + c * x3
        f1 = f2
        f2 = Kij_Qf(x2, Data(), datasets, Dato, isoterm)
      Else
        x3 = x2
        x2 = x1
        x1 = R * x2 + c * x0
        f2 = f1
        f1 = Kij_Qf(x1, Data(), datasets, Dato, isoterm)
      End If
    Wend
    If f1 < f2 Then
      Kij_GoldenSearch = f1
      Xmin = x1
    Else
      Kij_GoldenSearch = f2
      Xmin = x2
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

Kij_GoldenSearch_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_GoldenSearch:

End Function
Public Function Kij_BracketMin(ByRef ax As Double, ByRef bx As Double, ByRef _
cx As Double, ByRef fa As Double, ByRef fb As Double, ByRef fc As Double, _
datasets As Integer, Dato As Double, ByRef Data() As Double, isoterm As _
Boolean) As Double
On Error GoTo Kij_BracketMin_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Const GOLD = 1.618034
    Const GLIMIT = 100!
    Const TINY = 1E-20
    Dim dum As Double, DONE As Boolean, R As Double, Q As Double, U As _
    Double, Ulim As Double, FU As Double
    fa = Kij_Qf(ax, Data(), datasets, Dato, isoterm)
    fb = Kij_Qf(bx, Data(), datasets, Dato, isoterm)
    If fb > fa Then
      dum = ax
      ax = bx
      bx = dum
      dum = fb
      fb = fa
      fa = dum
    End If
    cx = bx + GOLD * (bx - ax)
    fc = Kij_Qf(cx, Data(), datasets, Dato, isoterm)
    Do
      DoEvents
      If fb < fc Then Exit Do
      DONE = -1
      R = (bx - ax) * (fb - fc)
      Q = (bx - cx) * (fb - fa)
      dum = Q - R
      If Abs(dum) < TINY Then dum = TINY
      U = bx - ((bx - cx) * Q - (bx - ax) * R) / (2! * dum)
      Ulim = bx + GLIMIT * (cx - bx)
      If (bx - U) * (U - cx) > 0! Then
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
        If FU < fc Then
          ax = bx
          fa = fb
          bx = U
          fb = FU
          Exit Function
        ElseIf FU > fb Then
          cx = U
          fc = FU
          Exit Function
        End If
        U = cx + GOLD * (cx - bx)
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
      ElseIf (cx - U) * (U - Ulim) > 0! Then
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
        If FU < fc Then
          bx = cx
          cx = U
          U = cx + GOLD * (cx - bx)
          fb = fc
          fc = FU
          FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
        End If
      ElseIf (U - Ulim) * (Ulim - cx) >= 0! Then
        U = Ulim
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
      Else
        U = cx + GOLD * (cx - bx)
        FU = Kij_Qf(U, Data(), datasets, Dato, isoterm)
      End If
      If DONE Then
        ax = bx
        bx = cx
        cx = U
        fa = fb
        fb = fc
        fc = FU
      Else
        DONE = 0
      End If
    Loop While Not DONE
' ---- Code Body Ends ---- (Default)
Exit Function

Kij_BracketMin_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_BracketMin:

End Function
Public Function Kij_Qf(mmkij As Double, ByRef Data() As Double, datasets As _
Integer, Dato As Double, isoterm As Boolean) As Double
On Error GoTo Kij_Qf_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Suma As Double, i As Integer, Fig1 As Double, Fil1 As Double, Fig2 _
    As Double, Fil2 As Double
    ReDim mvarMolarFraction(1 To 2) As Double
    ReDim mvarKij(1 To 2, 1 To 2) As Double

        mvarKij(1, 1) = 0
        mvarKij(1, 2) = mmkij
        mvarKij(2, 1) = mmkij
        mvarKij(2, 2) = 0

        Suma = 0
        For i = 1 To datasets
            DoEvents
            'liquido
            mvarMolarFraction(1) = Data(i, 2)
            mvarMolarFraction(2) = 1 - Data(i, 2)
            mvarPhaseOfMixture = PhaseIDLiquid
            If isoterm = True Then
                PartialFugacityCoeficientInSolution 1, Dato, Data(i, 1), Fil1
                PartialFugacityCoeficientInSolution 2, Dato, Data(i, 1), Fil2
            Else
                PartialFugacityCoeficientInSolution 1, Data(i, 1), Dato, Fil1
                PartialFugacityCoeficientInSolution 2, Data(i, 1), Dato, Fil2
            End If
            Fil1 = mvarMolarFraction(1) * Fil1
            Fil2 = mvarMolarFraction(2) * Fil2
            'vapor
            mvarMolarFraction(1) = Data(i, 3)
            mvarMolarFraction(2) = 1 - Data(i, 3)
            mvarPhaseOfMixture = PhaseIDvapor
            If isoterm = True Then
                PartialFugacityCoeficientInSolution 1, Dato, Data(i, 1), Fig1
                PartialFugacityCoeficientInSolution 2, Dato, Data(i, 1), Fig2
            Else
                PartialFugacityCoeficientInSolution 1, Data(i, 1), Dato, Fig1
                PartialFugacityCoeficientInSolution 2, Data(i, 1), Dato, Fig2
            End If
            Fig1 = mvarMolarFraction(1) * Fig1
            Fig2 = mvarMolarFraction(2) * Fig2
            Suma = Suma + (Abs(Fig1 - Fil1) / Fig1) + (Abs(Fig2 - Fil2) / Fil2)
        Next
        Kij_Qf = Suma
' ---- Code Body Ends ---- (Default)
Exit Function

Kij_Qf_ErrorHandler:
  Err.Raise Err.Number

Exit_Kij_Qf:

End Function
Private Sub C_cal(ByRef c As Double)
    Select Case mvarModel
        Case PR1976_Qbicp, PRL1997_Qbicp, PRATmng1997_Qbicp, _
        PRMmn1989_Qbicp, PROL1998_Qbicp, RP1978_Qbicp, PRSV1986_Qbicp
        
        c = -0.62322540138
        
        Case RK1949_Qbicp, RKS1972_Qbicp, RKSL1997_Qbicp, _
        RKSGD1978_Qbicp, RKSmn1980_Qbicp, RKSATmn1995_Qbicp, RKOL1998_Qbicp
        
        c = -0.69314718056
        
        Case VdW1870_Qbicp, VdWAda1984_Qbicp, VdWVald1989_Qbicp, _
        VdWOL1998_Qbicp
        
        c = -1
        
        Case Berth1899_Qbicp
    
    End Select

End Sub
Private Sub Parametros(Component As Integer, Temperature As Double, Pressure As Double)

End Sub
