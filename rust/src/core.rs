use crate::domain::{
    config::PurgeConfig,
    impls::{OsSafetyChecker, ParallelScanner, StandardCleaner},
    safety,
    traits::{Cleaner, SafetyChecker, ScanTier, Scanner},
};
use crate::ui::{confirm, preview};
use anyhow::{Context, Result};
use std::io::{self, Write};

pub fn run() -> Result<()> {
    let cli = crate::cli::parse();

    let scan_root = std::fs::canonicalize(&cli.path)
        .with_context(|| format!("failed to find path: {:?}", cli.path))?;

    let tier = if cli.aggressive {
        ScanTier::Aggressive
    } else if cli.deep {
        ScanTier::Deep
    } else if cli.cache {
        ScanTier::Cache
    } else {
        ScanTier::Project
    };

    if tier == ScanTier::Aggressive && !cli.yes {
        println!("\n🔴 AGGRESSIVE MODE WARNING");
        println!("This will clear system caches including:");
        println!("  • Docker build cache (dangling layers only)");
        println!("  • Package manager caches (apt, pacman, yay)");
        println!("  • All user application caches");
        println!("\nRunning containers and installed packages are preserved.");
        println!("Caches will be rebuilt automatically when needed.\n");

        print!("Type 'yes i understand' to continue: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim() != "yes i understand" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    safety::check(&scan_root, tier)?;

    println!("🛡️  {}  {}", "DEV-PURGE:".bold(), "Smart Cleanup".dimmed());
    match tier {
        ScanTier::Project => println!("🔍 Mode: {} (safe)", "Project Caches".green()),
        ScanTier::Cache => println!("🔍 Mode: {} (+ ~/.cargo, ~/.npm, etc)", "Global Caches".yellow()),
        ScanTier::Deep => println!("🔍 Mode: {} (+ Library/Caches, AppData)", "Deep Clean".yellow()),
        ScanTier::Aggressive => println!("🔍 Mode: {} (System Caches + Docker)", "AGGRESSIVE".red().bold()),
    }
    println!();

    // Compose the default pipeline using traits
    let config = PurgeConfig::for_tier(tier);
    let scanner = ParallelScanner::new(config);
    let safety_checker = OsSafetyChecker;
    let cleaner = StandardCleaner;

    // Scan for candidates
    let mut scan_results = scanner.scan(&scan_root)?;

    // Add Docker scan (only in Aggressive mode)
    if tier == ScanTier::Aggressive {
        let docker_scanner = crate::domain::docker::DockerScanner;
        if let Ok(docker_results) = docker_scanner.scan(&scan_root) {
            scan_results.extend(docker_results);
        }
    }

    // Filter safe results (Docker results are inherently safe as they use the API)
    let safe_results: Vec<_> = scan_results
        .into_iter()
        .filter(|result| {
            if result.artifact_type == crate::domain::traits::ArtifactType::Physical {
                safety_checker.is_safe(&result.path, tier)
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
