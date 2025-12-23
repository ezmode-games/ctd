//! Crash report building and submission.
//!
//! This module handles creating crash reports from captured crash data
//! and submitting them to the CTD API.

use std::sync::atomic::{AtomicBool, Ordering};

use ctd_core::api_client::ApiClient;
use ctd_core::crash_report::CreateCrashReport;
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::crash_handler::CrashData;
use crate::mod_scanner;

/// Errors that can occur during report submission.
#[derive(Error, Debug)]
pub enum ReportError {
    /// Failed to build the crash report.
    #[error("Failed to build crash report: {0}")]
    BuildFailed(String),

    /// Failed to create the API client.
    #[error("Failed to create API client: {0}")]
    ClientCreation(String),

    /// Failed to submit the report.
    #[error("Failed to submit crash report: {0}")]
    Submission(String),

    /// A submission is already in progress.
    #[error("Crash report submission already in progress")]
    AlreadySubmitting,
}

/// Result type for report operations.
pub type Result<T> = std::result::Result<T, ReportError>;

/// Guard to prevent multiple simultaneous submissions.
static SUBMISSION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Game ID for Cyberpunk 2077 crash reports.
const GAME_ID: &str = "cyberpunk-2077";

/// Submits a crash report asynchronously (fire-and-forget).
///
/// This spawns a new thread to handle submission, avoiding blocking
/// the exception handler. If a submission is already in progress,
/// this call is ignored.
///
/// # Arguments
///
/// * `crash_data` - The captured crash data to report.
pub fn submit_async(crash_data: CrashData) {
    // Check if we're already submitting
    if SUBMISSION_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        warn!("Crash report submission already in progress, skipping duplicate");
        return;
    }

    // Spawn a thread for submission
    // We use a thread instead of tokio::spawn because we may not have
    // a runtime active, and we want to guarantee this doesn't block
    std::thread::spawn(move || {
        let result = submit_sync(crash_data);

        // Reset the in-progress flag
        SUBMISSION_IN_PROGRESS.store(false, Ordering::SeqCst);

        match result {
            Ok(response_id) => {
                info!("Crash report submitted successfully: {}", response_id);
            }
            Err(e) => {
                // Log but don't crash - we're in a crash handler after all
                error!("Failed to submit crash report: {}", e);
            }
        }
    });
}

/// Submits a crash report synchronously.
///
/// This creates a tokio runtime to execute the async API call.
///
/// # Returns
///
/// The report ID on success, or an error on failure.
fn submit_sync(crash_data: CrashData) -> Result<String> {
    debug!(
        "Submitting crash report for exception 0x{:08X}",
        crash_data.exception_code
    );

    // Get cached mods (or empty if not scanned)
    let load_order = mod_scanner::get_cached_or_empty();

    // Build the crash report
    let report = build_report(&crash_data, load_order)?;

    // Create a single-threaded runtime for the API call
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| ReportError::Submission(format!("Failed to create runtime: {}", e)))?;

    // Execute the API call
    let response = rt.block_on(async {
        let client = ApiClient::from_config()
            .or_else(|_| ApiClient::with_defaults())
            .map_err(|e| ReportError::ClientCreation(e.to_string()))?;

        client
            .submit_crash_report(&report)
            .await
            .map_err(|e| ReportError::Submission(e.to_string()))
    })?;

    Ok(response.id)
}

/// Builds a crash report from crash data.
fn build_report(
    crash_data: &CrashData,
    load_order: ctd_core::load_order::LoadOrder,
) -> Result<CreateCrashReport> {
    let mut builder = CreateCrashReport::builder()
        .game_id(GAME_ID)
        .game_version(get_game_version())
        .stack_trace(&crash_data.stack_trace)
        .exception_code(format!("0x{:08X}", crash_data.exception_code))
        .exception_address(format!("0x{:016X}", crash_data.exception_address))
        .load_order(load_order)
        .crashed_now();

    // Add faulting module if available
    if let Some(ref module) = crash_data.faulting_module {
        builder = builder.faulting_module(module);
    }

    // Add RED4ext version if we can detect it
    if let Some(version) = get_red4ext_version() {
        builder = builder.script_extender_version(version);
    }

    // Add OS version
    if let Some(os_version) = get_os_version() {
        builder = builder.os_version(os_version);
    }

    builder
        .build()
        .map_err(|e| ReportError::BuildFailed(e.to_string()))
}

