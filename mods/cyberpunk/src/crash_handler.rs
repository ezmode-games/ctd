//! Vectored Exception Handler (VEH) for crash capture.
//!
//! This module registers a Windows VEH handler that captures fatal exceptions
//! and triggers crash report submission.

use std::sync::OnceLock;

use thiserror::Error;

use crate::report;

/// Errors that can occur during crash handler operations.
#[derive(Error, Debug)]
pub enum CrashHandlerError {
    /// The handler was already registered.
    #[error("Crash handler already registered")]
    AlreadyRegistered,

    /// Failed to register the VEH handler.
    #[error("Failed to register VEH handler: {0}")]
    RegistrationFailed(String),

    /// Failed to capture stack trace.
    #[error("Failed to capture stack trace: {0}")]
    StackTraceCapture(String),
}

/// Result type for crash handler operations.
pub type Result<T> = std::result::Result<T, CrashHandlerError>;

/// Captured data from a crash.
#[derive(Debug, Clone)]
pub struct CrashData {
    /// The Windows exception code (e.g., 0xC0000005 for ACCESS_VIOLATION).
    pub exception_code: u32,

    /// The address where the exception occurred.
    pub exception_address: u64,

    /// The captured stack trace as a formatted string.
    pub stack_trace: String,

    /// Module name where the crash occurred (if available).
    pub faulting_module: Option<String>,
}

/// Guard to ensure VEH is only registered once.
static HANDLER_REGISTERED: OnceLock<()> = OnceLock::new();

/// Registers the Vectored Exception Handler.
///
/// This should be called once during plugin initialization.
/// Subsequent calls will return an error.
///
/// # Errors
///
/// Returns `CrashHandlerError::AlreadyRegistered` if already registered.
/// Returns `CrashHandlerError::RegistrationFailed` if Windows API fails.
#[cfg(windows)]
pub fn register() -> Result<()> {
    use windows::Win32::System::Diagnostics::Debug::AddVectoredExceptionHandler;

    if HANDLER_REGISTERED.get().is_some() {
        return Err(CrashHandlerError::AlreadyRegistered);
    }

    // SAFETY: We're registering a valid exception handler function.
    // The handler must be careful not to allocate or do complex operations
    // as the process state may be corrupted.
    let result = unsafe { AddVectoredExceptionHandler(1, Some(veh_handler)) };

    if result.is_null() {
        return Err(CrashHandlerError::RegistrationFailed(
            "AddVectoredExceptionHandler returned null".to_string(),
        ));
    }

    // Mark as registered
    let _ = HANDLER_REGISTERED.set(());

    Ok(())
}

/// Non-Windows stub for register.
#[cfg(not(windows))]
pub fn register() -> Result<()> {
    let _ = HANDLER_REGISTERED.set(());
    Ok(())
}

