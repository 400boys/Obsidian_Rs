<#
.SYNOPSIS
    Script de compilation automatisé du package installateur Windows (.msi) pour Obsidian_Rs.
.DESCRIPTION
    1. Compile le binaire release (cargo build --release).
    2. Utilise le compilateur WiX Toolset (candle.exe et light.exe).
    3. Produit l'installateur dist/Obsidian_Rs-v1.0.8-x64.msi.
#>

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
Set-Location $root

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   COMPILATION DE L'INSTALLATEUR MSI - OBSIDIAN_RS        " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# 1. Compilation du binaire Rust en release
Write-Host "`n[1/3] Compilation du binaire Rust en mode release..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Échec de la compilation de cargo."
}

# 2. Localisation ou téléchargement de WiX Toolset
$wixDir = "$env:LOCALAPPDATA\Programs\wix3"
if (-not (Test-Path "$wixDir\candle.exe")) {
    Write-Host "`n[WiX] WiX Toolset non détecté dans $wixDir, téléchargement portable..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path $wixDir | Out-Null
    $zipFile = Join-Path $wixDir "wix311.zip"
    Invoke-WebRequest -Uri "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip" -OutFile $zipFile
    Expand-Archive -Path $zipFile -DestinationPath $wixDir -Force
    Remove-Item -Force $zipFile
}

# 3. Compilation du package MSI
Write-Host "`n[2/3] Génération du package MSI avec WiX..." -ForegroundColor Yellow
$wixObj = Join-Path $root "wix\Obsidian_Rs.wixobj"
$msiOut = Join-Path $root "dist\Obsidian_Rs-v1.0.8-x64.msi"
New-Item -ItemType Directory -Force -Path (Join-Path $root "dist") | Out-Null

& "$wixDir\candle.exe" -arch x64 -ext WixUIExtension -ext WixUtilExtension -out $wixObj (Join-Path $root "wix\Obsidian_Rs.wxs")
& "$wixDir\light.exe" -ext WixUIExtension -ext WixUtilExtension -sice:ICE64 -sice:ICE91 -out $msiOut $wixObj

# Nettoyage des fichiers temporaires WiX
Remove-Item -Force (Join-Path $root "wix\*.wixobj"), (Join-Path $root "wix\*.wixpdb") -ErrorAction SilentlyContinue

Write-Host "`n[3/3] Installateur généré avec succès !" -ForegroundColor Green
Write-Host "  Chemin : $msiOut" -ForegroundColor Green
$size = (Get-Item $msiOut).Length / 1MB
Write-Host "  Taille : $([math]::Round($size, 2)) Mo" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
