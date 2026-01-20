//! Load order types matching the API schema.
//!
//! These types represent mod load orders and are serialized to JSON
//! for the `loadOrderJson` field in crash reports.

use serde::{Deserialize, Serialize};

/// A single entry in a load order.
///
/// Matches the API's `loadOrderItemSchema`:
/// - `name`: required string
/// - `enabled`: optional boolean
/// - `index`: optional integer
/// - `fingerprint`: optional xxh3 hash for mod identification
/// - `size`: optional file size in bytes
/// - `version`: optional version string
/// - `mod_type`: optional mod type identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadOrderEntry {
    /// The name of the mod/plugin file (e.g., "SkyUI_SE.esp").
    pub name: String,

    /// xxh3 fingerprint of the mod file(s), lowercase hex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// File size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    /// Version string extracted from PE/manifest if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Mod type identifier (e.g., "archive", "dll", "redmod", "cet").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_type: Option<String>,

    /// Whether this plugin is enabled. Optional because some formats
    /// don't track enabled state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Position in the load order. Optional because some formats
    /// are ordered implicitly by file position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
}

impl LoadOrderEntry {
    /// Creates a builder starting with the mod name.
    pub fn builder(name: impl Into<String>) -> LoadOrderEntryBuilder {
        LoadOrderEntryBuilder::new(name)
    }

    /// Creates a new load order entry with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fingerprint: None,
            size: None,
            version: None,
            mod_type: None,
            enabled: None,
            index: None,
        }
    }

    /// Creates a new enabled/disabled entry.
    pub fn with_enabled(name: impl Into<String>, enabled: bool) -> Self {
        Self {
            name: name.into(),
            fingerprint: None,
            size: None,
            version: None,
            mod_type: None,
            enabled: Some(enabled),
            index: None,
        }
    }

    /// Creates a fully specified entry.
    pub fn full(name: impl Into<String>, enabled: bool, index: u32) -> Self {
        Self {
            name: name.into(),
            fingerprint: None,
            size: None,
            version: None,
            mod_type: None,
            enabled: Some(enabled),
            index: Some(index),
        }
    }
}

/// Builder for constructing LoadOrderEntry with optional fields.
pub struct LoadOrderEntryBuilder {
    name: String,
    fingerprint: Option<String>,
    size: Option<u64>,
    version: Option<String>,
    mod_type: Option<String>,
    enabled: Option<bool>,
    index: Option<u32>,
}

impl LoadOrderEntryBuilder {
    /// Creates a new builder with the mod name.
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fingerprint: None,
            size: None,
            version: None,
            mod_type: None,
            enabled: None,
            index: None,
        }
    }

    /// Sets the xxh3 fingerprint.
    pub fn fingerprint(mut self, fp: impl Into<String>) -> Self {
        self.fingerprint = Some(fp.into());
        self
    }

    /// Sets the file size in bytes.
    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the version string.
    pub fn version(mut self, v: impl Into<String>) -> Self {
        self.version = Some(v.into());
        self
    }

    /// Sets the mod type identifier.
    pub fn mod_type(mut self, t: impl Into<String>) -> Self {
        self.mod_type = Some(t.into());
        self
    }

    /// Sets whether the mod is enabled.
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = Some(e);
        self
    }

    /// Sets the load order index.
    pub fn index(mut self, i: u32) -> Self {
        self.index = Some(i);
        self
    }

    /// Builds the LoadOrderEntry.
    pub fn build(self) -> LoadOrderEntry {
        LoadOrderEntry {
            name: self.name,
            fingerprint: self.fingerprint,
            size: self.size,
            version: self.version,
            mod_type: self.mod_type,
            enabled: self.enabled,
            index: self.index,
        }
    }
}

/// A complete load order as a list of entries.
///
/// This gets serialized to JSON and sent as the `loadOrderJson` string field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoadOrder(pub Vec<LoadOrderEntry>);

impl LoadOrder {
    /// Creates a new empty load order.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Creates a load order from a vector of entries.
    pub fn from_entries(entries: Vec<LoadOrderEntry>) -> Self {
        Self(entries)
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Adds an entry.
    pub fn push(&mut self, entry: LoadOrderEntry) {
        self.0.push(entry);
    }

    /// Returns an iterator over entries.
    pub fn iter(&self) -> impl Iterator<Item = &LoadOrderEntry> {
        self.0.iter()
    }

    /// Serializes to JSON string for the API's `loadOrderJson` field.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.0)
    }

