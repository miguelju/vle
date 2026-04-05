Attribute VB_Name = "Controles"
Option Explicit
    Public Const Pi As Double = 3.14159265358979 'en radianes
    Public Const Imaginary As Double = 123456789
    Public MRule As Integer
    Public MGamma As Boolean
    Private Filter As New clsJMLFilter
    Private Declare Function SendMessage Lib "user32" Alias "SendMessageA" (ByVal hwnd As Long, ByVal wMsg As Long, ByVal wParam As Long, lParam As Any) As Long
    Public Declare Function SetWindowWord Lib "user32" (ByVal hwnd As Long, ByVal nIndex As Long, ByVal wNewWord As Long) As Long
    Public Const EM_CANUNDO = &HC6
    Public Const EM_UNDO = &HC7
    Public Const SWW_hParent = (-8)
    Public Const SW_SHOWNORMAL = 1 ' Restores Window if Minimized or
    Public Declare Function ShellExecute Lib "shell32.dll" Alias "ShellExecuteA" _
        (ByVal hwnd As Long, ByVal lpOperation As String, ByVal lpFile As String, _
        ByVal lpParameters As String, ByVal lpDirectory As String, _
        ByVal nShowCmd As Long) As Long
    Public Declare Function FindExecutable Lib "shell32.dll" Alias "FindExecutableA" _
        (ByVal lpFile As String, ByVal lpDirectory As String, ByVal lpResult As _
        String) As Long
'Sub CopyGrid(GridName As Control)
'    Clipboard.Clear
'    Clipboard.SetText GetGrdText(GridName)
'End Sub
'Public Sub MyGrid_Click(Box As Control, MSFlexGrid1 As MSFlexGrid)
'    On Error Resume Next
'    Box.Visible = True
'    Box.Left = MSFlexGrid1.CellLeft + MSFlexGrid1.Left
'    Box.Top = MSFlexGrid1.CellTop + MSFlexGrid1.Top
'    Box.Width = MSFlexGrid1.CellWidth
'    Box.Height = MSFlexGrid1.CellHeight
'    Box = MSFlexGrid1.Text
'    Box.SetFocus
'    Box.SelStart = 0
'    Box.SelLength = Len(Box.Text)
'End Sub
'Public Sub MyText_Change(Box As Control, MSFlexGrid1 As MSFlexGrid)
'    MSFlexGrid1.Col = MSFlexGrid1.ColSel
'    MSFlexGrid1.Row = MSFlexGrid1.RowSel
'    MSFlexGrid1.Text = Filter.InternationalValue(Box.Text)
'End Sub
'Public Sub MyText_KeyPress(KeyAscii As Integer, Box As Control, MSFlexGrid1 As MSFlexGrid, down As Boolean)
'    On Error Resume Next
'    If KeyAscii = 13 And MSFlexGrid1.RowSel < MSFlexGrid1.Rows - 1 Then
'        KeyAscii = 0
'        Box.Left = MSFlexGrid1.CellLeft + MSFlexGrid1.Left
'        If down = True Then
'            MSFlexGrid1.Row = MSFlexGrid1.Row + 1
'            Box.Top = MSFlexGrid1.CellTop + MSFlexGrid1.Top
'            Box.Width = MSFlexGrid1.CellWidth
'            Box.Height = MSFlexGrid1.CellHeight
'            Box = MSFlexGrid1.Text
'            Box.SetFocus
'            Box.SelStart = 0
'            Box.SelLength = Len(Box.Text)
'        Else
'            Box.Visible = False
'        End If
'    Else
'        If MSFlexGrid1.Row + 1 = MSFlexGrid1.Rows And KeyAscii = 13 Then
'            Box.Visible = False
'        End If
'    End If
'End Sub
'Public Function SearchCombo(cmb As ComboBox) As Boolean
'    Dim i As Integer, j As Integer
'    With cmb
'        j = .ListCount - 1
'        For i = 0 To j
'            If .Text = .List(i) Then
'                SearchCombo = True
'                Exit Function
'            End If
'        Next
'    End With
'    SearchCombo = False
'End Function
'Public Function GetGrdText(grd As MSFlexGrid) As String
'    Dim i As Integer, j As Integer, Filas As Integer, Columnas As Integer
'    Dim ClipText As String, CopyText As String, NC As String, NR As String, Count As Long, ColEnd As Long, HeadTxt As String
'    Filas = grd.Rows: Columnas = grd.Cols
'    ReDim Aux1(Filas, Columnas) As Variant
'    For j = 0 To Columnas - 1
'        For i = 0 To Filas - 1
'            grd.Col = j
'            grd.Row = i
'            Aux1(i, j) = grd.Text
'        Next i
'    Next j
'    NC = Chr(9)
'    NR = Chr(13) & Chr(10)
'    ColEnd = Columnas - 1
'    ClipText = grd.Clip
'    For i = 0 To Filas - 1
'        For Count = 0 To ColEnd
'            grd.Col = Count
'            HeadTxt = Aux1(i, Count)
'            If Count = 0 Then
'                CopyText = CopyText & HeadTxt
'            Else
'                CopyText = CopyText & NC & HeadTxt
'            End If
'            If Count = ColEnd Then CopyText = CopyText & NR
'        Next Count
'    Next i
'    GetGrdText = CopyText
'End Function
'Public Sub UndoEdit(ctrl As Object)
'    Dim foo As Long
'    foo = SendMessage(ctrl.hwnd, EM_CANUNDO, 0, 0)
'    If foo <> 0 Then
'         SendMessage ctrl.hwnd, EM_UNDO, 0, 0
'    End If
'End Sub
'Function LanguageChange(lng As Integer, toTADiP As Boolean) As enumLanguage
'    If toTADiP Then
'        Select Case lng
'            Case Is = 0
'                LanguageChange = English_TADiP
'            Case Is = 2000
'                LanguageChange = French_TADiP
'            Case Is = 4000
'                LanguageChange = Spanish_TADiP
'        End Select
'    Else
'        Select Case lng
'            Case Is = 0
'                LanguageChange = 0
'            Case Is = French_TADiP
'                LanguageChange = 2000
'            Case Is = Spanish_TADiP
'                LanguageChange = 4000
'        End Select
'    End If
'End Function
'Public Function FileExist(str As String) As Boolean
'    On Error GoTo ExitFunction
'    If Dir$(str) = "" Then
'        FileExist = False
'    Else
'        FileExist = True
'    End If
'ExitFunction:
'End Function
'Public Function CutFilePath(str As String, Max As Integer) As String
'    Dim i As Integer, lblLen As Integer, tempStr As String
'    tempStr = str
'    lblLen = Max
'    If Len(tempStr) <= lblLen Then CutFilePath = tempStr: Exit Function
'    lblLen = lblLen - 6
'    For i = Len(tempStr) - lblLen To Len(tempStr)
'        If Mid$(tempStr, i, 1) = "\" Then Exit For
'    Next
'    CutFilePath = Left$(tempStr, 3) & "..." & Right$(tempStr, Len(tempStr) - (i - 1))
'End Function
'Public Sub MySelText(MyTxt As TextBox)
'    MyTxt.SelStart = 0
'    MyTxt.SelLength = Len(MyTxt)
'End Sub
'Public Function MoveCell(KeyCode As Integer, MyGrd As MSFlexGrid) As Boolean
'    Select Case KeyCode
'    Case 37
'        If MyGrd.Col > 1 Then
'            MyGrd.Col = MyGrd.Col - 1
'        End If
'        MoveCell = True
'    Case 38
'        If MyGrd.Row > 1 Then
'            MyGrd.Row = MyGrd.Row - 1
'        End If
'        MoveCell = True
'    Case 39
'        If MyGrd.Col < MyGrd.Cols - 1 Then
'            MyGrd.Col = MyGrd.Col + 1
'        End If
'        MoveCell = True
'    Case 40
'        If MyGrd.Row < MyGrd.Rows - 1 Then
'            MyGrd.Row = MyGrd.Row + 1
'        End If
'        MoveCell = True
'    Case 13
'        If MyGrd.Row < MyGrd.Rows - 1 Then
'            MyGrd.Row = MyGrd.Row + 1
'        End If
'        MoveCell = True
'    End Select
'End Function
'Public Sub ChangeColor(Leave As Boolean, MyGrd As MSFlexGrid)
'    If Leave Then
'        MyGrd.CellForeColor = vbMenuText
'        MyGrd.CellBackColor = vbWhite
'    Else
'        MyGrd.CellForeColor = vbHighlightText
'        MyGrd.CellBackColor = vbHighlight
'    End If
'End Sub

