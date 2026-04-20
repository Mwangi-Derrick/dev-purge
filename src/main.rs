use anyhow::{bail, Context, Result};
use colored::Colorize;
use rayon::prelude::*;
use std::cell::RefCell;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Default,
    Check,
}

#[derive(Clone, Debug)]
struct Finding {
    path: PathBuf,
    bytes: u64,
}

fn main() -> Result<()> {
    let mode = parse_mode(env::args().skip(1).collect())?;

    let cwd = env::current_dir().context("failed to read current working directory")?;
    safety_check(&cwd)?;

    let candidates = scan(&cwd)?;
    let mut findings = estimate_sizes(&candidates);
    findings.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    print_preview(&cwd, &findings);

    if mode == Mode::Check || findings.is_empty() {
        return Ok(());
    }

    if !confirm(total_bytes(&findings), findings.len())? {
        return Ok(());
    }

    let (recovered, errors) = delete(&findings);
    print_summary(recovered, errors);

    Ok(())
}

fn parse_mode(args: Vec<String>) -> Result<Mode> {
    if args.is_empty() {
        return Ok(Mode::Default);
    }

    if args.len() == 1 && args[0] == "--check" {
        return Ok(Mode::Check);
    }

    eprintln!("Usage: dev-purge [--check]");
    bail!("invalid arguments")
}

fn safety_check(cwd: &Path) -> Result<()> {
    let cwd = canonicalize_best_effort(cwd);

    if is_home_dir(&cwd)? {
        eprintln!(
            "{}  Run dev-purge from a projects directory, not your home folder.\n   {}",
            "✗".red(),
            "cd ~/projects && dev-purge".dimmed()
        );
        bail!("refusing to run from home directory");
    }

    if is_system_dir(&cwd) {
        eprintln!(
            "{}  Refusing to run from a system directory: {}",
            "✗".red(),
            display_path_for_humans(&cwd).dimmed()
        );
        bail!("refusing to run from system directory");
    }

    Ok(())
}

fn scan(root: &Path) -> Result<Vec<PathBuf>> {
    let patterns = candidate_patterns();
    let candidates: RefCell<Vec<PathBuf>> = RefCell::new(Vec::new());

    let mut it = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| scan_filter_entry(e, &patterns, &candidates));

    while let Some(entry) = it.next() {
        if let Err(err) = entry {
            eprintln!("{} {}", "warning:".yellow(), err);
        }
    }

    let mut out = candidates.into_inner();
    out.sort();
    out.dedup();
    Ok(out)
}

fn scan_filter_entry(
    entry: &DirEntry,
    patterns: &[Pattern],
    candidates: &RefCell<Vec<PathBuf>>,
) -> bool {
    let name = entry.file_name();

    if is_protected_name(name) {
        return false;
    }

    if entry.file_type().is_dir() && matches_any_pattern(name, patterns) {
        candidates.borrow_mut().push(entry.path().to_path_buf());
        return false; // don't descend into it
    }

    true
}

fn estimate_sizes(candidates: &[PathBuf]) -> Vec<Finding> {
    candidates
        .par_iter()
        .map(|path| Finding {
            path: path.clone(),
            bytes: estimate_dir_size_bytes(path),
        })
        .filter(|f| f.bytes > 0)
        .collect()
}

fn estimate_dir_size_bytes(dir: &Path) -> u64 {
    let mut total: u64 = 0;

    let it = WalkDir::new(dir).follow_links(false).into_iter();
    for entry in it {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if entry.file_type().is_file() {
            if let Ok(md) = entry.metadata() {
                total = total.saturating_add(md.len());
            }
        }
    }

    total
}

fn delete(findings: &[Finding]) -> (u64, u64) {
    let mut recovered: u64 = 0;
    let mut errors: u64 = 0;

    for f in findings {
        match fs::remove_dir_all(&f.path) {
            Ok(()) => recovered = recovered.saturating_add(f.bytes),
            Err(err) => {
                errors += 1;
                eprintln!(
                    "{} {}: {}",
                    "error:".red(),
                    display_path_for_humans(&f.path),
                    err
                );
            }
        }
    }

    (recovered, errors)
}

fn confirm(total: u64, count: usize) -> Result<bool> {
    print!(
        "Recover {} across {} {}?  {} ",
        format_bytes(total).bold(),
        count,
        if count == 1 { "item" } else { "items" },
        "[y/N]".dimmed()
    );
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read confirmation input")?;

    Ok(matches!(input.trim(), "y" | "Y"))
}

