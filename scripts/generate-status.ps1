# Generate status.json for external consumption
# Usage: .\scripts\generate-status.ps1
# Output: status.json (repo root)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$OutputFile = "$RepoRoot/status.json"

# Game display names
$GameNames = @{
    "skyrim" = "The Elder Scrolls V: Skyrim"
    "fallout4" = "Fallout 4"
    "fallout3" = "Fallout 3"
    "newvegas" = "Fallout: New Vegas"
    "oblivion-remastered" = "Oblivion Remastered"
    "cyberpunk" = "Cyberpunk 2077"
    "ue5" = "Unreal Engine 5 (Generic)"
    "elden-ring" = "Elden Ring"
}

# Manual status/quality overrides (edit these as development progresses)
$ModMeta = @{
    "skyrim" = @{
        status = "beta"
        quality = "good"
        features = @("crash_capture", "load_order", "mod_fingerprinting")
    }
    "fallout4" = @{
        status = "beta"
        quality = "good"
        features = @("crash_capture", "load_order", "mod_fingerprinting")
    }
    "cyberpunk" = @{
        status = "beta"
        quality = "good"
        features = @("crash_capture", "mod_scanning")
    }
    "oblivion-remastered" = @{
        status = "alpha"
        quality = "experimental"
        features = @("crash_capture", "load_order")
    }
    "ue5" = @{
        status = "alpha"
        quality = "experimental"
        features = @("crash_capture", "pak_scanning")
    }
    "elden-ring" = @{
        status = "wip"
        quality = "scaffolding"
        features = @("crash_capture")
    }
    "fallout3" = @{
        status = "scaffolding"
        quality = "scaffolding"
        features = @()
    }
    "newvegas" = @{
        status = "scaffolding"
        quality = "scaffolding"
        features = @()
    }
}

function Get-ModVersion($modDir) {
    $cargoToml = "$modDir/Cargo.toml"
    if (Test-Path $cargoToml) {
        $content = Get-Content $cargoToml -Raw
        # Check for explicit version first
        if ($content -match '(?m)^version\s*=\s*"([^"]+)"') {
            return $matches[1]
        }
        # Check for workspace inheritance
        if ($content -match 'version\.workspace\s*=\s*true') {
            $workspaceToml = "$RepoRoot/Cargo.toml"
            $wsContent = Get-Content $workspaceToml -Raw
            if ($wsContent -match '\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"') {
                return $matches[1]
            }
        }
    }
    return $null
}

function Get-BuildType($modDir) {
    if (Test-Path "$modDir/CMakeLists.txt") {
        return "cmake"
    }
    if (Test-Path "$modDir/Cargo.toml") {
        return "cargo"
    }
    return "unknown"
}

function Get-GitHubRelease($mod, $version) {
    if (-not $version) { return $null }
    $tag = "$mod-v$version"
    return "https://github.com/ezmode-games/ctd/releases/tag/$tag"
}

function Get-NexusInfo($modDir) {
    $nexusToml = "$modDir/nexus.toml"
    if (-not (Test-Path $nexusToml)) { return $null }

    $content = Get-Content $nexusToml -Raw
    $result = @{ game_slug = $null; mod_id = $null }

    if ($content -match 'game_slug\s*=\s*"([^"]+)"') {
        $result.game_slug = $matches[1]
    }
    # Match mod_id only if not commented out (no # at start of line)
    if ($content -match '(?m)^mod_id\s*=\s*(\d+)') {
        $result.mod_id = [int]$matches[1]
    }

    return $result
}

function Get-Published($mod, $version, $nexusInfo, $status) {
    $published = [ordered]@{
        github = $null
        nexus = $null
        ezmode = $null
    }

    # GitHub - always available if version exists and not scaffolding
    if ($version -and $status -notin @("scaffolding", "wip", "unknown")) {
        $published.github = Get-GitHubRelease $mod $version
    }

    # Nexus - only if we have both game_slug and mod_id
    if ($nexusInfo -and $nexusInfo.game_slug -and $nexusInfo.mod_id) {
        $published.nexus = "https://www.nexusmods.com/$($nexusInfo.game_slug)/mods/$($nexusInfo.mod_id)"
    }

    # ezmode - placeholder for future
    # $published.ezmode = "https://ctd.ezmode.games/mods/$mod"

    return $published
}

# Build mod list
$mods = @{}
$modDirs = Get-ChildItem "$RepoRoot/mods" -Directory

foreach ($dir in $modDirs) {
    $modName = $dir.Name
    $modPath = $dir.FullName

    $version = Get-ModVersion $modPath
    $buildType = Get-BuildType $modPath
    $nexusInfo = Get-NexusInfo $modPath
    $meta = $ModMeta[$modName]

    if (-not $meta) {
        $meta = @{
            status = "unknown"
            quality = "unknown"
            features = @()
        }
    }

    $published = Get-Published $modName $version $nexusInfo $meta.status

    $modInfo = [ordered]@{
        name = "CTD Crash Reporter"
        game = $GameNames[$modName]
        status = $meta.status
        version = $version
        build_type = $buildType
        quality = $meta.quality
        features = $meta.features
        published = $published
    }

    # Remove null game names for unknown mods
    if (-not $modInfo.game) {
        $modInfo.game = $modName
    }

    $mods[$modName] = $modInfo
}

# Build final output
$output = [ordered]@{
    '$schema' = "https://ezmode.games/schemas/ctd-status.json"
    generated_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    repository = "https://github.com/ezmode-games/ctd"
    mods = $mods
}

# Write JSON
$json = $output | ConvertTo-Json -Depth 10
$json | Set-Content $OutputFile -Encoding UTF8

Write-Host "Generated: $OutputFile" -ForegroundColor Green
Write-Host $json
