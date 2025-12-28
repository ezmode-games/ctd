# CTD UE5 Development Handoff

## Current State
- **Branch**: `feature/ctd-ue5`
- **PR**: https://github.com/ezmode-games/ctd/pull/30
- **CI Status**: All failing (lint, test, build)

## What Was Built
Created scaffold for UE5/UE4SS crash reporter at `mods/ue5/`:
- `Cargo.toml` - Rust staticlib with crash-handler, cxx, ctd-core deps
- `src/lib.rs` - CXX bridge definition, init/shutdown functions
- `src/crash.rs` - crash-handler integration, uses blocking reqwest
- `cpp/bridge.hpp` - C++ header for CXX bridge
- `cpp/bridge.cpp` - Stub implementations (to be overridden by game-specific code)
- `build.rs` - CXX build script

## Known Issues to Fix
1. **Formatting** - Need to run `cargo fmt` (couldn't in WSL)
2. **Compilation errors** - Likely issues with crash-handler API usage
3. **Test failures** - Probably same compilation errors

## Next Steps
1. Run `cargo fmt` to fix formatting
2. Run `cargo check -p ctd-ue5` to see compilation errors
3. Fix crash-handler API usage (check their docs)
4. Add nexus.toml for the ue5 mod (or skip for now since it's generic)
5. Build UE4SS C++ mod wrapper for Oblivion Remastered

## Architecture Notes
- ctd-ue5 is a **generic UE5 crash reporter**
- Game-specific code (Oblivion Remastered) will override C++ bridge functions
- Uses crash-handler crate from Embark Studios (MIT/Apache-2.0)
- Uses blocking HTTP because async doesn't work in crash context

## Research Done
- UE4SS uses Lua and C++ mods, no native Rust support
- CXX bridge works same as SKSE/F4SE
- crash-handler provides Windows SEH exception handling
- Oblivion Remastered is UE5 remake with dual Gamebryo/UE5 architecture
- OBSE64 exists but is just a plugin loader (early stage)

## Commands to Run
```powershell
# Fix formatting
cargo fmt --all

# Check compilation
cargo check -p ctd-ue5

# Run all tests
cargo test --all

# Build for Windows
cargo build -p ctd-ue5 --target x86_64-pc-windows-msvc
```

## Related PRs
- PR #29 (merged): Added Fallout 4 F4SE plugin
- PR #30 (current): UE5 scaffold - needs fixes

## Release Status
- v0.1.1 released with Skyrim, Cyberpunk, Fallout4 plugins
- All three zips on GitHub Releases with FOMOD installers
