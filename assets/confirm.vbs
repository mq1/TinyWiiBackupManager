' SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
' SPDX-License-Identifier: GPL-3.0-only

' Usage: cscript confirm.vbs "title" "message "Info|Warning|Error""
' Returns: yes|no

Dim title, message, level, icon, result

title = WScript.Arguments(0)
message = WScript.Arguments(1)
level = WScript.Arguments(2)

Select Case level
    Case "Info"
        icon = vbInformation
    Case "Warning"
        icon = vbExclamation
    Case "Error"
        icon = vbCritical
End Select

result = MsgBox(title, vbYesNo + icon, message)

Select Case result
    Case vbYes
        WScript.Echo "yes"
    Case vbNo
        WScript.Echo "no"
End Select

