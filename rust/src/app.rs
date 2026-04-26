use crate::domain::{config::PurgeConfig, delete, safety, scan, size};
use crate::types::Finding;
use crate::ui::{confirm, preview};
use anyhow::{Context, Result};

pub fn run() -> Result<()> {
    let cli = crate::cli::parse();

    let scan_root = std::fs::canonicalize(&cli.path)
        .with_context(|| format!("failed to find path: {:?}", cli.path))?;

    safety::check(&scan_root)?;

    let config = PurgeConfig::hardcoded();
    let candidates = scan::scan(&scan_root, &config)?;

    let mut findings: Vec<Finding> = size::estimate_sizes(&candidates);
    findings.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    preview::print(&scan_root, &findings);

    if cli.check || findings.is_empty() {
        return Ok(());
    }

    if !confirm::prompt(preview::total_bytes(&findings), findings.len())? {
        return Ok(());
    }

    let stats = delete::delete(&findings);
    preview::print_summary(stats.recovered_bytes, stats.errors);

    Ok(())
}
