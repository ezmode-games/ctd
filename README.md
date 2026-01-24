# CTD - Crash To Desktop Reporter

Automatic crash reporting for modded games. Captures crash context and helps identify patterns across users.

**Hosted**: [ctd.ezmode.games](https://ctd.ezmode.games)
**API Docs**: [OpenAPI 3.1](https://ctd.ezmode.games/docs)
**License**: AGPL-3.0

## Supported Games

| Game | Plugin | Status | Version | Download |
|------|--------|--------|---------|----------|
| Cyberpunk 2077 | RED4ext | Beta | v0.1.4 | [v0.1.4](https://github.com/ezmode-games/ctd/releases/tag/cyberpunk-v0.1.4) |
| Fallout 3 | FOSE | Beta | v0.1.4 | [v0.1.4](https://github.com/ezmode-games/ctd/releases/tag/fallout3-v0.1.4) |
| Fallout 4 | F4SE | Beta | v0.1.4 | [v0.1.4](https://github.com/ezmode-games/ctd/releases/tag/fallout4-v0.1.4) |
| Fallout: New Vegas | NVSE | Beta | v0.1.4 | [v0.1.4](https://github.com/ezmode-games/ctd/releases/tag/newvegas-v0.1.4) |
| Oblivion Remastered | UE4SS | Alpha | v0.1.4 | [v0.1.4](https://github.com/ezmode-games/ctd/releases/tag/oblivion-remastered-v0.1.4) |
| The Elder Scrolls V: Skyrim | SKSE64 | Beta | v0.1.4 | [v0.1.4](https://github.com/ezmode-games/ctd/releases/tag/skyrim-v0.1.4) |
| Elden Ring | UE4SS | Wip | - | - |

## What It Captures

- Stack traces with module offsets
- Resolved function names (when PDB available)
- Load order at crash time
- Mod fingerprints (file hashes)
- Crash patterns across users

## Installation

Download from [Releases](https://github.com/ezmode-games/ctd/releases) or Nexus Mods.

Extract to your game's mod directory or install via Vortex/MO2.

## For Mod Creators

CTD helps you understand crashes affecting your users:

- **Crash visibility** - See reports where your mod is in the load order
- **Pattern detection** - Identify common crash signatures across users
- **Correlation analysis** - Find which mod combinations cause issues
- **Export data** - CSV export for your own analysis

For technical details on how CTD works internally, see [Architecture](docs/architecture.md).

### Providing Debug Symbols

Include your `.pdb` file alongside your DLL for resolved stack traces:

```
Data/SKSE/Plugins/
  MyMod.dll
  MyMod.pdb      <- Users get function names in crash reports
```

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

### API Endpoints

| Endpoint | Description |
|----------|-------------|
| `POST /crashes` | Submit a crash report |
| `GET /crashes/{id}` | Get crash report details |
| `POST /api-keys` | Create an API key |
| `GET /api-keys` | List your API keys |
| `GET /setup?key=<api_key>` | Download ctd.toml config |
| `GET /patterns` | List crash patterns |
| `GET /patterns/{id}` | Pattern details with mod correlations |
| `GET /calibration/metrics` | Prediction calibration metrics |
| `GET /docs` | Interactive API documentation |

### User Setup

Users need a `ctd.toml` config file with your server URL and their API key.

**Option 1: Download via API**
```bash
curl "https://your-server.com/setup?key=ctd_yourkey" -o ctd.toml
```

**Option 2: Manual creation**
```toml
# ctd.toml
[api]
url = "https://your-server.com"
api_key = "ctd_yourkey"
```

**Installation paths:**
| Game | Path |
|------|------|
| Skyrim SE | `Data/SKSE/Plugins/ctd.toml` |
| Fallout 4 | `Data/F4SE/Plugins/ctd.toml` |
| Fallout 3 | `Data/FOSE/Plugins/ctd.toml` |
| Fallout: New Vegas | `Data/NVSE/Plugins/ctd.toml` |
| Cyberpunk 2077 | `red4ext/plugins/ctd/ctd.toml` |

See [API Documentation](https://ctd.ezmode.games/docs) for full endpoint details.

## License

AGPL-3.0 - Modifications must be open sourced.
