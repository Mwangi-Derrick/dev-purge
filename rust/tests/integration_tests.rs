//! Integration tests for the trait-based domain implementations.
//!
//! These tests ensure that the core scanning, safety checking, and cleaning
//! functionality works correctly and can be extended by contributors.

use assert_fs::prelude::*;
use std::path::Path;

use dev_purge::domain::{
    config::PurgeConfig,
    impls::{ParallelScanner, OsSafetyChecker, StandardCleaner},
    traits::{Scanner, SafetyChecker, Cleaner},
};

#[test]
fn test_parallel_scanner_finds_artifacts() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create some test artifacts
    temp.child("target").child("debug").create_dir_all().unwrap();
    temp.child("target").child("debug").child("binary.exe").write_binary(b"fake binary").unwrap();

    temp.child("node_modules").child("some-package").create_dir_all().unwrap();
    temp.child("node_modules").child("some-package").child("index.js").write_binary(b"console.log('test')").unwrap();

    // Create a protected directory that should be ignored
    temp.child(".git").create_dir_all().unwrap();
    temp.child(".git").child("config").write_binary(b"[core]").unwrap();

    let config = PurgeConfig::hardcoded();
    let scanner = ParallelScanner::new(config);

    let results = scanner.scan(temp.path()).unwrap();

    // Should find target/ and node_modules/, but not .git/
    assert!(results.len() >= 2);
    assert!(results.iter().any(|r| r.path.to_string_lossy().contains("target")));
    assert!(results.iter().any(|r| r.path.to_string_lossy().contains("node_modules")));
    assert!(!results.iter().any(|r| r.path.to_string_lossy().contains(".git")));
}

#[test]
fn test_os_safety_checker() {
    let checker = OsSafetyChecker;

    // Safe paths
    assert!(checker.is_safe(Path::new("target")));
    assert!(checker.is_safe(Path::new("node_modules")));

    // Protected paths
    assert!(!checker.is_safe(Path::new(".git")));
    assert!(!checker.is_safe(Path::new(".vscode")));
    assert!(!checker.is_safe(Path::new("C:\\Windows\\System32")));
}

#[test]
fn test_standard_cleaner_dry_run() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("target").child("debug").create_dir_all().unwrap();
    temp.child("target").child("debug").child("binary.exe").write_binary(b"fake binary").unwrap();

    let config = PurgeConfig::hardcoded();
    let scanner = ParallelScanner::new(config);
    let cleaner = StandardCleaner;

    let results = scanner.scan(temp.path()).unwrap();
    let stats = cleaner.clean(&results, true).unwrap(); // dry run

    // In dry run, nothing should be deleted but we get stats about what would be deleted
    assert_eq!(stats.items_deleted, 1); // Would delete 1 item
    assert!(stats.total_bytes_freed > 0); // Would free some bytes
    assert!(stats.errors.is_empty());

    // But files should still exist
    assert!(temp.path().join("target").exists());
}

#[test]
fn test_standard_cleaner_actual_deletion() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child("target").child("debug").create_dir_all().unwrap();
    temp.child("target").child("debug").child("binary.exe").write_binary(b"fake binary").unwrap();

    let config = PurgeConfig::hardcoded();
    let scanner = ParallelScanner::new(config);
    let cleaner = StandardCleaner;

    let results = scanner.scan(temp.path()).unwrap();
    let stats = cleaner.clean(&results, false).unwrap(); // actual deletion

    // Files should be deleted
    assert!(!temp.path().join("target").exists());

    // Should have some bytes freed
    assert!(stats.total_bytes_freed > 0);
    assert!(stats.items_deleted > 0);
    assert!(stats.errors.is_empty());
}