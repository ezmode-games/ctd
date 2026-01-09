//! Fast file fingerprinting for mod identification.
//!
//! This module provides efficient file hashing for identifying mods without
//! reading entire files. It uses a partial hash approach (first 64KB + file size)
//! to generate unique fingerprints quickly, making it suitable for scanning
//! large mod directories.
//!
//! The partial hash approach is a tradeoff: it's much faster than hashing
//! entire files (especially for large BSA/BA2 archives), but two files with
//! identical first 64KB and size would produce the same hash. In practice,
//! this is extremely rare for mod files.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when computing file hashes.
#[derive(Error, Debug)]
pub enum HashError {
    /// Failed to open or read the file.
    #[error("Failed to open file: {0}")]
    IoError(#[from] std::io::Error),
}

/// Compute a fast fingerprint of a file.
/// Uses SHA256 of the first 64KB + file size for speed.
/// Returns (hash_hex, file_size) where hash_hex is 16 characters.
pub fn compute_file_hash(path: &Path) -> Result<(String, u64), HashError> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();

    let read_size = 65536.min(size as usize);
    let mut buffer = vec![0u8; read_size];
    file.read_exact(&mut buffer)?;

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    hasher.update(size.to_le_bytes());

    let result = hasher.finalize();
    Ok((hex::encode(&result[..8]), size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_small_file_hash() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"test content").unwrap();

        let (hash, size) = compute_file_hash(file.path()).unwrap();

        assert_eq!(hash.len(), 16);
        assert_eq!(size, 12);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_same_content_same_hash() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(b"identical").unwrap();
        file2.write_all(b"identical").unwrap();

        let (hash1, _) = compute_file_hash(file1.path()).unwrap();
        let (hash2, _) = compute_file_hash(file2.path()).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content_different_hash() {
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        file1.write_all(b"content a").unwrap();
        file2.write_all(b"content b").unwrap();

        let (hash1, _) = compute_file_hash(file1.path()).unwrap();
        let (hash2, _) = compute_file_hash(file2.path()).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_missing_file_error() {
        let result = compute_file_hash(Path::new("/nonexistent/file.esp"));
        assert!(result.is_err());
    }
}
