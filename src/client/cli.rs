use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rust-tuiw")]
#[command(about = "TUI applications wrapper with tmux for headless operation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Create {
        command: String,
    },
    Send {
        session_id: String,
        keys: String,
    },
    List,
    Close {
        session_id: String,
    },
}

pub async fn run_client(cli: Cli) -> Result<()> {
    tracing::info!("running client");
    Ok(())
}
