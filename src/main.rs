mod client;
mod store;
mod tmux;
mod types;

use anyhow::Result;
use clap::Parser;
use client::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async { client::cli::run_client(cli).await })
}
