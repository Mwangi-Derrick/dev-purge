//! Default implementations of domain traits.
//!
//! This module provides concrete implementations that can be used
//! directly or as examples for custom implementations.
//!
//! ## Extending dev-purge
//!
//! To add a new scanner for a specific tool:
//!
//! ```rust
//! use dev_purge::domain::traits::{Scanner, ScanResult};
//!
//! pub struct DockerScanner;
//!
//! impl Scanner for DockerScanner {
//!     fn scan(&self, root: &std::path::Path) -> anyhow::Result<Vec<ScanResult>> {
//!         // Scan for Docker-related artifacts
//!         Ok(vec![])
//!     }
//! }
//! ```

use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;

use super::config::{matches_any_pattern, PurgeConfig};
use super::os;
use super::traits::{
    ArtifactType, Cleaner, CleanupCategory, CleanupStats, SafetyChecker, ScanResult, ScanTier,
    Scanner,
};

/// Default scanner using walkdir and rayon for parallel processing.
pub struct ParallelScanner {
    config: PurgeConfig,
}

impl ParallelScanner {
    pub fn new(config: PurgeConfig) -> Self {
        Self { config }
    }
}

impl Scanner for ParallelScanner {
    fn scan(&self, root: &Path) -> Result<Vec<ScanResult>> {
        let patterns = self.config.patterns();
        let tier = self.config.tier;
        let results: Mutex<Vec<ScanResult>> = Mutex::new(Vec::new());

        WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Early skip of protected entries to save time and avoid permission errors
                let name = e.file_name();
                !os::is_protected_entry_name(name, tier)
            })
            .par_bridge()
            .for_each(|entry| {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => return, // Skip entries we can't access
                };
                let path = entry.path();

                if entry.file_type().is_dir()
                    && matches_any_pattern(path, entry.file_name(), patterns)
                {
                    if let Ok(_metadata) = entry.metadata() {
                        if let Ok(size) = estimate_dir_size(path) {
                            let result = ScanResult {
                                path: path.to_path_buf(),
                                size_bytes: size,
                                category: CleanupCategory::BuildArtifact, // TODO: categorize based on pattern
                                artifact_type: ArtifactType::Physical,
                            };
                            results.lock().unwrap().push(result);
                        }
                    }
                }
            });

        let mut final_results = results.into_inner().unwrap();
        final_results.sort_by_key(|r| std::cmp::Reverse(r.size_bytes));
        Ok(final_results)
    }
}

/// Default safety checker using OS-specific rules.
pub struct OsSafetyChecker;

impl SafetyChecker for OsSafetyChecker {
    fn is_safe(&self, path: &Path, tier: ScanTier) -> bool {
        !os::is_protected_root(path, tier)
            && !path.components().any(|comp| {
                comp.as_os_str()
                    .to_str()
                    .is_some_and(|s| os::is_protected_entry_name(s.as_ref(), tier))
            })
    }
}

/// Default cleaner that deletes files and reports progress.
pub struct StandardCleaner;

impl Cleaner for StandardCleaner {
    fn clean(
        &self,
        results: &[ScanResult],
        dry_run: bool,
        permanent: bool,
    ) -> Result<CleanupStats> {
        let mut stats = CleanupStats {
            total_bytes_freed: 0,
            items_deleted: 0,
            errors: Vec::new(),
        };

        for result in results {
            if dry_run {
                println!(
                    "[DRY RUN] Would {}delete: {} ({} bytes)",
                    if permanent { "permanently " } else { "" },
                    result.path.display(),
                    result.size_bytes
                );
                stats.total_bytes_freed += result.size_bytes;
                stats.items_deleted += 1;
            } else {
                let op_result = match &result.artifact_type {
                    ArtifactType::Physical => {
                        if permanent {
                            std::fs::remove_dir_all(&result.path).map_err(|e| anyhow::anyhow!(e))
                        } else {
                            trash::delete(&result.path).map_err(|e| anyhow::anyhow!(e))
                        }
                    }
                    ArtifactType::DockerImage(id) => {
                        let rt = tokio::runtime::Runtime::new()?;
                        let docker = bollard::Docker::connect_with_local_defaults()?;
                        rt.block_on(docker.remove_image(id, None, None))
                            .map(|_| ()) // Convert Vec<_> to () iterates over the results and returns () if successful
                            .map_err(|e| anyhow::anyhow!(e))
                    }
                    ArtifactType::DockerContainer(id) => {
                        let rt = tokio::runtime::Runtime::new()?;
                        let docker = bollard::Docker::connect_with_local_defaults()?;
                        rt.block_on(docker.remove_container(id, None))
                            .map_err(|e| anyhow::anyhow!(e))
                    }
                    _ => Ok(()), // Volumes not implemented yet
                };

                match op_result {
                    Ok(_) => {
                        println!(
                            "✓ {}: {} ({} bytes)",
                            match &result.artifact_type {
                                ArtifactType::Physical =>
                                    if permanent {
                                        "Permanently deleted"
                                    } else {
                                        "Moved to trash"
                                    },
                                ArtifactType::DockerImage(_) => "Removed Docker image",
                                ArtifactType::DockerContainer(_) => "Removed Docker container",
                                _ => "Cleaned",
                            },
                            result.path.display(),
                            result.size_bytes
                        );
                        stats.total_bytes_freed += result.size_bytes;
                        stats.items_deleted += 1;
                    }
                    Err(e) => {
                        let error = format!("Failed to clean {}: {}", result.path.display(), e);
                        eprintln!("✗ {}", error);
                        stats.errors.push(error);
                    }
                }
            }
        }

        Ok(stats)
    }
}

/// Estimate the size of a directory recursively.
fn estimate_dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                size += metadata.len();
            }
        }
    }
    Ok(size)
}
