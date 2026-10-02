@echo off
rem Inicia a versao mais recente do Aura (recompila so quando o codigo mudou).
rem Opcoes: -Demo  -Rebuild  -Pull  -NoStart  (veja scripts\iniciar-aura.ps1)
where pwsh >nul 2>nul
if %errorlevel%==0 (
  pwsh -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\iniciar-aura.ps1" %*
) else (
  powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\iniciar-aura.ps1" %*
)
if errorlevel 1 pause
