use crate::cli::Mode;
use crate::domain::{config::PurgeConfig, delete, safety, scan, size};
use crate::types::Finding;
use crate::ui::{confirm, preview};
use anyhow::{Context, Result};
use std::env;

pub fn run() -> Result<()> {
    let mode = crate::cli::parse_mode(env::args().skip(1))?;

    let cwd = env::current_dir().context("failed to read current working directory")?;
    safety::check(&cwd)?;

    let config = PurgeConfig::hardcoded();
    let candidates = scan::scan(&cwd, &config)?;

    let mut findings: Vec<Finding> = size::estimate_sizes(&candidates);
    findings.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    preview::print(&cwd, &findings);

    if mode == Mode::Check || findings.is_empty() {
        return Ok(());
    }

    if !confirm::prompt(preview::total_bytes(&findings), findings.len())? {
        return Ok(());
    }

    let stats = delete::delete(&findings);
    preview::print_summary(stats.recovered_bytes, stats.errors);

    Ok(())
}
