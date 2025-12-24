//! SKSE64 plugin for Skyrim Special Edition crash capture.
//!
//! This crate provides the Rust side of a hybrid C++/Rust SKSE plugin.
//! The C++ layer handles SKSE registration and VEH setup, while Rust
//! handles crash processing and API submission.

mod crash;

use ctd_core::load_order::{LoadOrder, LoadOrderEntry};
use tracing::info;

/// CXX bridge between C++ and Rust.
#[cxx::bridge(namespace = "ctd")]
mod ffi {
    /// Exception data passed from C++ VEH handler.
    #[derive(Debug, Clone)]
    struct ExceptionData {
        /// Windows exception code (e.g., 0xC0000005).
        code: u32,
        /// Address where the exception occurred.
        address: u64,
        /// Formatted stack trace with module offsets.
        stack_trace: String,
        /// Module name where the crash occurred (if known).
        faulting_module: String,
    }

    /// Mod information from TESDataHandler.
    #[derive(Debug, Clone)]
    struct ModInfo {
        /// Mod filename (e.g., "Skyrim.esm").
        name: String,
        /// Load order index.
        index: u8,
        /// Whether this is a light plugin (ESL).
        is_light: bool,
    }

    // Functions exported from Rust to C++
    extern "Rust" {
        /// Initialize the Rust side of the plugin.
        fn init();

        /// Called when SKSE's kDataLoaded message is received.
        fn on_data_loaded();

        /// Handle a crash from the VEH handler.
        fn handle_crash(data: ExceptionData);
    }

    // Functions imported from C++ to Rust
    unsafe extern "C++" {
        include!("ctd-skyrim/cpp/bridge.hpp");

        /// Get the current load order from TESDataHandler.
        fn get_load_order() -> Vec<ModInfo>;

        /// Get the Skyrim game version.
        fn get_game_version() -> String;

        /// Get the SKSE version string.
        fn get_skse_version() -> String;
    }
}

/// Initialize the Rust side of the plugin.
pub fn init() {
    info!("CTD Crash Reporter initializing");
}

/// Called when game data is loaded.
pub fn on_data_loaded() {
    info!("Game data loaded, load order available");
}

/// Handle a crash from the VEH handler.
pub fn handle_crash(data: ffi::ExceptionData) {
    info!(
        "Crash captured: 0x{:08X} at 0x{:016X}",
        data.code, data.address
    );

    // Delegate to crash module
    crash::process_crash(data);
}

/// Convert FFI mod info to ctd-core LoadOrder.
pub(crate) fn build_load_order(mods: Vec<ffi::ModInfo>) -> LoadOrder {
    let mut load_order = LoadOrder::new();

    for (index, mod_info) in mods.into_iter().enumerate() {
        // Create entry with full details
        // Note: is_light flag is informational but not stored in the simple LoadOrderEntry
        let entry = LoadOrderEntry::full(mod_info.name, true, index as u32);
        load_order.push(entry);
    }

    load_order
}
