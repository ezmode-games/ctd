# Build a CMake mod locally
# Usage: .\scripts\build-mod.ps1 <mod-name>
# Example: .\scripts\build-mod.ps1 skyrim

param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("skyrim", "fallout4", "oblivion-remastered")]
    [string]$Mod
)

$ErrorActionPreference = "Stop"
$ModDir = "mods/$Mod"

# Set up VS environment if cmake not found
if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsPath = & $vsWhere -latest -property installationPath
        $vcvars = "$vsPath\VC\Auxiliary\Build\vcvars64.bat"
        if (Test-Path $vcvars) {
            Write-Host "Setting up Visual Studio environment..." -ForegroundColor Gray
            cmd /c "`"$vcvars`" && set" | ForEach-Object {
                if ($_ -match "^([^=]+)=(.*)$") {
                    [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
                }
            }
        }
    }
}

if (-not (Test-Path "$ModDir/CMakeLists.txt")) {
    Write-Error "No CMakeLists.txt found in $ModDir"
    exit 1
}

Write-Host "Building $Mod..." -ForegroundColor Cyan

# Determine architecture from nexus.toml
$Arch = "x64"
$RustTarget = "x86_64-pc-windows-msvc"
$NexusToml = "$ModDir/nexus.toml"

if (Test-Path $NexusToml) {
    $content = Get-Content $NexusToml -Raw
    if ($content -match 'arch\s*=\s*"x86"') {
        $Arch = "Win32"
        $RustTarget = "i686-pc-windows-msvc"
    }
}

Write-Host "Architecture: $Arch ($RustTarget)" -ForegroundColor Gray

# Build Rust first (for hybrid mods that need it)
# Skip for oblivion-remastered - CMake handles Rust build via custom command
if ($Mod -ne "oblivion-remastered") {
    Write-Host "`n[1/3] Building Rust library..." -ForegroundColor Yellow
    Push-Location $ModDir
    try {
        cargo build --release --target $RustTarget --target-dir build/rust-build
        if ($LASTEXITCODE -ne 0) { throw "Rust build failed" }
    } finally {
        Pop-Location
    }
} else {
    Write-Host "`n[1/3] Rust library (built by CMake)..." -ForegroundColor Yellow
}

# Configure CMake
Write-Host "`n[2/3] Configuring CMake..." -ForegroundColor Yellow
Push-Location $ModDir
try {
    cmake -B build -G "Visual Studio 17 2022" -A $Arch
    if ($LASTEXITCODE -ne 0) { throw "CMake configure failed" }
} finally {
    Pop-Location
}

# Build
Write-Host "`n[3/3] Building..." -ForegroundColor Yellow
Push-Location $ModDir
try {
    cmake --build build --config Release
    if ($LASTEXITCODE -ne 0) { throw "CMake build failed" }
} finally {
    Pop-Location
}

# Find output
Write-Host "`nBuild complete!" -ForegroundColor Green

$Outputs = @(
    "$ModDir/build/Release/*.dll",
    "$ModDir/build/*/Release/*.dll",
    "$ModDir/build/Game__Shipping__Win64/*.dll"
) | ForEach-Object { Get-Item $_ -ErrorAction SilentlyContinue } | Select-Object -First 3

if ($Outputs) {
    Write-Host "Output:" -ForegroundColor Cyan
    $Outputs | ForEach-Object { Write-Host "  $_" }
}
