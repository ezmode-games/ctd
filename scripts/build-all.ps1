# Build all CMake mods
# Usage: .\scripts\build-all.ps1

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$Mods = @("skyrim", "fallout4", "oblivion-remastered")
$Failed = @()

foreach ($Mod in $Mods) {
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "Building $Mod" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    try {
        & "$ScriptDir/build-mod.ps1" -Mod $Mod
    } catch {
        Write-Host "FAILED: $Mod - $_" -ForegroundColor Red
        $Failed += $Mod
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
if ($Failed.Count -eq 0) {
    Write-Host "All builds succeeded!" -ForegroundColor Green
} else {
    Write-Host "Failed: $($Failed -join ', ')" -ForegroundColor Red
    exit 1
}
