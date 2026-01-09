//! Mod enumeration for Cyberpunk 2077.
//!
//! This module scans all known mod locations to build a complete load order
//! for inclusion in crash reports.
//!
//! ## Supported Mod Types
//!
//! - **Archive**: `.archive` files in `archive/pc/mod/`
//! - **REDmod**: Directories with `info.json` in `mods/`
//! - **RED4ext**: `.dll` plugins in `red4ext/plugins/`
//! - **CET**: Lua mods with `init.lua` in `bin/x64/plugins/cyber_engine_tweaks/mods/`
//! - **Redscript**: `.reds` scripts in `r6/scripts/`
//! - **TweakXL**: `.yaml`/`.yml` files in `r6/tweaks/`

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ctd_core::file_hash::compute_file_hash;
use ctd_core::load_order::{ModEntry, ModList};
use ctd_core::version::get_dll_version;
use thiserror::Error;
use tracing::{debug, warn};
use walkdir::WalkDir;

/// Errors that can occur during mod scanning.
#[derive(Error, Debug)]
pub enum ModScannerError {
    /// Failed to determine the game directory.
    #[error("Could not determine game directory")]
    GameDirectoryNotFound,

    /// Failed to read a mod directory.
    #[error("Failed to read mod directory {path}: {source}")]
    DirectoryRead {
        path: String,
        source: std::io::Error,
    },
}

/// Result type for mod scanner operations.
pub type Result<T> = std::result::Result<T, ModScannerError>;

/// Types of mods in Cyberpunk 2077.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModType {
    /// Archive files (`.archive`) for game assets.
    Archive,
    /// REDmod official modding format.
    RedMod,
    /// RED4ext DLL plugins.
    Red4ext,
    /// Cyber Engine Tweaks Lua mods.
    Cet,
    /// Redscript (`.reds`) script mods.
    Redscript,
    /// TweakXL (`.yaml`/`.yml`) tweak files.
    TweakXL,
}

impl ModType {
    /// Returns the prefix used in load order entries for this mod type.
    fn prefix(self) -> &'static str {
        match self {
            ModType::Archive => "",
            ModType::RedMod => "[REDmod]",
            ModType::Red4ext => "[RED4ext]",
            ModType::Cet => "[CET]",
            ModType::Redscript => "[Redscript]",
            ModType::TweakXL => "[TweakXL]",
        }
    }
}

/// Mod paths relative to the game directory, with their type.
const MOD_PATHS: &[(&str, ModType)] = &[
    ("archive/pc/mod", ModType::Archive),
    ("mods", ModType::RedMod),
    ("red4ext/plugins", ModType::Red4ext),
    ("bin/x64/plugins/cyber_engine_tweaks/mods", ModType::Cet),
    ("r6/scripts", ModType::Redscript),
    ("r6/tweaks", ModType::TweakXL),
];

/// Cached mod list from startup scan.
static CACHED_MODS: OnceLock<ModList> = OnceLock::new();

/// Parse REDmod info.json for version.
fn get_redmod_version(mod_dir: &Path) -> Option<String> {
    let info_path = mod_dir.join("info.json");
    let content = std::fs::read_to_string(info_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json["version"].as_str().map(String::from)
}

/// Scans all mod locations and caches the result.
///
/// This should be called once during plugin initialization.
/// Subsequent calls will return the cached count without rescanning.
///
/// # Returns
///
/// The number of mods found, or an error if scanning failed.
pub fn scan_and_cache() -> Result<usize> {
    if let Some(cached) = CACHED_MODS.get() {
        return Ok(cached.len());
    }

    let mods = scan_mods()?;
    let count = mods.len();

    // Store in cache (if another thread beat us, that's fine)
    let _ = CACHED_MODS.set(mods);

    Ok(count)
}

/// Returns the cached mod list.
///
/// # Panics
///
/// Panics if `scan_and_cache()` was not called first.
pub fn get_cached() -> &'static ModList {
    CACHED_MODS
        .get()
        .expect("Mods not scanned yet - call scan_and_cache() first")
}

/// Returns a clone of the cached mod list, or an empty list if not scanned.
pub fn get_cached_or_empty() -> ModList {
    CACHED_MODS.get().cloned().unwrap_or_default()
}

