//! CTD Core Library
//!
//! This crate provides core functionality for the CTD (Crash To Desktop) project:
//! - Configuration management
//! - Load order parsing and management
//! - Crash report generation and serialization
//! - API client for backend communication

pub mod api_client;
pub mod config;
pub mod crash_report;
pub mod load_order;

use thiserror::Error;

/// Error types for CTD operations.
#[derive(Error, Debug)]
pub enum CtdError {
    /// Failed to load or parse configuration.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Failed to parse a load order file or data.
    #[error("Failed to parse load order: {0}")]
    LoadOrderParse(String),

    /// Failed to serialize or deserialize data.
    #[error("Failed to serialize crash report: {0}")]
    Serialize(#[from] serde_json::Error),

    /// An API request failed.
    #[error("API request failed: {0}")]
    ApiRequest(String),
}

/// A specialized Result type for CTD operations.
pub type Result<T> = std::result::Result<T, CtdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_load_order_parse() {
        let err = CtdError::LoadOrderParse("invalid format".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to parse load order: invalid format"
        );
    }

    #[test]
    fn error_display_api_request() {
        let err = CtdError::ApiRequest("connection timeout".to_string());
        assert_eq!(err.to_string(), "API request failed: connection timeout");
    }
}
