# CTD - Crash To Desktop Reporter

Automatic crash reporting for modded games. Captures crash context and helps identify patterns across users.

**Hosted**: [ctd.ezmode.games](https://ctd.ezmode.games)
**API Docs**: [OpenAPI 3.1](https://ctd.ezmode.games/docs)
**License**: AGPL-3.0

## Supported Games

| Game | Plugin | Status |
|------|--------|--------|
| Skyrim SE/AE | SKSE64 | Beta |
| Fallout 4 | F4SE | Beta |
| Cyberpunk 2077 | RED4ext | Beta |
| Oblivion Remastered | UE4SS | Alpha |
| Unreal Engine 5 | UE4SS | Alpha |

## What It Captures

- Stack traces with module offsets
- Load order at crash time
- Mod fingerprints (file hashes)
- Crash patterns across users

## Installation

Download from [Releases](https://github.com/ezmode-games/ctd/releases) or [Nexus Mods](https://www.nexusmods.com).

Extract to your game's mod directory or install via Vortex/MO2.

## For Mod Creators

Subscribe to crashes mentioning your mods:

- See crash reports where your mod is present
- Pattern detection ("50 users crashed with mod A + mod B")
- Export data for analysis

## Building

### Cargo Mods (Cyberpunk)

```bash
cargo build --release -p ctd-cyberpunk
```

### CMake Mods (Skyrim, Fallout 4)

```powershell
.\scripts\build-mod.ps1 -Mod skyrim
.\scripts\build-mod.ps1 -Mod fallout4
```

### Packaging

```powershell
.\scripts\package-mod.ps1 -Mod skyrim -Version 0.1.2
```

## Self-Hosting

```bash
cd api
pnpm install
pnpm dev
```

## License

AGPL-3.0 - Modifications must be open sourced.