Public Sub ThirdGradePolySolver(Coef() As Double, ByRef Discriminante As Double, ByRef Sol() As Double)
'Este sub contiene la solucion analitica al problema de obtener
'las raices de un polinomio cubico, mediante el algoritmo de Cardano
'el arreglo Coef() contiene tres coeficientes del polinomio cubico
'el coeficiente del termino de tercer orden debe ser uno por eso se omite
'se debe definir Coef(0 to 2) as double
'donde coef(0) se corresponde con el termino independiente
'y Sol(1 to 3) es el arreglo que ha de almacenar las tres raices del polinomio
    Dim Alfa As Double
    Dim Beta As Double
'    Dim D As Double
    Dim a As Double
    Dim B As Double
    Dim X(1 To 3) As Double
    Dim phi As Double
    Dim i As Integer
    Dim Temp As Double
    Dim m1 As Double
    Dim m2 As Double
'    On Error GoTo ThirdGradePolySolverErr
    Alfa = 3 * Coef(1) - Coef(2) * Coef(2)
    Alfa = Alfa / 3
    Beta = 2 * Coef(2) ^ 3 - 9 * Coef(2) * Coef(1) + 27 * Coef(0)
    Beta = Beta / 27
    Discriminante = Beta * Beta / 4 + Alfa ^ 3 / 27
    If Discriminante >= 0 Then
        m1 = -Beta / 2 + Sqr(Discriminante)
        m2 = Beta / 2 + Sqr(Discriminante)
        If m1 > 0 Then
            a = m1 ^ (1 / 3)
        Else
            a = -Abs(m1) ^ (1 / 3)
        End If
        If m2 > 0 Then
            B = -m2 ^ (1 / 3)
        Else
            B = -(-Abs(m2) ^ (1 / 3))
        End If
        X(1) = a + B
        If Discriminante > 0 Then
            'habra una raiz real y dos raices complejas conjugadas
            'X 2,3 = - (A+B)/2 +/- ((A-B)/2*sqr(-3))
            X(2) = Imaginary
            X(3) = Imaginary
        Else
            'habra tres raices reales, con al menos dos raices iguales
            X(2) = -(a + B) / 2
            X(3) = X(2)
        End If
    ElseIf Discriminante < 0 Then
        'habra tres raices reales diferentes
        phi = ArcCos(-Beta / (2 * Sqr(Abs(Alfa ^ 3) / 27)))
        Temp = 2 * Sqr(Abs(Alfa) / 3)
        X(1) = Temp * Cos(phi / 3)
        X(2) = -Temp * Cos((phi + Pi) / 3)
        X(3) = -Temp * Cos((phi - Pi) / 3)
    End If
    For i = 1 To 3
        If Not X(i) = Imaginary Then
            Sol(i) = X(i) - Coef(2) / 3
        Else
            'Este caso es en realidad igual al anterior, pero se van a emplear solamente raices reales
            Sol(i) = Imaginary
        End If
    Next
