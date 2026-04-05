VERSION 1.0 CLASS
BEGIN
  MultiUse = -1  'True
END
Attribute VB_Name = "clsJMLFilter"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = True
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_Ext_KEY = "SavedWithClassBuilder" ,"Yes"
Attribute VB_Ext_KEY = "Top_Level" ,"Yes"
Option Explicit
    Private mvarAuthorizedKeyDownForNumericFilter As Integer
    Private MyDecimalSeparator As Integer
Public Sub FloatingTextBox_KeyDown(KeyCode As Integer)
On Error GoTo FloatingTextBox_KeyDown_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ValidNumericalKeypad KeyCode
' ---- Code Body Ends ---- (Default)
Exit Sub

FloatingTextBox_KeyDown_ErrorHandler:
  Err.Raise Err.Number

Exit_FloatingTextBox_KeyDown:

End Sub
Public Sub FloatingTextBox_KeyPress(KeyAscii As Integer, FloatingTextBox As Object)
On Error GoTo FloatingTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim j As Integer, k As Integer
    MyDecimalSeparator = Asc(Mid$(Format$(1.2, "0.0"), 2, 1))
    KeyAscii = CheckKeyDownEvent(KeyAscii)
    If CheckValidDigit(KeyAscii) Then Exit Sub
    If CheckValidExponent(KeyAscii) Then KeyAscii = 69
    If KeyAscii = 69 Or KeyAscii = MyDecimalSeparator Or KeyAscii = 45 Then
        j = InStr(FloatingTextBox.Text, Chr$(MyDecimalSeparator))
        If j > 0 And KeyAscii = MyDecimalSeparator Then
            KeyAscii = 0
            Exit Sub
        End If
        k = InStr(FloatingTextBox.Text, "E")
        If k > 0 And k < (FloatingTextBox.SelStart + 1) And KeyAscii = MyDecimalSeparator Then
            KeyAscii = 0
            Exit Sub
        End If
        If k > 0 And KeyAscii = 69 Then
            KeyAscii = 0
            Exit Sub
        End If
        If k > 0 And KeyAscii = 45 Then
            If FloatingTextBox.SelStart < k Then
                ChangeSign FloatingTextBox, 1
            Else
                ChangeSign FloatingTextBox, k + 1
            End If
            KeyAscii = 0
        ElseIf KeyAscii = 45 Then
            ChangeSign FloatingTextBox, 1
            KeyAscii = 0
        End If
    Else
        KeyAscii = 0
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

FloatingTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_FloatingTextBox_KeyPress:

End Sub
Public Sub FloatingPositiveTextBox_KeyDown(KeyCode As Integer)
On Error GoTo FloatingPositiveTextBox_KeyDown_ErrorHandler

' ---- Code Body Starts ---- (Default)
    ValidNumericalKeypad KeyCode
' ---- Code Body Ends ---- (Default)
Exit Sub

FloatingPositiveTextBox_KeyDown_ErrorHandler:
  Err.Raise Err.Number

Exit_FloatingPositiveTextBox_KeyDown:

End Sub
Public Sub FloatingPositiveTextBox_KeyPress(KeyAscii As Integer, FloatingPositiveTextBox As Object)
On Error GoTo FloatingPositiveTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim j As Integer, k As Integer
    On Error Resume Next
    MyDecimalSeparator = Asc(Mid$(Format$(1.2, "0.0"), 2, 1))
    KeyAscii = CheckKeyDownEvent(KeyAscii)
    If CheckValidDigit(KeyAscii) Then Exit Sub
    If CheckValidExponent(KeyAscii) Then KeyAscii = 69
    If KeyAscii = 69 Or KeyAscii = MyDecimalSeparator Or KeyAscii = 45 Then
        j = InStr(FloatingPositiveTextBox.Text, Chr$(MyDecimalSeparator))
        If j > 0 And KeyAscii = MyDecimalSeparator Then
            KeyAscii = 0
            Exit Sub
        End If
        k = InStr(FloatingPositiveTextBox.Text, "E")
        If k > 0 And k < (FloatingPositiveTextBox.SelStart + 1) And KeyAscii = MyDecimalSeparator Then
            KeyAscii = 0
            Exit Sub
        End If
        If k > 0 And KeyAscii = 69 Then
            KeyAscii = 0
            Exit Sub
        End If
        If k > 0 And KeyAscii = 45 Then
            If FloatingPositiveTextBox.SelStart < k Then
            Else
                ChangeSign FloatingPositiveTextBox, k + 1
            End If
            KeyAscii = 0
        ElseIf k = 0 And KeyAscii = 45 Then
            KeyAscii = 0
        End If
    Else
        KeyAscii = 0
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

FloatingPositiveTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_FloatingPositiveTextBox_KeyPress:

End Sub
Public Sub IntegerTextBox_KeyPress(KeyAscii As Integer, IntegerTextBox As Object)
On Error GoTo IntegerTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    If CheckValidDigit(KeyAscii) Then
        Exit Sub
    ElseIf KeyAscii = 45 Then
        ChangeSign IntegerTextBox, 1
        KeyAscii = 0
    Else
        KeyAscii = 0
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

IntegerTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_IntegerTextBox_KeyPress:

End Sub
Public Sub IntegerPositiveTextBox_KeyPress(KeyAscii As Integer)
On Error GoTo IntegerPositiveTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    If Not (CheckValidDigit(KeyAscii)) Then
        KeyAscii = 0
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

IntegerPositiveTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_IntegerPositiveTextBox_KeyPress:

End Sub
Public Sub LcaseTextBox_KeyPress(KeyAscii As Integer)
On Error GoTo LcaseTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    KeyAscii = Asc(LCase$(Chr$(KeyAscii)))
' ---- Code Body Ends ---- (Default)
Exit Sub

LcaseTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_LcaseTextBox_KeyPress:

End Sub
Public Sub UcaseFirstTextBox_KeyPress(KeyAscii As Integer, UcaseFirstTextBox As Object)
On Error GoTo UcaseFirstTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If UcaseFirstTextBox.SelStart = 0 Then
        KeyAscii = Asc(UCase$(Chr$(KeyAscii)))
    Else
        KeyAscii = Asc(LCase$(Chr$(KeyAscii)))
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

UcaseFirstTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_UcaseFirstTextBox_KeyPress:

End Sub
Public Sub UcaseTextBox_KeyPress(KeyAscii As Integer)
On Error GoTo UcaseTextBox_KeyPress_ErrorHandler

' ---- Code Body Starts ---- (Default)
    KeyAscii = Asc(UCase$(Chr$(KeyAscii)))
' ---- Code Body Ends ---- (Default)
Exit Sub

UcaseTextBox_KeyPress_ErrorHandler:
  Err.Raise Err.Number

Exit_UcaseTextBox_KeyPress:

End Sub
Private Sub ChangeSign(x As TextBox, Pos)
On Error GoTo ChangeSign_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim i As Integer
    On Error GoTo fin
    i = x.SelStart
    If Mid$(x.Text, Pos, 1) = "-" Then
        x.Text = Left$(x.Text, Pos - 1) + Mid$(x.Text, Pos + 1)
        x.SelStart = i - 1
    Else
        x.Text = Left$(x.Text, Pos - 1) + "-" + Mid$(x.Text, Pos)
        x.SelStart = i + 1
    End If
    Exit Sub
fin:
    x.Text = ""
' ---- Code Body Ends ---- (Default)
Exit Sub

ChangeSign_ErrorHandler:
  Err.Raise Err.Number

Exit_ChangeSign:

End Sub
Public Function InternationalValue(x As String) As Double
On Error GoTo InternationalValue_ErrorHandler

' ---- Code Body Starts ---- (Default)
    InternationalValue = modInternationalValue(x)
' ---- Code Body Ends ---- (Default)
Exit Function

InternationalValue_ErrorHandler:
  Err.Raise Err.Number

Exit_InternationalValue:

End Function
Public Function Display(x As Double, n As Integer, AutoCmplx As Integer) As String
On Error GoTo Display_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Display = modDisplay(x, n, AutoCmplx)
' ---- Code Body Ends ---- (Default)
Exit Function

