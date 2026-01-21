# Full release workflow for a mod
# Usage: .\scripts\release-mod.ps1 <mod-name> <version>
# Example: .\scripts\release-mod.ps1 oblivion-remastered 0.1.0

param(
    [Parameter(Mandatory=$true)]
    [string]$Mod,

    [Parameter(Mandatory=$true)]
    [string]$Version,

    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Tag = "$Mod-v$Version"
$ArchiveName = "ctd-$Mod-v$Version.7z"
$ArchivePath = "dist/$ArchiveName"

Write-Host "=== Releasing $Mod v$Version ===" -ForegroundColor Cyan

# Check gh is authenticated
$ghAuth = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "GitHub CLI not authenticated. Run: gh auth login"
    exit 1
}

# Check for uncommitted changes
$gitStatus = git status --porcelain
if ($gitStatus) {
    Write-Error "Uncommitted changes. Commit or stash first."
    exit 1
}

# Step 1: Build (if CMake mod and not skipped)
$HasCMake = Test-Path "mods/$Mod/CMakeLists.txt"
if ($HasCMake -and -not $SkipBuild) {
    Write-Host "`n[1/5] Building $Mod..." -ForegroundColor Yellow
    & "$ScriptDir/build-mod.ps1" -Mod $Mod
    if ($LASTEXITCODE -ne 0) { exit 1 }
} else {
    Write-Host "`n[1/5] Build skipped" -ForegroundColor Gray
}

# Step 2: Package
Write-Host "`n[2/5] Packaging..." -ForegroundColor Yellow
& "$ScriptDir/package-mod.ps1" -Mod $Mod -Version $Version
if ($LASTEXITCODE -ne 0) { exit 1 }

if (-not (Test-Path $ArchivePath)) {
    Write-Error "Package not found at $ArchivePath"
    exit 1
}

# Step 3: Create and push tag
Write-Host "`n[3/5] Creating tag $Tag..." -ForegroundColor Yellow
git tag $Tag 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tag already exists, using existing tag" -ForegroundColor Gray
}
git push origin $Tag 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Tag already pushed" -ForegroundColor Gray
}

# Step 4: Wait for release to be created (or create it)
Write-Host "`n[4/5] Creating/finding release..." -ForegroundColor Yellow
Start-Sleep -Seconds 5  # Give workflow time to start

# Check if release exists, if not create it
$release = gh release view $Tag 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Creating release..." -ForegroundColor Gray

    # Get description from nexus.toml or Cargo.toml
    $Desc = "CTD crash reporter for $Mod"
    $NexusToml = "mods/$Mod/nexus.toml"
    $CargoToml = "mods/$Mod/Cargo.toml"

    if (Test-Path $NexusToml) {
        $content = Get-Content $NexusToml -Raw
        if ($content -match 'description\s*=\s*"([^"]+)"') {
            $Desc = $matches[1]
        }
    } elseif (Test-Path $CargoToml) {
        $content = Get-Content $CargoToml -Raw
        if ($content -match 'description\s*=\s*"([^"]+)"') {
            $Desc = $matches[1]
        }
    }

    gh release create $Tag `
        --title "ctd-$Mod v$Version" `
        --notes "## $Desc`n`n**Mod:** ctd-$Mod`n**Version:** $Version" `
        --draft=$HasCMake
}

# Step 5: Upload artifact
Write-Host "`n[5/5] Uploading $ArchiveName..." -ForegroundColor Yellow
gh release upload $Tag $ArchivePath --clobber

# Publish if it was a draft
if ($HasCMake) {
    Write-Host "`nPublishing release..." -ForegroundColor Yellow
    gh release edit $Tag --draft=false
}

Write-Host "`n=== Release complete! ===" -ForegroundColor Green
Write-Host "https://github.com/ezmode-games/ctd/releases/tag/$Tag"
