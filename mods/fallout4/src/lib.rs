//! F4SE plugin for Fallout 4 crash capture.
//!
//! This crate provides the Rust side of a hybrid C++/Rust F4SE plugin.
//! The C++ layer handles F4SE registration and VEH setup, while Rust
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

    /// Plugin information from TESDataHandler.
    #[derive(Debug, Clone)]
    struct PluginInfo {
        /// Plugin filename (e.g., "Fallout4.esm").
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

        /// Called when F4SE's kDataLoaded message is received.
        fn on_data_loaded();

        /// Handle a crash from the VEH handler.
        fn handle_crash(data: ExceptionData);
    }

    // Functions imported from C++ to Rust
    unsafe extern "C++" {
        include!("cpp/bridge.hpp");

        /// Get the current load order from TESDataHandler.
        fn get_load_order() -> Vec<PluginInfo>;

        /// Get the Fallout 4 game version.
        fn get_game_version() -> String;

        /// Get the F4SE version string.
        fn get_f4se_version() -> String;
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

/// Convert FFI plugin info to ctd-core LoadOrder.
pub(crate) fn build_load_order(plugins: Vec<ffi::PluginInfo>) -> LoadOrder {
    let mut load_order = LoadOrder::new();

    for (index, plugin_info) in plugins.into_iter().enumerate() {
        // Create entry with full details
        // Note: is_light flag is informational but not stored in the simple LoadOrderEntry
        let entry = LoadOrderEntry::full(plugin_info.name, true, index as u32);
        load_order.push(entry);
    }

    load_order
}