fn print_preview(root: &Path, findings: &[Finding]) {
    let total = total_bytes(findings);
    let count = findings.len();
    let noun = if count == 1 { "item" } else { "items" };

    println!(
        "Found {} {} · {} recoverable",
        count.to_string().bold(),
        noun,
        format_bytes(total).bold()
    );
    println!();

    if findings.is_empty() {
        return;
    }

    let max_bytes = findings[0].bytes.max(1);
    let max_path_len = findings
        .iter()
        .map(|f| display_path_relative(root, &f.path).len())
        .max()
        .unwrap_or(0)
        .min(48);

    for f in findings {
        let rel = display_path_relative(root, &f.path);
        let rel = truncate_end(&rel, max_path_len);
        let size = format_bytes(f.bytes);
        let bar = bar(f.bytes, max_bytes, 15);
        let label = size_label(f.bytes);

        let line = if label.is_empty() {
            format!("  {:<width$}  {:>8}  {}", rel, size, bar, width = max_path_len)
        } else {
            format!(
                "  {:<width$}  {:>8}  {}  {}",
                rel,
                size,
                bar,
                label,
                width = max_path_len
            )
        };

        if is_dimmed(f.bytes) {
            println!("{}", line.bright_black());
        } else if label == "big win" {
            println!("{}", line);
        } else {
            println!("{}", line);
        }
    }
}

fn print_summary(recovered: u64, errors: u64) {
    if errors == 0 {
        println!("{} Recovered {}", "✓".green(), format_bytes(recovered).bold());
        return;
    }

    let err_word = if errors == 1 { "error" } else { "errors" };
    println!(
        "{} Recovered {}  ({} {} — see above)",
        "✓".green(),
        format_bytes(recovered).bold(),
        errors.to_string().red(),
        err_word
    );
}

fn total_bytes(findings: &[Finding]) -> u64 {
    findings.iter().map(|f| f.bytes).sum()
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.0} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn bar(bytes: u64, max_bytes: u64, width: usize) -> String {
    let ratio = (bytes as f64) / (max_bytes as f64);
    let filled = ((ratio * width as f64).round() as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn size_label(bytes: u64) -> &'static str {
    const MB: u64 = 1024 * 1024;
    if bytes > 500 * MB {
        "big win"
    } else {
        ""
    }
}

fn is_dimmed(bytes: u64) -> bool {
    const MB: u64 = 1024 * 1024;
    bytes < 50 * MB
}

fn truncate_end(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    if max_len <= 1 {
        return "…".to_string();
    }
    let keep = max_len - 1;
    format!("{}…", &s[..keep])
}

fn display_path_relative(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let mut s = rel.to_string_lossy().to_string();
    if s.is_empty() {
        s = ".".to_string();
    }
    s = s.replace('\\', "/");
    format!("./{}", s.trim_start_matches("./"))
}

fn display_path_for_humans(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn canonicalize_best_effort(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn is_home_dir(cwd: &Path) -> Result<bool> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from));
    let Some(home) = home else {
        return Ok(false);
    };

    let home = canonicalize_best_effort(&home);
    Ok(paths_equal_case_insensitive_if_windows(cwd, &home))
}

fn is_system_dir(cwd: &Path) -> bool {
    if cwd.parent().is_none() {
        return true;
    }

    #[cfg(windows)]
    {
        let s = cwd.to_string_lossy().to_lowercase().replace('/', "\\");
        return s.starts_with("c:\\windows")
            || s.starts_with("c:\\program files")
            || s.starts_with("c:\\program files (x86)");
    }

    #[cfg(not(windows))]
    {
        let s = cwd.to_string_lossy();
        return s == "/"
            || s.starts_with("/usr")
            || s.starts_with("/etc")
            || s.starts_with("/var")
            || s.starts_with("/bin");
    }
}

fn paths_equal_case_insensitive_if_windows(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}

#[derive(Clone, Copy)]
struct Pattern {
    kind: PatternKind,
    text: &'static str,
}

#[derive(Clone, Copy)]
enum PatternKind {
    Exact,
    Prefix,
}

fn candidate_patterns() -> Vec<Pattern> {
    vec![
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
    ]
}

fn matches_any_pattern(name: &OsStr, patterns: &[Pattern]) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    patterns.iter().any(|p| match p.kind {
        PatternKind::Exact => name == p.text,
        PatternKind::Prefix => name.starts_with(p.text),
    })
}

fn is_protected_name(name: &OsStr) -> bool {
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
