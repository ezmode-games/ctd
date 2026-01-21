# CTD Architecture

This document explains how CTD captures crashes, resolves symbols, and submits reports.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Game Process                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Script       │    │ CTD Plugin   │    │ Game + Mods      │  │
│  │ Extender     │───▶│ (DLL)        │───▶│                  │  │
│  │ SKSE/F4SE/   │    │              │    │                  │  │
│  │ RED4ext      │    └──────┬───────┘    └──────────────────┘  │
│  └──────────────┘           │                                   │
│                             │ VEH Handler                       │
│                             ▼                                   │
│                    ┌────────────────┐                          │
│                    │ Crash Capture  │                          │
│                    │ - Stack walk   │                          │
│                    │ - Symbol res.  │                          │
│                    │ - Load order   │                          │
│                    └───────┬────────┘                          │
└────────────────────────────┼────────────────────────────────────┘
                             │ HTTPS
                             ▼
                    ┌────────────────┐
                    │  CTD API       │
                    │  (Cloudflare)  │
                    └────────────────┘
```

## Crash Capture

### Vectored Exception Handler (VEH)

CTD installs a Vectored Exception Handler at plugin load time. VEH runs before Structured Exception Handling (SEH), giving us first chance at exceptions.

```cpp
// Simplified - actual implementation in mods/*/cpp/veh.cpp
LONG CALLBACK VectoredHandler(EXCEPTION_POINTERS* info) {
    if (IsFatalException(info->ExceptionRecord->ExceptionCode)) {
        CaptureCrash(info);
    }
    return EXCEPTION_CONTINUE_SEARCH;  // Let other handlers run
}
```

**Why VEH over SEH?**
- Runs before frame-based SEH handlers
- Can't be bypassed by mod code
- Works across all threads

### Stack Walking

We use `RtlCaptureStackBackTrace` on Windows to walk the stack:

```cpp
void CaptureStack(CONTEXT* ctx, std::vector<StackFrame>& frames) {
    STACKFRAME64 frame = {};
    frame.AddrPC.Offset = ctx->Rip;
    frame.AddrStack.Offset = ctx->Rsp;
    frame.AddrFrame.Offset = ctx->Rbp;

    while (StackWalk64(...)) {
        frames.push_back({
            .module = GetModuleName(frame.AddrPC.Offset),
            .offset = frame.AddrPC.Offset - moduleBase
        });
    }
}
```

Each frame captures:
- **Module name** - Which DLL/EXE contains this address
- **Offset** - Address relative to module base (survives ASLR)

## Symbol Resolution

### PDB Parsing

When a `.pdb` file is available, CTD resolves raw offsets to function names:

```
Before: SkyrimSE.exe+0x12A4B0
After:  SkyrimSE.exe+0x12A4B0 (Actor::UpdateAnimation)
```

**Implementation** (`lib/ctd-core/src/symbols.rs`):

```rust
pub struct SymbolResolver {
    modules: HashMap<String, ModuleSymbols>,
}

