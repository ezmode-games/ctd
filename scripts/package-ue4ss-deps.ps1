# Package UE4SS pre-built dependencies for CI
# Run after building mods/oblivion-remastered locally

param(
    [string]$BuildDir = "mods/oblivion-remastered/build",
    [string]$SourceDir = "mods/oblivion-remastered/RE-UE4SS",
    [string]$OutputDir = "dist/ue4ss-deps"
)

$ErrorActionPreference = "Stop"

# Get version from git
$Commit = (git -C $SourceDir rev-parse --short HEAD 2>$null)
if ($Commit) {
    $Version = "main-$Commit"
} else {
    $Version = "unknown"
}

Write-Host "Packaging UE4SS deps version: $Version"

# Create output structure
$OutPath = "$OutputDir/ue4ss-deps-$Version"
New-Item -ItemType Directory -Force -Path "$OutPath/lib" | Out-Null
New-Item -ItemType Directory -Force -Path "$OutPath/include" | Out-Null

# Copy libs
Write-Host "Copying libraries..."
Copy-Item "$BuildDir/Game__Shipping__Win64/lib/*.lib" "$OutPath/lib/" -Force

# Copy headers - UE4SS main
Write-Host "Copying headers..."
Copy-Item "$SourceDir/UE4SS/include" "$OutPath/include/UE4SS" -Recurse -Force

# Copy headers - deps/first (Unreal, Helpers, etc.)
$FirstDeps = @("Unreal", "Helpers", "Constructs", "File", "DynamicOutput", "Function", "String", "JSON", "Input")
foreach ($dep in $FirstDeps) {
    $src = "$SourceDir/deps/first/$dep/include"
    if (Test-Path $src) {
        Copy-Item $src "$OutPath/include/$dep" -Recurse -Force
    }
}

# Copy generated headers from build
if (Test-Path "$BuildDir/Game__Shipping__Win64/include") {
    Copy-Item "$BuildDir/Game__Shipping__Win64/include/*" "$OutPath/include/" -Recurse -Force
}

# Create version file
@{
    version = $Version
    commit = (git -C $SourceDir rev-parse HEAD)
    date = (Get-Date -Format "yyyy-MM-dd")
} | ConvertTo-Json | Set-Content "$OutPath/version.json"

# Create zip
$ZipPath = "$OutputDir/ue4ss-deps-$Version.zip"
Write-Host "Creating $ZipPath..."
Compress-Archive -Path $OutPath -DestinationPath $ZipPath -Force

Write-Host "Done! Upload $ZipPath to GitHub Releases"
Write-Host "Size: $([math]::Round((Get-Item $ZipPath).Length / 1MB, 2)) MB"
