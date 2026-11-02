use crate::grammar::{CliError, CliSpec, GlobalArgs, InputArgs, emit, parse_string};
use crate::store;
use crate::tmux::wrapper;
use crate::types::SessionId;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(
    name = "tuiw",
    version,
    about = "TUI applications wrapper with tmux for headless operation",
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// tmux sessions that run TUI applications
    #[command(subcommand)]
    Session(SessionCommand),
}

#[derive(Subcommand, Debug)]
pub enum SessionCommand {
    /// Start a command in a new tmux session and print its session ID
    CreateSession(CreateSessionArgs),
    /// List tuiw sessions
    ListSessions(ListSessionsArgs),
    /// Send keys to a session
    SendKeys(SendKeysArgs),
    /// Capture the current screen of a session
    GetScreen(GetScreenArgs),
    /// Kill a session
    DeleteSession(DeleteSessionArgs),
}

#[derive(Args, Debug, Clone)]
pub struct CreateSessionArgs {
    /// Shell command to run in the session
    #[arg(long, value_name = "STRING", value_parser = parse_string)]
    pub command: Option<String>,
    /// Working directory (default: current directory)
    #[arg(long, value_name = "STRING", value_parser = parse_string)]
    pub cwd: Option<String>,
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ListSessionsArgs {
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct SendKeysArgs {
    /// Session ID printed by create-session
    #[arg(long, value_name = "STRING", value_parser = parse_string)]
    pub session_id: Option<String>,
    /// Keys passed to tmux send-keys in order (literal text or key names such as Escape, C-c)
    #[arg(long, value_name = "STRING", num_args = 1.., value_parser = parse_string)]
    pub keys: Option<Vec<String>>,
    /// Send Enter after the keys (default)
    #[arg(long, overrides_with = "no_newline")]
    pub newline: bool,
    /// Do not send Enter after the keys
    #[arg(long, overrides_with = "newline")]
    pub no_newline: bool,
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GetScreenArgs {
    /// Session ID printed by create-session
    #[arg(long, value_name = "STRING", value_parser = parse_string)]
    pub session_id: Option<String>,
    /// Keep the ANSI color escape sequences (default)
    #[arg(long, overrides_with = "no_color")]
    pub color: bool,
    /// Strip the ANSI color escape sequences
    #[arg(long, overrides_with = "color")]
    pub no_color: bool,
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DeleteSessionArgs {
    /// Session ID printed by create-session
    #[arg(long, value_name = "STRING", value_parser = parse_string)]
    pub session_id: Option<String>,
    #[command(flatten)]
    pub input: InputArgs,
}

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CreateSessionOutput {
    pub session_id: String,
}

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SessionSummary {
    pub session_id: String,
    pub command: String,
    pub cwd: String,
}

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ListSessionsOutput {
    pub sessions: Vec<SessionSummary>,
}

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GetScreenOutput {
    pub session_id: String,
    pub screen: String,
}

impl CliSpec for Cli {
    fn global(&self) -> &GlobalArgs {
        &self.global
    }

    fn input(&self) -> &InputArgs {
        let Command::Session(command) = &self.command;
        match command {
            SessionCommand::CreateSession(args) => &args.input,
            SessionCommand::ListSessions(args) => &args.input,
            SessionCommand::SendKeys(args) => &args.input,
            SessionCommand::GetScreen(args) => &args.input,
            SessionCommand::DeleteSession(args) => &args.input,
        }
    }

    fn output_skeleton(&self) -> Value {
        let Command::Session(command) = &self.command;
        let value = match command {
            SessionCommand::CreateSession(_) => {
                serde_json::to_value(CreateSessionOutput::default())
            }
            SessionCommand::ListSessions(_) => serde_json::to_value(ListSessionsOutput {
                sessions: vec![SessionSummary::default()],
            }),
            SessionCommand::GetScreen(_) => serde_json::to_value(GetScreenOutput::default()),
            SessionCommand::SendKeys(_) | SessionCommand::DeleteSession(_) => {
                Ok(Value::Object(serde_json::Map::new()))
            }
        };
        value.expect("output skeleton serializes")
    }
}

fn required<T>(value: Option<T>, name: &str) -> Result<T, CliError> {
    value.ok_or_else(|| CliError::Param(format!("--{name} is required")))
}

fn target(err: anyhow::Error) -> CliError {
    CliError::Target(format!("{err:#}"))
}

fn print_output<T: Serialize>(global: &GlobalArgs, output: &T) -> Result<(), CliError> {
    let value = serde_json::to_value(output).map_err(|err| CliError::Other(err.to_string()))?;
    emit(global, &value)
}

pub async fn run_client(cli: Cli) -> Result<(), CliError> {
    let global = cli.global;
    let Command::Session(command) = cli.command;
    match command {
        SessionCommand::CreateSession(args) => {
            let command = required(args.command, "command")?;
            let current =
                std::env::current_dir().map_err(|err| CliError::Other(err.to_string()))?;
            let cwd = current
                .join(args.cwd.as_deref().unwrap_or("."))
                .canonicalize()
                .map_err(|err| CliError::Target(format!("--cwd: {err}")))?;
            let cwd_str = cwd.to_string_lossy().to_string();

            let session_id = SessionId::new();

            wrapper::create_session(&session_id.0, &command, &cwd_str)
                .await
                .map_err(target)?;
            store::add_session(session_id.0.clone(), command, cwd_str)
                .map_err(|err| CliError::Other(format!("{err:#}")))?;
            store::cleanup_stale_sessions().await.map_err(target)?;
            print_output(
                &global,
                &CreateSessionOutput {
                    session_id: session_id.0,
                },
            )
        }
        SessionCommand::SendKeys(args) => {
            let session_id = required(args.session_id, "session-id")?;
            let keys = required(args.keys, "keys")?;
            for key in &keys {
                wrapper::send_keys(&session_id, key).await.map_err(target)?;
            }
            if !args.no_newline {
                wrapper::send_keys(&session_id, "Enter")
                    .await
                    .map_err(target)?;
            }
            Ok(())
        }
        SessionCommand::ListSessions(_) => {
            let session_ids = wrapper::list_sessions().await.map_err(target)?;
            let session_store =
                store::load_store().map_err(|err| CliError::Other(format!("{err:#}")))?;
            let sessions = session_ids
                .into_iter()
                .map(|session_id| match session_store.sessions.get(&session_id) {
                    Some(info) => SessionSummary {
                        command: info.command.clone(),
                        cwd: info.cwd.clone(),
                        session_id,
                    },
                    None => SessionSummary {
                        session_id,
                        ..SessionSummary::default()
                    },
                })
                .collect();
            print_output(&global, &ListSessionsOutput { sessions })
        }
        SessionCommand::GetScreen(args) => {
            let session_id = required(args.session_id, "session-id")?;
            let screen = wrapper::capture_pane_with_color(&session_id, !args.no_color)
                .await
                .map_err(target)?;
            print_output(&global, &GetScreenOutput { session_id, screen })
        }
        SessionCommand::DeleteSession(args) => {
            let session_id = required(args.session_id, "session-id")?;
            let exists = wrapper::session_exists(&session_id).await.map_err(target)?;
            if !exists {
                return Err(CliError::Target(format!("session not found: {session_id}")));
            }
            wrapper::kill_session(&session_id).await.map_err(target)?;
            store::cleanup_stale_sessions().await.map_err(target)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{EXIT_PARAM, Parsed, parse};
    use std::ffi::OsString;

    fn argv(args: &[&str]) -> Vec<OsString> {
        std::iter::once("tuiw")
            .chain(args.iter().copied())
            .map(OsString::from)
            .collect()
    }

    fn run(args: &[&str]) -> (Cli, Option<Value>) {
        match parse::<Cli>(argv(args)) {
            Parsed::Run(cli, skeleton) => (cli, skeleton),
            Parsed::Exit(code) => panic!("exited with {code}"),
        }
    }

    fn exit_code(args: &[&str]) -> i32 {
        match parse::<Cli>(argv(args)) {
            Parsed::Run(..) => panic!("expected exit"),
            Parsed::Exit(code) => code,
        }
    }

    #[test]
    fn send_keys_takes_a_list() {
        let (cli, _) = run(&[
            "session",
            "send-keys",
            "--session-id",
            "2a638ef5",
            "--keys",
            "i",
            "Escape",
            "--no-newline",
        ]);
        let Command::Session(SessionCommand::SendKeys(args)) = cli.command else {
            panic!("unexpected command");
        };
        assert_eq!(args.session_id.as_deref(), Some("2a638ef5"));
        assert_eq!(args.keys, Some(vec!["i".to_string(), "Escape".to_string()]));
        assert!(args.no_newline);
    }

    #[test]
    fn rejects_the_positional_syntax() {
        assert_eq!(exit_code(&["create", "bash"]), EXIT_PARAM);
        assert_eq!(
            exit_code(&["session", "create-session", "bash"]),
            EXIT_PARAM
        );
    }

    #[test]
    fn help_at_every_level() {
        assert_eq!(exit_code(&["help"]), 0);
        assert_eq!(exit_code(&["session", "help"]), 0);
        assert_eq!(exit_code(&["session", "get-screen", "help"]), 0);
    }

    #[test]
    fn skeletons() {
        let (_, input) = run(&["session", "send-keys", "--generate-cli-skeleton"]);
        assert_eq!(
            input.unwrap(),
            serde_json::json!({"SessionId": "", "Keys": [""], "Newline": true})
        );
        let (_, output) = run(&[
            "session",
            "list-sessions",
            "--generate-cli-skeleton",
            "output",
        ]);
        assert_eq!(
            output.unwrap(),
            serde_json::json!({"Sessions": [{"SessionId": "", "Command": "", "Cwd": ""}]})
        );
    }

    #[test]
    fn cli_input_json_fills_parameters() {
        let (cli, _) = run(&[
            "session",
            "create-session",
            "--cli-input-json",
            r#"{"Command": "vim", "Cwd": "/tmp"}"#,
        ]);
        let Command::Session(SessionCommand::CreateSession(args)) = cli.command else {
            panic!("unexpected command");
        };
        assert_eq!(args.command.as_deref(), Some("vim"));
        assert_eq!(args.cwd.as_deref(), Some("/tmp"));
    }
}
