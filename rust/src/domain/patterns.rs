//! Pattern configuration using compact DSL.
//!
//! This module provides artifact pattern detection for the purge tool.
//! Patterns are defined in a compact tuple-based DSL for maintainability.

use std::ffi::OsStr;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternKind {
    /// Match exact directory name
    Exact,
    /// Match prefix (e.g., "cmake-build-" matches "cmake-build-debug")
    Prefix,
    /// Only match if sibling file exists (e.g., "bin" only if ".csproj" exists)
    Guarded(&'static str),
}

use PatternKind::*;

#[derive(Clone, Copy, Debug)]
pub struct Pattern {
    pub name: &'static str,
    pub kind: PatternKind,
}

/// Compact DSL-style pattern definitions
const PATTERNS_DSL: &[(PatternKind, &[&str])] = &[
    // Core / Language-agnostic
    (Exact, &["target", "dist", "build", "out"]),

    // Python
    (Exact, &[
        "__pycache__", ".venv", "venv", ".pytest_cache", ".mypy_cache",
        ".ruff_cache", ".tox", ".hypothesis", ".ipynb_checkpoints",
    ]),

    // Node.js / JavaScript / Web
    (Exact, &[
        "node_modules", ".next", ".nuxt", ".parcel-cache", ".turbo",
        ".nx", ".svelte-kit", ".astro", ".vite", ".angular", ".vercel",
    ]),

    // Go
    (Exact, &["vendor"]),

    // Java / Kotlin / Scala
    (Exact, &[".gradle", "gradle-app.setting", ".kotlin"]),

    // PHP / Ruby
    (Exact, &["vendor"]),

    // Swift / iOS
    (Exact, &[".build", "DerivedData"]),

    // Haskell
    (Exact, &["dist", "dist-newstyle"]),

    // Elixir / Erlang
    (Exact, &["_build", "deps"]),

    // Clojure
    (Exact, &[".cpcache"]),

    // R
    (Exact, &[".Rproj.user", ".Rhistory"]),

    // Julia / Crystal / Nim
    (Exact, &[".crystal", "libs", "nimcache"]),

    // Infrastructure
    (Exact, &[".terraform", "zig-cache", "zig-out", ".dart_tool"]),

    // Prefix patterns
    (Prefix, &["cmake-build-"]),

    // .NET / C# (guarded - only delete if project file exists)
    (Guarded(".csproj"), &["bin", "obj"]),
    (Guarded(".fsproj"), &["bin", "obj"]),
    (Guarded(".vbproj"), &["bin", "obj"]),
    (Guarded(".sln"), &["bin", "obj"]),
];

pub struct PurgeConfig;

impl PurgeConfig {
    /// Returns all artifact patterns
    pub fn patterns() -> Vec<Pattern> {
        let mut patterns = Vec::new();

        for (kind, names) in PATTERNS_DSL {
            for &name in *names {
                patterns.push(Pattern { name, kind: *kind });
            }
        }

        patterns
    }

    /// Get patterns for a specific language (extensibility point)
    pub fn patterns_for_language(language: &str) -> Vec<Pattern> {
        match language {
            "python" => vec![
                Pattern { name: "__pycache__", kind: Exact },
                Pattern { name: ".venv", kind: Exact },
                Pattern { name: "venv", kind: Exact },
                Pattern { name: ".pytest_cache", kind: Exact },
                Pattern { name: ".mypy_cache", kind: Exact },
            ],
            "rust" => vec![
                Pattern { name: "target", kind: Exact },
            ],
            "node" | "javascript" | "typescript" => vec![
                Pattern { name: "node_modules", kind: Exact },
                Pattern { name: ".next", kind: Exact },
                Pattern { name: ".nuxt", kind: Exact },
                Pattern { name: ".vite", kind: Exact },
            ],
            "go" => vec![
                Pattern { name: "vendor", kind: Exact },
            ],
            "java" | "kotlin" => vec![
                Pattern { name: ".gradle", kind: Exact },
                Pattern { name: "target", kind: Exact },
            ],
            "dotnet" | "csharp" => vec![
                Pattern { name: "bin", kind: Guarded(".csproj") },
                Pattern { name: "obj", kind: Guarded(".csproj") },
            ],
            _ => Self::patterns(),
        }
    }
}

/// Check if a directory name and path match any pattern
pub fn matches_any_pattern(path: &Path, name: &OsStr, patterns: &[Pattern]) -> bool {
    let Some(name_str) = name.to_str() else {
        return false;
    };

    patterns.iter().any(|pattern| {
        match pattern.kind {
            Exact => pattern.name == name_str,
            Prefix => name_str.starts_with(pattern.name),
            Guarded(marker) => {
                if pattern.name != name_str {
                    return false;
                }
                // Check if marker file exists in sibling directories
                if let Some(parent) = path.parent() {
                    if let Ok(entries) = std::fs::read_dir(parent) {
                        for entry in entries.flatten() {
                            let s_name = entry.file_name();
                            if let Some(s) = s_name.to_str() {
                                if marker.starts_with('.') {
                                    // Extension check
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
        }
    })
}