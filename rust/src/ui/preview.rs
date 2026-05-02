use crate::domain::types::Finding;
use colored::Colorize;
use std::path::Path;

pub fn print(root: &Path, findings: &[Finding]) {
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

    let display_count = 20;
    let visible_findings = &findings[..findings.len().min(display_count)];

    let max_bytes = findings[0].bytes.max(1);
    let max_path_len = visible_findings
        .iter()
        .map(|f| display_path_relative(root, &f.path).len())
        .max()
        .unwrap_or(0)
        .min(48);

    for f in visible_findings {
        let rel = display_path_relative(root, &f.path);
        let rel = truncate_end(&rel, max_path_len);
        let size = format_bytes(f.bytes);
        let bar = bar(f.bytes, max_bytes, 15);
        let label = size_label(f.bytes);

        let line = if label.is_empty() {
            format!(
                "  {:<width$}  {:>8}  {}",
                rel,
                size,
                bar,
                width = max_path_len
            )
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
        } else {
            println!("{}", line);
        }
    }

    if count > display_count {
        let remaining_count = count - display_count;
        let remaining_bytes: u64 = findings[display_count..].iter().map(|f| f.bytes).sum();
        println!(
            "\n  {} smaller items totaling {} (not shown)",
            remaining_count.to_string().dimmed(),
            format_bytes(remaining_bytes).dimmed()
        );
    }
}

pub fn print_summary(recovered: u64, errors: u64) {
    if errors == 0 {
        println!(
            "{} Recovered {}",
            "✓".green(),
            format_bytes(recovered).bold()
        );
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

pub fn total_bytes(findings: &[Finding]) -> u64 {
    findings.iter().map(|f| f.bytes).sum()
}

pub fn format_bytes(bytes: u64) -> String {
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
