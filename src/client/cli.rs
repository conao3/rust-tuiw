use crate::tmux::wrapper;
use crate::types::SessionId;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tuiw")]
#[command(about = "TUI applications wrapper with tmux for headless operation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Create {
        command: String,
        #[arg(short, long, default_value = ".")]
        cwd: String,
    },
    Send {
        session_id: String,
        keys: String,
        #[arg(short = 'n', long)]
        no_newline: bool,
    },
    List,
    View {
        session_id: String,
        #[arg(long)]
        no_color: bool,
    },
    Status {
        session_id: String,
    },
    Close {
        session_id: String,
    },
}

pub async fn run_client(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Create { command, cwd } => {
            let cwd = std::env::current_dir()?.join(&cwd).canonicalize()?;
            let cwd_str = cwd.to_string_lossy().to_string();

            let session_id = SessionId::new();
            let tmux_session = format!("tuiw-{}", session_id.0);

            wrapper::create_session(&tmux_session, &command, &cwd_str).await?;
            println!("{}", session_id.0);
        }
        Commands::Send {
            session_id,
            keys,
            no_newline,
        } => {
            let tmux_session = format!("tuiw-{}", session_id);
            wrapper::send_keys(&tmux_session, &keys).await?;

            if !no_newline {
                wrapper::send_keys(&tmux_session, "Enter").await?;
            }
        }
        Commands::List => {
            let sessions = wrapper::list_sessions().await?;
            for session in sessions {
                if let Some(id) = session.strip_prefix("tuiw-") {
                    println!("{}", id);
                }
            }
        }
        Commands::View {
            session_id,
            no_color,
        } => {
            let tmux_session = format!("tuiw-{}", session_id);
            let output = wrapper::capture_pane_with_color(&tmux_session, !no_color).await?;
            print!("{}", output);
        }
        Commands::Status { session_id } => {
            let tmux_session = format!("tuiw-{}", session_id);
            let exists = wrapper::session_exists(&tmux_session).await?;
            if exists {
                println!("Running");
            } else {
                println!("Stopped");
            }
        }
        Commands::Close { session_id } => {
            let tmux_session = format!("tuiw-{}", session_id);
            let exists = wrapper::session_exists(&tmux_session).await?;
            if !exists {
                anyhow::bail!("Session not found");
            }
            wrapper::kill_session(&tmux_session).await?;
        }
    }

    Ok(())
}
