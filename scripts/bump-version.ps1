# Bump version across workspace and create release tags
# Usage: .\scripts\bump-version.ps1 -Version 0.1.3
#
# This script:
# 1. Updates version in Cargo.toml workspace
# 2. Updates Cargo.lock
# 3. Regenerates status.json
# 4. Updates CHANGELOG.md (moves [Unreleased] to new version)
# 5. Commits changes
# 6. Creates tags for each releasable mod
# 7. Pushes to origin (tags trigger CI draft releases)

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,

    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir

# Mods that get released (excludes UE4SS-based mods that need local builds)
$ReleasableMods = @("skyrim", "fallout4", "fallout3", "newvegas", "cyberpunk")

Write-Host "=== Bumping to v$Version ===" -ForegroundColor Cyan

# Validate version format
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Invalid version format. Use semver (e.g., 0.1.3)"
    exit 1
}

# Check for clean working directory
$gitStatus = git -C $RepoRoot status --porcelain
if ($gitStatus -and -not $DryRun) {
    Write-Error "Working directory not clean. Commit or stash changes first."
    exit 1
}

# 1. Update Cargo.toml workspace version
Write-Host "`n[1/6] Updating Cargo.toml..." -ForegroundColor Yellow
$cargoToml = "$RepoRoot/Cargo.toml"
$content = Get-Content $cargoToml -Raw
# Only replace version in [workspace.package] section, not dependency versions
$newContent = $content -replace '(\[workspace\.package\][\s\S]*?version\s*=\s*")[^"]+(")', "`${1}$Version`${2}"
if ($DryRun) {
    Write-Host "Would update $cargoToml" -ForegroundColor Gray
} else {
    $newContent | Set-Content $cargoToml -NoNewline
}

# 2. Update Cargo.lock
Write-Host "`n[2/6] Updating Cargo.lock..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "Would run: cargo update --workspace" -ForegroundColor Gray
} else {
    Push-Location $RepoRoot
    cargo update --workspace
    Pop-Location
}

# 3. Regenerate status.json
Write-Host "`n[3/6] Regenerating status.json..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "Would run: generate-status.ps1" -ForegroundColor Gray
} else {
    & "$ScriptDir/generate-status.ps1"
}

# 4. Update CHANGELOG.md
Write-Host "`n[4/6] Updating CHANGELOG.md..." -ForegroundColor Yellow
$changelogPath = "$RepoRoot/CHANGELOG.md"
$today = (Get-Date).ToString("yyyy-MM-dd")
$prevVersion = "0.1.$(([int]$Version.Split('.')[-1]) - 1)"

if (Test-Path $changelogPath) {
    $changelog = Get-Content $changelogPath -Raw

    # Check if there's content under [Unreleased]
    if ($changelog -match '## \[Unreleased\]\s*\n\s*\n## \[') {
        Write-Warning "No changes under [Unreleased] section"
    }

    # Replace [Unreleased] header and add new version
    $newChangelog = $changelog -replace '## \[Unreleased\]', "## [Unreleased]`n`n## [$Version] - $today"

    # Add version link at bottom (before the last empty lines)
    $versionLink = "[$Version]: https://github.com/ezmode-games/ctd/compare/skyrim-v$prevVersion...skyrim-v$Version"
    if ($newChangelog -notmatch "\[$Version\]:") {
        $newChangelog = $newChangelog.TrimEnd() + "`n$versionLink`n"
    }

    if ($DryRun) {
        Write-Host "Would update CHANGELOG.md with version $Version dated $today" -ForegroundColor Gray
    } else {
        [System.IO.File]::WriteAllText($changelogPath, $newChangelog, [System.Text.UTF8Encoding]::new($false))
    }
} else {
    Write-Warning "CHANGELOG.md not found, skipping"
}

# 5. Commit changes
Write-Host "`n[5/6] Committing changes..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "Would commit: chore: bump version to $Version" -ForegroundColor Gray
} else {
    Push-Location $RepoRoot
    git add Cargo.toml Cargo.lock status.json CHANGELOG.md
    git commit -m "chore: bump version to $Version"
    Pop-Location
}

# 6. Create tags
Write-Host "`n[6/6] Creating release tags..." -ForegroundColor Yellow
$tags = @()
foreach ($mod in $ReleasableMods) {
    $tag = "$mod-v$Version"
    $tags += $tag
    if ($DryRun) {
        Write-Host "Would create tag: $tag" -ForegroundColor Gray
    } else {
        Push-Location $RepoRoot
        # Check if tag already exists
        git rev-parse -q --verify "refs/tags/$tag" 2>$null | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Write-Warning "Tag '$tag' already exists; skipping creation."
        } else {
            git tag $tag
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Failed to create tag '$tag'"
            }
        }
        Pop-Location
    }
}

# 6. Push commit and tags together
if (-not $DryRun) {
    Write-Host "`nPushing to origin..." -ForegroundColor Yellow
    Push-Location $RepoRoot
    git push origin main --tags
    Pop-Location
}

Write-Host "`n=== Version bump complete ===" -ForegroundColor Green
Write-Host "`nTags created:"
foreach ($tag in $tags) {
    Write-Host "  - $tag"
}
Write-Host "`nCI will create draft releases. Review and publish at:"
Write-Host "  https://github.com/ezmode-games/ctd/releases" -ForegroundColor Yellow
