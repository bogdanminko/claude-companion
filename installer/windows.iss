; Windows installer (Inno Setup 6): per-user, no admin rights.
;   iscc /DAppVersion=0.2.3 installer\windows.iss   →  dist\ClaudeCompanion-Setup.exe
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{8F3C2A51-6B7E-4D2A-9C1F-2E5B7A9D4C10}
AppName=Claude Companion
AppVersion={#AppVersion}
AppPublisher=Claude Companion contributors
AppPublisherURL=https://github.com/bogdanminko/claude-companion
DefaultDirName={localappdata}\Programs\ClaudeCompanion
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\dist
OutputBaseFilename=ClaudeCompanion-Setup
SetupIconFile=..\Resources\claude-companion.ico
UninstallDisplayIcon={app}\claude-companion.exe
UninstallDisplayName=Claude Companion
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Files]
Source: "..\target\release\claude-companion.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Claude Companion"; Filename: "{app}\claude-companion.exe"; Parameters: "--supervise"

[Registry]
; start at login; --supervise restarts Pixel after a crash, Quit ends it until next login
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "ClaudeCompanion"; ValueData: """{app}\claude-companion.exe"" --supervise"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\claude-companion.exe"; Parameters: "--supervise"; Description: "Start Claude Companion"; Flags: nowait postinstall

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM claude-companion.exe"; Flags: runhidden; RunOnceId: "StopPixel"

[Code]
// replace a running copy: stop it before the files are copied
function InitializeSetup(): Boolean;
var
  Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM claude-companion.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Result := True;
end;
