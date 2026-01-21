<#
.SYNOPSIS
    Build a UE4SS-based mod.
.DESCRIPTION
    Builds the Rust library first, then runs CMake configure and build.
.PARAMETER ModDir
    Path to the mod directory.
.PARAMETER BuildType
    CMake build type (Debug or Release). Default: Release.
#>
param(
    [Parameter(Mandatory=$true)]
    [string]$ModDir,

    [ValidateSet("Debug", "Release")]
    [string]$BuildType = "Release"
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir

Push-Location $ModDir
try {
    Write-Host "Building UE4SS mod in $ModDir" -ForegroundColor Cyan

    # Build Rust library first
    Write-Host "Building Rust library..." -ForegroundColor Yellow
    $RustBuildDir = "build/rust-build"
    New-Item -ItemType Directory -Force -Path $RustBuildDir | Out-Null

    # The Rust crate is built as part of CMake via corrosion, but we need
    # to ensure the workspace is set up correctly

    # CMake configure
    Write-Host "Configuring CMake..." -ForegroundColor Yellow
    $BuildDir = "build"
    cmake --preset=default -S . -B $BuildDir

    # CMake build
    Write-Host "Building with CMake..." -ForegroundColor Yellow
    cmake --build $BuildDir --config $BuildType

    # Find output DLL
    $DllPath = Get-ChildItem -Path $BuildDir -Recurse -Filter "*.dll" |
        Where-Object { $_.Name -notmatch "deps|RE-UE4SS" } |
        Select-Object -First 1

    if ($DllPath) {
        Write-Host "Build complete: $($DllPath.FullName)" -ForegroundColor Green
    } else {
        Write-Host "Build complete (DLL location may vary)" -ForegroundColor Green
    }
}
finally {
    Pop-Location
}
