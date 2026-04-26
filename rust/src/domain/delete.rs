use crate::domain::types::{DeleteStats, Finding};
use colored::Colorize;
use std::fs;

pub fn delete(findings: &[Finding]) -> DeleteStats {
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

    DeleteStats {
        recovered_bytes: recovered,
        errors,
    }
}

fn display_path_for_humans(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
