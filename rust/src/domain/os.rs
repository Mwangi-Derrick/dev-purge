//! OS-specific path protection rules for dev-purge.
//!
//! This module defines platform-aware rules for protecting system directories,
//! IDE configurations, tool binaries, and caches from accidental deletion.
//!
//! ## Adding New Rules
//!
//! To add protection for a new path or tool:
//!
//! 1. Choose the appropriate `ProtectedPathCategory`
//! 2. Select the `OsFamily` (Any, Unix, Windows, MacOS)
//! 3. Add a tuple to `PROTECTED_PATH_RULES`: (category, os, root_prefix, dir_name, description)
//! 4. For root_prefix: Use absolute paths like "/usr" or "c:\\windows"
//! 5. For dir_name: Use relative names like ".vscode" or "node_modules"
//!
//! ## Examples
//!
//! - Protect VS Code extensions: (IdeConfig, Any, None, Some(".vscode"), "...")
//! - Protect system binaries: (System, Unix, Some("/bin"), None, "...")
//! - Protect Windows app data: (IdeConfig, Windows, None, Some("AppData"), "...")

use crate::domain::traits::ScanTier;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub enum OsFamily {
    Any,
    Unix,
    Windows,
    MacOS,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtectedPathCategory {
    System,
    IdeConfig,
    IdeExtension,
    ToolBinary,
    ToolCache,
    ProjectMetadata,
    SecretConfig,
}

pub struct ProtectedPathRule {
    pub category: ProtectedPathCategory,
    pub os: OsFamily,
    pub root_prefix: Option<&'static str>,
    pub dir_name: Option<&'static str>,
    pub description: &'static str,
}

// Compact tuple: (category, os, root_prefix, dir_name, description)
type ProtectedPathRuleTuple = (
    ProtectedPathCategory,
    OsFamily,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
);

const PROTECTED_PATH_RULES: &[ProtectedPathRuleTuple] = &[
    // System-managed roots
    (
        ProtectedPathCategory::System,
        OsFamily::Any,
        Some("/"),
        None,
        "Unix root filesystem",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Unix,
        Some("/usr"),
        None,
        "Unix system binaries and libraries",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Unix,
        Some("/etc"),
        None,
        "Unix system configuration",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Unix,
        Some("/var"),
        None,
        "Unix variable data and caches",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Unix,
        Some("/bin"),
        None,
        "Unix system binaries",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Unix,
        Some("/sbin"),
        None,
        "Unix system binaries",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Unix,
        Some("/lib"),
        None,
        "Unix system libraries",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::MacOS,
        Some("/Applications"),
        None,
        "macOS application bundles",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::MacOS,
        Some("/Library"),
        None,
        "macOS system libraries and caches",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Windows,
        Some("c:\\windows"),
        None,
        "Windows system directory",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Windows,
        Some("c:\\program files"),
        None,
        "Windows program files",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Windows,
        Some("c:\\program files (x86)"),
        None,
        "Windows program files x86",
    ),
    (
        ProtectedPathCategory::System,
        OsFamily::Windows,
        None,
        Some("Programs"),
        "Windows programs installation",
    ),
    // IDE configurations and caches
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::Any,
        None,
        Some(".vscode"),
        "VS Code settings",
    ),
    (
        ProtectedPathCategory::IdeExtension,
        OsFamily::Any,
        None,
        Some("extensions"),
        "IDE extensions",
    ),
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::Any,
        None,
        Some(".idea"),
        "JetBrains IDE settings",
    ),
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::Any,
        None,
        Some(".cursor"),
        "Cursor IDE settings",
    ),
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::Any,
        None,
        Some(".config"),
        "User application configuration",
    ),
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::Unix,
        None,
        Some(".cache"),
        "User cache directory (Linux)",
    ),
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::MacOS,
        None,
        Some("Library"),
        "macOS user library",
    ),
    (
        ProtectedPathCategory::IdeConfig,
        OsFamily::Windows,
        None,
        Some("AppData"),
        "Windows app data",
    ),
    // Project metadata
    (
        ProtectedPathCategory::ProjectMetadata,
        OsFamily::Any,
        None,
        Some(".git"),
        "Git repository metadata",
    ),
    (
        ProtectedPathCategory::ProjectMetadata,
        OsFamily::Any,
        None,
        Some(".github"),
        "GitHub workflow metadata",
    ),
    (
        ProtectedPathCategory::ProjectMetadata,
        OsFamily::Any,
        None,
        Some(".gitignore"),
        "Git ignore file",
    ),
    (
        ProtectedPathCategory::ProjectMetadata,
        OsFamily::Any,
        None,
        Some(".editorconfig"),
        "Editor configuration",
    ),
    // Secret configurations
    (
        ProtectedPathCategory::SecretConfig,
        OsFamily::Any,
        None,
        Some(".env"),
        "Environment variables and secrets",
    ),
    (
        ProtectedPathCategory::SecretConfig,
        OsFamily::Any,
        None,
        Some(".env.local"),
        "Local environment variables",
    ),
    (
        ProtectedPathCategory::SecretConfig,
        OsFamily::Any,
        None,
        Some(".env.production"),
        "Production secrets",
    ),
    // Tool binaries and configurations
    (
        ProtectedPathCategory::ToolBinary,
        OsFamily::Any,
        None,
        Some(".cargo"),
        "Cargo configuration and binaries",
    ),
    (
        ProtectedPathCategory::ToolBinary,
        OsFamily::Any,
        None,
        Some(".npm-global"),
        "Global npm binaries",
    ),
    (
        ProtectedPathCategory::ToolBinary,
        OsFamily::Any,
        None,
        Some(".local"),
        "Local user binaries",
    ),
    (
        ProtectedPathCategory::ToolBinary,
        OsFamily::Any,
        None,
        Some("go"),
        "Go installation directory",
    ),
    (
        ProtectedPathCategory::ToolBinary,
        OsFamily::Any,
        None,
        Some(".gradle"),
        "Gradle configuration",
    ),
    (
        ProtectedPathCategory::ToolBinary,
        OsFamily::Any,
        None,
        Some(".m2"),
        "Maven repository",
    ),
    // Tool caches (safe to protect to avoid accidental deletion)
    (
        ProtectedPathCategory::ToolCache,
        OsFamily::Any,
        None,
        Some(".cargo"),
        "Cargo registry cache",
    ),
    (
        ProtectedPathCategory::ToolCache,
        OsFamily::Any,
        None,
        Some(".npm"),
        "npm cache",
    ),
    (
        ProtectedPathCategory::ToolCache,
        OsFamily::Any,
        None,
        Some(".gradle"),
        "Gradle caches",
    ),
    (
        ProtectedPathCategory::ToolCache,
        OsFamily::Any,
        None,
        Some(".m2"),
        "Maven local repository",
    ),
];