Display_ErrorHandler:
  Err.Raise Err.Number

Exit_Display:

End Function
Private Function CheckValidDigit(KeyAscii As Integer) As Boolean
On Error GoTo CheckValidDigit_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If (KeyAscii > 47 And KeyAscii < 58) Or KeyAscii = 13 Or KeyAscii = 8 Then
        CheckValidDigit = True
    Else
        CheckValidDigit = False
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

CheckValidDigit_ErrorHandler:
  Err.Raise Err.Number

Exit_CheckValidDigit:

End Function
Private Function CheckKeyDownEvent(KeyAscii As Integer) As Integer
On Error GoTo CheckKeyDownEvent_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If mvarAuthorizedKeyDownForNumericFilter = 110 Then
        CheckKeyDownEvent = MyDecimalSeparator
    Else
        CheckKeyDownEvent = KeyAscii
    End If
    mvarAuthorizedKeyDownForNumericFilter = 0
' ---- Code Body Ends ---- (Default)
Exit Function

CheckKeyDownEvent_ErrorHandler:
  Err.Raise Err.Number

Exit_CheckKeyDownEvent:

End Function
Private Sub ValidNumericalKeypad(KeyCode As Integer)
On Error GoTo ValidNumericalKeypad_ErrorHandler

' ---- Code Body Starts ---- (Default)
    mvarAuthorizedKeyDownForNumericFilter = 0
    If KeyCode = 110 Then
        mvarAuthorizedKeyDownForNumericFilter = KeyCode
    End If
' ---- Code Body Ends ---- (Default)
Exit Sub

ValidNumericalKeypad_ErrorHandler:
  Err.Raise Err.Number

Exit_ValidNumericalKeypad:

End Sub
Private Function CheckValidExponent(KeyAscii As Integer) As Boolean
On Error GoTo CheckValidExponent_ErrorHandler

' ---- Code Body Starts ---- (Default)
    If KeyAscii = 68 Or KeyAscii = 69 Or KeyAscii = 100 Or KeyAscii = 101 Then
        CheckValidExponent = True
    Else
        CheckValidExponent = False
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

CheckValidExponent_ErrorHandler:
  Err.Raise Err.Number

Exit_CheckValidExponent:

End Function
Private Function modDisplay(x As Double, n As Integer, AutoCmplx As Integer) As String
On Error GoTo modDisplay_ErrorHandler

' ---- Code Body Starts ---- (Default)
'   --------------
'   Display
'               Transforme une variable double précision en string
'               suivant un format calcule en fonction des paramètres
'
'   Paramètres:
'               X variable à présenter
'               n nombre de caractères qui doivent être affichées
'               AutoCmplx est le choix du format
'                   0 -> Automatique
'                   1 -> Fixe
'                   2 -> Exposant
'                   3 -> Scientifique
'                   +4 -> utilise 0 au lieu de #
'                   +8 -> utilise caractère pour séparer les mille
'                   +16 -> utilise " " au lieu du caractère défini en International pour séparer les mille
'
'   Auteur:
'               Jean-Marie Ledanois (Mai-93)
'   --------------
    Dim Auto As Integer
    Dim st As String
    Dim Sp As String
    Dim Res As String
    Dim i As Integer
    Dim j As Integer
    Dim Debut As String
    Dim fin As String
    Dim Expo As String

    If (AutoCmplx And 4) <> 0 Then
        st = "0"
    Else
        st = "#"
    End If
    If (AutoCmplx And 8) <> 0 Then
        If (AutoCmplx And 16) <> 0 Then
            Sp = " "
        Else
            Sp = Format$(1234, "#,###")
            If Mid$(Sp, 2, 1) = "2" Then
                Sp = ""
            Else
                Sp = Mid$(Sp, 2, 1)
            End If
        End If
    Else
        Sp = ""
    End If
    Auto = AutoCmplx Mod 4
    If Auto = 2 Then
        Res = DisplayExp(x, n, st)
    ElseIf Auto = 1 Then
        Res = DisplayFix(x, n, st)
    ElseIf Auto = 3 Then
        Res = DisplaySci(x, n, st)
    Else
        Res = DisplayAut(x, n, st)
    End If
