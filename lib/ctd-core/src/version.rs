//! Windows PE version extraction utilities.
//!
//! This module provides functionality to extract version information from
//! Windows PE files (DLLs and EXEs) using the Windows API.

use std::path::Path;
use thiserror::Error;

/// Errors that can occur when extracting version information.
#[derive(Error, Debug)]
pub enum VersionError {
    /// Failed to read or access the file.
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    /// The file does not contain version information.
    #[error("No version info found")]
    NoVersionInfo,

    /// Failed to parse the version information structure.
    #[error("Failed to parse version info")]
    ParseError,
}

/// Extract version string from a Windows PE file (DLL/EXE).
///
/// Returns the file version in the format "major.minor.build.revision".
///
/// # Arguments
///
/// * `path` - Path to the PE file to extract version from
///
/// # Returns
///
/// * `Ok(String)` - The version string (e.g., "10.0.19041.1")
/// * `Err(VersionError)` - If version extraction fails
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use ctd_core::version::get_dll_version;
///
/// let version = get_dll_version(Path::new("C:\\Windows\\System32\\kernel32.dll")).unwrap();
/// println!("kernel32.dll version: {}", version);
/// ```
#[cfg(windows)]
pub fn get_dll_version(path: &Path) -> Result<String, VersionError> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
    };
    use windows::core::PCWSTR;

    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(wide_path.as_ptr()), None);
        if size == 0 {
            return Err(VersionError::NoVersionInfo);
        }

        let mut buffer = vec![0u8; size as usize];
        GetFileVersionInfoW(
            PCWSTR(wide_path.as_ptr()),
            0,
            size,
            buffer.as_mut_ptr().cast(),
        )
        .map_err(|_| VersionError::ParseError)?;

        let mut ver_info: *mut VS_FIXEDFILEINFO = std::ptr::null_mut();
        let mut ver_len: u32 = 0;

        let sub_block: Vec<u16> = OsStr::new("\\")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let success = VerQueryValueW(
            buffer.as_ptr().cast(),
            PCWSTR(sub_block.as_ptr()),
            std::ptr::addr_of_mut!(ver_info).cast(),
            &mut ver_len,
        );

        if !success.as_bool() || ver_info.is_null() {
            return Err(VersionError::NoVersionInfo);
        }

        let info = &*ver_info;
        let major = (info.dwFileVersionMS >> 16) & 0xFFFF;
        let minor = info.dwFileVersionMS & 0xFFFF;
        let build = (info.dwFileVersionLS >> 16) & 0xFFFF;
        let revision = info.dwFileVersionLS & 0xFFFF;

        Ok(format!("{}.{}.{}.{}", major, minor, build, revision))
    }
}

/// Stub implementation for non-Windows platforms.
///
/// Always returns `VersionError::NoVersionInfo` since PE version extraction
/// is only supported on Windows.
#[cfg(not(windows))]
pub fn get_dll_version(_path: &Path) -> Result<String, VersionError> {
    Err(VersionError::NoVersionInfo)
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[test]
    fn test_system_dll_has_version() {
        let path = Path::new("C:\\Windows\\System32\\kernel32.dll");
        let version = get_dll_version(path).unwrap();

        assert!(version.contains('.'));
        let parts: Vec<&str> = version.split('.').collect();
        assert!(parts.len() >= 2);
    }

    #[test]
    fn test_nonexistent_file_error() {
        let result = get_dll_version(Path::new("C:\\nonexistent.dll"));
        assert!(result.is_err());
    }
}
