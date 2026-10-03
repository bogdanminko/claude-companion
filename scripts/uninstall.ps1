# Stops and removes Claude Companion, its autostart entry and Start menu shortcut.
$ErrorActionPreference = "SilentlyContinue"

Get-Process claude-companion | Stop-Process -Force
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "ClaudeCompanion"
Remove-Item (Join-Path ([Environment]::GetFolderPath("Programs")) "Claude Companion.lnk")
Start-Sleep -Milliseconds 300
Remove-Item -Recurse -Force (Join-Path $env:LOCALAPPDATA "Programs\ClaudeCompanion")

Write-Host "OK: Claude Companion removed"
