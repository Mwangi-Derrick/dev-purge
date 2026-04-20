use std::ffi::OsStr;

#[derive(Clone, Copy)]
pub struct Pattern {
    pub kind: PatternKind,
    pub text: &'static str,
}

#[derive(Clone, Copy)]
pub enum PatternKind {
    Exact,
    Prefix,
}

#[derive(Clone)]
pub struct PurgeConfig {
    patterns: Vec<Pattern>,
}

impl PurgeConfig {
    pub fn hardcoded() -> Self {
        Self {
            patterns: vec![
                // Spec patterns (core)
                Pattern {
                    kind: PatternKind::Exact,
                    text: "target",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "node_modules",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "__pycache__",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: ".venv",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "venv",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: ".next",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: ".nuxt",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "dist",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "build",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "out",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: ".pytest_cache",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: ".parcel-cache",
                },
                // Extra stacks mentioned (still hardcoded)
                Pattern {
                    kind: PatternKind::Exact,
                    text: ".dart_tool",
                },
                Pattern {
                    kind: PatternKind::Prefix,
                    text: "cmake-build-",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "zig-cache",
                },
                Pattern {
                    kind: PatternKind::Exact,
                    text: "zig-out",
                },
            ],
        }
    }

    pub fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }
}

pub fn matches_any_pattern(name: &OsStr, patterns: &[Pattern]) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };

    patterns.iter().any(|p| match p.kind {
        PatternKind::Exact => name == p.text,
        PatternKind::Prefix => name.starts_with(p.text),
    })
}

pub fn is_protected_entry_name(name: &OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(
            ".git"
                | ".vscode"
                | ".idea"
                | ".cursor"
                | ".env"
                | ".env.local"
                | ".gitignore"
                | ".dockerignore"
                | ".github"
        )
    )
}