    /// Deserializes from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let entries: Vec<LoadOrderEntry> = serde_json::from_str(json)?;
        Ok(Self(entries))
    }
}

impl IntoIterator for LoadOrder {
    type Item = LoadOrderEntry;
    type IntoIter = std::vec::IntoIter<LoadOrderEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a LoadOrder {
    type Item = &'a LoadOrderEntry;
    type IntoIter = std::slice::Iter<'a, LoadOrderEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<LoadOrderEntry> for LoadOrder {
    fn from_iter<I: IntoIterator<Item = LoadOrderEntry>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_minimal() {
        let entry = LoadOrderEntry::new("Test.esp");
        assert_eq!(entry.name, "Test.esp");
        assert!(entry.enabled.is_none());
        assert!(entry.index.is_none());
    }

    #[test]
    fn entry_with_enabled() {
        let entry = LoadOrderEntry::with_enabled("Test.esp", true);
        assert_eq!(entry.enabled, Some(true));
    }

    #[test]
    fn entry_full() {
        let entry = LoadOrderEntry::full("Test.esp", true, 5);
        assert_eq!(entry.enabled, Some(true));
        assert_eq!(entry.index, Some(5));
    }

    #[test]
    fn load_order_json_roundtrip() {
        let mut lo = LoadOrder::new();
        lo.push(LoadOrderEntry::full("Skyrim.esm", true, 0));
        lo.push(LoadOrderEntry::full("Update.esm", true, 1));
        lo.push(LoadOrderEntry::with_enabled("SkyUI_SE.esp", true));

        let json = lo.to_json().unwrap();
        let parsed = LoadOrder::from_json(&json).unwrap();

        assert_eq!(lo, parsed);
    }

    #[test]
    fn json_skips_none_fields() {
        let entry = LoadOrderEntry::new("Test.esp");
        let json = serde_json::to_string(&entry).unwrap();

        // Should not contain "enabled" or "index" keys
        assert!(!json.contains("enabled"));
        assert!(!json.contains("index"));
    }

    #[test]
    fn collect_from_iter() {
        let entries = vec![LoadOrderEntry::new("A.esp"), LoadOrderEntry::new("B.esp")];
        let lo: LoadOrder = entries.into_iter().collect();
        assert_eq!(lo.len(), 2);
    }

    #[test]
    fn builder_all_fields() {
        let entry = LoadOrderEntry::builder("test.dll")
            .fingerprint("a1b2c3d4e5f6a7b8")
            .size(12345)
            .version("1.0.0")
            .mod_type("dll")
            .enabled(true)
            .index(5)
            .build();

        assert_eq!(entry.name, "test.dll");
        assert_eq!(entry.fingerprint, Some("a1b2c3d4e5f6a7b8".to_string()));
        assert_eq!(entry.size, Some(12345));
        assert_eq!(entry.version, Some("1.0.0".to_string()));
        assert_eq!(entry.mod_type, Some("dll".to_string()));
        assert_eq!(entry.enabled, Some(true));
        assert_eq!(entry.index, Some(5));
    }

    #[test]
    fn builder_minimal() {
        let entry = LoadOrderEntry::builder("test.esp").build();

        assert_eq!(entry.name, "test.esp");
        assert!(entry.fingerprint.is_none());
        assert!(entry.size.is_none());
    }

    #[test]
    fn json_skips_none_new_fields() {
        let entry = LoadOrderEntry::new("Test.esp");
        let json = serde_json::to_string(&entry).unwrap();

        assert!(!json.contains("fingerprint"));
        assert!(!json.contains("size"));
        assert!(!json.contains("version"));
        assert!(!json.contains("mod_type"));
    }

    #[test]
    fn json_roundtrip_with_new_fields() {
        let entry = LoadOrderEntry::builder("test.dll")
            .fingerprint("abc123")
            .size(999)
            .build();

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: LoadOrderEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry, parsed);
    }

    #[test]
    fn backwards_compat_old_json() {
        // Old format without new fields should still parse
        let old_json = r#"{"name":"Test.esp","enabled":true,"index":0}"#;
        let entry: LoadOrderEntry = serde_json::from_str(old_json).unwrap();

        assert_eq!(entry.name, "Test.esp");
        assert!(entry.fingerprint.is_none());
    }
}
