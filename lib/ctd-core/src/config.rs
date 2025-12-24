//! Configuration management for CTD.
//!
//! Configuration is loaded from a TOML file with the following search order:
//! 1. Path specified via `CTD_CONFIG` environment variable
//! 2. `./ctd.toml` (current directory)
//! 3. `~/.config/ctd/config.toml` (user config directory)
//!
//! Environment variables can override config file values:
//! - `CTD_API_URL` - Base URL for the API server
//! - `CTD_API_KEY` - API key for authentication

use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{CtdError, Result};

/// Default base URL for local development.
pub const DEFAULT_API_URL: &str = "http://localhost:3000";

/// Default API path for crash reports.
pub const DEFAULT_CRASHES_PATH: &str = "/crashes";

/// Default request timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Configuration for the CTD client.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// API configuration.
    pub api: ApiConfig,
}

/// API-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// Base URL of the API server.
    pub url: String,
    /// API path for crash reports endpoint.
    pub crashes_path: String,
    /// Optional API key for authentication.
    pub api_key: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_API_URL.to_string(),
            crashes_path: DEFAULT_CRASHES_PATH.to_string(),
            api_key: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

impl Config {
    /// Loads configuration from file and environment variables.
    ///
    /// Search order for config file:
    /// 1. `CTD_CONFIG` environment variable
    /// 2. `./ctd.toml`
    /// 3. `~/.config/ctd/config.toml`
    ///
    /// Environment variables override file values:
    /// - `CTD_API_URL`
    /// - `CTD_API_KEY`
    pub fn load() -> Result<Self> {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();
        Ok(config)
    }

    /// Loads configuration from a specific file path.
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| CtdError::Config(format!("Failed to read config file: {}", e)))?;

        let mut config: Config = toml::from_str(&contents)
            .map_err(|e| CtdError::Config(format!("Failed to parse config file: {}", e)))?;

        config.apply_env_overrides();
        Ok(config)
    }

    /// Attempts to load config from default file locations.
    fn load_from_file() -> Option<Self> {
        // 1. Check CTD_CONFIG env var
        if let Ok(path) = env::var("CTD_CONFIG")
            && let Ok(config) = Self::load_from_path(&PathBuf::from(path))
        {
            return Some(config);
        }

        // 2. Check ./ctd.toml
        let local_path = PathBuf::from("ctd.toml");
        if local_path.exists()
            && let Ok(config) = Self::load_from_path(&local_path)
        {
            return Some(config);
        }

        // 3. Check ~/.config/ctd/config.toml
        if let Some(config_dir) = dirs::config_dir() {
            let user_path = config_dir.join("ctd").join("config.toml");
            if user_path.exists()
                && let Ok(config) = Self::load_from_path(&user_path)
            {
                return Some(config);
            }
        }

        None
    }

    /// Applies environment variable overrides to the config.
    fn apply_env_overrides(&mut self) {
        if let Ok(url) = env::var("CTD_API_URL") {
            self.api.url = url;
        }

        if let Ok(key) = env::var("CTD_API_KEY") {
            self.api.api_key = Some(key);
        }
    }

    /// Returns an example config file as a string.
    pub fn example() -> &'static str {
        r#"# CTD Configuration File
# Place this file at ./ctd.toml or ~/.config/ctd/config.toml

[api]
# Base URL of the crash report server
url = "http://localhost:3000"

# API path for crash reports endpoint
crashes_path = "/crashes"

# Optional API key for authentication
# api_key = "your-api-key-here"

# Request timeout in seconds
timeout_secs = 30
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.api.url, DEFAULT_API_URL);
        assert_eq!(config.api.crashes_path, DEFAULT_CRASHES_PATH);
        assert!(config.api.api_key.is_none());
        assert_eq!(config.api.timeout_secs, DEFAULT_TIMEOUT_SECS);
    }

    #[test]
    fn parse_toml() {
        let toml = r#"
            [api]
            url = "https://custom.example.com"
            crashes_path = "/api/v2/crashes"
            api_key = "secret123"
            timeout_secs = 60
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.api.url, "https://custom.example.com");
        assert_eq!(config.api.crashes_path, "/api/v2/crashes");
        assert_eq!(config.api.api_key, Some("secret123".to_string()));
        assert_eq!(config.api.timeout_secs, 60);
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let toml = r#"
            [api]
            url = "https://custom.example.com"
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.api.url, "https://custom.example.com");
        assert_eq!(config.api.crashes_path, DEFAULT_CRASHES_PATH);
        assert!(config.api.api_key.is_none());
    }

    #[test]
    fn example_config_is_valid() {
        let example = Config::example();
        let _config: Config = toml::from_str(example).unwrap();
    }
}
