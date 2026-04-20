use anyhow::{bail, Result};
use colored::Colorize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn check(cwd: &Path) -> Result<()> {
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

fn canonicalize_best_effort(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn display_path_for_humans(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
