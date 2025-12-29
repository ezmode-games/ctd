# Releasing Mods

## Version Scheme

We use `0.x.y` versions until we're proud. Bump patch (`0.1.x`) for regular releases.

## Quick Reference

| Mod | Type | Auto-build? |
|-----|------|-------------|
| cyberpunk | Cargo | ✅ Yes |
| skyrim | CMake | ❌ Manual |
| fallout4 | CMake | ❌ Manual |
| oblivion-remastered | CMake | ❌ Manual |

**Don't tag:** `ue5` (library, not a standalone mod)

## Releasing a Cargo Mod (cyberpunk)

Automatic build and release:

```bash
git tag cyberpunk-v0.1.2
git push origin cyberpunk-v0.1.2
```

The workflow builds, packages with FOMOD, and creates a GitHub Release.

## Releasing a CMake Mod (skyrim, fallout4, oblivion-remastered)

### 1. Build locally

```powershell
.\scripts\build-mod.ps1 skyrim
```

### 2. Package

```powershell
.\scripts\package-mod.ps1 skyrim 0.1.2
```

Output: `dist/ctd-skyrim-v0.1.2.zip`

### 3. Create draft release

```bash
git tag skyrim-v0.1.2
git push origin skyrim-v0.1.2
```

This creates a **draft** release on GitHub.

### 4. Upload and publish

1. Go to https://github.com/ezmode-games/ctd/releases
2. Find the draft release
3. Upload `dist/ctd-skyrim-v0.1.2.zip`
4. Click "Publish release"

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
