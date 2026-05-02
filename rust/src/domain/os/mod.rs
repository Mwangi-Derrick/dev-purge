//! Platform-agnostic interface for OS-specific operations.
//!
//! This module dispatches requests to the appropriate platform implementation
//! (Windows, Linux, macOS) while providing a unified API to the rest of the application.

use crate::domain::traits::ScanTier;
use std::ffi::OsStr;
use std::path::Path;

// Unix module contains linux and mac submodules
pub mod unix;
pub mod windows;

// Re-export platform-specific functionality
pub use unix::linux as linux;
pub use unix::mac as macos;  // Note: renaming 'mac' to 'macos' for API consistency

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

/// The main safety entry point.
pub fn is_safe(path: &Path, tier: ScanTier) -> bool {
    #[cfg(windows)]
    {
        windows::is_safe(path, tier)
    }
    #[cfg(target_os = "macos")]
    {
        macos::is_safe(path, tier)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        linux::is_safe(path, tier)
    }
}

/// Checks if an entry name is protected at the current tier.
pub fn is_protected_entry_name(name: &OsStr, tier: ScanTier) -> bool {
    #[cfg(windows)]
    {
        windows::is_protected_entry_name(name, tier)
    }
    #[cfg(unix)]
    {
        unix::is_protected_entry_name(name, tier)
    }
}

/// Checks if a path is a protected root at the current tier.
pub fn is_protected_root(path: &Path, tier: ScanTier) -> bool {
    #[cfg(windows)]
    {
        windows::is_protected_root(path, tier)
    }
    #[cfg(target_os = "macos")]
    {
        macos::is_protected_root(path, tier)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        linux::is_protected_root(path, tier)
    }
}

/// Internal helper for tier-based protection.
pub(crate) fn is_category_protected(category: ProtectedPathCategory, tier: ScanTier) -> bool {
    match tier {
        ScanTier::Project => true,
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