fn is_category_protected(category: ProtectedPathCategory, tier: ScanTier) -> bool {
    match tier {
        ScanTier::Project => true, // Everything in registry is protected for Project tier
        ScanTier::Cache => {
            category != ProtectedPathCategory::ToolCache
                && category != ProtectedPathCategory::ToolBinary
        }
        ScanTier::Deep => {
            category != ProtectedPathCategory::ToolCache
                && category != ProtectedPathCategory::ToolBinary
                && category != ProtectedPathCategory::IdeConfig
        }
        ScanTier::Aggressive => {
            category == ProtectedPathCategory::System
                || category == ProtectedPathCategory::ProjectMetadata
                || category == ProtectedPathCategory::IdeExtension
        }
    }
}

pub fn is_protected_entry_name(name: &OsStr, tier: ScanTier) -> bool {
    let name = match name.to_str() {
        Some(value) => value,
        None => return false,
    };

    PROTECTED_PATH_RULES
        .iter()
        .any(|(category, os, _root_prefix, dir_name, _description)| {
            dir_name == &Some(name)
                && matches_os_family(*os)
                && is_category_protected(*category, tier)
        })
}

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
            .is_some_and(|s| is_protected_entry_name(s.as_ref(), tier))
    })
}

pub fn is_protected_root(path: &Path, tier: ScanTier) -> bool {
    let normalized = normalize_path(path);
    PROTECTED_PATH_RULES
        .iter()
        .any(|(category, os, root_prefix, _dir_name, _description)| {
            if !matches_os_family(*os) || !is_category_protected(*category, tier) {
                return false;
            }

            if let Some(prefix) = root_prefix {
                normalized == *prefix
                    || normalized.starts_with(&format!("{}{}", prefix, std::path::MAIN_SEPARATOR))
            } else {
                false
            }
        })
        || is_protected_home_subpath(path, tier)
}

fn matches_os_family(rule_os: OsFamily) -> bool {
    match rule_os {
        OsFamily::Any => true,
        OsFamily::Unix => cfg!(unix),
        OsFamily::Windows => cfg!(windows),
        OsFamily::MacOS => cfg!(target_os = "macos"),
    }
}

fn normalize_path(path: &Path) -> String {
    let mut normalized = path.to_string_lossy().to_string();

    if cfg!(windows) {
        normalized = normalized.replace('/', "\\").to_lowercase();
    }

    normalized
}

fn is_protected_home_subpath(path: &Path, tier: ScanTier) -> bool {
    if cfg!(windows) {
        let local_app_data = env::var_os("LOCALAPPDATA");
        let app_data = env::var_os("APPDATA");
        let user_profile = env::var_os("USERPROFILE").map(PathBuf::from);

        let home_subdirs = if tier >= ScanTier::Deep {
            vec![]
        } else if tier >= ScanTier::Cache {
            vec![".config", ".vscode", ".idea", ".cursor"]
        } else {
            vec![".cargo", ".config", ".vscode", ".idea", ".cursor"]
        };

        if matches_any_home_subpath(path, &user_profile, &home_subdirs) {
            return true;
        }

        if tier < ScanTier::Deep
            && matches_any_path_prefix(
                path,
                local_app_data.as_deref(),
                &["Local", "LocalLow", "Temp"],
            )
            || matches_any_path_prefix(path, app_data.as_deref(), &[])
        {
            return true;
        }

        return false;
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    let home_subdirs = if tier >= ScanTier::Deep {
        vec![".local"] // Still protect binaries maybe?
    } else if tier >= ScanTier::Cache {
        vec![".config", ".local", ".vscode", ".idea", ".cursor"]
    } else {
        vec![".cargo", ".config", ".local", ".vscode", ".idea", ".cursor"]
    };

    matches_any_home_subpath(path, &home, &home_subdirs)
}

fn matches_any_home_subpath(path: &Path, home: &Option<PathBuf>, subdirs: &[&str]) -> bool {
    let Some(home) = home else {
        return false;
    };

    for subdir in subdirs {
        let candidate = home.join(subdir);
        if path == candidate || path.starts_with(&candidate) {
            return true;
        }
    }

    false
}

fn matches_any_path_prefix(path: &Path, prefix: Option<&OsStr>, _subdirs: &[&str]) -> bool {
    let Some(prefix) = prefix else {
        return false;
    };
    let prefix_path = PathBuf::from(prefix);
    path == prefix_path || path.starts_with(&prefix_path)
}
