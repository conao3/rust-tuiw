use crate::config::get_daemon_endpoint;
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "tuiw")]
#[command(about = "TUI applications wrapper with tmux for headless operation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Daemon,
    Create {
        command: String,
        #[arg(short, long, default_value = ".")]
        cwd: String,
    },
    Send {
        session_id: String,
        keys: String,
    },
    List,
    Capture {
        session_id: String,
    },
    Status {
        session_id: String,
    },
    Close {
        session_id: String,
    },
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

pub async fn run_client(cli: Cli) -> Result<()> {
    tracing::info!("running client");

    let command = cli.command.unwrap_or_else(|| {
        eprintln!("No command provided. Use --help for usage information.");
        std::process::exit(1);
    });

    match command {
        Commands::Daemon => {
            unreachable!("daemon command should be handled in main");
        }
        Commands::Create { command, cwd } => {
            let cwd = std::env::current_dir()?.join(&cwd).canonicalize()?;
            let cwd_str = cwd.to_string_lossy().to_string();

            #[derive(Deserialize)]
            struct CreateSessionData {
                #[serde(rename = "createSession")]
                create_session: String,
            }

            let query = r#"
                mutation CreateSession($input: CreateSessionInput!) {
                    createSession(input: $input)
                }
            "#;

            let variables = serde_json::json!({
                "input": {
                    "command": command,
                    "cwd": cwd_str,
                }
            });

            let response = send_graphql_request::<CreateSessionData>(query, variables).await?;
            println!("{}", response.create_session);
        }
        Commands::Send { session_id, keys } => {
            #[derive(Deserialize)]
            struct SendKeysData {
                #[serde(rename = "sendKeys")]
                _send_keys: bool,
            }

            let query = r#"
                mutation SendKeys($input: SendKeysInput!) {
                    sendKeys(input: $input)
                }
            "#;

            let variables = serde_json::json!({
                "input": {
                    "sessionId": session_id,
                    "keys": keys,
                }
            });

            send_graphql_request::<SendKeysData>(query, variables).await?;
        }
        Commands::List => {
            #[derive(Deserialize)]
            struct Session {
                id: String,
                command: String,
                cwd: String,
            }

            #[derive(Deserialize)]
            struct SessionsData {
                sessions: Vec<Session>,
            }

            let query = r#"
                query Sessions {
                    sessions {
                        id
                        command
                        cwd
                    }
                }
            "#;

            let response =
                send_graphql_request::<SessionsData>(query, serde_json::json!({})).await?;

            for session in response.sessions {
                println!("{}\t{}\t{}", session.id, session.command, session.cwd);
            }
        }
        Commands::Capture { session_id } => {
            #[derive(Deserialize)]
            struct SessionCaptureData {
                #[serde(rename = "sessionCapture")]
                session_capture: String,
            }

            let query = r#"
                query SessionCapture($sessionId: SessionId!) {
                    sessionCapture(sessionId: $sessionId)
                }
            "#;

            let variables = serde_json::json!({
                "sessionId": session_id,
            });

            let response = send_graphql_request::<SessionCaptureData>(query, variables).await?;
            println!("{}", response.session_capture);
        }
        Commands::Status { session_id } => {
            #[derive(Deserialize)]
            struct SessionStatusData {
                #[serde(rename = "sessionStatus")]
                session_status: String,
            }

            let query = r#"
                query SessionStatus($sessionId: SessionId!) {
                    sessionStatus(sessionId: $sessionId)
                }
            "#;

            let variables = serde_json::json!({
                "sessionId": session_id,
            });

            let response = send_graphql_request::<SessionStatusData>(query, variables).await?;
            println!("{}", response.session_status);
        }
        Commands::Close { session_id } => {
            #[derive(Deserialize)]
            struct CloseSessionData {
                #[serde(rename = "closeSession")]
                close_session: bool,
            }

            let query = r#"
                mutation CloseSession($sessionId: SessionId!) {
                    closeSession(sessionId: $sessionId)
                }
            "#;

            let variables = serde_json::json!({
                "sessionId": session_id,
            });

            let response = send_graphql_request::<CloseSessionData>(query, variables).await?;
            if !response.close_session {
                anyhow::bail!("Session not found");
            }
        }
    }

    Ok(())
}

async fn send_graphql_request<T: for<'de> Deserialize<'de>>(
    query: &str,
    variables: serde_json::Value,
) -> Result<T> {
    let endpoint = get_daemon_endpoint();
    let client = reqwest::Client::new();

    let request = GraphQLRequest {
        query: query.to_string(),
        variables,
    };

    let response = client
        .post(format!("{}/graphql", endpoint))
        .json(&request)
        .send()
        .await?;

    let graphql_response: GraphQLResponse<T> = response.json().await?;

    if let Some(errors) = graphql_response.errors {
        let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        anyhow::bail!("GraphQL errors: {}", error_messages.join(", "));
    }

    graphql_response
        .data
        .ok_or_else(|| anyhow::anyhow!("No data in response"))
}
