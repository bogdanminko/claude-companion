# Builds Claude Companion and installs it for the current user: starts at login, restarts after a crash.
#   powershell -ExecutionPolicy Bypass -File scripts\install.ps1
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

$dir = Join-Path $env:LOCALAPPDATA "Programs\ClaudeCompanion"
$exe = Join-Path $dir "claude-companion.exe"

# stop the running copy, if any
Get-Process claude-companion -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300

New-Item -ItemType Directory -Force -Path $dir | Out-Null
Copy-Item "target\release\claude-companion.exe" $exe -Force

# autostart: --supervise restarts Pixel after a crash; Quit ends it until next login
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "ClaudeCompanion" -Value "`"$exe`" --supervise"

# Start menu shortcut
$lnk = Join-Path ([Environment]::GetFolderPath("Programs")) "Claude Companion.lnk"
$shell = New-Object -ComObject WScript.Shell
$s = $shell.CreateShortcut($lnk)
$s.TargetPath = $exe
$s.Arguments = "--supervise"
$s.IconLocation = "$exe,0"
$s.Save()

Start-Process -FilePath $exe -ArgumentList "--supervise"
Write-Host "OK: Claude Companion installed and running ($exe)"
