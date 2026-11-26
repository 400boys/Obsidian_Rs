@echo off
chcp 65001 >nul
echo [Obsidian_Rs] Desinstallation et restauration...
if exist "%~dp0target\release\obsidian_rs.exe" (
    "%~dp0target\release\obsidian_rs.exe" desinstaller %*
) else (
    cargo run --release --bin obsidian_rs -- desinstaller %*
)
