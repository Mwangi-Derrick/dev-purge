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

    /// Optional path to scan (defaults to current directory).
    #[arg(default_value = ".")]
    pub path: std::path::PathBuf,
}

pub fn parse() -> Cli {
    Cli::parse()
}
