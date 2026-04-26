//! Configuration for the purge tool.
//!
//! This is a thin wrapper around the patterns module.
//! It provides the PurgeConfig struct that the core uses.

use crate::domain::patterns::{Pattern, PurgeConfig as PatternsConfig};
use std::ffi::OsStr;
use std::path::Path;

#[derive(Clone)]
pub struct PurgeConfig {
    patterns: Vec<Pattern>,
}

impl PurgeConfig {
    /// Create configuration with hardcoded defaults
    pub fn hardcoded() -> Self {
        Self {
            patterns: PatternsConfig::patterns().to_vec(),
        }
    }

    /// Get all patterns
    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

/// Check if a path matches any configured pattern
pub fn matches_any_pattern(path: &Path, name: &OsStr, patterns: &[Pattern]) -> bool {
    crate::domain::patterns::matches_any_pattern(path, name, patterns)
}


   