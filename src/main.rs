use anyhow::Result;
use clap::Parser;
use pontis::bootstrap::{Cli, run};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(&cli)
}
