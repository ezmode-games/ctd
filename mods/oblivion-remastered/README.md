# CTD Crash Reporter - Oblivion Remastered

UE4SS C++ mod that provides automatic crash reporting for Oblivion Remastered.

## Requirements

### Build Requirements
- Visual Studio 2022 with C++ workload
- CMake 3.22+
- Rust toolchain (stable)
- Epic Games account linked to GitHub (for Unreal Engine source access)
- UE4SS SDK (fetched automatically by CMake)

### Runtime Requirements
- [UE4SS](https://github.com/UE4SS-RE/RE-UE4SS) installed in Oblivion Remastered

## Building

1. **Link your Epic Games account to GitHub**
   - Required for UE4SS SDK which needs Unreal Engine headers
   - See: https://www.unrealengine.com/en-US/ue-on-github

2. **Configure and build**
   ```powershell
   cmake -B build -G "Visual Studio 17 2022" -A x64
   cmake --build build --config Release
   ```

3. **Output**
   - `build/CTDCrashReporter/dlls/main.dll`

## Installation

1. Install UE4SS in Oblivion Remastered
2. Copy the `CTDCrashReporter` folder to:
   ```
   OblivionRemastered/Binaries/Win64/ue4ss/Mods/CTDCrashReporter/
   ```
3. Enable the mod in `ue4ss/Mods/mods.txt`:
   ```
   CTDCrashReporter : 1
   ```

## How It Works

1. On game startup, UE4SS loads the CTDCrashReporter mod
2. The mod installs a Windows SEH exception handler via the Rust crash-handler crate
3. When a crash occurs:
   - Exception data is captured (code, address, stack trace)
   - Load order is collected (esp/esm plugins)
   - Report is submitted to https://ctd.ezmode.games

## Architecture

```
CTDCrashReporter (UE4SS C++ Mod)
    └── ctd-ue5 (Rust staticlib)
            └── ctd-core (shared Rust library)
                    └── crash-handler (Embark Studios crate)
```

The C++ mod provides the UE4SS integration, while the Rust libraries handle
crash capture and API submission.
