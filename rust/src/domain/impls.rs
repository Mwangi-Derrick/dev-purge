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
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::env;
use std::path::{Path, PathBuf};
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

    fn get_jump_points(&self, tier: ScanTier) -> Vec<PathBuf> {
        let mut points = Vec::new();

        if tier >= ScanTier::Cache {
            // Rust, Bun, Go, npm
            let home = env::var_os("HOME")
                .map(PathBuf::from)
                .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from));

            if let Some(home) = home {
                points.push(home.join(".cargo"));
                points.push(home.join(".bun"));
                points.push(home.join("go"));
                points.push(home.join(".npm"));
                points.push(home.join(".cache/pip"));
                points.push(home.join(".cache/uv"));
            }

            // Windows specific global caches
            if let Some(appdata) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
                points.push(appdata.join("npm-cache"));
                points.push(appdata.join("uv"));
                points.push(appdata.join("pip/cache"));
            }
        }

        if tier >= ScanTier::Deep {
            if let Some(temp) = env::var_os("TEMP").map(PathBuf::from) {
                points.push(temp);
            }
            #[cfg(unix)]
            points.push(PathBuf::from("/tmp"));

            if let Some(appdata) = env::var_os("LOCALAPPDATA").map(PathBuf::from) {
                points.push(appdata.join("Microsoft/TypeScript"));
            }
        }

        points
            .into_iter()
            .filter(|p| p.exists())
            .map(|p| std::fs::canonicalize(&p).unwrap_or(p))
            .collect()
    }
}

impl Scanner for ParallelScanner {
    fn scan(&self, root: &Path) -> Result<Vec<ScanResult>> {
        let patterns = self.config.patterns();
        let tier = self.config.tier;
        let results: Mutex<Vec<ScanResult>> = Mutex::new(Vec::new());

        let mut entry_points =
            vec![std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf())];
        entry_points.extend(self.get_jump_points(tier));
        entry_points.sort();
        entry_points.dedup();

        // Filter out entry points that are sub-paths of other entry points
        let mut final_entry_points = Vec::new();
        for i in 0..entry_points.len() {
            let mut is_subpath = false;
            for j in 0..entry_points.len() {
                if i != j && entry_points[i].starts_with(&entry_points[j]) {
                    is_subpath = true;
                    break;
                }
            }
            if !is_subpath {
                final_entry_points.push(entry_points[i].clone());
            }
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Scanning...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        final_entry_points.par_iter().for_each(|entry_root| {
            let mut it = WalkDir::new(entry_root).follow_links(false).into_iter();

            loop {
                let entry = match it.next() {
                    None => break,
                    Some(Ok(e)) => e,
                    Some(Err(_)) => continue,
                };

                let path = entry.path();
                let name = entry.file_name();

                // 1. Hard system exclusions
                let path_str = path.to_string_lossy().to_string().replace('\\', "/");
                if path_str.contains("/Windows")
                    || path_str.contains("/Program Files")
                    || path_str.contains("/$Recycle.Bin")
                    || path_str.contains("/msys64")
                    || path_str.contains("/proc")
                    || path_str.contains("/sys")
                    || path_str.contains("/dev")
                {
                    it.skip_current_dir();
                    continue;
                }

                // 2. Allow the root itself
                if path == entry_root {
                    continue;
                }

                // 3. Match patterns
                if entry.file_type().is_dir() && matches_any_pattern(path, name, patterns) {
                    if let Ok(size) = estimate_dir_size(path) {
                        results.lock().unwrap().push(ScanResult {
                            path: path.to_path_buf(),
                            size_bytes: size,
                            category: CleanupCategory::BuildArtifact,
                            artifact_type: ArtifactType::Physical,
                        });
                        pb.set_message(format!("Found {} items...", results.lock().unwrap().len()));
                    }
                    // IMPORTANT: Once matched, do NOT look inside. Skip contents.
                    it.skip_current_dir();
                    continue;
                }

                // 4. Safety skip for non-matches
                if entry.file_type().is_dir() && !os::is_safe(path, tier) {
                    it.skip_current_dir();
                    continue;
                }
            }
        });

        pb.finish_and_clear();

        let mut final_results = results.into_inner().unwrap();
        final_results.sort_by(|a, b| a.path.cmp(&b.path));
        final_results.dedup_by(|a, b| a.path == b.path);
        final_results.sort_by_key(|r| std::cmp::Reverse(r.size_bytes));
        Ok(final_results)
    }
}

/// Default safety checker using OS-specific rules.
pub struct OsSafetyChecker;

impl SafetyChecker for OsSafetyChecker {
    fn is_safe(&self, path: &Path, tier: ScanTier) -> bool {
        os::is_safe(path, tier)
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
                            trash::delete(&result.path).map_err(|e| {
                                #[cfg(windows)]
                                {
                                    anyhow::anyhow!("{}\n   Tip: Windows aborted the trash operation. Files might be in use or paths too long.\n   Try running with: dev-purge --permanent", e)
                                }
                                #[cfg(not(windows))]
                                { anyhow::anyhow!(e) }
                            })
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
