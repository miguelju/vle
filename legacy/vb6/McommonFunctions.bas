Attribute VB_Name = "McommonFunctions"
Option Explicit
    Public Const UnivGasConst = 8.31451
    Global MaxVar As Double
    Public Declare Sub CopyMemory Lib "kernel32" Alias "RtlMoveMemory" (Dest As Any, Source As Any, ByVal Bytes As Long)
    Public Filter As New clsJMLFilter
Function SWAPN(ByRef a1 As Variant, ByRef a2 As Variant)
On Error GoTo SWAPN_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim AI As Double
        AI = a2
        a2 = a1
        a1 = AI
' ---- Code Body Ends ---- (Default)
Exit Function

SWAPN_ErrorHandler:
  Err.Raise Err.Number

Exit_SWAPN:

End Function
Function MatrizGauss(n As Integer, nvc As Integer, ByRef A() As Double, ByRef P() As Integer, ByRef d As Integer) As Boolean
On Error GoTo MatrizGauss_ErrorHandler

' ---- Code Body Starts ---- (Default)

' Resolucion de un sistema lineal de la forma A x = b
' por el metodo de Gauss con pivote maximo
' n, dimension de la matriz
' nvc, numero de vectores columna
' A(), matriz aumentada (contiene los vectores b)
' El numero de vectores b es nvc
' pivote maximo
' p(), arreglo monodimensional que contiene el orden
' de las permutaciones
' ip, indice del pivote
' il, indice de linea
' ic, indice de columna
    Dim ip As Integer, il As Integer, ic As Integer
    Dim PivoteMax As Double, R As Double
    d = 1
    For il = 1 To n: P(il) = il: Next
    For ip = 1 To n - 1
        PivoteMax = Abs(A(P(ip), ip))
        For il = ip + 1 To n
            If Abs(A(P(il), ip)) > PivoteMax Then
                PivoteMax = Abs(A(P(il), ip))
                SWAPN P(ip), P(il)
                d = -d
            End If
        Next
        If A(P(ip), ip) = 0 Then
            MatrizGauss = 0#
            Exit Function
        End If
        For il = ip + 1 To n
            R = A(P(il), ip) / A(P(ip), ip)
            For ic = ip To n
            '   Este FOR debe ir de ip+1 a n+nvc para reducuir el tiempo (no aparecen los ceros)
                A(P(il), ic) = A(P(il), ic) - A(P(ip), ic) * R
            Next
        Next
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

MatrizGauss_ErrorHandler:
  Err.Raise Err.Number

Exit_MatrizGauss:

End Function
Public Function DeterDiag(n As Integer, ByRef A() As Double, ByRef P() As Integer, ByVal d As Integer) As Double
On Error GoTo DeterDiag_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim il As Integer, Prod As Double
    Prod = 1
    For il = 1 To n
            Prod = Prod * A(P(il), il)
    Next
    DeterDiag = Prod * d
' ---- Code Body Ends ---- (Default)
Exit Function

DeterDiag_ErrorHandler:
  Err.Raise Err.Number

Exit_DeterDiag:

End Function
Public Function BackSub(n As Integer, ByRef A() As Double, ByRef P() As Integer, ByRef X() As Double)
On Error GoTo BackSub_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim nvc As Integer, i As Integer, j As Integer, Sum As Double
    nvc = 1: Sum = 0
   'backsubstitution, Remember that in this case b=0 and Xn=1
    If nvc <> 0 Then
        X(P(n)) = 1
        For i = (n - 1) To 1 Step -1
            For j = (i + 1) To n
                Sum = Sum + X(P(j)) * A(P(i), j)
            Next
            X(P(i)) = -Sum / A(P(i), i)
            Sum = 0
        Next
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

BackSub_ErrorHandler:
  Err.Raise Err.Number

Exit_BackSub:

