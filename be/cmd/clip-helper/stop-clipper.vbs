' Stop clip-helper process
Set WshShell = CreateObject("WScript.Shell")

' Kill the process
WshShell.Run "taskkill /F /IM web-term-clip.exe", 0, False

' Log it
Set fso = CreateObject("Scripting.FileSystemObject")
scriptPath = WScript.ScriptFullName
scriptDir = fso.GetAbsolutePathName(fso.GetParentFolderName(scriptPath))
Set logFile = fso.CreateTextFile(scriptDir & "\vbs-stop.log")
logFile.WriteLine "VBS stopped at " & Now
logFile.Close