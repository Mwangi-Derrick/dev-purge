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

#[derive(Clone, Copy, Debug)]
pub enum ProtectedPathCategory {
    System,
    IdeConfig,
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

const PROTECTED_PATH_RULES: &[ProtectedPathRule] = &[
    // System-managed roots
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Any,
        root_prefix: Some("/"),
        dir_name: None,
        description: "Unix root filesystem",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Unix,
        root_prefix: Some("/usr"),
        dir_name: None,
        description: "Unix system binaries and libraries",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Unix,
        root_prefix: Some("/etc"),
        dir_name: None,
        description: "Unix system configuration",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Unix,
        root_prefix: Some("/var"),
        dir_name: None,
        description: "Unix variable data and caches",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Unix,
        root_prefix: Some("/bin"),
        dir_name: None,
        description: "Unix system binaries",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Unix,
        root_prefix: Some("/sbin"),
        dir_name: None,
        description: "Unix system binaries",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Unix,
        root_prefix: Some("/lib"),
        dir_name: None,
        description: "Unix system libraries",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::MacOS,
        root_prefix: Some("/Applications"),
        dir_name: None,
        description: "macOS application bundles",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::MacOS,
        root_prefix: Some("/Library"),
        dir_name: None,
        description: "macOS system libraries and caches",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Windows,
        root_prefix: Some("c:\\windows"),
        dir_name: None,
        description: "Windows system directory",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Windows,
        root_prefix: Some("c:\\program files"),
        dir_name: None,
        description: "Windows program files",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::System,
        os: OsFamily::Windows,
        root_prefix: Some("c:\\program files (x86)"),
        dir_name: None,
        description: "Windows program files x86",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::IdeConfig,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".vscode"),
        description: "VS Code settings and extensions",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::IdeConfig,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".idea"),
        description: "JetBrains IDE settings",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::IdeConfig,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".cursor"),
        description: "Cursor IDE settings",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::IdeConfig,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".config"),
        description: "User application configuration",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::ProjectMetadata,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".git"),
        description: "Git repository metadata",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::ProjectMetadata,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".github"),
        description: "GitHub workflow metadata",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::SecretConfig,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".env"),
        description: "Environment variables and secrets",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::SecretConfig,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".env.local"),
        description: "Local environment variables and secrets",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::ToolBinary,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".cargo"),
        description: "Cargo configuration and caches",
    },
    ProtectedPathRule {
        category: ProtectedPathCategory::ToolBinary,
        os: OsFamily::Any,
        root_prefix: None,
        dir_name: Some(".npm-global"),
        description: "Global npm binaries",
    },
];

pub fn is_protected_entry_name(name: &OsStr) -> bool {
    let name = match name.to_str() {
        Some(value) => value,
        None => return false,
    };

    PROTECTED_PATH_RULES.iter().any(|rule| {
        rule.dir_name == Some(name) && matches_os_family(rule.os)
    })
}

pub fn is_protected_root(path: &Path) -> bool {
    let normalized = normalize_path(path);
    PROTECTED_PATH_RULES.iter().any(|rule| {
        if !matches_os_family(rule.os) {
            return false;
        }

        if let Some(prefix) = rule.root_prefix {
            normalized == prefix || normalized.starts_with(&format!("{}{}", prefix, std::path::MAIN_SEPARATOR))
        } else {
            false
        }
    }) || is_protected_home_subpath(path)
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

fn is_protected_home_subpath(path: &Path) -> bool {
    if cfg!(windows) {
        let local_app_data = env::var_os("LOCALAPPDATA");
        let app_data = env::var_os("APPDATA");
        let user_profile = env::var_os("USERPROFILE").map(PathBuf::from);

        return matches_any_home_subpath(path, &user_profile, &[".cargo", ".config", ".vscode", ".idea", ".cursor"])
            || matches_any_path_prefix(path, local_app_data.as_ref().map(|v| &**v), &["Local", "LocalLow", "Temp"])
            || matches_any_path_prefix(path, app_data.as_ref().map(|v| &**v), &[]);
    }

    let home = env::var_os("HOME").map(PathBuf::from);
    matches_any_home_subpath(path, &home, &[".cargo", ".config", ".local", ".vscode", ".idea", ".cursor"])
}

fn matches_any_home_subpath(path: &Path, home: &Option<PathBuf>, subdirs: &[&str]) -> bool {
    let Some(home) = home else { return false; };

    for subdir in subdirs {
        let candidate = home.join(subdir);
        if path == candidate || path.starts_with(&candidate) {
            return true;
        }
    }

    false
}

fn matches_any_path_prefix(path: &Path, prefix: Option<&OsStr>, _subdirs: &[&str]) -> bool {
    let Some(prefix) = prefix else { return false; };
    let prefix_path = PathBuf::from(prefix);
    path == prefix_path || path.starts_with(&prefix_path)
}
