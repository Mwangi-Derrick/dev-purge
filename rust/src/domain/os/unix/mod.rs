//! Shared Unix-like (Linux/macOS) path protection rules.

use super::{is_category_protected, ProtectedPathCategory};
use crate::domain::traits::ScanTier;
pub mod linux;
pub mod mac;
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
    // Linux system caches
    (
        ProtectedPathCategory::System,
        None,
        Some("/var/cache/apt/archives"),
    ), // APT
    (
        ProtectedPathCategory::System,
        None,
        Some("/var/cache/pacman/pkg"),
    ), // Pacman
    (ProtectedPathCategory::System, None, Some("/var/cache/dnf")), // DNF
    (ProtectedPathCategory::System, None, Some("/var/cache/yum")), // YUM
    (ProtectedPathCategory::System, None, Some("/var/cache/zypp")), // Zypper
    (
        ProtectedPathCategory::System,
        None,
        Some("/var/cache/snapd"),
    ), // Snap
    // Linux
    (ProtectedPathCategory::System, None, Some("~/.bun/bin/bun")),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/bun"),
    ),
    (ProtectedPathCategory::System, None, Some("/opt/bun/bin/bun")),

    // macOS system
    (ProtectedPathCategory::System, None, Some("~/Library/Caches")),
    (ProtectedPathCategory::System, None, Some("/Library/Caches")),
    (ProtectedPathCategory::System, None, Some("~/Library/Logs")),
    (ProtectedPathCategory::System, None, Some("/Library/Logs")),
    // macOS
    (ProtectedPathCategory::System, None, Some("~/.bun/bin/bun")),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/bun"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/opt/homebrew/bin/bun"),
    ),
    // Linux
    (ProtectedPathCategory::System, None, Some("/usr/bin/node")),
    (ProtectedPathCategory::System, None, Some("/usr/bin/npm")),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/node"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/npm"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.nvm/versions/node/*/bin/node"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.nvm/versions/node/*/bin/npm"),
    ),
    // macOS
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/node"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/npm"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/opt/homebrew/bin/node"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/opt/homebrew/bin/npm"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.nvm/versions/node/*/bin/node"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.nvm/versions/node/*/bin/npm"),
    ),
    // Linux
    (ProtectedPathCategory::System, None, Some("/usr/bin/yarn")),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/yarn"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.yarn/bin/yarn"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/opt/yarn/bin/yarn"),
    ),
    // macOS
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/yarn"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/opt/homebrew/bin/yarn"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.yarn/bin/yarn"),
    ),
    // Linux
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.deno/bin/deno"),
    ),
    (ProtectedPathCategory::System, None, Some("/usr/bin/deno")),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/deno"),
    ),
    // macOS
    (
        ProtectedPathCategory::System,
        None,
        Some("~/.deno/bin/deno"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/usr/local/bin/deno"),
    ),
    (
        ProtectedPathCategory::System,
        None,
        Some("/opt/homebrew/bin/deno"),
    ),
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