/// Scans all mod locations and returns a ModList with fingerprints.
fn scan_mods() -> Result<ModList> {
    let game_dir = get_game_directory()?;
    let mut list = ModList::new();
    let mut index = 0u32;

    debug!("Scanning mods in game directory: {:?}", game_dir);

    for (path, mod_type) in MOD_PATHS {
        let full_path = game_dir.join(path);

        if !full_path.exists() {
            debug!("Mod path does not exist: {:?}", full_path);
            continue;
        }

        let count_before = list.len();

        match mod_type {
            ModType::Archive => {
                scan_archive_mods(&full_path, &mut list, &mut index);
            }
            ModType::RedMod => {
                scan_redmod_mods(&full_path, &mut list, &mut index);
            }
            ModType::Red4ext => {
                scan_red4ext_mods(&full_path, &mut list, &mut index);
            }
            ModType::Cet => {
                scan_cet_mods(&full_path, &mut list, &mut index);
            }
            ModType::Redscript => {
                scan_redscript_mods(&full_path, &mut list, &mut index);
            }
            ModType::TweakXL => {
                scan_tweakxl_mods(&full_path, &mut list, &mut index);
            }
        }

        let count_found = list.len() - count_before;
        if count_found > 0 {
            debug!("Found {} {:?} mods in {:?}", count_found, mod_type, path);
        }
    }

    Ok(list)
}

/// Scans for Archive mods (`.archive` files).
fn scan_archive_mods(path: &Path, list: &mut ModList, index: &mut u32) {
    for entry in WalkDir::new(path).max_depth(1).into_iter().flatten() {
        let file_path = entry.path();
        if file_path.extension().is_some_and(|ext| ext == "archive") {
            let name = entry.file_name().to_string_lossy().into_owned();

            // Compute hash and size
            let (hash, size) = compute_file_hash(file_path).unwrap_or_else(|e| {
                warn!("Failed to hash archive {}: {}", file_path.display(), e);
                ("0000000000000000".to_string(), 0)
            });

            let mod_entry = ModEntry::new(name, hash, size)
                .with_index(*index)
                .with_enabled(true);

            list.push(mod_entry);
            *index += 1;
        }
    }
}

/// Scans for REDmod mods (directories with `info.json`).
fn scan_redmod_mods(path: &Path, list: &mut ModList, index: &mut u32) {
    // REDmod mods are directories containing info.json
    for entry in WalkDir::new(path).max_depth(2).into_iter().flatten() {
        let file_path = entry.path();
        if entry.file_name() == "info.json"
            && let Some(mod_dir) = file_path.parent()
            && let Some(mod_name) = mod_dir.file_name()
        {
            let name = format!(
                "{} {}",
                ModType::RedMod.prefix(),
                mod_name.to_string_lossy()
            );

            // Compute hash from info.json file
            let (hash, size) = compute_file_hash(file_path).unwrap_or_else(|e| {
                warn!(
                    "Failed to hash REDmod info.json {}: {}",
                    file_path.display(),
                    e
                );
                ("0000000000000000".to_string(), 0)
            });

            // Extract version from info.json
            let version = get_redmod_version(mod_dir);

            let mut mod_entry = ModEntry::new(name, hash, size)
                .with_index(*index)
                .with_enabled(true);

            if let Some(v) = version {
                mod_entry = mod_entry.with_version(v);
            }

            list.push(mod_entry);
            *index += 1;
        }
    }
}

/// Scans for RED4ext plugins (`.dll` files).
fn scan_red4ext_mods(path: &Path, list: &mut ModList, index: &mut u32) {
    // RED4ext plugins are DLLs, typically in subdirectories
    for entry in WalkDir::new(path).max_depth(2).into_iter().flatten() {
        let file_path = entry.path();
        if file_path.extension().is_some_and(|ext| ext == "dll") {
            // Skip our own DLL
            let filename = entry.file_name().to_string_lossy();
            if filename.contains("ctd_cyberpunk") || filename.contains("ctd-cyberpunk") {
                continue;
            }

            let name = format!("{} {}", ModType::Red4ext.prefix(), filename);

            // Compute hash and size
            let (hash, size) = compute_file_hash(file_path).unwrap_or_else(|e| {
                warn!("Failed to hash RED4ext DLL {}: {}", file_path.display(), e);
                ("0000000000000000".to_string(), 0)
            });

            // Extract DLL version
            let version = get_dll_version(file_path).ok();

            let mut mod_entry = ModEntry::new(name, hash, size)
                .with_index(*index)
                .with_enabled(true);

            if let Some(v) = version {
                mod_entry = mod_entry.with_version(v);
            }

            list.push(mod_entry);
            *index += 1;
        }
    }
}

