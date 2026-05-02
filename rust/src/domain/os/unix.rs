//! Shared Unix-like (Linux/macOS) path protection rules.

use super::{is_category_protected, ProtectedPathCategory};
use crate::domain::traits::ScanTier;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn is_protected_entry_name(name: &OsStr, tier: ScanTier) -> bool {
    let name_str = match name.to_str() {
        Some(v) => v,
        None => return false,
    };

    BASE_RULES.iter().any(|(cat, n, _)| {
        if let Some(rule_name) = n {
            *rule_name == name_str && is_category_protected(*cat, tier)
        } else {
            false
        }
    })
}

pub fn is_protected_home_subpath(path: &Path, tier: ScanTier) -> bool {
    let home = env::var_os("HOME").map(PathBuf::from);
    let home_subdirs = if tier >= ScanTier::Deep {
        vec![".local"]
    } else if tier >= ScanTier::Cache {
        vec![".config", ".local", ".vscode", ".idea", ".cursor"]
    } else {
        vec![".cargo", ".config", ".local", ".vscode", ".idea", ".cursor"]
    };

    if let Some(home_path) = home {
        for subdir in home_subdirs {
            let candidate = home_path.join(subdir);
            if path == candidate || path.starts_with(&candidate) {
                return true;
            }
        }
    }
    false
}

pub const BASE_RULES: &[(ProtectedPathCategory, Option<&str>, Option<&str>)] = &[
    (ProtectedPathCategory::System, None, Some("/")),
    (ProtectedPathCategory::System, None, Some("/usr")),
    (ProtectedPathCategory::System, None, Some("/etc")),
    (ProtectedPathCategory::System, None, Some("/var")),
    (ProtectedPathCategory::System, None, Some("/bin")),
    (ProtectedPathCategory::System, None, Some("/sbin")),
    (ProtectedPathCategory::System, None, Some("/lib")),
    (ProtectedPathCategory::IdeConfig, Some(".vscode"), None),
    (
        ProtectedPathCategory::IdeExtension,
        Some("extensions"),
        None,
    ),
    (ProtectedPathCategory::IdeConfig, Some(".idea"), None),
    (ProtectedPathCategory::IdeConfig, Some(".cursor"), None),
    (ProtectedPathCategory::IdeConfig, Some(".config"), None),
    (ProtectedPathCategory::IdeConfig, Some(".cache"), None),
    (ProtectedPathCategory::ProjectMetadata, Some(".git"), None),
    (
        ProtectedPathCategory::ProjectMetadata,
        Some(".github"),
        None,
    ),
    (ProtectedPathCategory::SecretConfig, Some(".env"), None),
    (ProtectedPathCategory::ToolBinary, Some(".cargo"), None),
    (ProtectedPathCategory::ToolCache, Some(".cargo"), None),
    (ProtectedPathCategory::ToolCache, Some(".npm"), None),
];
