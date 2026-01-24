# Update README.md with status from status.json
# Usage: .\scripts\update-readme.ps1

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$StatusFile = "$RepoRoot/status.json"
$ReadmeFile = "$RepoRoot/README.md"

if (-not (Test-Path $StatusFile)) {
    Write-Error "status.json not found. Run generate-status.ps1 first."
    exit 1
}

# Read status.json
$status = Get-Content $StatusFile -Raw | ConvertFrom-Json

# Plugin names for each game
$Plugins = @{
    "skyrim" = "SKSE64"
    "fallout4" = "F4SE"
    "fallout3" = "FOSE"
    "newvegas" = "NVSE"
    "cyberpunk" = "RED4ext"
    "oblivion-remastered" = "UE4SS"
    "ue5" = "UE4SS"
    "elden-ring" = "UE4SS"
}

# Build table rows
$rows = @()
$rows += "| Game | Plugin | Status | Version | Download |"
$rows += "|------|--------|--------|---------|----------|"

# Order: released first (by game name), then unreleased
# Skip internal mods (not meant for public release)
$mods = $status.mods.PSObject.Properties | Where-Object {
    $_.Value.status -ne "internal"
} | Sort-Object {
    $mod = $_.Value
    $hasRelease = $null -ne $mod.version
    if ($hasRelease) { "0_$($mod.game)" } else { "1_$($mod.game)" }
}

foreach ($prop in $mods) {
    $name = $prop.Name
    $mod = $prop.Value

    $game = $mod.game
    $plugin = $Plugins[$name]
    $statusText = (Get-Culture).TextInfo.ToTitleCase($mod.status)

    if ($mod.version) {
        $version = "v$($mod.version)"
        $github = $mod.published.github
        if ($github) {
            $download = "[$version]($github)"
        } else {
            $download = $version
        }
    } else {
        $version = "-"
        $download = "-"
    }

    $rows += "| $game | $plugin | $statusText | $version | $download |"
}

$table = $rows -join "`n"

# Read README
$readme = Get-Content $ReadmeFile -Raw

# Replace the table (between "## Supported Games" and the next ##)
$pattern = '(?s)(## Supported Games\r?\n\r?\n)(\|[\s\S]*?)(\r?\n\r?\n## )'
$replacement = "`$1$table`$3"

$newReadme = $readme -replace $pattern, $replacement

# Write back
Set-Content $ReadmeFile $newReadme -NoNewline

Write-Host "README.md updated with current status" -ForegroundColor Green
