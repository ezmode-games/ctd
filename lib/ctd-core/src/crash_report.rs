//! Crash report types matching the API schema.
//!
//! These types exactly match the API's `createCrashReportSchema`.

use serde::{Deserialize, Serialize};

use crate::load_order::{LoadOrder, ModList};
use crate::{CtdError, Result};

/// Current schema version for crash reports.
/// - v1: LoadOrderEntry with name/enabled/index only
/// - v2: ModEntry with file_hash/file_size/version for pattern detection
const CURRENT_SCHEMA_VERSION: u32 = 2;

/// A crash report to be submitted to the API.
///
/// Matches the API's `createCrashReportSchema` exactly.
/// Required fields are non-optional to enforce at compile time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCrashReport {
    /// Schema version for forward compatibility. Default: 1
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// Game identifier (e.g., "skyrim-se", "fallout4").
    /// Required, min length 1.
    pub game_id: String,

    /// Full stack trace from the crash.
    /// Required, min length 1, max 100000.
    pub stack_trace: String,

    /// Pre-computed crash hash for deduplication.
    /// Optional - server computes if not provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_hash: Option<String>,

    /// Exception code (e.g., "0xC0000005").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_code: Option<String>,

    /// Exception address (e.g., "0x7FF712345678").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_address: Option<String>,

    /// Module that caused the crash (e.g., "SkyrimSE.exe").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faulting_module: Option<String>,

    /// Game version string (e.g., "1.6.1170").
    /// Required, min length 1, max 50.
    pub game_version: String,

    /// Script extender version (e.g., "2.2.3").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_extender_version: Option<String>,

    /// Operating system version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,

    /// Load order as JSON string. Required.
    /// Use `LoadOrder::to_json()` to create this.
    pub load_order_json: String,

    /// Number of plugins in the load order.
    /// Required, 0-10000.
    pub plugin_count: u32,

    /// Unix timestamp (milliseconds) when the crash occurred.
    /// Required.
    pub crashed_at: u64,

    /// User notes about the crash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}

/// Response from the API after creating a crash report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashReportResponse {
    /// The assigned report ID (ULID).
    pub id: String,
    /// Share token for accessing the report.
    pub share_token: String,
}

/// Internal enum to hold either v1 or v2 load order data.
#[derive(Debug)]
enum LoadOrderData {
    V1(LoadOrder),
    V2(ModList),
}

/// Builder for creating crash reports with validation.
#[derive(Debug, Default)]
pub struct CrashReportBuilder {
    game_id: Option<String>,
    stack_trace: Option<String>,
    crash_hash: Option<String>,
    exception_code: Option<String>,
    exception_address: Option<String>,
    faulting_module: Option<String>,
    game_version: Option<String>,
    script_extender_version: Option<String>,
    os_version: Option<String>,
    load_order_data: Option<LoadOrderData>,
    crashed_at: Option<u64>,
    notes: Option<String>,
}

