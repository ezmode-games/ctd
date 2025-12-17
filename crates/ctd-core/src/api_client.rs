//! API client for backend communication.
//!
//! This module provides an HTTP client for communicating with the CTD backend API.

use tracing::{debug, instrument};

use crate::crash_report::CrashReport;
use crate::{CtdError, Result};

/// Configuration for the API client.
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    /// Base URL of the API server.
    pub base_url: String,
    /// Optional API key for authentication.
    pub api_key: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.ctd.ezmode.games".to_string(),
            api_key: None,
            timeout_secs: 30,
        }
    }
}

impl ApiClientConfig {
    /// Creates a new configuration with the specified base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            ..Default::default()
        }
    }

    /// Sets the API key.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the timeout in seconds.
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// HTTP client for the CTD API.
#[derive(Debug, Clone)]
pub struct ApiClient {
    config: ApiClientConfig,
    client: reqwest::Client,
}

impl ApiClient {
    /// Creates a new API client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the HTTP client cannot be created.
    pub fn new(config: ApiClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| CtdError::ApiRequest(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// Creates a new API client with default configuration.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the HTTP client cannot be created.
    pub fn with_defaults() -> Result<Self> {
        Self::new(ApiClientConfig::default())
    }

    /// Submits a crash report to the API.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::ApiRequest` if the request fails.
    #[instrument(skip(self, report), fields(game = %report.game))]
    pub async fn submit_crash_report(&self, report: &CrashReport) -> Result<String> {
        let url = format!("{}/v1/crash-reports", self.config.base_url);
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
        let url = format!("{}/v1/crash-reports/{}", self.config.base_url, id);
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
        &self.config.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = ApiClientConfig::default();
        assert_eq!(config.base_url, "https://api.ctd.ezmode.games");
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn config_builder() {
        let config = ApiClientConfig::new("https://test.example.com")
            .with_api_key("secret")
            .with_timeout(60);

        assert_eq!(config.base_url, "https://test.example.com");
        assert_eq!(config.api_key, Some("secret".to_string()));
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn client_creation() {
        let client = ApiClient::with_defaults();
        assert!(client.is_ok());
    }

    #[test]
    fn client_base_url() {
        let client = ApiClient::with_defaults().unwrap();
        assert_eq!(client.base_url(), "https://api.ctd.ezmode.games");
    }
}