End Function
Public Function Norm(ByRef A() As Double, Sum As Double)
On Error GoTo Norm_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    For i = 1 To UBound(A())
        A(i) = A(i) / Sum
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

Norm_ErrorHandler:
  Err.Raise Err.Number

Exit_Norm:

End Function
Public Function SumFrac(ByRef X() As Double, ByRef Sum As Double)
On Error GoTo SumFrac_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    Sum = 0
    For i = 1 To UBound(X())
        Sum = Sum + X(i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

SumFrac_ErrorHandler:
  Err.Raise Err.Number

Exit_SumFrac:

End Function
Public Function Compara(X() As Double, y() As Double, E As Double) As Boolean
On Error GoTo Compara_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer, Sum As Double
    For i = 1 To UBound(X())
        Sum = Sum + Abs(X(i) - y(i))
    Next
    If Sum < E Then Compara = True Else Compara = False
' ---- Code Body Ends ---- (Default)
Exit Function

Compara_ErrorHandler:
  Err.Raise Err.Number

Exit_Compara:

End Function
Public Sub IncrementarYRodar(ByRef x1 As Double, ByRef x2 As Double, ByRef x3 As Double, ByVal Incremento As Double, ByRef f1 As Double _
, ByRef f2 As Double, ByRef f3 As Double)
On Error GoTo IncrementarYRodar_ErrorHandler

' ---- Code Body Starts ---- (Default)
    x1 = x2
    x2 = x3
    x3 = x3 + Incremento
    f1 = f2
    f2 = f3
' ---- Code Body Ends ---- (Default)
Exit Sub

IncrementarYRodar_ErrorHandler:
  Err.Raise Err.Number

Exit_IncrementarYRodar:

End Sub
Public Sub Parabola(x1 As Double, x2 As Double, x3 As Double, fx1 As Double, fx2 As Double, fx3 As Double, ByRef Incremento As Double, Lineal As Boolean)
On Error GoTo Parabola_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim AA As Double, BB As Double, cc As Double, h1 As Double, Ho As Double
'Warning el x3 debe ser el ultimo calculado y asi sucesivamente
    If (x3 - x1) / x3 = 0 Then Lineal = True
    If Lineal = False Then
        Ho = x1 - x2
        h1 = x2 - x3
        cc = (fx1 - fx2 - (fx2 - fx3) * Ho / h1) / ((x1 * x1) - (x2 * x2) - ((x2 * x2) - (x3 * x3)) * Ho / h1)
        BB = (fx1 - fx2 - cc * ((x1 * x1) - (x2 * x2))) / Ho
        AA = fx1 - BB * x1 - cc * x1 * x1
        h1 = BB * BB - 4 * AA * cc
    Else
        h1 = -1
    End If
    If h1 > 0 Then
        Ho = (-BB - Sqr(h1)) / (2 * cc)
        h1 = (-BB + Sqr(h1)) / (2 * cc)
        If Abs(Ho - x3) < Abs(h1 - x3) Then
            Incremento = Ho - x3
        Else
            Incremento = h1 - x3
        End If
    Else
        h1 = (fx2 - fx3) / (x2 - x3)
        Incremento = -fx3 / h1
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

Parabola_ErrorHandler:
  Err.Raise Err.Number

Exit_Parabola:

End Sub
Public Function CopySign(A As Double, B As Double) As Double
On Error GoTo CopySign_ErrorHandler

' ---- Code Body Starts ---- (Default)
    CopySign = B * Sgn(A)
' ---- Code Body Ends ---- (Default)
Exit Function

CopySign_ErrorHandler:
  Err.Raise Err.Number

Exit_CopySign:

End Function
Public Function NuevoIncremento(x1 As Double, x2 As Double, x3 As Double, f1 As Double, f2 As Double, f3 As Double, ByVal Iteracion As Integer) As Double
On Error GoTo NuevoIncremento_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Aux As Double
    MaxVar = 0.25
    If Iteracion > 2 Then
        Parabola x1, x2, x3, f1, f2, f3, Aux, False
    Else
        If Iteracion = 1 Then
            Aux = -x3 * 0.01
        Else
            Parabola x1, x2, x3, f1, f2, f3, Aux, True
        End If
    End If
    If Abs(Aux) > MaxVar * x3 Then
        Aux = CopySign(Aux, MaxVar * x3)
        NuevoIncremento = Aux
    End If
    NuevoIncremento = Aux
' ---- Code Body Ends ---- (Default)
Exit Function

NuevoIncremento_ErrorHandler:
  Err.Raise Err.Number

Exit_NuevoIncremento:

End Function
Public Sub GeneralConstantsEOS(intEOS As TADiPEDC, ByRef K1 As Integer, ByRef K2 As Integer, ByRef K3 As Integer, ByRef OmA As Double, ByRef OmB As Double, ByRef OmC As Double, ByRef h() As Double)
On Error GoTo GeneralConstantsEOS_ErrorHandler

' ---- Code Body Starts ---- (Default)
'Dado un modelo termodinamico, selecciona el subprocedimiento adecuado para
'calcular las constantes del mismo
    Select Case intEOS
        Case PR1976_Qbic, RP1978_Qbic, PRL1997_Qbic, PRATmng1997_Qbic, PRSV1986_Qbic, PROL1998_Qbic, PRMmn1989_Qbic
            K1 = 2
            K2 = -1
            K3 = 1
            OmA = 0.457235528921382
            OmB = 7.77960739038885E-02
            h(0) = -3.355301398
            h(1) = 0.397564209
            h(2) = -0.258753915
            h(3) = 0.061054087
            h(4) = -0.005015492
        Case RKS1972_Qbic, RKSL1997_Qbic, RKSGD1978_Qbic, RK1949_Qbic, VdWVald1989_Qbic, RKSATmn1995_Qbic, RKOL1998_Qbic, RKSmn1980_Qbic
            K1 = 1
            K2 = 0
            K3 = 1
            OmA = 0.427480233540341
            OmB = 8.66403499649577E-02
            h(0) = -3.053621576
            h(1) = 0.257388077
            h(2) = -0.17122157
            h(3) = 0.039528771
            h(4) = -0.003183878
    Case VdW1870_Qbic, Berth1899_Qbic, VdWAda1984_Qbic, VdWOL1998_Qbic
            K1 = 0
            K2 = 0
            K3 = 1
            OmA = 27# / 64#
            OmB = 1# / 8#
            h(0) = -3
            h(1) = 0.235541536
            h(2) = -0.157703463
            h(3) = 0.036236158
            h(4) = -0.002905731
    End Select
    OmC = OmA * OmB * (K3 - 1)
' ---- Code Body Ends ---- (Default)
Exit Sub

GeneralConstantsEOS_ErrorHandler:
  Err.Raise Err.Number

Exit_GeneralConstantsEOS:

End Sub
Public Function CubicSolverOld(ByVal a0 As Double, ByVal a1 As Double, ByVal a2 As Double, ByVal a3 As Double, ByRef y1 As Double, ByRef y2 As Double, ByRef y3 As Double) As Integer
On Error GoTo CubicSolver_ErrorHandler

' ---- Code Body Starts ---- (Default)
    'a0+a1*x+a2*x2+a3*x3
    Dim P As Double, Q As Double, R As Double
    Dim E As Double, f As Double, h As Double
    Dim sqh As Double, teta As Double
    Dim Alfa As Double, Beta As Double
    Dim Untercio As Double
    Const DOSPI = 2# * 3.14159265358979
    Untercio = (1# / 3#)
    P = -a2 / a3 / 3#
    Q = a1 / a3 / 3#
    R = -a0 / a3 / 2#
    Alfa = Q - P * P
    Beta = R - (Q / 2# + Alfa) * P
    h = Beta * Beta + Alfa * Alfa * Alfa
    If h > 0 Then '1 real root
        CubicSolverOld = 1
        sqh = Sqr(h)
        E = XALAY(-Beta + sqh, Untercio)
        f = XALAY(-Beta - sqh, Untercio)
        y1 = -(-P + E + f)
        y2 = y1
        y3 = y1
    ElseIf h = 0 Then '1 double real root & 1 real root
        CubicSolverOld = 2
        E = XALAY(-Beta, Untercio)
        y1 = -P + 2# * E
        y2 = -P - E
        If y1 < y2 Then SWAP y1, y2
        y3 = y2
    Else '3 real roots
        CubicSolverOld = 2
        teta = Arccos(-Beta / Sqr(-Alfa * Alfa * Alfa))
        sqh = 2 * Sqr(-Alfa)
        y1 = -(-P + sqh * Cos(teta / 3#))
        y2 = -(-P + sqh * Cos((teta + DOSPI) / 3#))
        y3 = -(-P + sqh * Cos((teta + 2# * DOSPI) / 3#))
        If y1 < y2 Then SWAP y1, y2
        If y1 < y3 Then SWAP y1, y3
        If y2 < y3 Then SWAP y2, y3
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

CubicSolver_ErrorHandler:
  Err.Raise Err.Number

Exit_CubicSolver:

End Function
Private Sub SWAP(i As Double, j As Double)
On Error GoTo SWAP_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim k As Double
    k = i
    i = j
    j = k
' ---- Code Body Ends ---- (Default)
Exit Sub

SWAP_ErrorHandler:
  Err.Raise Err.Number

Exit_SWAP:

End Sub
Public Sub CopyX(ByRef X() As Double, ByRef XC() As Double)
On Error GoTo CopyX_ErrorHandler

' ---- Code Body Starts ---- (Default)
    CopyMemory XC(1), X(1), UBound(X()) * Len(X(1))
' ---- Code Body Ends ---- (Default)
Exit Sub

CopyX_ErrorHandler:
  Err.Raise Err.Number

Exit_CopyX:

End Sub
Private Function Arccos(Alfa As Double) As Double
On Error GoTo Arccos_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Arccos = Atn(-Alfa / Sqr(-Alfa * Alfa + 1#)) + 1.5707963268
' ---- Code Body Ends ---- (Default)
Exit Function

Arccos_ErrorHandler:
  Err.Raise Err.Number

Exit_Arccos:

End Function
Private Function XALAY(X As Double, y As Double) As Double
On Error GoTo XALAY_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If X > 0 Then
        XALAY = X ^ y
    ElseIf X < 0 Then
        XALAY = -((-X) ^ y)
    Else
        XALAY = 0
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

XALAY_ErrorHandler:
  Err.Raise Err.Number

Exit_XALAY:

End Function
Public Function LUbksb(ByRef A() As Double, ByRef n As Integer, NP As Integer, ByRef INDX() As Double, ByRef B() As Double)
On Error GoTo LUbksb_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim ii As Integer, j As Integer, ll As Integer, i As Integer
    Dim Sum As Double
    ii = 0
    For i = 1 To n
        ll = INDX(i)
        Sum = B(ll)
        B(ll) = B(i)
        If ii <> 0 Then
            For j = ii To i - 1: Sum = Sum - A(i, j) * B(j): Next j
        ElseIf Sum <> 0! Then
            ii = i
        End If
        B(i) = Sum
    Next
    For i = n To 1 Step -1
        Sum = B(i)
        For j = i + 1 To n: Sum = Sum - A(i, j) * B(j): Next
        B(i) = Sum / A(i, i)
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

LUbksb_ErrorHandler:
  Err.Raise Err.Number

Exit_LUbksb:

End Function
Public Function LUDecomp(ByVal n As Integer, NP As Integer, ByRef A() As Double, ByRef INDX() As Double, ByRef d As Double) As Boolean
On Error GoTo LUDecomp_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Const TINY = 1E-20
    ReDim VV(1 To n) As Double
    Dim i As Integer, j As Integer, k As Integer, imax As Integer
    Dim AAMAX As Double, dum As Double, Sum As Double
    d = 1!
    For i = 1 To n
        AAMAX = 0!
        For j = 1 To n
            If Abs(A(i, j)) > AAMAX Then AAMAX = Abs(A(i, j))
        Next
        If AAMAX = 0! Then MsgBox "Singular matrix.": Exit Function
        VV(i) = 1! / AAMAX
    Next
    For j = 1 To n
        For i = 1 To j - 1
            Sum = A(i, j)
            For k = 1 To i - 1
                Sum = Sum - A(i, k) * A(k, j)
            Next
            A(i, j) = Sum
        Next
        AAMAX = 0!
        For i = j To n
            Sum = A(i, j)
            For k = 1 To j - 1
                Sum = Sum - A(i, k) * A(k, j)
            Next
            A(i, j) = Sum
            dum = VV(i) * Abs(Sum)
            If dum >= AAMAX Then
                imax = i
                AAMAX = dum
            End If
        Next
        If j <> imax Then
            For k = 1 To n
                dum = A(imax, k)
                A(imax, k) = A(j, k)
                A(j, k) = dum
            Next
            d = -d
            VV(imax) = VV(j)
        End If
        INDX(j) = imax
        If A(j, j) = 0! Then A(j, j) = TINY
        If j <> n Then
            dum = 1! / A(j, j)
            For i = j + 1 To n
                A(i, j) = A(i, j) * dum
            Next
        End If
    Next
' ---- Code Body Ends ---- (Default)
Exit Function

LUDecomp_ErrorHandler:
  Err.Raise Err.Number

Exit_LUDecomp:

End Function
Public Function SolveAx(n As Integer, A() As Double, B() As Double, ByRef X() As Double) As Boolean
On Error GoTo SolveAx_ErrorHandler

' ---- Code Body Starts ---- (Default)
'These proccedure solve a set of linear equations by aplying the LU
'n  dimension of the array
    ReDim INDX(1 To n) As Double
    Dim d As Double
    LUDecomp n, n, A(), INDX(), d
    LUbksb A(), n, n, INDX(), B()
    CopyMemory X(1), B(1), UBound(B) * Len(B(1))
' ---- Code Body Ends ---- (Default)
Exit Function

SolveAx_ErrorHandler:
  Err.Raise Err.Number

Exit_SolveAx:

End Function
Public Function CheckFunConv(ByRef f() As Double, ByVal E As Double) As Boolean
On Error GoTo CheckFunConv_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Sum As Double, i As Integer
    For i = 1 To UBound(f())
        Sum = Sum + Abs(f(i))
    Next
    If Sum < E Then CheckFunConv = True Else CheckFunConv = False
' ---- Code Body Ends ---- (Default)
Exit Function

CheckFunConv_ErrorHandler:
  Err.Raise Err.Number

Exit_CheckFunConv:

End Function
Public Function CheckRootConv(ByRef X() As Double, ByVal E As Double) As Boolean
On Error GoTo CheckRootConv_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Sum As Double, i As Integer
    For i = 1 To UBound(X())
        Sum = Sum + Abs(X(i))
    Next
    If Sum < E Then CheckRootConv = True Else CheckRootConv = False
' ---- Code Body Ends ---- (Default)
Exit Function

CheckRootConv_ErrorHandler:
  Err.Raise Err.Number

Exit_CheckRootConv:

End Function
Public Function FirstDerivate(fo As Double, f1 As Double, h As Double) As Double
On Error GoTo FirstDerivate_ErrorHandler

' ---- Code Body Starts ---- (Default)
    FirstDerivate = (f1 - fo) / h
' ---- Code Body Ends ---- (Default)
Exit Function

FirstDerivate_ErrorHandler:
  Err.Raise Err.Number

Exit_FirstDerivate:

End Function
Public Function VectorMod(f() As Double) As Double
On Error GoTo VectorMod_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    Dim Sum As Double
    Sum = 0
    For i = 1 To UBound(f())
        Sum = Sum + f(i) * f(i)
    Next
    VectorMod = Sqr(Sum)
' ---- Code Body Ends ---- (Default)
Exit Function

VectorMod_ErrorHandler:
  Err.Raise Err.Number

Exit_VectorMod:

End Function
Public Function CubicSolverNR(ByVal A As Double, ByVal B As Double, ByVal C As Double, ByRef x1 As Double, ByRef x2 As Double, ByRef x3 As Double, Optional Xsup As Double = 0.5) As Integer
On Error GoTo CubicSolvernr_ErrorHandler

' ---- Code Body Starts ---- (Default)
'c+bx+ax2+x3
    Dim Q As Double, R As Double, Phi As Double, Aux As Double, AA As Double, BB As Double
    Const DOSPI = 2# * 3.14159265358979
    Q = (A * A - 3 * B) / 9
    R = ((2 * A * A * A) - (9 * A * B) + (27 * C)) / 54
    If R * R < Q * Q * Q Then
    'Three real roots
        Aux = R / (Sqr(Q * Q * Q))
        Phi = Arccos(Aux)
        x1 = (-2 * Sqr(Q) * Cos(Phi / 3)) - (A / 3)
        x2 = (-2 * Sqr(Q) * Cos((Phi + DOSPI) / 3)) - (A / 3)
        x3 = (-2 * Sqr(Q) * Cos((Phi - DOSPI) / 3)) - (A / 3)
        If x1 < x2 Then SWAP x1, x2
        If x1 < x3 Then SWAP x1, x3
        If x2 < x3 Then SWAP x2, x3
        CubicSolverNR = 3
    Else
    'One Real root
        AA = -Sgn(R) * ((Abs(R) + Sqr(R ^ 2 - Q ^ 3)) ^ (1 / 3))
        If AA <> 0 Then
            BB = Q / AA
        Else
            BB = 0
        End If
        x1 = (AA + BB) - (A / 3)
        x2 = x1
        x3 = x1
        CubicSolverNR = 3
    End If
' ---- Code Body Ends ---- (Default)
Exit Function
'On Error GoTo CubicSolver_ErrorHandler
''
''    Dim x As Double, fun As Double, der As Double, b2 As Double, B1 As Double, b0 As Double
''    Dim h As Double
''    Do
''        x = Xsup
''        fun = Xsup ^ 3 + A * Xsup ^ 2 + B * Xsup + C
''        der = 3 * Xsup ^ 2 + 2 * A * Xsup + B
''        Xsup = Xsup - fun / der
''    Loop Until Abs((x - Xsup) / Xsup) < 0.000000000001
''    b2 = 1
''    B1 = x + A
''    b0 = x * x + A * x + B
''    h = B1 ^ 2 - 4 * b0 * b2
''    If h < 0 Then
''        CubicSolverNR = 3
''        x1 = x
''        x2 = x
''        x3 = x
''    Else
''        CubicSolverNR = 3
''        x1 = x
''        x2 = (-B1 + Sqr(h)) / (2 * b2)
''        x3 = (-B1 - Sqr(h)) / (2 * b2)
''        If x1 < x2 Then SWAP x1, x2
''        If x1 < x3 Then SWAP x1, x3
''        If x2 < x3 Then SWAP x2, x3
''    End If
'' ---- Code Body Ends ---- (Default)
'Exit Function
'
CubicSolvernr_ErrorHandler:
  Err.Raise Err.Number

Exit_CubicSolver:

End Function
''Public Function CubicSolver(a0 As Double, a1 As Double, a2 As Double, a3 As Double, x1 As Double, x2 As Double, x3 As Double, Optional Xsup As Double = 0.5) As Integer
''On Error GoTo CubicSolver_ErrorHandler
''
''    Dim x As Double, fun As Double, der As Double, b2 As Double, B1 As Double, b0 As Double
''    Dim h As Double, i As Integer
''    'esto fue hecho por Ronny Rodriguez
''    i = 1
''    Do
''        i = i + 1
''        x = Xsup
''        fun = a3 * Xsup ^ 3 + a2 * Xsup ^ 2 + a1 * Xsup + a0
''        der = 3 * a3 * Xsup ^ 2 + 2 * a2 * Xsup + a1
''        Xsup = Xsup - (fun / der)
''        If i = 100 Then
''            Xsup = 10
''        End If
''    Loop Until Abs((x - Xsup) / Xsup) < 0.000000000001
''    b2 = a3
''    B1 = (a3) * x + a2
''    b0 = a3 * x * x + a2 * x + a1
''    h = B1 ^ 2 - 4 * b0 * b2
''    If h < 0 Then
''        CubicSolver = 1
''        x1 = x
''        x2 = x
''        x3 = x
''    Else
''        CubicSolver = 2
''        x1 = x
''        x2 = (-B1 + Sqr(h)) / (2 * b2)
''        x3 = (-B1 - Sqr(h)) / (2 * b2)
''        If x1 < x2 Then SWAP x1, x2
''        If x1 < x3 Then SWAP x1, x3
''        If x2 < x3 Then SWAP x2, x3
''    End If
''' ---- Code Body Ends ---- (Default)
''Exit Function
''
''CubicSolver_ErrorHandler:
''  Err.Raise Err.Number
''
''Exit_CubicSolver:
''
''End Function


Public Function CubicSolver(ByVal a0 As Double, ByVal a1 As Double, ByVal a2 As Double, ByVal a3 As Double, ByRef y1 As Double, ByRef y2 As Double, ByRef y3 As Double) As Integer
On Error GoTo CubicSolver_ErrorHandler

' ---- Code Body Starts ---- (Default)
    'a0+a1*x+a2*x2+a3*x3
    Dim P As Double, Q As Double, R As Double
    Dim E As Double, f As Double, h As Double
    Dim sqh As Double, teta As Double
    Dim Alfa As Double, Beta As Double
    Dim Untercio As Double
    Const DOSPI = 2# * 3.14159265358979
    Untercio = (1# / 3#)
    P = -a2 / a3 / 3#
    Q = a1 / a3 / 3#
    R = -a0 / a3 / 2#
    Alfa = Q - P * P
    Beta = R - (Q / 2# + Alfa) * P
    h = Beta * Beta + Alfa * Alfa * Alfa
    If h > 0 Then '1 real root
        CubicSolver = 1
        sqh = Sqr(h)
        E = XALAY(-Beta + sqh, Untercio)
        f = XALAY(-Beta - sqh, Untercio)
        y1 = -(-P + E + f)
        y2 = y1
        y3 = y1
    ElseIf h = 0 Then '1 double real root & 1 real root
        CubicSolver = 2
        E = XALAY(-Beta, Untercio)
        y1 = -P + 2# * E
        y2 = -P - E
        If y1 < y2 Then SWAP y1, y2
        y3 = y2
    Else '3 real roots
        CubicSolver = 2
        teta = Arccos(-Beta / Sqr(-Alfa * Alfa * Alfa))
        sqh = 2 * Sqr(-Alfa)
        y1 = -(-P + sqh * Cos(teta / 3#))
        y2 = -(-P + sqh * Cos((teta + DOSPI) / 3#))
        y3 = -(-P + sqh * Cos((teta + 2# * DOSPI) / 3#))
        If y1 < y2 Then SWAP y1, y2
        If y1 < y3 Then SWAP y1, y3
        If y2 < y3 Then SWAP y2, y3
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

CubicSolver_ErrorHandler:
  Err.Raise Err.Number

Exit_CubicSolver:

End Function


