//! File fingerprinting utilities for mod identification.
//!
//! This module provides fast, consistent file hashing using xxh3-64 for
//! fingerprinting mod files. These fingerprints can be used to correlate
//! crash patterns with specific mod versions.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use xxhash_rust::xxh3::Xxh3;

/// Buffer size for reading files in chunks (8KB).
const CHUNK_SIZE: usize = 8 * 1024;

/// Computes xxh3-64 fingerprint of a file, returned as lowercase hex string.
///
/// Reads the file in 8KB chunks to avoid loading the entire file into memory,
/// making it suitable for large archive files.
///
/// # Returns
/// 16-character hex string (64-bit hash), e.g., "a1b2c3d4e5f6a7b8"
///
/// # Errors
/// Returns an error if the file cannot be opened or read.
pub fn fingerprint_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Xxh3::new();
    let mut buffer = [0u8; CHUNK_SIZE];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:016x}", hasher.digest()))
}

/// Returns file size in bytes.
///
/// # Errors
/// Returns an error if the file metadata cannot be read.
pub fn file_size(path: &Path) -> io::Result<u64> {
    Ok(std::fs::metadata(path)?.len())
}

/// Extracts PE version info from a Windows executable/DLL.
///
/// Returns version string like "1.2.3.4" or None if not available.
/// This is useful for extracting version information from RED4ext DLLs.
#[cfg(windows)]
pub fn pe_version(path: &Path) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };
    use windows::core::PCWSTR;

    if !path.exists() {
        return None;
    }

    // Convert path to wide string
    let wide_path: Vec<u16> = OsStr::new(path)
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

    if result.is_err() {
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

/// Extracts PE version info from a Windows executable/DLL.
///
/// On non-Windows platforms, this always returns None.
#[cfg(not(windows))]
pub fn pe_version(_path: &Path) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn fingerprint_consistent_for_same_content() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();

        let fp1 = fingerprint_file(file.path()).unwrap();
        let fp2 = fingerprint_file(file.path()).unwrap();

        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 16); // 64-bit hex
    }

    #[test]
    fn fingerprint_different_for_different_content() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(b"content a").unwrap();
        file2.write_all(b"content b").unwrap();

        let fp1 = fingerprint_file(file1.path()).unwrap();
        let fp2 = fingerprint_file(file2.path()).unwrap();

        assert_ne!(fp1, fp2);
    }

    #[test]
    fn fingerprint_missing_file_returns_error() {
        let result = fingerprint_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn file_size_returns_correct_size() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"12345").unwrap();

        let size = file_size(file.path()).unwrap();
        assert_eq!(size, 5);
    }

    #[test]
    fn fingerprint_empty_file() {
        let file = NamedTempFile::new().unwrap();

        let fp = fingerprint_file(file.path()).unwrap();
        assert_eq!(fp.len(), 16);
    }

    #[test]
    fn pe_version_nonexistent_file() {
        let result = pe_version(Path::new("/nonexistent/file.dll"));
        assert!(result.is_none());
    }
}
