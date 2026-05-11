' Run clip-helper with output redirected to file in user's home
Set WshShell = CreateObject("WScript.Shell")

Set fso = CreateObject("Scripting.FileSystemObject")

' Get user home directory
userHome = WshShell.ExpandEnvironmentStrings("%USERPROFILE%")
outputPath = userHome & "\clipboard-output.log"

' Use explicit testing-clip folder path
clipperFolder = userHome & "\testing-clip"
exePath = clipperFolder & "\web-term-clip.exe"

' Run with output to userprofile folder
cmdLine = "cmd /c """ & exePath & """>""" & outputPath & """ 2>&1"
WshShell.Run cmdLine, 0, False