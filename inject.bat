@echo off
chcp 65001 >nul
if exist "%~dp0target\release\obsidian_rs.exe" (
    "%~dp0target\release\obsidian_rs.exe" injecter %*
) else (
    cargo run --release --bin obsidian_rs -- injecter %*
)
