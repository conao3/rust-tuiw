mod client;
mod config;
mod daemon;
mod graphql;
mod tmux;
mod types;

use anyhow::Result;
use clap::Parser;
use client::cli::{Cli, Commands, run_client};
use config::get_daemon_endpoint;
use daemon::server::run_daemon;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Some(Commands::Daemon) = cli.command {
        tracing::info!("starting daemon");
        let runtime = tokio::runtime::Runtime::new()?;
        return runtime.block_on(async { run_daemon().await });
    }

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        if !is_daemon_running().await {
            eprintln!("Error: daemon is not running");
            eprintln!("Start the daemon with: tuiw daemon");
            std::process::exit(1);
        }

        tracing::info!("running as client");
        run_client(cli).await?;

        Ok(())
    })
}

async fn is_daemon_running() -> bool {
    let endpoint = get_daemon_endpoint();
    let client = reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_millis(100))
        .build()
        .unwrap();

    match client.get(format!("{}/health", endpoint)).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}