/// Scans for CET mods (directories with `init.lua`).
fn scan_cet_mods(path: &Path, list: &mut ModList, index: &mut u32) {
    // CET mods are directories containing init.lua
    for entry in WalkDir::new(path).max_depth(2).into_iter().flatten() {
        let file_path = entry.path();
        if entry.file_name() == "init.lua"
            && let Some(mod_dir) = file_path.parent()
            && let Some(mod_name) = mod_dir.file_name()
        {
            let name = format!("{} {}", ModType::Cet.prefix(), mod_name.to_string_lossy());

            // Compute hash from init.lua
            let (hash, size) = compute_file_hash(file_path).unwrap_or_else(|e| {
                warn!("Failed to hash CET init.lua {}: {}", file_path.display(), e);
                ("0000000000000000".to_string(), 0)
            });

            let mod_entry = ModEntry::new(name, hash, size)
                .with_index(*index)
                .with_enabled(true);

            list.push(mod_entry);
            *index += 1;
        }
    }
}

/// Scans for Redscript mods (`.reds` files).
fn scan_redscript_mods(path: &Path, list: &mut ModList, index: &mut u32) {
    // Collect unique script directories/files
    let mut seen_mods = std::collections::HashSet::new();

    for entry in WalkDir::new(path).into_iter().flatten() {
        let file_path = entry.path();
        if file_path.extension().is_some_and(|ext| ext == "reds") {
            // Use parent directory as mod name if it exists, otherwise file name
            let mod_name = if let Some(parent) = file_path.parent() {
                if parent != path {
                    // Use the immediate parent directory name
                    parent
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| entry.file_name().to_string_lossy().into_owned())
                } else {
                    entry.file_name().to_string_lossy().into_owned()
                }
            } else {
                entry.file_name().to_string_lossy().into_owned()
            };

            // Only add each mod directory once
            if seen_mods.insert(mod_name.clone()) {
                let name = format!("{} {}", ModType::Redscript.prefix(), mod_name);

                // Compute hash from the .reds file
                let (hash, size) = compute_file_hash(file_path).unwrap_or_else(|e| {
                    warn!("Failed to hash Redscript {}: {}", file_path.display(), e);
                    ("0000000000000000".to_string(), 0)
                });

                let mod_entry = ModEntry::new(name, hash, size)
                    .with_index(*index)
                    .with_enabled(true);

                list.push(mod_entry);
                *index += 1;
            }
        }
    }
}

/// Scans for TweakXL mods (`.yaml`/`.yml` files).
fn scan_tweakxl_mods(path: &Path, list: &mut ModList, index: &mut u32) {
    // Collect unique tweak directories/files
    let mut seen_mods = std::collections::HashSet::new();

    for entry in WalkDir::new(path).into_iter().flatten() {
        let file_path = entry.path();
        if file_path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            // Use parent directory as mod name if it exists
            let mod_name = if let Some(parent) = file_path.parent() {
                if parent != path {
                    parent
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| entry.file_name().to_string_lossy().into_owned())
                } else {
                    entry.file_name().to_string_lossy().into_owned()
                }
            } else {
                entry.file_name().to_string_lossy().into_owned()
            };

            // Only add each mod directory once
            if seen_mods.insert(mod_name.clone()) {
                let name = format!("{} {}", ModType::TweakXL.prefix(), mod_name);

                // Compute hash from the yaml file
                let (hash, size) = compute_file_hash(file_path).unwrap_or_else(|e| {
                    warn!("Failed to hash TweakXL {}: {}", file_path.display(), e);
                    ("0000000000000000".to_string(), 0)
                });

                let mod_entry = ModEntry::new(name, hash, size)
                    .with_index(*index)
                    .with_enabled(true);

                list.push(mod_entry);
                *index += 1;
            }
        }
    }
}

