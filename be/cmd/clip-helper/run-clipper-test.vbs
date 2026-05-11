' Test VBS - write to file first
Set fso = CreateObject("Scripting.FileSystemObject")
dir = fso.GetParentFolderName(WScript.ScriptFullName)

' Write test file
Set f = fso.CreateTextFile(dir & "\vbs-test.log")
f.WriteLine "VBS is running! dir: " & dir
f.Close

' Then run clip-helper
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run dir & "\clip-helper.exe", 0, False