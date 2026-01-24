# Package a built mod for release
# Usage: .\scripts\package-mod.ps1 <mod-name> <version>
# Example: .\scripts\package-mod.ps1 skyrim 0.1.2

param(
    [Parameter(Mandatory=$true)]
    [string]$Mod,

    [Parameter(Mandatory=$true)]
    [string]$Version
)

$ErrorActionPreference = "Stop"
$ModDir = "mods/$Mod"

# Find the DLL - check Cargo builds first, then CMake builds
$ModUnderscore = $Mod.Replace("-", "_")
$DllPaths = @(
    # Cargo builds (default target)
    "target/release/ctd_$ModUnderscore.dll",
    # Cargo builds (explicit x64)
    "target/x86_64-pc-windows-msvc/release/ctd_$ModUnderscore.dll",
    # Cargo builds (x86)
    "target/i686-pc-windows-msvc/release/ctd_$ModUnderscore.dll",
    # CMake builds
    "$ModDir/build/Release/ctd-$Mod.dll",
    "$ModDir/build/Release/*.dll",
    # UE4SS builds
    "$ModDir/build/CTDCrashReporter/dlls/main.dll",
    "$ModDir/build/CTDCrashReporter/dlls/Game__Shipping__Win64/main.dll",
    "$ModDir/build/Game__Shipping__Win64/CTDCrashReporter/dlls/main.dll"
)

$Dll = $null
foreach ($Path in $DllPaths) {
    $Found = Get-Item $Path -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($Found) {
        $Dll = $Found
        break
    }
}

if (-not $Dll) {
    Write-Error "No DLL found. Run build-mod.ps1 first."
    exit 1
}

Write-Host "Found DLL: $Dll" -ForegroundColor Gray

# Get config from nexus.toml
$PluginPath = "plugins"
$ScriptExtender = "Unknown"
$NexusToml = "$ModDir/nexus.toml"

if (Test-Path $NexusToml) {
    $content = Get-Content $NexusToml -Raw
    if ($content -match 'plugin_path\s*=\s*"([^"]+)"') {
        $PluginPath = $matches[1]
    }
    if ($content -match 'script_extender\s*=\s*"([^"]+)"') {
        $ScriptExtender = $matches[1]
    }
}

# Create package
$DistDir = "dist/ctd-$Mod-v$Version"
$ArchiveName = "ctd-$Mod-v$Version.7z"

Write-Host "Packaging to $ArchiveName..." -ForegroundColor Cyan

# Clean and create dirs
if (Test-Path $DistDir) { Remove-Item $DistDir -Recurse -Force }
New-Item -ItemType Directory -Force -Path "$DistDir/$PluginPath" | Out-Null
New-Item -ItemType Directory -Force -Path "$DistDir/fomod" | Out-Null

# Copy DLL
$DllName = if ($Mod -eq "oblivion-remastered" -or $Mod -eq "elden-ring") { "main.dll" } else { "ctd-$Mod.dll" }
Copy-Item $Dll "$DistDir/$PluginPath/$DllName"

# Create config
@"
# CTD (Crash to Desktop Reporter) Configuration
[api]
url = "https://ctd.ezmode.games"
"@ | Set-Content "$DistDir/$PluginPath/ctd.toml" -Encoding UTF8

# Check for pre-made nexus folder with FOMOD
$NexusDir = "$ModDir/nexus"
$HasNexusFolder = Test-Path "$NexusDir/fomod"

if ($HasNexusFolder) {
    Write-Host "Using pre-made FOMOD from $NexusDir" -ForegroundColor Gray

    # Copy fomod folder
    Copy-Item "$NexusDir/fomod/*" "$DistDir/fomod/" -Recurse

    # Copy images if present
    if (Test-Path "$NexusDir/images") {
        New-Item -ItemType Directory -Force -Path "$DistDir/images" | Out-Null
        Copy-Item "$NexusDir/images/*" "$DistDir/images/" -Recurse
    }

    # Update version in info.xml
    $InfoXml = "$DistDir/fomod/info.xml"
    if (Test-Path $InfoXml) {
        $infoContent = Get-Content $InfoXml -Raw
        # Update Version element and MachineVersion attribute
        $infoContent = $infoContent -replace '<Version[^>]*>[^<]*</Version>', "<Version MachineVersion=`"$Version`">$Version</Version>"
        Set-Content $InfoXml $infoContent -Encoding UTF8
    }
} else {
    Write-Host "Generating FOMOD (no nexus/ folder found)" -ForegroundColor Yellow

    # FOMOD info.xml
    @"
<?xml version="1.0" encoding="UTF-8"?>
<fomod>
    <Name>CTD - Crash Reporter ($Mod)</Name>
    <Author>ezmode.games</Author>
    <Version MachineVersion="$Version">$Version</Version>
    <Id>ctd-$Mod</Id>
    <Website>https://ctd.ezmode.games</Website>
    <Description>Automatic crash reporting. Helps identify and fix mod conflicts.</Description>
    <Groups>
        <element>Utilities</element>
    </Groups>
</fomod>
"@ | Set-Content "$DistDir/fomod/info.xml" -Encoding UTF8

    # FOMOD ModuleConfig.xml
    $RootFolder = $PluginPath.Split("/")[0]
    @"
<?xml version="1.0" encoding="UTF-8"?>
<config xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://qconsulting.ca/fo3/ModConfig5.0.xsd">
    <moduleName position="RightOfImage" colour="990000">CTD - Crash to Desktop Reporter</moduleName>

    <requiredInstallFiles>
        <folder source="$RootFolder" destination="$RootFolder"/>
    </requiredInstallFiles>
</config>
"@ | Set-Content "$DistDir/fomod/ModuleConfig.xml" -Encoding UTF8
}

# README
@"
CTD - Crash to Desktop Reporter
================================
Game: $Mod
Version: $Version
Requires: $ScriptExtender

https://ctd.ezmode.games
https://github.com/ezmode-games/ctd
"@ | Set-Content "$DistDir/README.txt" -Encoding UTF8

# 7z archive
$ArchivePath = "dist/$ArchiveName"
if (Test-Path $ArchivePath) { Remove-Item $ArchivePath -Force }

# Find 7z executable
$7zPath = "C:\Program Files\7-Zip\7z.exe"
if (-not (Test-Path $7zPath)) {
    $7zPath = "C:\Program Files (x86)\7-Zip\7z.exe"
}
if (-not (Test-Path $7zPath)) {
    $7zPath = "7z"  # Try PATH
}

Push-Location $DistDir
& $7zPath a -t7z -mx=9 "../$ArchiveName" * | Out-Null
Pop-Location

$Size = [math]::Round((Get-Item $ArchivePath).Length / 1KB, 1)
Write-Host "Created: $ArchivePath ($Size KB)" -ForegroundColor Green
Write-Host "Upload to: https://github.com/ezmode-games/ctd/releases" -ForegroundColor Yellow
