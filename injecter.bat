@echo off
chcp 65001 >nul
echo [Obsidian_Rs] Lancement de l'injection...
if exist "%~dp0target\release\obsidian_rs.exe" (
    "%~dp0target\release\obsidian_rs.exe" injecter %*
) else (
    cargo run --release --bin obsidian_rs -- injecter %*
)
