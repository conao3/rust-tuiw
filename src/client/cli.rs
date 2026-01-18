use crate::store;
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

            wrapper::create_session(&session_id.0, &command, &cwd_str).await?;
            store::add_session(session_id.0.clone(), command, cwd_str)?;
            store::cleanup_stale_sessions().await?;
            println!("{}", session_id.0);
        }
        Commands::Send {
            session_id,
            keys,
            no_newline,
        } => {
            wrapper::send_keys(&session_id, &keys).await?;

            if !no_newline {
                wrapper::send_keys(&session_id, "Enter").await?;
            }
        }
        Commands::List => {
            let session_ids = wrapper::list_sessions().await?;
            let session_store = store::load_store()?;
            for session_id in session_ids {
                match session_store.sessions.get(&session_id) {
                    Some(info) => {
                        println!("{}\t{}\t{}", session_id, info.command, info.cwd);
                    }
                    None => {
                        println!("{}\t\t", session_id);
                    }
                }
            }
        }
        Commands::View {
            session_id,
            no_color,
        } => {
            let output = wrapper::capture_pane_with_color(&session_id, !no_color).await?;
            print!("{}", output);
        }
        Commands::Status { session_id } => {
            let exists = wrapper::session_exists(&session_id).await?;
            if exists {
                println!("Running");
            } else {
                println!("Stopped");
            }
        }
        Commands::Close { session_id } => {
            let exists = wrapper::session_exists(&session_id).await?;
            if !exists {
                anyhow::bail!("Session not found");
            }
            wrapper::kill_session(&session_id).await?;
            store::cleanup_stale_sessions().await?;
        }
    }

    Ok(())
}
