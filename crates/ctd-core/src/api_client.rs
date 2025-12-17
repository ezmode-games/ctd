//! API client for backend communication.
//!
//! This module provides an HTTP client for communicating with the CTD backend API.
//! Configuration is loaded from `ctd.toml` or environment variables.

use tracing::{debug, instrument};

use crate::config::{ApiConfig, Config};
use crate::crash_report::CrashReport;
use crate::{CtdError, Result};

/// HTTP client for the CTD API.
#[derive(Debug, Clone)]
pub struct ApiClient {
    config: ApiConfig,
    client: reqwest::Client,
}

impl ApiClient {
    /// Creates a new API client with the given API configuration.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the HTTP client cannot be created.
    pub fn new(config: ApiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| CtdError::ApiRequest(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// Creates a new API client by loading configuration from file/environment.
    ///
    /// Loads config from `ctd.toml` or environment variables.
    /// See [`Config::load`] for search order.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::Config` if config loading fails, or
    /// `CtdError::ApiRequest` if the HTTP client cannot be created.
    pub fn from_config() -> Result<Self> {
        let config = Config::load()?;
        Self::new(config.api)
    }

    /// Creates a new API client with default configuration.
    ///
    /// Uses hardcoded defaults (localhost:3000). Prefer [`ApiClient::from_config`]
    /// for production use.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the HTTP client cannot be created.
    pub fn with_defaults() -> Result<Self> {
        Self::new(ApiConfig::default())
    }

    /// Submits a crash report to the API.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the request fails.
    #[instrument(skip(self, report), fields(game = %report.game))]
    pub async fn submit_crash_report(&self, report: &CrashReport) -> Result<String> {
        let url = format!("{}{}", self.config.url, self.config.crashes_path);
        debug!("Submitting crash report to {}", url);

        let mut request = self.client.post(&url).json(report);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| CtdError::ApiRequest(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CtdError::ApiRequest(format!(
                "Server returned status {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| CtdError::ApiRequest(e.to_string()))?;

        Ok(body)
    }

    /// Retrieves a crash report by ID.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the request fails.
    #[instrument(skip(self))]
    pub async fn get_crash_report(&self, id: &str) -> Result<CrashReport> {
        let url = format!("{}{}/{}", self.config.url, self.config.crashes_path, id);
        debug!("Fetching crash report from {}", url);

        let mut request = self.client.get(&url);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| CtdError::ApiRequest(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CtdError::ApiRequest(format!(
                "Server returned status {}",
                response.status()
            )));
        }

        let report: CrashReport = response
            .json()
            .await
            .map_err(|e| CtdError::ApiRequest(e.to_string()))?;

        Ok(report)
    }

    /// Returns the base URL of the API.
    pub fn base_url(&self) -> &str {
        &self.config.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_API_URL;

    #[test]
    fn client_creation() {
        let client = ApiClient::with_defaults();
        assert!(client.is_ok());
    }

    #[test]
    fn client_base_url() {
        let client = ApiClient::with_defaults().unwrap();
        assert_eq!(client.base_url(), DEFAULT_API_URL);
    }

    #[test]
    fn client_with_custom_config() {
        let config = ApiConfig {
            url: "https://custom.example.com".to_string(),
            crashes_path: "/api/v2/crashes".to_string(),
            api_key: Some("test-key".to_string()),
            timeout_secs: 60,
        };
        let client = ApiClient::new(config).unwrap();
        assert_eq!(client.base_url(), "https://custom.example.com");
    }
}
