@echo off
setlocal
set SCRIPT_DIR=%~dp0
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%Start-Parkside.ps1"
if errorlevel 1 (
  echo.
  echo Der Starter wurde mit einem Fehler beendet.
  pause
)
