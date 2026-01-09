//! Crash processing and report submission.

use ctd_core::api_client::ApiClient;
use ctd_core::crash_report::CreateCrashReport;
use tracing::{error, info};

use crate::ffi;
use crate::ffi::ExceptionData;
use crate::fingerprint::build_mod_list;

/// Game ID for Skyrim Special Edition.
const GAME_ID: &str = "skyrim-se";

/// Process a crash and submit it to the API.
pub fn process_crash(data: ExceptionData) {
    // Spawn a thread for submission to avoid blocking
    std::thread::spawn(move || {
        if let Err(e) = submit_crash_report(data) {
            error!("Failed to submit crash report: {}", e);
        }
    });
}

/// Build and submit a crash report.
fn submit_crash_report(
    data: ExceptionData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get load order from game and build mod list with file hashes
    let mods = ffi::get_load_order();
    let mod_names: Vec<String> = mods.into_iter().map(|m| m.name).collect();
    let mod_list = build_mod_list(mod_names);

    // Build the crash report
    let mut builder = CreateCrashReport::builder()
        .game_id(GAME_ID)
        .game_version(ffi::get_game_version())
        .stack_trace(&data.stack_trace)
        .exception_code(format!("0x{:08X}", data.code))
        .exception_address(format!("0x{:016X}", data.address))
        .load_order_v2(mod_list)
        .script_extender_version(ffi::get_skse_version())
        .crashed_now();

    // Add faulting module if available
    if !data.faulting_module.is_empty() {
        builder = builder.faulting_module(&data.faulting_module);
    }

    let report = builder.build()?;

    // Create runtime for async API call
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Submit the report
    let response = rt.block_on(async {
        let client = ApiClient::from_config().or_else(|_| ApiClient::with_defaults())?;

        client.submit_crash_report(&report).await
    })?;

    info!("Crash report submitted: {}", response.id);
    Ok(())
}