/// The VEH handler callback.
///
/// This is called by Windows when an exception occurs. We filter for fatal
/// exceptions and trigger crash report submission.
#[cfg(windows)]
unsafe extern "system" fn veh_handler(
    exception_info: *mut windows::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> i32 {
    use windows::Win32::System::Diagnostics::Debug::{
        EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS,
    };

    // SAFETY: Windows guarantees exception_info is valid when this callback is invoked
    let info: &EXCEPTION_POINTERS = unsafe { &*exception_info };

    let Some(record) = (unsafe { info.ExceptionRecord.as_ref() }) else {
        return EXCEPTION_CONTINUE_SEARCH.0;
    };

    let code = record.ExceptionCode.0 as u32;

    // Only handle fatal exceptions
    if !is_fatal_exception(code) {
        return EXCEPTION_CONTINUE_SEARCH.0;
    }

    // Capture crash data
    // Note: We're in an exception handler, so we must be very careful
    // about what we do here. Avoid allocations if possible.
    let crash_data = CrashData {
        exception_code: code,
        exception_address: record.ExceptionAddress as u64,
        stack_trace: capture_stack_trace(info),
        faulting_module: get_module_at_address(record.ExceptionAddress as u64),
    };

    // Fire-and-forget report submission
    // This spawns a thread to avoid blocking the exception handler
    report::submit_async(crash_data);

    // Continue searching for other handlers (let the game/debugger handle it too)
    EXCEPTION_CONTINUE_SEARCH.0
}

/// Returns true if the exception code represents a fatal crash.
#[cfg(windows)]
fn is_fatal_exception(code: u32) -> bool {
    // Common fatal exception codes
    const ACCESS_VIOLATION: u32 = 0xC0000005;
    const STACK_OVERFLOW: u32 = 0xC00000FD;
    const ILLEGAL_INSTRUCTION: u32 = 0xC000001D;
    const INTEGER_DIVIDE_BY_ZERO: u32 = 0xC0000094;
    const INTEGER_OVERFLOW: u32 = 0xC0000095;
    const PRIVILEGED_INSTRUCTION: u32 = 0xC0000096;
    const IN_PAGE_ERROR: u32 = 0xC0000006;
    const INVALID_HANDLE: u32 = 0xC0000008;
    const HEAP_CORRUPTION: u32 = 0xC0000374;
    const STACK_BUFFER_OVERRUN: u32 = 0xC0000409;

    matches!(
        code,
        ACCESS_VIOLATION
            | STACK_OVERFLOW
            | ILLEGAL_INSTRUCTION
            | INTEGER_DIVIDE_BY_ZERO
            | INTEGER_OVERFLOW
            | PRIVILEGED_INSTRUCTION
            | IN_PAGE_ERROR
            | INVALID_HANDLE
            | HEAP_CORRUPTION
            | STACK_BUFFER_OVERRUN
    )
}

/// Captures a stack trace from the exception context.
#[cfg(windows)]
fn capture_stack_trace(
    exception_info: &windows::Win32::System::Diagnostics::Debug::EXCEPTION_POINTERS,
) -> String {
    use std::fmt::Write;

    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Diagnostics::Debug::{
        ADDRESS_MODE, CONTEXT, STACKFRAME64, StackWalk64,
    };
    use windows::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW,
    };
    use windows::Win32::System::Threading::GetCurrentProcess;

    let mut result = String::with_capacity(4096);

    let Some(context) = (unsafe { exception_info.ContextRecord.as_ref() }) else {
        return "Failed to get exception context".to_string();
    };

    // Get process handle
    let process: HANDLE = unsafe { GetCurrentProcess() };

    // Initialize stack frame from context
    let mut frame = STACKFRAME64::default();

    // x64 architecture
    #[cfg(target_arch = "x86_64")]
    {
        frame.AddrPC.Offset = context.Rip;
        frame.AddrPC.Mode = ADDRESS_MODE(3); // AddrModeFlat
        frame.AddrFrame.Offset = context.Rbp;
        frame.AddrFrame.Mode = ADDRESS_MODE(3);
        frame.AddrStack.Offset = context.Rsp;
        frame.AddrStack.Mode = ADDRESS_MODE(3);
    }

    let machine_type = 0x8664u32; // IMAGE_FILE_MACHINE_AMD64

    // Walk the stack (limit to 64 frames to avoid infinite loops)
    let mut frame_count = 0;
    let max_frames = 64;

    // Make a mutable copy of the context for StackWalk64
    let mut context_copy: CONTEXT = *context;

    while frame_count < max_frames {
        // SAFETY: StackWalk64 is safe to call with valid handles and pointers
        let success = unsafe {
            StackWalk64(
                machine_type,
                process,
                HANDLE::default(), // Use 0 for current thread in exception context
                &mut frame,
                std::ptr::addr_of_mut!(context_copy).cast(),
                None,
                None,
                None,
                None,
            )
        };

        if !success.as_bool() {
            break;
        }

        if frame.AddrPC.Offset == 0 {
            break;
        }

        // Get module name for this address
        let module_name =
            get_module_at_address(frame.AddrPC.Offset).unwrap_or_else(|| "unknown".to_string());

        // Calculate offset within module
        let module_base = get_module_base(frame.AddrPC.Offset).unwrap_or(0);
        let offset = frame.AddrPC.Offset.saturating_sub(module_base);

        let _ = writeln!(
            result,
            "[{:2}] {}+0x{:X} (0x{:016X})",
            frame_count, module_name, offset, frame.AddrPC.Offset
        );

        frame_count += 1;
    }

    if result.is_empty() {
        // Fallback: just report the crash address
        let Some(record) = (unsafe { exception_info.ExceptionRecord.as_ref() }) else {
            return "Failed to capture stack trace".to_string();
        };

        let addr = record.ExceptionAddress as u64;
        let module_name = get_module_at_address(addr).unwrap_or_else(|| "unknown".to_string());
        let module_base = get_module_base(addr).unwrap_or(0);
        let offset = addr.saturating_sub(module_base);

        let _ = writeln!(
            result,
            "[0] {}+0x{:X} (0x{:016X})",
            module_name, offset, addr
        );
    }

    result
}

