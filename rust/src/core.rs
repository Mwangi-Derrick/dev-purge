use crate::domain::{
    config::PurgeConfig,
    impls::{OsSafetyChecker, ParallelScanner, StandardCleaner},
    safety,
    traits::{Cleaner, SafetyChecker, Scanner},
};
use crate::ui::{confirm, preview};
use anyhow::{Context, Result};

pub fn run() -> Result<()> {
    let cli = crate::cli::parse();

    let scan_root = std::fs::canonicalize(&cli.path)
        .with_context(|| format!("failed to find path: {:?}", cli.path))?;

    safety::check(&scan_root)?;

    // Compose the default pipeline using traits
    let config = PurgeConfig::hardcoded();
    let scanner = ParallelScanner::new(config);
    let safety_checker = OsSafetyChecker;
    let cleaner = StandardCleaner;

    // Scan for candidates
    let mut scan_results = scanner.scan(&scan_root)?;

    // Add Docker scan
    let docker_scanner = crate::domain::docker::DockerScanner;
    if let Ok(docker_results) = docker_scanner.scan(&scan_root) {
        scan_results.extend(docker_results);
    }

    // Filter safe results (Docker results are inherently safe as they use the API)
    let safe_results: Vec<_> = scan_results
        .into_iter()
        .filter(|result| {
            if result.artifact_type == crate::domain::traits::ArtifactType::Physical {
                safety_checker.is_safe(&result.path)
            } else {
                true
            }
        })
        .collect();

    // Convert to UI format (temporary compatibility)
    let findings: Vec<_> = safe_results
        .iter()
        .map(|r| crate::domain::types::Finding {
            path: r.path.clone(),
            bytes: r.size_bytes,
        })
        .collect();

    preview::print(&scan_root, &findings);

    if cli.check || findings.is_empty() {
        return Ok(());
    }

    if !cli.yes && !confirm::prompt(preview::total_bytes(&findings), findings.len())? {
        return Ok(());
    }

    // Clean up
    let stats = cleaner.clean(&safe_results, false, cli.permanent)?;
    preview::print_summary(stats.total_bytes_freed, stats.errors.len() as u64);

    Ok(())
}
