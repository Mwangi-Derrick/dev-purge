//! Default implementations of domain traits.
//!
//! This module provides concrete implementations that can be used
//! directly or as examples for custom implementations.

use std::path::Path;
use std::sync::Mutex;
use anyhow::Result;
use rayon::prelude::*;
use walkdir::WalkDir;

use super::config::{matches_any_pattern, Pattern, PurgeConfig};
use super::os;
use super::traits::{Scanner, SafetyChecker, Cleaner, ScanResult, CleanupStats, CleanupCategory};
use super::types::{Finding, DeleteStats};

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
        let results: Mutex<Vec<ScanResult>> = Mutex::new(Vec::new());

        WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .par_bridge()
            .try_for_each(|entry| -> Result<()> {
                let entry = entry?;
                let path = entry.path();

                if entry.file_type().is_dir() && matches_any_pattern(path, entry.file_name(), patterns) {
                    if let Ok(metadata) = entry.metadata() {
                        let size = estimate_dir_size(path)?;
                        let result = ScanResult {
                            path: path.to_path_buf(),
                            size_bytes: size,
                            category: CleanupCategory::BuildArtifact, // TODO: categorize based on pattern
                        };
                        results.lock().unwrap().push(result);
                    }
                }
                Ok(())
            })?;

        let mut final_results = results.into_inner().unwrap();
        final_results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        Ok(final_results)
    }
}

/// Default safety checker using OS-specific rules.
pub struct OsSafetyChecker;

impl SafetyChecker for OsSafetyChecker {
    fn is_safe(&self, path: &Path) -> bool {
        !os::is_protected_root(path) && !path.components().any(|comp| {
            comp.as_os_str().to_str().map_or(false, |s| os::is_protected_entry_name(s.as_ref()))
        })
    }
}

/// Default cleaner that deletes files and reports progress.
pub struct StandardCleaner;

impl Cleaner for StandardCleaner {
    fn clean(&self, results: &[ScanResult], dry_run: bool) -> Result<CleanupStats> {
        let mut stats = CleanupStats {
            total_bytes_freed: 0,
            items_deleted: 0,
            errors: Vec::new(),
        };

        for result in results {
            if dry_run {
                println!("[DRY RUN] Would delete: {} ({} bytes)", result.path.display(), result.size_bytes);
                stats.total_bytes_freed += result.size_bytes;
                stats.items_deleted += 1;
            } else {
                match std::fs::remove_dir_all(&result.path) {
                    Ok(_) => {
                        println!("✓ Deleted: {} ({} bytes)", result.path.display(), result.size_bytes);
                        stats.total_bytes_freed += result.size_bytes;
                        stats.items_deleted += 1;
                    }
                    Err(e) => {
                        let error = format!("Failed to delete {}: {}", result.path.display(), e);
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