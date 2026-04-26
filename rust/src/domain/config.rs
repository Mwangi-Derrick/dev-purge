use std::ffi::OsStr;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub enum PatternKind {
    Exact,
    Prefix,
    /// Only matches if a sibling file/dir matches the marker glob.
    /// Marker is a simple extension check for now (e.g. ".csproj") or exact name.
    Guarded {
        marker: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Pattern {
    pub kind: PatternKind,
    pub text: &'static str,
}

#[derive(Clone)]
pub struct PurgeConfig {
    patterns: Vec<Pattern>,
}

impl PurgeConfig {
    pub fn hardcoded() -> Self {
        let mut patterns = Vec::new();

        // Core / Common
        patterns.extend(&[
            Pattern { kind: PatternKind::Exact, text: "target" },
            Pattern { kind: PatternKind::Exact, text: "node_modules" },
            Pattern { kind: PatternKind::Exact, text: "dist" },
            Pattern { kind: PatternKind::Exact, text: "build" },
            Pattern { kind: PatternKind::Exact, text: "out" },
        ]);

        // Python
        patterns.extend(&[
            Pattern { kind: PatternKind::Exact, text: "__pycache__" },
            Pattern { kind: PatternKind::Exact, text: ".venv" },
            Pattern { kind: PatternKind::Exact, text: "venv" },
            Pattern { kind: PatternKind::Exact, text: ".pytest_cache" },
            Pattern { kind: PatternKind::Exact, text: ".mypy_cache" },
            Pattern { kind: PatternKind::Exact, text: ".ruff_cache" },
            Pattern { kind: PatternKind::Exact, text: ".tox" },
            Pattern { kind: PatternKind::Exact, text: ".hypothesis" },
            Pattern { kind: PatternKind::Exact, text: ".ipynb_checkpoints" },
        ]);

        // JS / TS / Web Ecosystem
        patterns.extend(&[
            Pattern { kind: PatternKind::Exact, text: ".next" },
            Pattern { kind: PatternKind::Exact, text: ".nuxt" },
            Pattern { kind: PatternKind::Exact, text: ".parcel-cache" },
            Pattern { kind: PatternKind::Exact, text: ".turbo" },
            Pattern { kind: PatternKind::Exact, text: ".nx" },
            Pattern { kind: PatternKind::Exact, text: ".svelte-kit" },
            Pattern { kind: PatternKind::Exact, text: ".astro" },
            Pattern { kind: PatternKind::Exact, text: ".vite" },
            Pattern { kind: PatternKind::Exact, text: ".angular" },
            Pattern { kind: PatternKind::Exact, text: ".vercel" },
        ]);

        // Mobile / Other Stacks
        patterns.extend(&[
            Pattern { kind: PatternKind::Exact, text: ".dart_tool" },
            Pattern { kind: PatternKind::Exact, text: "zig-cache" },
            Pattern { kind: PatternKind::Exact, text: "zig-out" },
            Pattern { kind: PatternKind::Prefix, text: "cmake-build-" },
        ]);

        // Infrastructure / DevOps
        patterns.extend(&[
            Pattern { kind: PatternKind::Exact, text: ".terraform" },
            Pattern { kind: PatternKind::Exact, text: ".gradle" },
        ]);

        // Guarded Patterns (.NET, etc)
        patterns.extend(&[
            Pattern { kind: PatternKind::Guarded { marker: ".csproj" }, text: "bin" },
            Pattern { kind: PatternKind::Guarded { marker: ".csproj" }, text: "obj" },
            Pattern { kind: PatternKind::Guarded { marker: ".sln" }, text: "bin" },
            Pattern { kind: PatternKind::Guarded { marker: ".sln" }, text: "obj" },
        ]);

        Self { patterns }
    }

    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

pub fn matches_any_pattern(path: &Path, name: &OsStr, patterns: &[Pattern]) -> bool {
    let Some(name_str) = name.to_str() else {
        return false;
    };

    patterns.iter().any(|p| match p.kind {
        PatternKind::Exact => name_str == p.text,
        PatternKind::Prefix => name_str.starts_with(p.text),
        PatternKind::Guarded { marker } => {
            if name_str != p.text {
                return false;
            }
            // Check siblings
            if let Some(parent) = path.parent() {
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let s_name = entry.file_name();
                        if let Some(s) = s_name.to_str() {
                            if marker.starts_with('.') {
                                if s.ends_with(marker) { return true; }
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

