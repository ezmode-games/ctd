//! CTD Crash Reporter for Cyberpunk 2077
//!
//! This RED4ext plugin captures crashes via Windows Vectored Exception Handler
//! and submits crash reports to the CTD backend API.
//!
//! ## Installation
//!
//! Place the compiled `ctd_cyberpunk.dll` in:
//! `<game>/red4ext/plugins/ctd-cyberpunk/ctd_cyberpunk.dll`
//!
//! ## Features
//!
//! - Captures crashes with stack traces and exception information
//! - Enumerates all installed mods from multiple sources
//! - Fire-and-forget submission (never blocks the game)
//! - Graceful failure (never crashes the crash handler)
//!
//! ## Platform Support
//!
//! This crate is designed for Windows (x86_64-pc-windows-msvc) but can be
//! compiled on other platforms for development purposes. Non-Windows builds
//! produce a stub library.

// Allow dead code on non-Windows platforms where the actual implementation isn't used
#![cfg_attr(not(windows), allow(dead_code, unused_imports))]

pub mod crash_handler;
pub mod mod_scanner;
pub mod report;

#[cfg(windows)]
use red4ext_rs::{
    Exportable, Plugin, PluginOps, SemVer, U16CStr, export_plugin_symbols, exports, wcstr,
};

#[cfg(windows)]
use tracing::{error, info};

/// The CTD Crash Reporter plugin for RED4ext.
#[cfg(windows)]
pub struct CtdReporter;

#[cfg(windows)]
impl Plugin for CtdReporter {
    const AUTHOR: &'static U16CStr = wcstr!("ezmode-games");
    const NAME: &'static U16CStr = wcstr!("CTD Crash Reporter");
    const VERSION: SemVer = SemVer::new(0, 1, 0);

    /// Called when the plugin is loaded.
    ///
    /// Initializes the VEH handler and caches the mod list.
    fn on_init(env: &red4ext_rs::SdkEnv) {
        // Initialize tracing to RED4ext logging
        init_logging(env);

        info!("CTD Crash Reporter initializing...");

        // Register VEH handler for crash capture
        if let Err(e) = crash_handler::register() {
            error!("Failed to register crash handler: {}", e);
        } else {
            info!("VEH crash handler registered");
        }

        // Cache mod list on startup (filesystem scan is expensive)
        match mod_scanner::scan_and_cache() {
            Ok(count) => info!("Cached {} mods from all sources", count),
            Err(e) => error!("Failed to scan mods: {}", e),
        }

        info!("CTD Crash Reporter initialized successfully");
    }

    fn exports() -> impl Exportable {
        // No exported functions needed - we only handle crashes
        exports![]
    }
}

/// Initialize tracing to output to RED4ext's logging system.
#[cfg(windows)]
fn init_logging(_env: &red4ext_rs::SdkEnv) {
    // red4ext-rs with the "log" feature handles this automatically
    // via the tracing subscriber integration
}

#[cfg(windows)]
export_plugin_symbols!(CtdReporter);

// ===========================================================================
// Non-Windows stubs for development/testing on other platforms
// ===========================================================================

/// Stub initialization function for non-Windows platforms.
///
/// This is only used for development testing - the actual plugin
/// functionality only works on Windows.
#[cfg(not(windows))]
pub fn init() {
    eprintln!("ctd-cyberpunk: This plugin only works on Windows");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_scanner_types() {
        // Basic smoke test that types are correct
        use mod_scanner::ModType;
        let _archive = ModType::Archive;
        let _redmod = ModType::RedMod;
    }
}