'    Display = Res
'Exit Function
    
    Auto = InStr(UCase$(Res), "E")
    If Auto = 0 Then Auto = InStr(UCase$(Res), "D")
    If Auto <> 0 Then
        If Auto = Len(Res) Then
            Res = Left$(Res, Auto - 1)
            Expo = ""
        Else
            Expo = Mid$(Res, Auto)
            Res = Left$(Res, Auto - 1)
        End If
        
    End If
    st = Format$(1234.5, "#,###.#")
    Auto = InStr(st, Mid$(st, InStr(st, "4") + 1, 1))
    If Auto <> 0 Then
        st = Mid$(st, Auto, 1)
    Else
        st = "."
    End If
    Auto = InStr(Res, st)
    If Auto > 0 Then
        Debut = Left$(Res, Auto - 1)
        If Auto <> Len(Res) Then
            fin = Mid$(Res, Auto + 1)
        Else
            fin = ""
        End If
    Else
        Debut = Res
        fin = ""
    End If
    If Len(Debut) > 0 Then
        Res = ""
        j = 0
        For i = Len(Debut) To 1 Step -1
            Res = Mid$(Debut, i, 1) + Res
            j = j + 1
            If j = 3 Then
                Res = Sp + Res
                j = 0
            End If
        Next
        If Left$(Res, 1) = Sp Then Res = Mid$(Res, 2)
        Debut = Res
    End If
    If Len(fin) > 0 Then
        Res = ""
        j = 0
        For i = 1 To Len(fin)
            Res = Res + Mid$(fin, i, 1)
            j = j + 1
            If j = 3 Then
                Res = Res + Sp
                j = 0
            End If
        Next
        If Left$(Res, 1) = Sp Then Res = Mid$(Res, 2)
        fin = Res
    End If
    If Right$(fin, 1) = Sp And Len(fin) > 0 Then fin = Left$(fin, Len(fin) - 1)
    'If Right$(Debut, 1) = Sp Then Debut = Left$(Debut, Len(Debut) - 1)
    If fin = "" Then
        'Display = Debut + " " + Expo
        Debut = Debut + " " + Expo
    Else
        'Display = Debut + St + fin + " " + Expo
        Debut = Debut + st + fin + " " + Expo
    End If
    'If Right$(Debut, 1) = Sp Then Debut = Left$(Debut, Len(Debut) - 1)
    modDisplay = Trim$(Debut)
    'modDisplay = Debut
' ---- Code Body Ends ---- (Default)
Exit Function

modDisplay_ErrorHandler:
  Err.Raise Err.Number

Exit_modDisplay:

End Function
Private Function modInternationalValue(x As String) As Double
On Error GoTo modInternationalValue_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim Sep1 As String
    Dim Sep2 As String
    Dim Dummy As String
    Dim Y As String
    Dim i As Integer
    Dummy = Format$(1234.5, "#,###.#")
    If Mid$(Dummy, 2, 1) = "2" Then
        Sep1 = " "
    Else
        Sep1 = Mid$(Dummy, 2, 1)
    End If
    Sep2 = Mid$(Dummy, InStr(Dummy, "4") + 1, 1)
    If Sep2 = "5" Then Sep2 = ""
    Dummy = x
    If Sep1 <> "" And Sep1 <> " " Then
        i = InStr(Dummy, Sep1)
        Do While i <> 0
            Mid$(Dummy, i, 1) = " "
            i = InStr(Dummy, Sep1)
        Loop
    End If
    If Sep2 <> "" And Sep2 <> "." Then
        i = InStr(Dummy, Sep2)
        Do While i <> 0
            Mid$(Dummy, i, 1) = "."
            i = InStr(Dummy, Sep2)
        Loop
    End If
    modInternationalValue = Val(Dummy)
' ---- Code Body Ends ---- (Default)
Exit Function

modInternationalValue_ErrorHandler:
  Err.Raise Err.Number

Exit_modInternationalValue:

