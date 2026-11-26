@echo off
chcp 65001 >nul
if "%~1"=="--premier-plan" (
    "%~dp0target\release\obsidian_rs.exe" surveiller
) else if "%~1"=="-f" (
    "%~dp0target\release\obsidian_rs.exe" surveiller
) else (
    echo [Obsidian_Rs] Lancement du demon de surveillance en arriere-plan...
    powershell -NoProfile -NonInteractive -WindowStyle Hidden -Command "Start-Process -FilePath '%~dp0target\release\obsidian_rs.exe' -ArgumentList 'surveiller' -WindowStyle Hidden"
    echo [Obsidian_Rs] Demon de surveillance actif en arriere-plan (surveille l'IDE et VS Code).
)
