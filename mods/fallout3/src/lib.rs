//! FOSE plugin for Fallout 3 crash capture.
//!
//! This crate provides the Rust side of a hybrid C++/Rust FOSE plugin.
//! The C++ layer handles FOSE registration and VEH setup, while Rust
//! handles crash processing and API submission.

mod crash;
mod fingerprint;

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
        /// Plugin filename (e.g., "Fallout3.esm").
        name: String,
        /// Load order index.
        index: u8,
    }

    // Functions exported from Rust to C++
    extern "Rust" {
        /// Initialize the Rust side of the plugin.
        fn init();

        /// Called when FOSE's kDataLoaded message is received.
        fn on_data_loaded();

        /// Handle a crash from the VEH handler.
        fn handle_crash(data: ExceptionData);
    }

    // Functions imported from C++ to Rust
    unsafe extern "C++" {
        include!("cpp/bridge.hpp");

        /// Get the current load order from TESDataHandler.
        fn get_load_order() -> Vec<PluginInfo>;

        /// Get the Fallout 3 game version.
        fn get_game_version() -> String;

        /// Get the FOSE version string.
        fn get_fose_version() -> String;
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
