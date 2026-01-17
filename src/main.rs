mod client;
mod daemon;
mod graphql;
mod tmux;
mod types;

use anyhow::Result;
use clap::Parser;
use client::cli::{run_client, Cli};
use daemon::server::run_daemon;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if is_daemon_running().await {
        run_client(cli).await?;
    } else {
        tokio::spawn(async {
            if let Err(e) = run_daemon().await {
                tracing::error!("daemon error: {}", e);
            }
        });
        run_client(cli).await?;
    }

    Ok(())
}

async fn is_daemon_running() -> bool {
    false
}
