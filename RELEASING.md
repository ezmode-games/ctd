# Releasing Mods

## Version Scheme

We use `0.x.y` versions until we're proud. Bump patch (`0.1.x`) for regular releases.

## Quick Release (Recommended)

One command does everything - builds, packages, tags, uploads, publishes:

```powershell
.\scripts\release-mod.ps1 skyrim 0.1.2
```

Requires: `gh auth login` (GitHub CLI authenticated)

## Manual Release

### Cargo Mods (cyberpunk)

Automatic build and release via CI:

```bash
git tag cyberpunk-v0.1.2
git push origin cyberpunk-v0.1.2
```

### CMake Mods (skyrim, fallout4, oblivion-remastered)

```powershell
# Build
.\scripts\build-mod.ps1 skyrim

# Package
.\scripts\package-mod.ps1 skyrim 0.1.2

# Tag and push
git tag skyrim-v0.1.2
git push origin skyrim-v0.1.2

# Upload (manual or use gh)
gh release upload skyrim-v0.1.2 dist/ctd-skyrim-v0.1.2.zip
```

## Scripts

| Script | Purpose |
|--------|---------|
| `scripts/build-mod.ps1 <mod>` | Build a single CMake mod |
| `scripts/build-all.ps1` | Build all CMake mods |
| `scripts/package-mod.ps1 <mod> <version>` | Package mod for release |
| `scripts/package-ue4ss-deps.ps1` | Package UE4SS deps for CI (future) |

## Tag Format

```
<mod>-v<version>
```

Examples:
- `skyrim-v0.1.2`
- `fallout4-v0.1.3`
- `oblivion-remastered-v0.1.0`
- `cyberpunk-v0.2.0`