impl SymbolResolver {
    pub fn resolve(&mut self, module: &Path, offset: u64) -> ResolvedFrame {
        // 1. Find PDB matching module name
        // 2. Parse PDB if not cached
        // 3. Binary search for function containing offset
        // 4. Return function name or just offset if not found
    }
}
```

### PDB Discovery

CTD searches for PDBs in:
1. Same directory as the DLL
2. Configured `search_dirs` in `ctd.toml`
3. Symbol cache directory

**For mod authors**: Place your `.pdb` next to your `.dll` and users automatically get resolved stack traces.

### Symbol Cache

Parsed symbol tables are cached to avoid re-parsing PDBs on every crash. The cache stores a sorted list of `(address, function_name)` pairs for O(log n) lookup.

## Load Order Capture

### Bethesda Games (Skyrim, Fallout)

CTD reads the active plugin list from:
- `plugins.txt` (user plugins)
- `loadorder.txt` (full order)
- Data directory scanning (for ESL files)

```rust
pub struct ModEntry {
    pub name: String,
    pub file_hash: String,   // SHA-256 of first 64KB
    pub file_size: u64,
    pub version: Option<String>,
    pub index: Option<u32>,
}
```

**File Hashing**: We hash the first 64KB of each plugin file. This identifies specific mod versions without hashing entire large files.

### Cyberpunk 2077

RED4ext mods are discovered by scanning:
- `red4ext/plugins/`
- `r6/scripts/` (REDscript)
- `archive/pc/mod/` (archives)

### UE4SS Games

Unreal Engine games use PAK files. CTD scans:
- `Mods/` directory
- `~mods/` directory
- PAK mount points

## Crash Deduplication

Crashes are grouped by a hash of:
1. Exception code
2. Faulting module
3. Top N stack frames (module + offset)

This identifies "same crash" across different users even with different load orders.

```
Hash = SHA256(
    exception_code +
    faulting_module +
    frame[0].module + frame[0].offset +
    frame[1].module + frame[1].offset +
    ...
)
```

## API Submission

### Crash Report Schema

```json
{
  "schemaVersion": 2,
  "gameId": "skyrim-se",
  "stackTrace": "[0] SkyrimSE.exe+0x12A4B0 (Actor::Update)\n...",
  "crashHash": "a1b2c3...",
  "exceptionCode": "0xC0000005",
  "faultingModule": "SkyrimSE.exe",
  "gameVersion": "1.6.1170",
  "loadOrderJson": "[{\"name\":\"Skyrim.esm\",...}]",
  "pluginCount": 255,
  "crashedAt": 1704067200000
}
```

### Network Flow

1. VEH captures crash
2. Symbol resolution (blocking)
3. Load order snapshot
4. Build JSON payload
5. POST to `https://ctd.ezmode.games/api/crash-reports`
6. Receive report ID + share token

All network calls use HTTPS with certificate pinning.

## Configuration

`ctd.toml` in plugin directory:

```toml
[api]
url = "https://ctd.ezmode.games"
timeout_secs = 30

[symbols]
enabled = true
cache_dir = "~/.ctd/symcache"
search_dirs = ["Data/SKSE/Plugins"]
```

## Repository Structure

```
ctd/
├── lib/
│   └── ctd-core/           # Rust core library
│       ├── api_client.rs   # HTTP client
│       ├── config.rs       # TOML config
│       ├── crash_report.rs # Report builder
│       ├── load_order.rs   # Plugin parsing
│       ├── symbols.rs      # PDB resolution
│       └── file_hash.rs    # Mod fingerprinting
├── mods/
│   ├── skyrim/            # SKSE64 plugin
│   │   ├── cpp/           # C++ VEH + SKSE hooks
│   │   ├── src/           # Rust FFI bridge
│   │   └── CMakeLists.txt
│   ├── fallout4/          # F4SE plugin
│   ├── cyberpunk/         # RED4ext plugin (pure Rust)
│   └── oblivion-remastered/  # UE4SS plugin
├── api/                   # Hono API (TypeScript)
└── scripts/               # Build/package scripts
```

## Build System

### Rust + C++ Hybrid (Skyrim, Fallout)

These games require native script extender integration:

```
┌─────────────────┐     ┌──────────────────┐
│ Rust Library    │────▶│ C++ Plugin       │
│ (staticlib)     │ FFI │ (SKSE/F4SE DLL)  │
│ - ctd-core      │     │ - VEH handler    │
│ - API client    │     │ - Game hooks     │
└─────────────────┘     └──────────────────┘
```

Build with: `.\scripts\build-mod.ps1 -Mod skyrim`

### Pure Rust (Cyberpunk)

RED4ext supports Rust directly via `red4ext-rs`:

```
┌─────────────────┐
│ Rust Plugin     │
│ (cdylib)        │
│ - ctd-core      │
│ - red4ext-rs    │
└─────────────────┘
```

Build with: `cargo build -p ctd-cyberpunk --release`

## Privacy

- **No PII collected** - No usernames, paths, or identifiers
- **Load order only** - Mod names, not file paths
- **Optional account linking** - Anonymous by default
- **90-day retention** - Anonymous reports auto-delete
