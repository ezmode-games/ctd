//! PDB symbol resolution for enhanced stack traces.
//!
//! This module provides functionality to resolve raw stack trace addresses
//! into function names, file paths, and line numbers using PDB debug symbols.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use pdb::{FallibleIterator, PDB};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::{CtdError, Result};

/// A resolved stack frame with optional symbol information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFrame {
    /// Module name (e.g., "SkyrimSE.exe", "ctd-skyrim.dll").
    pub module: String,
    /// Raw offset within the module.
    pub offset: u64,
    /// Resolved function name, if available.
    pub function: Option<String>,
    /// Source file path, if available.
    pub file: Option<String>,
    /// Line number in source file, if available.
    pub line: Option<u32>,
}

impl ResolvedFrame {
    /// Creates a new unresolved frame with just module and offset.
    pub fn unresolved(module: impl Into<String>, offset: u64) -> Self {
        Self {
            module: module.into(),
            offset,
            function: None,
            file: None,
            line: None,
        }
    }

    /// Creates a resolved frame with function information.
    pub fn resolved(
        module: impl Into<String>,
        offset: u64,
        function: impl Into<String>,
        file: Option<String>,
        line: Option<u32>,
    ) -> Self {
        Self {
            module: module.into(),
            offset,
            function: Some(function.into()),
            file,
            line,
        }
    }

    /// Returns true if this frame has symbol information.
    pub fn is_resolved(&self) -> bool {
        self.function.is_some()
    }

    /// Formats the frame for display in a stack trace.
    pub fn format(&self) -> String {
        let base = format!("{}+0x{:X}", self.module, self.offset);
        match (&self.function, &self.file, self.line) {
            (Some(func), Some(file), Some(line)) => {
                format!("{} ({} at {}:{})", base, func, file, line)
            }
            (Some(func), Some(file), None) => {
                format!("{} ({} at {})", base, func, file)
            }
            (Some(func), None, _) => {
                format!("{} ({})", base, func)
            }
            (None, _, _) => base,
        }
    }
}

/// Cached symbol information for a single module.
struct ModuleSymbols {
    /// Function addresses sorted for binary search.
    /// Each entry is (rva, function_name).
    functions: Vec<(u32, String)>,
}

impl ModuleSymbols {
    /// Looks up the function containing the given RVA.
    fn lookup(&self, rva: u32) -> Option<&str> {
        // Binary search for the largest RVA <= target
        match self.functions.binary_search_by_key(&rva, |(addr, _)| *addr) {
            Ok(idx) => Some(&self.functions[idx].1),
            Err(0) => None, // RVA is before first function
            Err(idx) => Some(&self.functions[idx - 1].1),
        }
    }
}

/// Symbol resolver that parses PDB files and resolves addresses.
pub struct SymbolResolver {
    /// Cache directory for parsed symbols.
    cache_dir: PathBuf,
    /// Directories to search for PDB files.
    search_dirs: Vec<PathBuf>,
    /// Cached parsed symbols by module name (lowercase).
    modules: HashMap<String, ModuleSymbols>,
}