'    Exit Sub
'ThirdGradePolySolverErr:
'    Call RaiseError(MyUnhandledError, "modPublicCode: ThirdGradePolySolver")
End Sub
Public Function ArcCos(ByVal y As Double) As Double
'    On Error GoTo ArcCosErr
    ArcCos = Atn(-y / Sqr(-y * y + 1)) + 2 * Atn(1)
'    Exit Function
'ArcCosErr:
'    Call RaiseError(MyUnhandledError, "modPublicCode: ArcCos")
End Function
Public Sub GetZfZg(ZRoots() As Double, Discriminante As Double, ByRef Zf As Double, ByRef Zg As Double)
    Dim Temp2 As Double
    Dim Temp3 As Double
'    On Error GoTo GetZfZgErr
    'ZRoots(1) siempre va a ser real, pero pudiera ser negativa, recordar que este arreglo va de 1 a 3
    Zf = ZRoots(1)
    Temp2 = ZRoots(2)
    Temp3 = ZRoots(3)
    If Discriminante > 0 Then
        'una sola raiz real
        If Zf > 0 Then
            Zg = Imaginary
        Else
            Zf = Imaginary
            Zg = Imaginary
        End If
    Else
        If Discriminante = 0 Then
            'tres raices reales, con al menos dos raices iguales
            If Zf > 0 Then
                If Temp2 > 0 Then
                    Zg = Temp2
                Else
                    Zg = Imaginary
                End If
            Else
                If Temp2 > 0 Then
                    Zf = Temp2
                    Zg = Temp3
                Else
                    Zf = Imaginary
                    Zg = Imaginary
                End If
            End If
        Else
            'tres raices reales diferentes
            If Zf > 0 Then
                If Temp2 > 0 Then
                    If Zf < Temp2 Then
                        If Temp2 < Temp3 Then
                            Zg = Temp3
                        Else
                            Zg = Temp2
                        End If
                    Else
                        Zg = Zf
                        Zf = Temp2
                        If Zg < Temp3 Then
                            Zg = Temp3
                        ElseIf Temp3 > 0 Then
                            If Temp3 < Zf Then
                                Zf = Temp3
                            End If
                        End If
                    End If
                Else
                    If Temp3 > 0 Then
                        If Temp3 > Zf Then
                            Zg = Temp3
                        Else
                            Zg = Zf
                            Zf = Temp3
                        End If
                    Else
                        'en realidad no tiene que ser zg imaginario, lo que sucede es que hay una sola raiz posible
                        Zg = Imaginary
                    End If
                End If
            Else
                If Temp2 > 0 Then
                    Zf = Temp2
                    If Temp3 > 0 Then
                        If Temp3 > Zf Then
                            Zg = Temp3
                        Else
                            Zg = Zf
                            Zf = Temp3
                        End If
                    Else
                        'en realidad no tiene que ser zg imaginario, lo que sucede es que hay una sola raiz posible
                        Zg = Imaginary
                    End If
                Else
                    If Temp3 > 0 Then
                        Zf = Temp3
                        'en realidad no tiene que ser zg imaginario, lo que sucede es que hay una sola raiz posible
                        Zg = Imaginary
                    Else
                        Zf = Imaginary
                        Zg = Imaginary
                    End If
                End If
            End If
        End If
    End If
'    Exit Sub
'GetZfZgErr:
'    Call RaiseError(MyUnhandledError, "modPublicCode: GetZfZg Function")
End Sub
Public Function PolyEval(Coef() As Double, y As Double) As Double
    PolyEval = y ^ 3 + Coef(2) * y ^ 2 + Coef(1) * y + Coef(0)
End Function


