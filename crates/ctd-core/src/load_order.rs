//! Load order parsing and management.
//!
//! This module handles parsing and representing game mod load orders.

use serde::{Deserialize, Serialize};

use crate::Result;

/// Represents a single entry in a load order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadOrderEntry {
    /// The name or identifier of the mod/plugin.
    pub name: String,
    /// Whether this entry is enabled in the load order.
    pub enabled: bool,
    /// Optional index position in the load order.
    pub index: Option<usize>,
}

impl LoadOrderEntry {
    /// Creates a new load order entry.
    pub fn new(name: impl Into<String>, enabled: bool) -> Self {
        Self {
            name: name.into(),
            enabled,
            index: None,
        }
    }

    /// Creates a new load order entry with a specific index.
    pub fn with_index(name: impl Into<String>, enabled: bool, index: usize) -> Self {
        Self {
            name: name.into(),
            enabled,
            index: Some(index),
        }
    }
}

/// Represents a complete load order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadOrder {
    /// The entries in the load order.
    pub entries: Vec<LoadOrderEntry>,
}

impl LoadOrder {
    /// Creates a new empty load order.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a load order from a string representation.
    ///
    /// # Errors
    ///
    /// Returns `CtdError::LoadOrderParse` if the input cannot be parsed.
    pub fn parse(_input: &str) -> Result<Self> {
        // TODO: Implement actual parsing logic
        Ok(Self::new())
    }

    /// Returns the number of entries in the load order.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the load order is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Adds an entry to the load order.
    pub fn push(&mut self, entry: LoadOrderEntry) {
        self.entries.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_load_order_is_empty() {
        let lo = LoadOrder::new();
        assert!(lo.is_empty());
        assert_eq!(lo.len(), 0);
    }

    #[test]
    fn push_entry() {
        let mut lo = LoadOrder::new();
        lo.push(LoadOrderEntry::new("test_mod", true));
        assert_eq!(lo.len(), 1);
        assert!(!lo.is_empty());
    }

    #[test]
    fn entry_with_index() {
        let entry = LoadOrderEntry::with_index("test_mod", true, 5);
        assert_eq!(entry.index, Some(5));
    }

    #[test]
    fn parse_returns_empty() {
        let lo = LoadOrder::parse("").unwrap();
        assert!(lo.is_empty());
    }
}
