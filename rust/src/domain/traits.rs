//! Domain traits for dev-purge operations.
//!
//! This module defines the core abstractions for scanning, safety checking,
//! and cleaning operations. Implement these traits to extend dev-purge
//! with new scanning strategies, safety rules, or cleanup methods.
//!
//! ## Implementing New Scanners
//!
//! ```rust
//! use dev_purge::domain::traits::{Scanner, ScanResult};
//!
//! pub struct CustomScanner;
//!
//! impl Scanner for CustomScanner {
//!     fn scan(&self, root: &std::path::Path) -> anyhow::Result<Vec<ScanResult>> {
//!         // Your scanning logic here
//!         Ok(vec![])
//!     }
//! }
//! ```

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Result of scanning a single path for potential cleanup.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub category: CleanupCategory,
    pub artifact_type: ArtifactType,
}

/// Type of artifact, either a physical file path or a virtual resource (like Docker).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    Physical,
    DockerImage(String),     // Image ID
    DockerContainer(String), // Container ID
    DockerVolume(String),    // Volume Name
}

/// Categories of files/directories that can be cleaned up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupCategory {
    BuildArtifact,
    Cache,
    Log,
    Temp,
    Other,
}

/// Statistics from a cleanup operation.
#[derive(Debug, Clone)]
pub struct CleanupStats {
    pub total_bytes_freed: u64,
    pub items_deleted: usize,
    pub errors: Vec<String>,
}

/// Trait for scanning directories for cleanup candidates.
pub trait Scanner {
    /// Scan the given root directory and return potential cleanup targets.
    fn scan(&self, root: &Path) -> Result<Vec<ScanResult>>;
}

/// Tiers of scanning depth and safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScanTier {
    /// Only project-level build artifacts (safest).
    Project,
    /// Include global tool caches (e.g., ~/.cargo, ~/.npm).
    Cache,
    /// Include application caches (e.g., Library/Caches, AppData).
    Deep,
    /// Include system-level caches (e.g., Docker, package managers).
    Aggressive,
}

/// Trait for checking if paths are safe to delete.
pub trait SafetyChecker {
    /// Return true if the path is safe to delete at the given tier.
    fn is_safe(&self, path: &Path, tier: ScanTier) -> bool;
}

/// Trait for performing cleanup operations.
pub trait Cleaner {
    /// Clean up the given scan results. If dry_run is true, only simulate.
    /// If permanent is true, delete files immediately instead of moving to trash.
    fn clean(&self, results: &[ScanResult], dry_run: bool, permanent: bool)
        -> Result<CleanupStats>;
}
