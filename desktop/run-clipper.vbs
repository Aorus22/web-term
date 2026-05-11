' Run clip-helper with debug logging
Set WshShell = CreateObject("WScript.Shell")

' Get current directory
currentDir = WshShell.CurrentDirectory

' Or use the path where the script is located
Set fso = CreateObject("Scripting.FileSystemObject")
scriptPath = WScript.ScriptFullName
scriptDir = fso.GetAbsolutePathName(fso.GetParentFolderName(scriptPath))

' Write to log file to confirm VBS is running
Set logFile = fso.CreateTextFile(scriptDir & "\vbs-start.log")
logFile.WriteLine "VBS started at " & Now & " dir: " & scriptDir
logFile.Close

' Run clip-helper from same directory
exePath = scriptDir & "\clip-helper.exe"
WshShell.Run exePath, 0, False