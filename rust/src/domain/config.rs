//! Configuration for the purge tool.
//!
//! This is a thin wrapper around the patterns module.
//! It provides the PurgeConfig struct that the core uses.

use crate::domain::patterns::{Pattern, PurgeConfig as PatternsConfig};
use crate::domain::traits::ScanTier;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Clone)]
pub struct PurgeConfig {
    patterns: Vec<Pattern>,
    pub tier: ScanTier,
}

impl PurgeConfig {
    /// Create configuration for a specific scan tier
    pub fn for_tier(tier: ScanTier) -> Self {
        let all_patterns = PatternsConfig::patterns();
        let filtered_patterns = all_patterns
            .iter()
            .filter(|p| p.tier <= tier)
            .cloned()
            .collect();

        Self {
            patterns: filtered_patterns,
            tier,
        }
    }

    /// Create configuration with hardcoded defaults (Project tier)
    pub fn hardcoded() -> Self {
        Self::for_tier(ScanTier::Project)
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
