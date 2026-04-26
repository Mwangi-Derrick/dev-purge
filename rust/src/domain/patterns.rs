//! Pattern configuration using a high-fidelity declarative DSL.
//!
//! Engineered for clarity, safety, and rapid extensibility.
//! This registry serves as the heuristic engine for artifact detection.

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
    Other,
}

#[derive(Clone, Copy, Debug)]
pub struct Pattern {
    pub category: Category,
    pub kind: PatternKind,
    pub name: &'static str,
    pub description: &'static str,
}

macro_rules! register_artifacts {
    ($( ($cat:ident, $kind:expr, $name:expr, $desc:expr) ),* $(,)?) => {
        const ARTIFACT_REGISTRY: &[Pattern] = &[
            $(
                Pattern {
                    category: Category::$cat,
                    kind: $kind,
                    name: $name,
                    description: $desc,
                }
            ),*
        ];
    };
}

use PatternKind::*;

register_artifacts![
    // Core / Language-agnostic
    (Core, Exact, "target", "Rust build artifacts"),
    (Core, Exact, "dist", "Generic distribution folder"),
    (Core, Exact, "build", "Generic build artifacts"),
    (Core, Exact, "out", "Generic output directory"),

    // Python Ecosystem
    (Python, Exact, "__pycache__", "Python bytecode cache"),
    (Python, Exact, ".venv", "Python virtual environment"),
    (Python, Exact, "venv", "Python virtual environment"),
    (Python, Exact, ".pytest_cache", "Pytest execution cache"),
    (Python, Exact, ".mypy_cache", "Mypy type check cache"),

    // Web / JavaScript / TypeScript
    (Node, Exact, "node_modules", "Node.js dependencies"),
    (Node, Exact, ".next", "Next.js build artifacts"),
    (Node, Exact, ".nuxt", "Nuxt.js build artifacts"),
    (Node, Exact, ".turbo", "Turborepo build cache"),
    (Node, Exact, ".vite", "Vite build cache"),

    // Go / PHP / Ruby
    (Go, Exact, "vendor", "Dependency vendor directory"),

    // Java / Kotlin / Gradle
    (Java, Exact, ".gradle", "Gradle build cache"),
    (Java, Exact, ".kotlin", "Kotlin compiler metadata"),

    // .NET / C# (Heuristic-based)
    (DotNet, Guarded(".csproj"), "bin", ".NET binary output"),
    (DotNet, Guarded(".csproj"), "obj", ".NET intermediate artifacts"),
    (DotNet, Guarded(".sln"), "bin", ".NET solution binaries"),
    (DotNet, Guarded(".sln"), "obj", ".NET solution intermediates"),

    // Mobile & Cross-Platform
    (Mobile, Exact, ".dart_tool", "Dart/Flutter metadata"),
    (Mobile, Exact, "DerivedData", "Xcode build artifacts"),

    // Infrastructure & Tooling
    (Infra, Exact, ".terraform", "Terraform state/plugins"),
    (Infra, Exact, "zig-cache", "Zig build cache"),
    (Infra, Exact, "zig-out", "Zig binary output"),

    // Prefixes
    (Other, Prefix, "cmake-build-", "CMake build directory"),
];

pub struct PurgeConfig;

impl PurgeConfig {
    pub fn patterns() -> &'static [Pattern] {
        ARTIFACT_REGISTRY
    }
}

pub fn matches_any_pattern(path: &Path, name: &OsStr, patterns: &[Pattern]) -> bool {
    let Some(name_str) = name.to_str() else { return false };

    patterns.iter().any(|pattern| match pattern.kind {
        Exact => pattern.name == name_str,
        Prefix => name_str.starts_with(pattern.name),
        Guarded(marker) => {
            if pattern.name != name_str { return false }
            if let Some(parent) = path.parent() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let s_name = entry.file_name();
                        if let Some(s) = s_name.to_str() {
                            if marker.starts_with('.') {
                                if s.ends_with(marker) { return true }
                            } else if s == marker {
                                return true
                            }
                        }
                    }
                }
            }
            false
        }
    })
}