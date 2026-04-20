use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Default,
    Check,
}

pub fn parse_mode<I>(args: I) -> Result<Mode>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();

    if args.is_empty() {
        return Ok(Mode::Default);
    }

    if args.len() == 1 && args[0] == "--check" {
        return Ok(Mode::Check);
    }

    eprintln!("Usage: dev-purge [--check]");
    bail!("invalid arguments")
}