impl CrashReportBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the game ID (required).
    pub fn game_id(mut self, id: impl Into<String>) -> Self {
        self.game_id = Some(id.into());
        self
    }

    /// Sets the stack trace (required).
    pub fn stack_trace(mut self, trace: impl Into<String>) -> Self {
        self.stack_trace = Some(trace.into());
        self
    }

    /// Sets the crash hash (optional).
    pub fn crash_hash(mut self, hash: impl Into<String>) -> Self {
        self.crash_hash = Some(hash.into());
        self
    }

    /// Sets the exception code (optional).
    pub fn exception_code(mut self, code: impl Into<String>) -> Self {
        self.exception_code = Some(code.into());
        self
    }

    /// Sets the exception address (optional).
    pub fn exception_address(mut self, addr: impl Into<String>) -> Self {
        self.exception_address = Some(addr.into());
        self
    }

    /// Sets the faulting module (optional).
    pub fn faulting_module(mut self, module: impl Into<String>) -> Self {
        self.faulting_module = Some(module.into());
        self
    }

    /// Sets the game version (required).
    pub fn game_version(mut self, version: impl Into<String>) -> Self {
        self.game_version = Some(version.into());
        self
    }

    /// Sets the script extender version (optional).
    pub fn script_extender_version(mut self, version: impl Into<String>) -> Self {
        self.script_extender_version = Some(version.into());
        self
    }

    /// Sets the OS version (optional).
    pub fn os_version(mut self, version: impl Into<String>) -> Self {
        self.os_version = Some(version.into());
        self
    }

    /// Sets the load order using v1 schema (LoadOrderEntry).
    ///
    /// For new implementations, prefer `load_order_v2` which includes
    /// file hashes and versions for better pattern detection.
    #[allow(deprecated)]
    #[deprecated(
        since = "0.2.0",
        note = "Use load_order_v2 with ModList for pattern detection"
    )]
    pub fn load_order(mut self, lo: LoadOrder) -> Self {
        self.load_order_data = Some(LoadOrderData::V1(lo));
        self
    }

    /// Sets the load order using v2 schema (ModEntry with fingerprints).
    ///
    /// This is the preferred method for new implementations. ModEntry
    /// includes file_hash, file_size, and version for pattern detection.
    pub fn load_order_v2(mut self, list: ModList) -> Self {
        self.load_order_data = Some(LoadOrderData::V2(list));
        self
    }

    /// Sets the crash timestamp in milliseconds (required).
    pub fn crashed_at(mut self, timestamp: u64) -> Self {
        self.crashed_at = Some(timestamp);
        self
    }

    /// Sets crash timestamp to now.
    pub fn crashed_now(self) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.crashed_at(now)
    }

    /// Sets user notes (optional).
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Builds the crash report, validating all required fields.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::Validation` if required fields are missing or invalid.
    pub fn build(self) -> Result<CreateCrashReport> {
        let game_id = self
            .game_id
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CtdError::Validation("game_id is required".into()))?;

        let stack_trace = self
            .stack_trace
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CtdError::Validation("stack_trace is required".into()))?;

        if stack_trace.len() > 100_000 {
            return Err(CtdError::Validation(
                "stack_trace exceeds 100000 characters".into(),
            ));
        }

        let game_version = self
            .game_version
            .filter(|s| !s.is_empty())
            .ok_or_else(|| CtdError::Validation("game_version is required".into()))?;

        if game_version.len() > 50 {
            return Err(CtdError::Validation(
                "game_version exceeds 50 characters".into(),
            ));
        }

        let load_order_data = self
            .load_order_data
            .ok_or_else(|| CtdError::Validation("load_order is required".into()))?;

        let (plugin_count, load_order_json, schema_version) = match load_order_data {
            LoadOrderData::V1(lo) => {
                let count = lo.len() as u32;
                let json = lo.to_json().map_err(|e| {
                    CtdError::Validation(format!("failed to serialize load_order: {}", e))
                })?;
                (count, json, 1)
            }
            LoadOrderData::V2(ml) => {
                let count = ml.len() as u32;
                let json = ml.to_json().map_err(|e| {
                    CtdError::Validation(format!("failed to serialize load_order: {}", e))
                })?;
                (count, json, 2)
            }
        };

        if plugin_count > 10_000 {
            return Err(CtdError::Validation("plugin_count exceeds 10000".into()));
        }

        let crashed_at = self
            .crashed_at
            .ok_or_else(|| CtdError::Validation("crashed_at is required".into()))?;

        // Validate optional field lengths
        if let Some(ref hash) = self.crash_hash
            && (hash.is_empty() || hash.len() > 64)
        {
            return Err(CtdError::Validation(
                "crash_hash must be 1-64 characters".into(),
            ));
        }

        if let Some(ref code) = self.exception_code
            && code.len() > 50
        {
            return Err(CtdError::Validation(
                "exception_code exceeds 50 characters".into(),
            ));
        }

        if let Some(ref addr) = self.exception_address
            && addr.len() > 50
        {
            return Err(CtdError::Validation(
                "exception_address exceeds 50 characters".into(),
            ));
        }

        if let Some(ref module) = self.faulting_module
            && module.len() > 255
        {
            return Err(CtdError::Validation(
                "faulting_module exceeds 255 characters".into(),
            ));
        }

        if let Some(ref ver) = self.script_extender_version
            && ver.len() > 50
        {
            return Err(CtdError::Validation(
                "script_extender_version exceeds 50 characters".into(),
            ));
        }

        if let Some(ref ver) = self.os_version
            && ver.len() > 100
        {
            return Err(CtdError::Validation(
                "os_version exceeds 100 characters".into(),
            ));
        }

        if let Some(ref notes) = self.notes
            && notes.len() > 5000
        {
            return Err(CtdError::Validation("notes exceeds 5000 characters".into()));
        }

        Ok(CreateCrashReport {
            schema_version,
            game_id,
            stack_trace,
            crash_hash: self.crash_hash,
            exception_code: self.exception_code,
            exception_address: self.exception_address,
            faulting_module: self.faulting_module,
            game_version,
            script_extender_version: self.script_extender_version,
            os_version: self.os_version,
            load_order_json,
            plugin_count,
            crashed_at,
            notes: self.notes,
        })
    }
}

impl CreateCrashReport {
    /// Creates a builder for constructing a crash report.
    pub fn builder() -> CrashReportBuilder {
        CrashReportBuilder::new()
    }

    /// Serializes to JSON for the API.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(CtdError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_order::{LoadOrderEntry, ModEntry, ModList};

    #[allow(deprecated)]
    fn sample_load_order() -> LoadOrder {
        let entries = vec![
            LoadOrderEntry::full("Skyrim.esm", true, 0),
            LoadOrderEntry::full("Update.esm", true, 1),
        ];
        LoadOrder::from_entries(entries)
    }

    fn sample_mod_list() -> ModList {
        let mut list = ModList::new();
        list.push(ModEntry::new("Skyrim.esm", "a1b2c3d4e5f67890", 1000).with_index(0));
        list.push(
            ModEntry::new("SkyUI_SE.esp", "b2c3d4e5f6789012", 2000)
                .with_version("5.2.1")
                .with_index(1),
        );
        list
    }

    #[test]
    #[allow(deprecated)]
    fn builder_creates_valid_report_v1() {
        let report = CreateCrashReport::builder()
            .game_id("skyrim-se")
            .game_version("1.6.1170")
            .stack_trace("SkyrimSE.exe+0x12345")
            .load_order(sample_load_order())
            .crashed_at(1700000000000)
            .build()
            .unwrap();

        assert_eq!(report.game_id, "skyrim-se");
        assert_eq!(report.game_version, "1.6.1170");
        assert_eq!(report.plugin_count, 2);
        assert_eq!(report.schema_version, 1);
    }

    #[test]
    fn builder_creates_valid_report_v2() {
        let report = CreateCrashReport::builder()
            .game_id("skyrim-se")
            .game_version("1.6.1170")
            .stack_trace("SkyrimSE.exe+0x12345")
            .load_order_v2(sample_mod_list())
            .crashed_at(1700000000000)
            .build()
            .unwrap();

        assert_eq!(report.game_id, "skyrim-se");
        assert_eq!(report.plugin_count, 2);
        assert_eq!(report.schema_version, 2);
        // Check that load_order_json contains fileHash
        assert!(report.load_order_json.contains("fileHash"));
        assert!(report.load_order_json.contains("a1b2c3d4e5f67890"));
    }

    #[test]
    #[allow(deprecated)]
    fn builder_requires_game_id() {
        let result = CreateCrashReport::builder()
            .game_version("1.0")
            .stack_trace("trace")
            .load_order(LoadOrder::new())
            .crashed_at(1000)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("game_id"));
    }

    #[test]
    #[allow(deprecated)]
    fn builder_requires_stack_trace() {
        let result = CreateCrashReport::builder()
            .game_id("skyrim-se")
            .game_version("1.0")
            .load_order(LoadOrder::new())
            .crashed_at(1000)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stack_trace"));
    }

    #[test]
    #[allow(deprecated)]
    fn builder_validates_field_lengths() {
        let long_string = "x".repeat(51);
        let result = CreateCrashReport::builder()
            .game_id("skyrim-se")
            .game_version(&long_string)
            .stack_trace("trace")
            .load_order(LoadOrder::new())
            .crashed_at(1000)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("game_version"));
    }

    #[test]
    #[allow(deprecated)]
    fn json_uses_camel_case() {
        let report = CreateCrashReport::builder()
            .game_id("skyrim-se")
            .game_version("1.0")
            .stack_trace("trace")
            .load_order(LoadOrder::new())
            .crashed_at(1000)
            .build()
            .unwrap();

        let json = report.to_json().unwrap();

        assert!(json.contains("gameId"));
        assert!(json.contains("gameVersion"));
        assert!(json.contains("stackTrace"));
        assert!(json.contains("loadOrderJson"));
        assert!(json.contains("pluginCount"));
        assert!(json.contains("crashedAt"));
        assert!(json.contains("schemaVersion"));
    }

    #[test]
    fn response_deserializes() {
        let json = r#"{"id":"01ABC","shareToken":"xyz123"}"#;
        let response: CrashReportResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.id, "01ABC");
        assert_eq!(response.share_token, "xyz123");
    }
}
