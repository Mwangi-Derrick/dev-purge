//! Pattern configuration using a high-fidelity declarative DSL.
//!
//! Engineered for clarity, safety, and rapid extensibility.
//! This registry serves as the heuristic engine for artifact detection.

use crate::domain::traits::ScanTier;
use std::ffi::OsStr;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternKind {
    /// Match exact directory name
    Exact,
    /// Match prefix (e.g., "cmake-build-")
    Prefix,
    /// Only match if sibling file/extension exists
    Guarded(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub enum Category {
    Core,
    Python,
    Node,
    Go,
    Java,
    DotNet,
    Mobile,
    Infra,
    Docker,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub struct Pattern {
    pub category: Category,
    pub kind: PatternKind,
    pub name: &'static str,
    pub tier: ScanTier,
    pub description: &'static str,
}

macro_rules! register_artifacts {
    ($( ($cat:ident, $kind:expr, $name:expr, $tier:ident, $desc:expr) ),* $(,)?) => {
        const ARTIFACT_REGISTRY: &[Pattern] = &[
            $(
                Pattern {
                    category: Category::$cat,
                    kind: $kind,
                    name: $name,
                    tier: ScanTier::$tier,
                    description: $desc,
                }
            ),*
        ];
    };
}

use PatternKind::*;

register_artifacts![
    // Core / Language-agnostic
    (Core, Exact, "target", Project, "Rust build artifacts"),
    (Core, Exact, "dist", Project, "Generic distribution folder"),
    (Core, Exact, "build", Project, "Generic build artifacts"),
    (Core, Exact, "out", Project, "Generic output directory"),
    // Python Ecosystem
    (
        Python,
        Exact,
        "__pycache__",
        Project,
        "Python bytecode cache"
    ),
    (
        Python,
        Exact,
        ".venv",
        Project,
        "Python virtual environment"
    ),
    (Python, Exact, "venv", Project, "Python virtual environment"),
    (
        Python,
        Exact,
        ".pytest_cache",
        Project,
        "Pytest execution cache"
    ),
    (
        Python,
        Exact,
        ".mypy_cache",
        Project,
        "Mypy type check cache"
    ),
    (Python, Exact, "pip", Cache, "Pip cache directory"),
    // Web / JavaScript / TypeScript
    (Node, Exact, "node_modules", Project, "Node.js dependencies"),
    (Node, Exact, ".next", Project, "Next.js build artifacts"),
    (Node, Exact, ".nuxt", Project, "Next.js build artifacts"),
    (Node, Exact, ".turbo", Project, "Turborepo build cache"),
    (Node, Exact, ".vite", Project, "Vite build cache"),
    (Node, Exact, "_cacache", Cache, "npm cache internals"),
    // Go / PHP / Ruby
    (Go, Exact, "vendor", Project, "Dependency vendor directory"),
    (Go, Exact, "cache", Cache, "Go build cache"),
    // Java / Kotlin / Gradle
    (Java, Exact, ".gradle", Project, "Gradle build cache"),
    (Java, Exact, ".kotlin", Project, "Kotlin compiler metadata"),
    // .NET / C# (Heuristic-based)
    (
        DotNet,
        Guarded(".csproj"),
        "bin",
        Project,
        ".NET binary output"
    ),
    (
        DotNet,
        Guarded(".csproj"),
        "obj",
        Project,
        ".NET intermediate artifacts"
    ),
    (
        DotNet,
        Guarded(".sln"),
        "bin",
        Project,
        ".NET solution binaries"
    ),
    (
        DotNet,
        Guarded(".sln"),
        "obj",
        Project,
        ".NET solution intermediates"
    ),
    // Mobile & Cross-Platform
    (
        Mobile,
        Exact,
        ".dart_tool",
        Project,
        "Dart/Flutter metadata"
    ),
    (
        Mobile,
        Exact,
        "DerivedData",
        Project,
        "Xcode build artifacts"
    ),
    // Infrastructure & Tooling
    (
        Infra,
        Exact,
        ".terraform",
        Project,
        "Terraform state/plugins"
    ),
    (Infra, Exact, "zig-cache", Project, "Zig build cache"),
    (Infra, Exact, "zig-out", Project, "Zig binary output"),
    // Docker
    (
        Docker,
        Exact,
        ".docker",
        Project,
        "Local Docker configuration/context"
    ),
    (
        Docker,
        Prefix,
        "docker-build-",
        Project,
        "Docker build artifacts"
    ),
    // Prefixes
    (
        Other,
        Prefix,
        "cmake-build-",
        Project,
        "CMake build directory"
    ),
    // Global Caches (Cache Tier)
    (Core, Exact, "registry", Cache, "Cargo registry cache"),
    (Core, Exact, "index", Cache, "Cargo index cache"),
    (Core, Exact, "db", Cache, "Cargo git database"),
    (Core, Exact, "mod", Cache, "Go module cache"),
    // Deep Cleaning (Deep Tier)
    (Core, Exact, "Caches", Deep, "System/App cache directory"),
    (Core, Exact, "Temp", Deep, "Temporary files"),
    (Core, Exact, ".cache", Deep, "User cache directory"),
    // System Caches (Aggressive Tier)
    (Other, Exact, "archives", Aggressive, "apt package archives"),
    (Other, Exact, "pkg", Aggressive, "pacman/yay package cache"),
];

pub struct PurgeConfig;

impl PurgeConfig {
    pub fn patterns() -> &'static [Pattern] {
        ARTIFACT_REGISTRY
    }
}

pub fn matches_any_pattern(path: &Path, name: &OsStr, patterns: &[Pattern]) -> bool {
    let Some(name_str) = name.to_str() else {
        return false;
    };

    patterns.iter().any(|pattern| match pattern.kind {
        Exact => pattern.name == name_str,
        Prefix => name_str.starts_with(pattern.name),
        Guarded(marker) => {
            if pattern.name != name_str {
                return false;
            }
            if let Some(parent) = path.parent() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let s_name = entry.file_name();
                        if let Some(s) = s_name.to_str() {
                            if marker.starts_with('.') {
                                if s.ends_with(marker) {
                                    return true;
                                }
                            } else if s == marker {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
    })
}
