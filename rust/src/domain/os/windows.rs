//! Windows-specific path protection rules.

use super::{is_category_protected, ProtectedPathCategory};
use crate::domain::traits::ScanTier;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub fn is_safe(path: &Path, tier: ScanTier) -> bool {
    if is_protected_root(path, tier) {
        return false;
    }

    let path_str = path.to_string_lossy().to_string().replace('\\', "/");

    // Stricter path-based checks for extensions and programs
    if path_str.contains(".vscode/extensions")
        || path_str.contains(".antigravity/extensions")
        || path_str.contains(".cursor/extensions")
        || path_str.contains("AppData/Local/Programs")
    {
        return false;
    }

    !path.components().any(|comp| {
        comp.as_os_str()
            .to_str()
            .is_some_and(|s| is_protected_entry_name(OsStr::new(s), tier))
    })
}

pub fn is_protected_entry_name(name: &OsStr, tier: ScanTier) -> bool {
    let name_str = match name.to_str() {
        Some(v) => v.to_lowercase(),
        None => return false,
    };

    RULES.iter().any(|(cat, n, _)| {
        if let Some(rule_name) = n {
            rule_name.to_lowercase() == name_str && is_category_protected(*cat, tier)
        } else {
            false
        }
    })
}

pub fn is_protected_root(path: &Path, tier: ScanTier) -> bool {
    let normalized = path.to_string_lossy().to_lowercase().replace('/', "\\");

    // Check system roots
    for (cat, _, root) in RULES {
        if let Some(prefix) = root {
            let prefix = prefix.to_lowercase();
            if is_category_protected(*cat, tier) {
                if normalized == prefix || normalized.starts_with(&format!("{}\\", prefix)) {
                    return true;
                }
            }
        }
    }

    is_protected_home_subpath(path, tier)
}

fn is_protected_home_subpath(path: &Path, tier: ScanTier) -> bool {
    let local_app_data = env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let app_data = env::var_os("APPDATA").map(PathBuf::from);
    let user_profile = env::var_os("USERPROFILE").map(PathBuf::from);

    let home_subdirs = if tier >= ScanTier::Deep {
        vec![]
    } else if tier >= ScanTier::Cache {
        vec![".config", ".vscode", ".idea", ".cursor"]
    } else {
        vec![".cargo", ".config", ".vscode", ".idea", ".cursor"]
    };

    if let Some(profile) = user_profile {
        for subdir in home_subdirs {
            let candidate = profile.join(subdir);
            if path == candidate || path.starts_with(&candidate) {
                return true;
            }
        }
    }

    if tier < ScanTier::Deep {
        if let Some(local) = local_app_data {
            if path == local
                || path.starts_with(&local.join("Local"))
                || path.starts_with(&local.join("LocalLow"))
                || path.starts_with(&local.join("Temp"))
            {
                return true;
            }
        }
        if let Some(roaming) = app_data {
            if path == roaming || path.starts_with(&roaming) {
                return true;
            }
        }
    }

    false
}

const RULES: &[(ProtectedPathCategory, Option<&str>, Option<&str>)] = &[
    (ProtectedPathCategory::System, None, Some("c:\\windows")),
    (
        ProtectedPathCategory::System,
        None,
        Some("c:\\program files"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("c:\\program files (x86)"),
    ),
    (ProtectedPathCategory::System, Some("Programs"), None),
    (ProtectedPathCategory::IdeConfig, Some(".vscode"), None),
    (
        ProtectedPathCategory::IdeExtension,
        Some("extensions"),
        None,
    ),
    (ProtectedPathCategory::IdeConfig, Some(".idea"), None),
    (ProtectedPathCategory::IdeConfig, Some(".cursor"), None),
    (ProtectedPathCategory::IdeConfig, Some("AppData"), None),
    (ProtectedPathCategory::ProjectMetadata, Some(".git"), None),
    (
        ProtectedPathCategory::ProjectMetadata,
        Some(".github"),
        None,
    ),
    (
        ProtectedPathCategory::ProjectMetadata,
        Some(".gitignore"),
        None,
    ),
    (ProtectedPathCategory::SecretConfig, Some(".env"), None),
    (ProtectedPathCategory::ToolBinary, Some(".cargo"), None),
    (ProtectedPathCategory::ToolCache, Some(".cargo"), None),
    (ProtectedPathCategory::ToolCache, Some(".npm"), None),
];