/// Gets the Cyberpunk 2077 game version.
///
/// Attempts to read the version from the game executable.
/// Falls back to "unknown" if detection fails.
fn get_game_version() -> String {
    // Try to detect version from game files
    #[cfg(windows)]
    if let Some(version) = detect_game_version_windows() {
        return version;
    }

    // Fallback to unknown
    "unknown".to_string()
}

/// Attempts to detect the game version on Windows.
#[cfg(windows)]
fn detect_game_version_windows() -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };
    use windows::core::PCWSTR;

    let game_dir = mod_scanner::get_game_directory_path()?;
    let exe_path = game_dir.join("bin/x64/Cyberpunk2077.exe");

    if !exe_path.exists() {
        return None;
    }

    // Convert path to wide string
    let wide_path: Vec<u16> = OsStr::new(&exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let path_pcwstr = PCWSTR::from_raw(wide_path.as_ptr());

    // Get version info size
    let size = unsafe { GetFileVersionInfoSizeW(path_pcwstr, None) };
    if size == 0 {
        return None;
    }

    // Allocate buffer and get version info
    let mut buffer = vec![0u8; size as usize];
    let result = unsafe { GetFileVersionInfoW(path_pcwstr, 0, size, buffer.as_mut_ptr().cast()) };

    if !result.as_bool() {
        return None;
    }

    // Query the root block for VS_FIXEDFILEINFO
    let mut info_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut info_len: u32 = 0;

    let query_path: Vec<u16> = OsStr::new("\\")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        VerQueryValueW(
            buffer.as_ptr().cast(),
            PCWSTR::from_raw(query_path.as_ptr()),
            &mut info_ptr,
            &mut info_len,
        )
    };

    if !result.as_bool() || info_ptr.is_null() {
        return None;
    }

    // VS_FIXEDFILEINFO structure
    #[repr(C)]
    struct VsFixedFileInfo {
        dw_signature: u32,
        dw_struc_version: u32,
        dw_file_version_ms: u32,
        dw_file_version_ls: u32,
        // ... other fields we don't need
    }

    let info = unsafe { &*(info_ptr as *const VsFixedFileInfo) };

    // Extract version numbers
    let major = (info.dw_file_version_ms >> 16) & 0xFFFF;
    let minor = info.dw_file_version_ms & 0xFFFF;
    let patch = (info.dw_file_version_ls >> 16) & 0xFFFF;
    let build = info.dw_file_version_ls & 0xFFFF;

    Some(format!("{}.{}.{}.{}", major, minor, patch, build))
}

/// Gets the RED4ext version if available.
///
/// RED4ext doesn't expose a Rust-friendly version API yet,
/// so this returns None for now.
fn get_red4ext_version() -> Option<String> {
    // RED4ext doesn't expose a Rust-friendly version API yet
    // This would need to be implemented using the SDK
    None
}

/// Gets the Windows version string.
#[cfg(windows)]
fn get_os_version() -> Option<String> {
    use windows::Win32::System::SystemInformation::{GetVersionExW, OSVERSIONINFOW};

    let mut info = OSVERSIONINFOW::default();
    info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

    // SAFETY: GetVersionExW is safe with a properly sized OSVERSIONINFOW
    #[allow(deprecated)]
    let result = unsafe { GetVersionExW(&mut info) };

    if result.as_bool() {
        Some(format!(
            "Windows {}.{}.{}",
            info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
        ))
    } else {
        None
    }
}

/// Non-Windows stub.
#[cfg(not(windows))]
fn get_os_version() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctd_core::load_order::LoadOrder;

    #[test]
    fn test_build_report() {
        let crash_data = CrashData {
            exception_code: 0xC0000005,
            exception_address: 0x7FF712345678,
            stack_trace: "test stack trace".to_string(),
            faulting_module: Some("test.dll".to_string()),
        };

        let load_order = LoadOrder::new();
        let result = build_report(&crash_data, load_order);

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.game_id, GAME_ID);
        assert_eq!(report.exception_code, Some("0xC0000005".to_string()));
        assert_eq!(
            report.exception_address,
            Some("0x00007FF712345678".to_string())
        );
    }

    #[test]
    fn test_game_id_constant() {
        assert_eq!(GAME_ID, "cyberpunk-2077");
    }
}
