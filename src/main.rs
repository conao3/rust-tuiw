mod client;
mod config;
mod daemon;
mod graphql;
mod tmux;
mod types;

use anyhow::Result;
use clap::Parser;
use client::cli::{run_client, Cli};
use config::get_daemon_endpoint;
use daemon::server::run_daemon;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if is_daemon_running().await {
        tracing::info!("daemon is already running, running as client");
        run_client(cli).await?;
    } else {
        tracing::info!("daemon is not running, starting daemon");
        tokio::spawn(async {
            if let Err(e) = run_daemon().await {
                tracing::error!("daemon error: {}", e);
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        run_client(cli).await?;
    }

    Ok(())
}

async fn is_daemon_running() -> bool {
    let endpoint = get_daemon_endpoint();
    let client = reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_millis(100))
        .build()
        .unwrap();

    match client.get(&format!("{}/health", endpoint)).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
