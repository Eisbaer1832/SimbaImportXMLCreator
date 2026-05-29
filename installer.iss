[Setup]
AppName=SimbaImportHelper
AppVersion=1.0
DefaultDirName={autopf}\SimbaImportHelper
DefaultGroupName=CapputinoDevelopment
OutputDir=Output
OutputBaseFilename=SimbaImportHelperInstaller

[Files]
Source: "target\x86_64-pc-windows-gnu\release\SimbaImportXMLCreator.exe"; DestDir: "{app}"

[Icons]
Name: "{group}\SimbaImportHelper"; Filename: "{app}\SimbaImportHelper.exe"
Name: "{commondesktop}\SimbaImportHelper"; Filename: "{app}\SimbaImportHelper.exe"
