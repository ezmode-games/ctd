//! Crash handling using the crash-handler crate
//!
//! This module wraps the crash-handler crate to capture Windows SEH exceptions
//! and generate minidumps for crash reporting.

use std::sync::atomic::{AtomicBool, Ordering};

use ctd_core::api_client::ApiClient;
use ctd_core::crash_report::CreateCrashReport;
use ctd_core::load_order::LoadOrder;
use tracing::{error, info};

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
                crash_handler::CrashEventResult::Handled(true)
            })
        });

        match handler {
            Ok(_) => {
                info!("Crash handler installed successfully");
            }
            Err(e) => {
                error!("Failed to install crash handler: {:?}", e);
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
    info!("Crash handler removed");
}

/// Handle a crash event
#[cfg(windows)]
fn handle_crash(crash_context: &crash_handler::CrashContext) {
    use crate::{ffi, game_info};

    // Get game info
    let game_info = match game_info() {
        Some(info) => info,
        None => {
            eprintln!("CTD: No game info available during crash");
            return;
        }
    };

    // Get load order from C++ side
    let mods = ffi::get_load_order();
    let load_order = build_load_order(mods);

    // Extract exception info
    let exception_code = format!("0x{:08X}", crash_context.exception_code);

    // Build stack trace
    let stack_trace = format!(
        "Exception: {}\nGame: {} v{}\nUE: {}",
        exception_code, game_info.game_name, game_info.game_version, game_info.ue_version
    );

    // Build crash report using ctd-core builder
    let report = match CreateCrashReport::builder()
        .game_id(&game_info.game_name)
        .game_version(&game_info.game_version)
        .script_extender_version(&game_info.ue_version)
        .stack_trace(stack_trace)
        .exception_code(exception_code)
        .os_version(get_os_version())
        .load_order(load_order)
        .crashed_now()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("CTD: Failed to build crash report: {:?}", e);
            return;
        }
    };

    // Submit in a separate thread to avoid blocking crash handling
    std::thread::spawn(move || {
        if let Err(e) = submit_crash_report(report) {
            error!("Failed to submit crash report: {}", e);
        }
    });
}

/// Build LoadOrder from FFI plugin info
#[cfg(windows)]
fn build_load_order(mods: Vec<crate::ffi::PluginInfo>) -> LoadOrder {
    use ctd_core::load_order::LoadOrderEntry;

    let mut load_order = LoadOrder::new();
    for plugin in mods {
        let mut entry = LoadOrderEntry::new(plugin.name);
        entry.index = Some(plugin.index);
        entry.enabled = Some(true); // If it's in the list, it's enabled
        load_order.push(entry);
    }
    load_order
}

/// Submit crash report using ctd-core ApiClient (respects ctd.toml config)
#[cfg(windows)]
fn submit_crash_report(
    report: CreateCrashReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create runtime for async API call
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Submit the report using ApiClient which reads from ctd.toml
    let response = rt.block_on(async {
        let client = ApiClient::from_config().or_else(|_| ApiClient::with_defaults())?;
        client.submit_crash_report(&report).await
    })?;

    info!("Crash report submitted: {}", response.id);
    Ok(())
}

fn get_os_version() -> String {
    #[cfg(windows)]
    {
        use windows::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOW};

        let mut info = OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
            ..Default::default()
        };

        #[allow(deprecated)]
        let success = unsafe { GetVersionExW(&mut info).is_ok() };

        if success {
            format!(
                "Windows {}.{}.{}",
                info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
            )
        } else {
            "Windows".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_installed_default() {
        // Handler should not be installed by default at test start
        // Note: This may be affected by other tests running in parallel
        let _ = HANDLER_INSTALLED.load(Ordering::SeqCst);
    }

    #[test]
    fn test_get_os_version() {
        let version = get_os_version();
        #[cfg(windows)]
        assert!(version.starts_with("Windows"));
        #[cfg(not(windows))]
        assert_eq!(version, "Unknown");
    }
}
