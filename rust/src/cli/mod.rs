use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Only scan and show what can be recovered, without deleting.
    #[arg(short, long)]
    pub check: bool,

    /// Delete files permanently instead of moving them to the trash bin.
    #[arg(long)]
    pub permanent: bool,

    /// Skip confirmation prompt (non-interactive mode).
    #[arg(short, long)]
    pub yes: bool,

    /// Include global tool caches (~/.cargo, ~/.npm, etc).
    #[arg(long)]
    pub cache: bool,

    /// Include application caches (macOS Library/Caches, Windows AppData).
    #[arg(long)]
    pub deep: bool,

    /// Include system caches (Docker, apt, pacman) - requires confirmation.
    #[arg(long)]
    pub aggressive: bool,

    /// Optional path to scan (defaults to current directory).
    #[arg(default_value = ".")]
    pub path: std::path::PathBuf,
}

pub fn parse() -> Cli {
    Cli::parse()
}
