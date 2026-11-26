@echo off
chcp 65001 >nul
echo [Obsidian_Rs] Lancement de l'installateur...
if exist "%~dp0target\release\obsidian_rs.exe" (
    "%~dp0target\release\obsidian_rs.exe" installer %*
) else (
    cargo run --release --bin obsidian_rs -- installer %*
)