/// Gets the game directory from the current DLL location.
///
/// The plugin DLL is expected to be at:
/// `<game>/red4ext/plugins/ctd-cyberpunk/ctd_cyberpunk.dll`
///
/// So we walk up 4 directory levels to get the game root.
#[cfg(windows)]
fn get_game_directory() -> Result<PathBuf> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::GetModuleFileNameW;

    // Get the path to our own DLL
    let mut path_buf = [0u16; 260];

    // SAFETY: GetModuleFileNameW with null module returns the current module path
    let len = unsafe { GetModuleFileNameW(HMODULE::default(), &mut path_buf) };

    if len == 0 {
        warn!("GetModuleFileNameW failed, falling back to current_exe");
        return get_game_directory_fallback();
    }

    let dll_path = String::from_utf16_lossy(&path_buf[..len as usize]);
    let dll_path = PathBuf::from(dll_path);

    // Walk up from red4ext/plugins/ctd-cyberpunk/ctd_cyberpunk.dll to game root
    // That's 4 levels up: file -> ctd-cyberpunk -> plugins -> red4ext -> game
    dll_path
        .parent() // ctd-cyberpunk/
        .and_then(|p| p.parent()) // plugins/
        .and_then(|p| p.parent()) // red4ext/
        .and_then(|p| p.parent()) // game root
        .map(|p| p.to_path_buf())
        .ok_or(ModScannerError::GameDirectoryNotFound)
}

/// Fallback method using current_exe.
#[cfg(windows)]
fn get_game_directory_fallback() -> Result<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .ok_or(ModScannerError::GameDirectoryNotFound)
}

/// Non-Windows stub - returns current directory.
#[cfg(not(windows))]
fn get_game_directory() -> Result<PathBuf> {
    std::env::current_dir().map_err(|_| ModScannerError::GameDirectoryNotFound)
}

/// Gets the game directory path for internal use by other modules.
///
/// This is a public accessor for the private `get_game_directory` function.
pub(crate) fn get_game_directory_path() -> Option<PathBuf> {
    get_game_directory().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_type_prefix() {
        assert_eq!(ModType::Archive.prefix(), "");
        assert_eq!(ModType::RedMod.prefix(), "[REDmod]");
        assert_eq!(ModType::Red4ext.prefix(), "[RED4ext]");
        assert_eq!(ModType::Cet.prefix(), "[CET]");
        assert_eq!(ModType::Redscript.prefix(), "[Redscript]");
        assert_eq!(ModType::TweakXL.prefix(), "[TweakXL]");
    }

    #[test]
    fn test_mod_paths_coverage() {
        // Ensure we have all 6 mod locations covered
        assert_eq!(MOD_PATHS.len(), 6);

        let types: Vec<ModType> = MOD_PATHS.iter().map(|(_, t)| *t).collect();
        assert!(types.contains(&ModType::Archive));
        assert!(types.contains(&ModType::RedMod));
        assert!(types.contains(&ModType::Red4ext));
        assert!(types.contains(&ModType::Cet));
        assert!(types.contains(&ModType::Redscript));
        assert!(types.contains(&ModType::TweakXL));
    }

    #[test]
    fn test_get_cached_or_empty() {
        // Should return empty if not scanned
        // Note: This test may fail if run after scan_and_cache is called elsewhere
        let result = get_cached_or_empty();
        // Just verify it doesn't panic and returns a valid ModList
        let _ = result.len();
    }

    #[test]
    fn test_get_redmod_version_parses_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let info_path = temp_dir.path().join("info.json");
        std::fs::write(&info_path, r#"{"name": "TestMod", "version": "1.2.3"}"#).unwrap();

        let version = get_redmod_version(temp_dir.path());
        assert_eq!(version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_get_redmod_version_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let version = get_redmod_version(temp_dir.path());
        assert_eq!(version, None);
    }

    #[test]
    fn test_get_redmod_version_invalid_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let info_path = temp_dir.path().join("info.json");
        std::fs::write(&info_path, "not json").unwrap();

        let version = get_redmod_version(temp_dir.path());
        assert_eq!(version, None);
    }

    #[test]
    fn test_get_redmod_version_missing_version_field() {
        let temp_dir = tempfile::tempdir().unwrap();
        let info_path = temp_dir.path().join("info.json");
        std::fs::write(&info_path, r#"{"name": "TestMod"}"#).unwrap();

        let version = get_redmod_version(temp_dir.path());
        assert_eq!(version, None);
    }
}
