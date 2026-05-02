//! Linux-specific path protection rules.

use super::is_category_protected;
use super::unix::{is_protected_home_subpath, BASE_RULES};
use crate::domain::traits::ScanTier;
use std::path::Path;

pub fn is_safe(path: &Path, tier: ScanTier) -> bool {
    if is_protected_root(path, tier) {
        return false;
    }

    let path_str = path.to_string_lossy();
    if path_str.contains(".vscode/extensions")
        || path_str.contains(".antigravity/extensions")
        || path_str.contains(".cursor/extensions")
    {
        return false;
    }

    !path.components().any(|comp| {
        comp.as_os_str()
            .to_str()
            .is_some_and(|s| super::unix::is_protected_entry_name(std::ffi::OsStr::new(s), tier))
    })
}

pub fn is_protected_root(path: &Path, tier: ScanTier) -> bool {
    for (cat, _, root) in BASE_RULES {
        if let Some(prefix) = root {
            if is_category_protected(*cat, tier) {
                if path == Path::new(prefix) || path.starts_with(prefix) {
                    return true;
                }
            }
        }
    }
    is_protected_home_subpath(path, tier)
}
