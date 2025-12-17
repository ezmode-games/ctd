//! Crash report generation and serialization.
//!
//! This module handles creating and serializing crash reports for submission.

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::load_order::LoadOrder;

/// Represents a crash report to be submitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    /// Unique identifier for this crash report.
    pub id: Option<String>,
    /// The game that crashed.
    pub game: String,
    /// Version of the game.
    pub game_version: Option<String>,
    /// The load order at the time of crash.
    pub load_order: LoadOrder,
    /// Optional crash log or stack trace.
    pub crash_log: Option<String>,
    /// Optional user-provided description.
    pub description: Option<String>,
    /// Timestamp when the crash occurred (Unix timestamp).
    pub timestamp: Option<u64>,
}

impl CrashReport {
    /// Creates a new crash report with the given game and load order.
    pub fn new(game: impl Into<String>, load_order: LoadOrder) -> Self {
        Self {
            id: None,
            game: game.into(),
            game_version: None,
            load_order,
            crash_log: None,
            description: None,
            timestamp: None,
        }
    }

    /// Sets the game version.
    pub fn with_game_version(mut self, version: impl Into<String>) -> Self {
        self.game_version = Some(version.into());
        self
    }

    /// Sets the crash log.
    pub fn with_crash_log(mut self, log: impl Into<String>) -> Self {
        self.crash_log = Some(log.into());
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the timestamp.
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Serializes the crash report to JSON.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::Serialize` if serialization fails.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Serializes the crash report to pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::Serialize` if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserializes a crash report from JSON.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::Serialize` if deserialization fails.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_crash_report() {
        let report = CrashReport::new("Skyrim", LoadOrder::new());
        assert_eq!(report.game, "Skyrim");
        assert!(report.load_order.is_empty());
        assert!(report.id.is_none());
    }

    #[test]
    fn builder_pattern() {
        let report = CrashReport::new("Skyrim", LoadOrder::new())
            .with_game_version("1.6.1170")
            .with_description("CTD on startup")
            .with_timestamp(1700000000);

        assert_eq!(report.game_version, Some("1.6.1170".to_string()));
        assert_eq!(report.description, Some("CTD on startup".to_string()));
        assert_eq!(report.timestamp, Some(1700000000));
    }

    #[test]
    fn json_roundtrip() {
        let report = CrashReport::new("Skyrim", LoadOrder::new()).with_game_version("1.6.1170");

        let json = report.to_json().unwrap();
        let parsed = CrashReport::from_json(&json).unwrap();

        assert_eq!(parsed.game, report.game);
        assert_eq!(parsed.game_version, report.game_version);
    }
}