/// Non-Windows stub for stack trace capture.
#[cfg(not(windows))]
fn capture_stack_trace(_exception_info: &std::ffi::c_void) -> String {
    "Stack trace not available on non-Windows platforms".to_string()
}

/// Gets the module name containing the given address.
#[cfg(windows)]
fn get_module_at_address(address: u64) -> Option<String> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleFileNameW, GetModuleHandleExW,
    };

    let mut module: HMODULE = HMODULE::default();

    // SAFETY: GetModuleHandleExW is safe with valid parameters
    let success = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            windows::core::PCWSTR::from_raw(address as *const u16),
            &mut module,
        )
    };

    if !success.is_ok() {
        return None;
    }

    // Get module filename
    let mut filename = [0u16; 260];
    // SAFETY: GetModuleFileNameW is safe with valid buffer
    let len = unsafe { GetModuleFileNameW(module, &mut filename) };

    if len == 0 {
        return None;
    }

    let path = String::from_utf16_lossy(&filename[..len as usize]);

    // Extract just the filename
    path.rsplit('\\').next().map(|s| s.to_string())
}

/// Non-Windows stub.
#[cfg(not(windows))]
fn get_module_at_address(_address: u64) -> Option<String> {
    None
}

/// Gets the base address of the module containing the given address.
#[cfg(windows)]
fn get_module_base(address: u64) -> Option<u64> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleHandleExW,
    };

    let mut module: HMODULE = HMODULE::default();

    // SAFETY: GetModuleHandleExW is safe with valid parameters
    let success = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            windows::core::PCWSTR::from_raw(address as *const u16),
            &mut module,
        )
    };

    if success.is_ok() {
        Some(module.0 as u64)
    } else {
        None
    }
}

/// Non-Windows stub.
#[cfg(not(windows))]
fn get_module_base(_address: u64) -> Option<u64> {
    None
}

/// Returns a human-readable name for a Windows exception code.
#[allow(dead_code)]
pub fn exception_code_name(code: u32) -> &'static str {
    match code {
        0xC0000005 => "ACCESS_VIOLATION",
        0xC00000FD => "STACK_OVERFLOW",
        0xC000001D => "ILLEGAL_INSTRUCTION",
        0xC0000094 => "INTEGER_DIVIDE_BY_ZERO",
        0xC0000095 => "INTEGER_OVERFLOW",
        0xC0000096 => "PRIVILEGED_INSTRUCTION",
        0xC0000006 => "IN_PAGE_ERROR",
        0xC0000008 => "INVALID_HANDLE",
        0xC0000374 => "HEAP_CORRUPTION",
        0xC0000409 => "STACK_BUFFER_OVERRUN",
        _ => "UNKNOWN_EXCEPTION",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exception_code_name() {
        assert_eq!(exception_code_name(0xC0000005), "ACCESS_VIOLATION");
        assert_eq!(exception_code_name(0xC00000FD), "STACK_OVERFLOW");
        assert_eq!(exception_code_name(0x12345678), "UNKNOWN_EXCEPTION");
    }

    #[test]
    fn test_crash_data_clone() {
        let data = CrashData {
            exception_code: 0xC0000005,
            exception_address: 0x7FF712345678,
            stack_trace: "test trace".to_string(),
            faulting_module: Some("test.dll".to_string()),
        };

        let cloned = data.clone();
        assert_eq!(cloned.exception_code, data.exception_code);
        assert_eq!(cloned.stack_trace, data.stack_trace);
    }
}
