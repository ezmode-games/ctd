<#
.SYNOPSIS
    Setup UE4SS submodule and apply necessary patches.
.DESCRIPTION
    Handles git URL rewriting (SSH to HTTPS), submodule initialization,
    and patches for patternsleuth compilation issues.
.PARAMETER ModDir
    Path to the mod directory containing RE-UE4SS submodule.
#>
param(
    [Parameter(Mandatory=$true)]
    [string]$ModDir
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir

Push-Location $ModDir
try {
    Write-Host "Setting up UE4SS in $ModDir" -ForegroundColor Cyan

    # Configure git to use HTTPS instead of SSH for GitHub
    # This fixes nested submodule issues where SSH URLs fail
    Write-Host "Configuring git URL rewriting..." -ForegroundColor Yellow
    git config --global url."https://github.com/".insteadOf "git@github.com:"

    # Initialize and update submodules
    Write-Host "Initializing submodules..." -ForegroundColor Yellow
    git submodule update --init --recursive

    # Patch patternsleuth to fix unused import warning that fails with -D warnings
    $PatternsleuthMod = "RE-UE4SS/deps/first/patternsleuth/patternsleuth/src/resolvers/unreal/mod.rs"
    if (Test-Path $PatternsleuthMod) {
        Write-Host "Patching patternsleuth..." -ForegroundColor Yellow
        $Content = Get-Content $PatternsleuthMod -Raw
        if ($Content -match "bail_out") {
            $Content = $Content -replace "use crate::resolvers::bail_out;`r?`n", ""
            $Content = $Content -replace "use crate::resolvers::\{bail_out, ", "use crate::resolvers::{"
            Set-Content $PatternsleuthMod $Content -NoNewline
            Write-Host "  Removed unused bail_out import" -ForegroundColor Green
        } else {
            Write-Host "  Already patched or pattern not found" -ForegroundColor Gray
        }
    }

    Write-Host "Setup complete!" -ForegroundColor Green
}
finally {
    Pop-Location
}
