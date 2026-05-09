use crate::domain::traits::{ArtifactType, ScanResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct ReduceOptions {
    /// If a directory contains >= this many sibling `out/` directories, collapse them
    /// to the cargo `.../build/` directory.
    pub cargo_out_promotion_min: usize,
}

impl Default for ReduceOptions {
    fn default() -> Self {
        Self {
            cargo_out_promotion_min: 8,
        }
    }
}

/// Reduce redundant scan results by:
/// 1) Removing descendants when an ancestor is already selected.
/// 2) Promoting cargo build-script `.../build/*/out` directories to `.../build/` when many exist.
pub fn reduce_scan_results(mut results: Vec<ScanResult>, opts: ReduceOptions) -> Vec<ScanResult> {
    results = promote_cargo_out_dirs(results, opts.cargo_out_promotion_min);
    remove_descendants(results)
}

fn promote_cargo_out_dirs(results: Vec<ScanResult>, min_siblings: usize) -> Vec<ScanResult> {
    let mut by_build_dir: HashMap<PathBuf, Vec<usize>> = HashMap::new();
    let mut kept: Vec<Option<ScanResult>> = results.into_iter().map(Some).collect();

    for (idx, maybe) in kept.iter().enumerate() {
        let Some(r) = maybe.as_ref() else { continue };
        if r.artifact_type != ArtifactType::Physical {
            continue;
        }

        let path = &r.path;
        if !is_out_dir(path) {
            continue;
        }

        let Some(build_dir) = cargo_build_dir_for_out(path) else {
            continue;
        };
        by_build_dir.entry(build_dir).or_default().push(idx);
    }

    for (build_dir, indices) in by_build_dir {
        if indices.len() < min_siblings {
            continue;
        }

        // Replace the first `out/` with the build dir and drop the rest.
        let mut first = true;
        for idx in indices {
            if first {
                first = false;
                if let Some(mut r) = kept[idx].take() {
                    r.path = build_dir.clone();
                    r.size_bytes = estimate_dir_size_bytes(&build_dir);
                    kept[idx] = Some(r);
                }
            } else {
                kept[idx] = None;
            }
        }
    }

    kept.into_iter().flatten().collect()
}

fn remove_descendants(mut results: Vec<ScanResult>) -> Vec<ScanResult> {
    // Prefer stable behavior: sort by path string, then keep the shortest roots.
    results.sort_by(|a, b| a.path.cmp(&b.path));

    let mut reduced: Vec<ScanResult> = Vec::with_capacity(results.len());
    for r in results {
        if r.artifact_type != ArtifactType::Physical {
            reduced.push(r);
            continue;
        }

        let is_descendant = reduced.iter().any(|kept| {
            kept.artifact_type == ArtifactType::Physical
                && r.path != kept.path
                && is_strict_descendant(&r.path, &kept.path)
        });
        if !is_descendant {
            reduced.push(r);
        }
    }

    // Re-sort for UI: biggest first.
    reduced.sort_by_key(|r| std::cmp::Reverse(r.size_bytes));
    reduced
}

fn is_strict_descendant(path: &Path, ancestor: &Path) -> bool {
    path.starts_with(ancestor) && path.components().count() > ancestor.components().count()
}

fn is_out_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("out"))
}

/// Recognize Cargo build-script output directories:
/// - `.../target/{debug|release}/build/<crate>/out`
/// - `.../{debug|release}/build/<crate>/out` (when `CARGO_TARGET_DIR` is set)
///
/// Returns the `.../build/` directory to delete instead of many `out/` dirs.
fn cargo_build_dir_for_out(out_dir: &Path) -> Option<PathBuf> {
    let crate_dir = out_dir.parent()?;
    let build_dir = crate_dir.parent()?;

    if !build_dir
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("build"))
    {
        return None;
    }

    let parent = build_dir.parent();

    // Variant A: .../target/<profile>/build/<crate>/out
    if let Some(parent) = parent {
        if is_cargo_profile_dir(parent) && parent.parent().is_some_and(|p| is_named(p, "target")) {
            return Some(build_dir.to_path_buf());
        }
    }

    // Variant B: .../<profile>/build/<crate>/out
    if build_dir.parent().is_some_and(is_cargo_profile_dir) {
        return Some(build_dir.to_path_buf());
    }

    None
}

fn is_named(path: &Path, name: &str) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case(name))
}

fn is_cargo_profile_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| matches!(n.to_lowercase().as_str(), "debug" | "release"))
}

fn estimate_dir_size_bytes(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    let it = walkdir::WalkDir::new(dir).follow_links(false).into_iter();
    for entry in it {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            if let Ok(md) = entry.metadata() {
                total = total.saturating_add(md.len());
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::traits::{CleanupCategory, ScanResult};

    fn sr(path: &str, bytes: u64) -> ScanResult {
        ScanResult {
            path: PathBuf::from(path),
            size_bytes: bytes,
            category: CleanupCategory::BuildArtifact,
            artifact_type: ArtifactType::Physical,
        }
    }

    #[test]
    fn reduces_descendants_when_parent_selected() {
        let results = vec![sr("C:/p/target", 100), sr("C:/p/target/debug", 50)];
        let reduced = reduce_scan_results(
            results,
            ReduceOptions {
                cargo_out_promotion_min: 99,
            },
        );
        assert_eq!(reduced.len(), 1);
        assert_eq!(reduced[0].path, PathBuf::from("C:/p/target"));
    }

    #[test]
    fn promotes_many_cargo_out_dirs_to_build_dir() {
        let mut results = Vec::new();
        for i in 0..10 {
            results.push(sr(&format!("C:/t/target/debug/build/crate{i}/out"), 10));
        }

        let reduced = reduce_scan_results(
            results,
            ReduceOptions {
                cargo_out_promotion_min: 8,
            },
        );

        assert_eq!(reduced.len(), 1);
        assert_eq!(reduced[0].path, PathBuf::from("C:/t/target/debug/build"));
    }
}
