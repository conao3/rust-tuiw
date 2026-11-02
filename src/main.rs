mod client;
mod grammar;
mod store;
mod tmux;
mod types;

use client::cli::Cli;
use grammar::CliError;

fn execute(cli: Cli) -> Result<(), CliError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|err| CliError::Other(format!("failed to start tokio runtime: {err}")))?;
    runtime.block_on(client::cli::run_client(cli))
}

fn main() {
    std::process::exit(grammar::run::<Cli>(std::env::args_os().collect(), execute));
}
