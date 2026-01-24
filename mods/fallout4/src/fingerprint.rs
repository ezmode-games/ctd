//! File fingerprinting for Fallout 4 mods.

use ctd_core::fingerprint::{file_size, fingerprint_file};
use ctd_core::load_order::{ModEntry, ModList};
use std::path::PathBuf;

/// Get the game's Data directory from the DLL location.
#[cfg(windows)]
pub fn get_data_dir() -> Option<PathBuf> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

    let mut path_buf = [0u16; 260];
    let len = unsafe { GetModuleFileNameW(HMODULE::default(), &mut path_buf) };

    if len == 0 {
        return None;
    }

    let dll_path = String::from_utf16_lossy(&path_buf[..len as usize]);
    let dll_path = PathBuf::from(dll_path);

    // Walk up from Data/F4SE/Plugins/ctd_fallout4.dll to Data/
    dll_path
        .parent() // Plugins/
        .and_then(|p| p.parent()) // F4SE/
        .and_then(|p| p.parent()) // Data/
        .map(|p| p.to_path_buf())
}

#[cfg(not(windows))]
pub fn get_data_dir() -> Option<PathBuf> {
    None
}

/// Build ModList with hashes for all loaded mods.
pub fn build_mod_list(mod_names: Vec<String>) -> ModList {
    let data_dir = get_data_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut list = ModList::new();

    for (index, name) in mod_names.into_iter().enumerate() {
        let path = data_dir.join(&name);

        let hash = fingerprint_file(&path).unwrap_or_else(|_| "0000000000000000".to_string());
        let size = file_size(&path).unwrap_or(0);

        // ESP/ESM files don't have version info, just use hash
        let entry = ModEntry::new(&name, hash, size)
            .with_index(index as u32)
            .with_enabled(true);

        list.push(entry);
    }

    list
}
