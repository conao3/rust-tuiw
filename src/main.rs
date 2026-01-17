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
        tracing::info!("starting daemon in foreground");
        let runtime = tokio::runtime::Runtime::new()?;
        return runtime.block_on(async { run_daemon().await });
    }

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        if !is_daemon_running().await {
            tracing::info!("daemon is not running, starting daemon in background");
            start_daemon_background()?;

            for i in 1..=20 {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                if is_daemon_running().await {
                    tracing::info!("daemon started successfully");
                    break;
                }
                if i == 20 {
                    anyhow::bail!("failed to start daemon after 4 seconds");
                }
            }
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

    match client.get(&format!("{}/health", endpoint)).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

fn start_daemon_background() -> Result<()> {
    use std::os::unix::process::CommandExt;

    let exe = std::env::current_exe()?;
    let log_path = std::env::temp_dir().join("tuiw-daemon.log");
    let err_path = std::env::temp_dir().join("tuiw-daemon.err");

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let err_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&err_path)?;

    unsafe {
        std::process::Command::new(exe)
            .arg("daemon")
            .stdin(std::process::Stdio::null())
            .stdout(log_file)
            .stderr(err_file)
            .process_group(0)
            .pre_exec(|| {
                libc::setsid();
                Ok(())
            })
            .spawn()?;
    }

    tracing::info!(
        "daemon started in background, logs: {} / {}",
        log_path.display(),
        err_path.display()
    );

    Ok(())
}
