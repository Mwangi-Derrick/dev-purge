use crate::domain::config::{is_protected_entry_name, matches_any_pattern, Pattern, PurgeConfig};
use anyhow::Result;
use colored::Colorize;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub fn scan(root: &Path, config: &PurgeConfig) -> Result<Vec<PathBuf>> {
    let candidates: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());
    let patterns = config.patterns();

    let mut it = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| scan_filter_entry(e, patterns, &candidates));

    while let Some(entry) = it.next() {
        if let Err(err) = entry {
            eprintln!("{} {}", "warning:".yellow(), err);
        }
    }

    let mut out = candidates.into_inner();
    out.sort();
    out.dedup();
    Ok(out)
}

fn scan_filter_entry(
    entry: &DirEntry,
    patterns: &[Pattern],
    candidates: &RefCell<Vec<PathBuf>>,
) -> bool {
    let name = entry.file_name();

    if is_protected_entry_name(name) {
        return false;
    }

    if entry.file_type().is_dir() && matches_any_pattern(name, patterns) {
        candidates.borrow_mut().push(entry.path().to_path_buf());
        return false; // don't descend into it
    }

    true
}
