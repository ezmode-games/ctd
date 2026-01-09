//! Crash processing and report submission.

use ctd_core::api_client::ApiClient;
use ctd_core::crash_report::CreateCrashReport;
use tracing::{error, info};

use crate::ffi::ExceptionData;
use crate::{build_load_order, ffi};

/// Game ID for Fallout 3.
const GAME_ID: &str = "fallout3";

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
    // Get load order from game
    let plugins = ffi::get_load_order();
    let load_order = build_load_order(plugins);

    // Build the crash report
    let mut builder = CreateCrashReport::builder()
        .game_id(GAME_ID)
        .game_version(ffi::get_game_version())
        .stack_trace(&data.stack_trace)
        .exception_code(format!("0x{:08X}", data.code))
        .exception_address(format!("0x{:016X}", data.address))
        .load_order(load_order)
        .script_extender_version(ffi::get_fose_version())
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
