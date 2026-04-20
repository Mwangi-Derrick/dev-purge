use crate::ui::preview::format_bytes;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};

pub fn prompt(total: u64, count: usize) -> Result<bool> {
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