impl SymbolResolver {
    /// Creates a new symbol resolver with the given cache directory.
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            search_dirs: Vec::new(),
            modules: HashMap::new(),
        }
    }

    /// Adds a directory to search for PDB files.
    pub fn add_search_dir(&mut self, dir: impl Into<PathBuf>) {
        self.search_dirs.push(dir.into());
    }

    /// Loads a PDB file and caches its symbols.
    pub fn add_pdb(&mut self, pdb_path: &Path) -> Result<()> {
        let module_name = pdb_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| CtdError::Symbol("Invalid PDB path".into()))?
            .to_lowercase();

        debug!("Loading PDB for module: {}", module_name);

        let file = File::open(pdb_path)
            .map_err(|e| CtdError::Symbol(format!("Failed to open PDB: {}", e)))?;

        let mut pdb = PDB::open(BufReader::new(file))
            .map_err(|e| CtdError::Symbol(format!("Failed to parse PDB: {}", e)))?;

        let symbols = self.extract_symbols(&mut pdb)?;
        self.modules.insert(module_name, symbols);

        Ok(())
    }

    /// Extracts function symbols from a PDB file.
    fn extract_symbols<'s, S: pdb::Source<'s> + 's>(
        &self,
        pdb: &mut PDB<'s, S>,
    ) -> Result<ModuleSymbols> {
        let mut functions = Vec::new();

        // Get the global symbols
        let symbol_table = pdb
            .global_symbols()
            .map_err(|e| CtdError::Symbol(format!("Failed to read global symbols: {}", e)))?;

        let address_map = pdb
            .address_map()
            .map_err(|e| CtdError::Symbol(format!("Failed to read address map: {}", e)))?;

        let mut symbols = symbol_table.iter();
        while let Some(symbol) = symbols
            .next()
            .map_err(|e| CtdError::Symbol(format!("Failed to iterate symbols: {}", e)))?
        {
            if let Ok(pdb::SymbolData::Public(data)) = symbol.parse() {
                if let Some(rva) = data.offset.to_rva(&address_map) {
                    let name = data.name.to_string();
                    functions.push((rva.0, name.to_string()));
                }
            }
        }

        // Sort by address for binary search
        functions.sort_by_key(|(addr, _)| *addr);
        functions.dedup_by_key(|(addr, _)| *addr);

        debug!("Loaded {} symbols", functions.len());

        Ok(ModuleSymbols { functions })
    }

    /// Resolves a single frame, returning symbol info if available.
    pub fn resolve(&mut self, module_path: &Path, offset: u64) -> ResolvedFrame {
        let module_name = module_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let module_key = module_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Try to load PDB if not already cached
        if !self.modules.contains_key(&module_key) {
            if let Some(pdb_path) = self.find_pdb(&module_key) {
                if let Err(e) = self.add_pdb(&pdb_path) {
                    debug!("Failed to load PDB for {}: {}", module_key, e);
                }
            }
        }

        // Look up the symbol
        if let Some(symbols) = self.modules.get(&module_key) {
            if let Some(func_name) = symbols.lookup(offset as u32) {
                return ResolvedFrame::resolved(&module_name, offset, func_name, None, None);
            }
        }

        ResolvedFrame::unresolved(&module_name, offset)
    }

    /// Resolves multiple frames.
    pub fn resolve_all(&mut self, frames: &[(PathBuf, u64)]) -> Vec<ResolvedFrame> {
        frames
            .iter()
            .map(|(module, offset)| self.resolve(module, *offset))
            .collect()
    }

    /// Searches for a PDB file matching the given module name.
    fn find_pdb(&self, module_name: &str) -> Option<PathBuf> {
        let pdb_name = format!("{}.pdb", module_name);

        // Search in configured directories
        for dir in &self.search_dirs {
            let path = dir.join(&pdb_name);
            if path.exists() {
                debug!("Found PDB: {:?}", path);
                return Some(path);
            }
        }

        // Search in cache directory
        let cache_path = self.cache_dir.join(&pdb_name);
        if cache_path.exists() {
            return Some(cache_path);
        }

        debug!("PDB not found for module: {}", module_name);
        None
    }

    /// Discovers and loads all PDB files in the search directories.
    pub fn discover_pdbs(&mut self) -> usize {
        let mut count = 0;

        for dir in self.search_dirs.clone() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "pdb").unwrap_or(false) {
                        if let Err(e) = self.add_pdb(&path) {
                            warn!("Failed to load {:?}: {}", path, e);
                        } else {
                            count += 1;
                        }
                    }
                }
            }
        }

        debug!("Discovered {} PDB files", count);
        count
    }

    /// Returns the number of loaded modules.
    pub fn loaded_module_count(&self) -> usize {
        self.modules.len()
    }

    /// Returns the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

/// Formats a stack trace string with resolved symbols.
pub fn format_stack_trace(frames: &[ResolvedFrame]) -> String {
    frames
        .iter()
        .enumerate()
        .map(|(i, frame)| format!("[{}] {}", i, frame.format()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn unresolved_frame_formats_correctly() {
        let frame = ResolvedFrame::unresolved("test.dll", 0x1234);
        assert_eq!(frame.format(), "test.dll+0x1234");
        assert!(!frame.is_resolved());
    }

    #[test]
    fn resolved_frame_formats_with_function() {
        let frame = ResolvedFrame::resolved("test.dll", 0x1234, "MyFunction", None, None);
        assert_eq!(frame.format(), "test.dll+0x1234 (MyFunction)");
        assert!(frame.is_resolved());
    }

    #[test]
    fn resolved_frame_formats_with_file_and_line() {
        let frame = ResolvedFrame::resolved(
            "test.dll",
            0x1234,
            "MyFunction",
            Some("src/main.cpp".into()),
            Some(42),
        );
        assert_eq!(
            frame.format(),
            "test.dll+0x1234 (MyFunction at src/main.cpp:42)"
        );
    }

    #[test]
    fn resolver_creates_with_cache_dir() {
        let dir = tempdir().unwrap();
        let resolver = SymbolResolver::new(dir.path());
        assert_eq!(resolver.cache_dir(), dir.path());
        assert_eq!(resolver.loaded_module_count(), 0);
    }

    #[test]
    fn resolver_returns_unresolved_for_unknown_module() {
        let dir = tempdir().unwrap();
        let mut resolver = SymbolResolver::new(dir.path());
        let frame = resolver.resolve(Path::new("unknown.dll"), 0x1234);
        assert!(!frame.is_resolved());
        assert_eq!(frame.module, "unknown.dll");
        assert_eq!(frame.offset, 0x1234);
    }

    #[test]
    fn format_stack_trace_numbers_frames() {
        let frames = vec![
            ResolvedFrame::unresolved("a.dll", 0x100),
            ResolvedFrame::resolved("b.dll", 0x200, "Func", None, None),
        ];
        let trace = format_stack_trace(&frames);
        assert!(trace.contains("[0] a.dll+0x100"));
        assert!(trace.contains("[1] b.dll+0x200 (Func)"));
    }
}