End Function
Private Function DisplayAut(x As Double, n As Integer, st As String) As String
On Error GoTo DisplayAut_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim forma1 As String
    Dim forma2 As String
    forma1 = DisplayFix(x, n, st)
    forma2 = DisplayExp(x, n, st)
    If modInternationalValue(forma1) = modInternationalValue(forma2) Then
        DisplayAut = forma1
    Else
        DisplayAut = forma2
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

DisplayAut_ErrorHandler:
  Err.Raise Err.Number

Exit_DisplayAut:

End Function
Private Function DisplayExp(x As Double, n As Integer, st As String) As String
On Error GoTo DisplayExp_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim forma As String
    Dim k As Integer, ps As Integer
    If x < 0 Then k = n + 2 Else k = n + 1
    If n > 0 And n < 15 Then
        forma = Format$(x, "0." + String$(n - 1, st) + "E+00")
        ps = InStr(forma, "E")
        If ps > k Then
            forma = Left$(forma, k) + Mid$(forma, ps)
        Else
            forma = Left$(forma, ps - 1) + Mid$(forma, ps)
        End If
    Else
        forma = Format$(x)
        If InStr(forma, "E") = 0 Then forma = Format$(x, "0." + String$(14, st) + "E+00")
    End If
    DisplayExp = forma
' ---- Code Body Ends ---- (Default)
Exit Function

DisplayExp_ErrorHandler:
  Err.Raise Err.Number

Exit_DisplayExp:

End Function
Private Function DisplayFix(x As Double, n As Integer, st As String) As String
On Error GoTo DisplayFix_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim forma As String
    Dim Sep As String
    Dim k As Integer, ps As Integer
    forma = Format$(1.2, "#.#")
    Sep = Mid$(forma, 2, 1)
    If Sep = "" Then Sep = "."
    If x < 0 Then k = n + 3 Else k = n + 2
    If n > 0 And n < 16 Then
        forma = Format$(x, "0." + String$(n - 1, st))
    Else
        forma = Format$(x, "0." + String$(15, st))
    End If
    ps = InStr(forma, Sep)
    If ps >= k Then
        forma = DisplayExp(x, n, st)
    Else
        If ps > 2 And ps < Len(forma) Then
            If n > ps Then
                forma = Format$(x, String$(ps - 2, st) + "0." + String$(n - ps + 1, st))
            Else
                forma = Format$(x, String$(ps - 2, st) + "0.")
            End If
        End If
        If modInternationalValue(forma) = 0 And x <> 0 Then
            forma = DisplayExp(x, n, st)
        Else
            If st = "#" Then
                Do While Right$(forma, 1) = "0"
                    forma = Left$(forma, Len(forma) - 1)
                Loop
            End If
        End If
    End If
    If Right$(forma, 1) = Sep Then
        DisplayFix = Left$(forma, Len(forma) - 1)
    Else
        DisplayFix = forma
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

DisplayFix_ErrorHandler:
  Err.Raise Err.Number

Exit_DisplayFix:

End Function
Private Function DisplaySci(x As Double, n As Integer, st As String) As String
On Error GoTo DisplaySci_ErrorHandler

' ---- Code Body Starts ---- (Default)
    Dim forma As String
    Dim k As Integer, ps As Integer
    Dim v As Double
    If x < 0 Then k = n + 2 Else k = n + 1
    forma = DisplayExp(x, n, st)
    ps = InStr(forma, "E")
    k = modInternationalValue(Mid$(forma, ps + 1))
    v = modInternationalValue(Left$(forma, ps - 1))
    If k < 0 Then
        ps = k Mod 3
        If ps = -1 Then
            v = v * 100
            k = k - 2
        ElseIf ps = -2 Then
            v = v * 10
            k = k - 1
        End If
    Else
        ps = k Mod 3
        If ps = 1 Then
            v = v * 10
        ElseIf ps = 2 Then
            v = v * 100
        End If
        k = k - ps
    End If
    
    If k = 0 Then
        DisplaySci = DisplayFix(v, n, st)
    Else
        DisplaySci = DisplayFix(v, n, st) + "E" + Format$(k, "+00;-00")
    End If
' ---- Code Body Ends ---- (Default)
Exit Function

DisplaySci_ErrorHandler:
  Err.Raise Err.Number

Exit_DisplaySci:

End Function
