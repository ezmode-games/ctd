//! File fingerprinting for UE4SS mods.

use ctd_core::file_hash::compute_file_hash;
use ctd_core::load_order::{ModEntry, ModList};
use ctd_core::version::get_dll_version;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Check if mod is enabled via enabled.txt.
pub fn is_mod_enabled(mod_dir: &Path) -> bool {
    let enabled_path = mod_dir.join("enabled.txt");
    if !enabled_path.exists() {
        return false;
    }

    std::fs::read_to_string(enabled_path)
        .map(|s| s.trim() == "1")
        .unwrap_or(false)
}

/// Scan the Mods directory and build ModList with fingerprints.
pub fn scan_ue4ss_mods(game_dir: &Path) -> ModList {
    let mods_dir = game_dir.join("Mods");
    let mut list = ModList::new();
    let mut index = 0u32;

    if !mods_dir.exists() {
        return list;
    }

    // Each subdirectory in Mods/ is a mod
    for entry in WalkDir::new(&mods_dir).max_depth(1).into_iter().flatten() {
        if !entry.file_type().is_dir() || entry.path() == mods_dir {
            continue;
        }

        let mod_dir = entry.path();
        let mod_name = mod_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip common non-mod directories
        if mod_name == "shared" || mod_name == "Keybinds" {
            continue;
        }

        // Check if enabled
        let enabled = is_mod_enabled(mod_dir);

        // Find and hash the main DLL or Lua
        let dll_path = mod_dir.join("dlls").join("main.dll");
        let lua_path = mod_dir.join("Scripts").join("main.lua");

        let (hash, size, version) = if dll_path.exists() {
            let (h, s) =
                compute_file_hash(&dll_path).unwrap_or(("0000000000000000".to_string(), 0));
            let v = get_dll_version(&dll_path).ok();
            (h, s, v)
        } else if lua_path.exists() {
            let (h, s) =
                compute_file_hash(&lua_path).unwrap_or(("0000000000000000".to_string(), 0));
            (h, s, None)
        } else {
            ("0000000000000000".to_string(), 0, None)
        };

        let mut mod_entry = ModEntry::new(&mod_name, hash, size)
            .with_index(index)
            .with_enabled(enabled);

        if let Some(v) = version {
            mod_entry = mod_entry.with_version(v);
        }

        list.push(mod_entry);
        index += 1;
    }

    list
}

/// Get game directory from DLL location.
#[cfg(windows)]
pub fn get_game_directory() -> Option<PathBuf> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

    let mut path_buf = [0u16; 260];
    let len = unsafe { GetModuleFileNameW(HMODULE::default(), &mut path_buf) };

    if len == 0 {
        return None;
    }

    let dll_path = String::from_utf16_lossy(&path_buf[..len as usize]);
    let dll_path = PathBuf::from(dll_path);

    // Walk up from Mods/CTD/dlls/main.dll to game root
    dll_path
        .parent() // dlls/
        .and_then(|p| p.parent()) // CTD/
        .and_then(|p| p.parent()) // Mods/
        .and_then(|p| p.parent()) // game root
        .map(|p| p.to_path_buf())
}

#[cfg(not(windows))]
pub fn get_game_directory() -> Option<PathBuf> {
    None
}
