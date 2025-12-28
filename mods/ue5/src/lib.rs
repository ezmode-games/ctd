//! CTD Crash Reporter for Unreal Engine 5 games
//!
//! This module provides crash capture and reporting for UE5 games via UE4SS.
//! It uses the crash-handler crate for Windows SEH exception handling and
//! integrates with ctd-core for report submission.

mod crash;

use std::sync::OnceLock;

/// Game information provided by the UE4SS mod
static GAME_INFO: OnceLock<GameInfo> = OnceLock::new();

/// Runtime game information
pub struct GameInfo {
    pub game_name: String,
    pub game_version: String,
    pub ue_version: String,
}

#[cxx::bridge]
mod ffi {
    /// Plugin info for load order
    struct PluginInfo {
        name: String,
        index: u32,
        is_light: bool,
    }

    extern "Rust" {
        /// Initialize the crash reporter
        fn init(game_name: &str, game_version: &str, ue_version: &str);

        /// Called when game data is loaded
        fn on_data_loaded();

        /// Shutdown the crash reporter
        fn shutdown();
    }

    unsafe extern "C++" {
        include!("ctd-ue5/cpp/bridge.hpp");

        /// Get the load order from UE4SS (game-specific)
        fn get_load_order() -> Vec<PluginInfo>;

        /// Get game-specific version info
        fn get_game_version() -> String;
    }
}

/// Initialize the crash reporter with game info
pub fn init(game_name: &str, game_version: &str, ue_version: &str) {
    // Store game info
    let _ = GAME_INFO.set(GameInfo {
        game_name: game_name.to_string(),
        game_version: game_version.to_string(),
        ue_version: ue_version.to_string(),
    });

    // Install crash handler
    crash::install_handler();

    tracing::info!(
        "CTD initialized for {} v{} (UE {})",
        game_name,
        game_version,
        ue_version
    );
}

/// Called when game data is loaded
pub fn on_data_loaded() {
    tracing::info!("Game data loaded, crash reporter active");
}

/// Shutdown and cleanup
pub fn shutdown() {
    crash::remove_handler();
    tracing::info!("CTD shutdown");
}

/// Get the current game info
pub fn game_info() -> Option<&'static GameInfo> {
    GAME_INFO.get()
}
