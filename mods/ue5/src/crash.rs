//! Crash handling using the crash-handler crate
//!
//! This module wraps the crash-handler crate to capture Windows SEH exceptions
//! and generate minidumps for crash reporting.

use std::sync::atomic::{AtomicBool, Ordering};

static HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Install the crash handler
pub fn install_handler() {
    if HANDLER_INSTALLED.swap(true, Ordering::SeqCst) {
        tracing::warn!("Crash handler already installed");
        return;
    }

    #[cfg(windows)]
    {
        use crash_handler::CrashHandler;

        let handler = CrashHandler::attach(unsafe {
            crash_handler::make_crash_event(move |crash_context| {
                handle_crash(crash_context);
            })
        });

        match handler {
            Ok(_) => {
                tracing::info!("Crash handler installed successfully");
            }
            Err(e) => {
                tracing::error!("Failed to install crash handler: {:?}", e);
                HANDLER_INSTALLED.store(false, Ordering::SeqCst);
            }
        }
    }

    #[cfg(not(windows))]
    {
        tracing::warn!("Crash handler only supported on Windows");
    }
}

/// Remove the crash handler
pub fn remove_handler() {
    if !HANDLER_INSTALLED.swap(false, Ordering::SeqCst) {
        return;
    }
    tracing::info!("Crash handler removed");
}

/// Handle a crash event
#[cfg(windows)]
fn handle_crash(crash_context: &crash_handler::CrashContext) {
    use crate::game_info;
    use ctd_core::crash_report::CreateCrashReport;
    use ctd_core::load_order::LoadOrder;

    // Get game info
    let game_info = match game_info() {
        Some(info) => info,
        None => {
            eprintln!("CTD: No game info available during crash");
            return;
        }
    };

    // Extract exception info
    let exception_code = crash_context
        .exception_code
        .map(|c| format!("0x{:08X}", c));

    // Build stack trace (basic for now)
    let stack_trace = format!(
        "Exception: {}\nGame: {} v{}\nUE: {}",
        exception_code.as_deref().unwrap_or("UNKNOWN"),
        game_info.game_name,
        game_info.game_version,
        game_info.ue_version
    );

    // Build crash report using ctd-core builder
    let report = match CreateCrashReport::builder()
        .game_id(&game_info.game_name)
        .game_version(&game_info.game_version)
        .script_extender_version(&game_info.ue_version)
        .stack_trace(stack_trace)
        .exception_code(exception_code.unwrap_or_default())
        .os_version(get_os_version().unwrap_or_default())
        .load_order(LoadOrder::new()) // TODO: Get from C++ side
        .crashed_now()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("CTD: Failed to build crash report: {:?}", e);
            return;
        }
    };

    // Try to submit synchronously (we're in a crash context, async won't work)
    if let Err(e) = submit_crash_report_sync(&report) {
        eprintln!("CTD: Failed to submit crash report: {:?}", e);
    }
}

#[cfg(windows)]
fn submit_crash_report_sync(
    report: &ctd_core::crash_report::CreateCrashReport,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use blocking reqwest since we're in a crash context
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client
        .post("https://ctd.ezmode.games/api/crashes")
        .json(report)
        .send()?;

    if response.status().is_success() {
        eprintln!("CTD: Crash report submitted successfully");
        Ok(())
    } else {
        Err(format!("Server returned {}", response.status()).into())
    }
}

fn get_os_version() -> Option<String> {
    #[cfg(windows)]
    {
        Some("Windows".to_string()) // TODO: Get actual version
    }
    #[cfg(not(windows))]
    {
        None
    }
}
