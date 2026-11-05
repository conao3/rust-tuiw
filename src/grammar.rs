use std::ffi::OsString;

use clap::error::ErrorKind;
use clap::{Args, Parser, ValueEnum};
use serde_json::{Map, Value};

pub const EXIT_PARAM: i32 = 252;
pub const EXIT_TARGET: i32 = 254;
pub const EXIT_OTHER: i32 = 255;

#[derive(Debug)]
pub enum CliError {
    Param(String),
    Target(String),
    Other(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Param(_) => EXIT_PARAM,
            CliError::Target(_) => EXIT_TARGET,
            CliError::Other(_) => EXIT_OTHER,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            CliError::Param(msg) | CliError::Target(msg) | CliError::Other(msg) => msg,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    /// Output format
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Json)]
    pub output: OutputFormat,
    /// Print parsed parameters to stderr
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Text,
    Table,
}

pub enum Parsed<C> {
    Run(C),
    Exit(i32),
}

pub fn parse<C: Parser + std::fmt::Debug>(args: Vec<OsString>) -> Parsed<C> {
    let args = normalize_help(args);
    let debug = args.iter().any(|arg| arg == "--debug");
    if debug {
        eprintln!("[debug] argv: {args:?}");
    }
    let cli = match C::try_parse_from(&args) {
        Ok(cli) => cli,
        Err(err) => return Parsed::Exit(print_clap_error(err)),
    };
    if debug {
        eprintln!("[debug] parsed: {cli:?}");
    }
    Parsed::Run(cli)
}

fn print_clap_error(err: clap::Error) -> i32 {
    let _ = err.print();
    match err.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
        _ => EXIT_PARAM,
    }
}

fn normalize_help(mut args: Vec<OsString>) -> Vec<OsString> {
    let mut i = 1;
    while i < args.len() {
        let arg = args[i].to_string_lossy();
        if arg == "--output" {
            i += 2;
        } else if arg.starts_with("--") {
            i += 1;
        } else {
            break;
        }
    }
    let leaf = i + 2;
    if args.get(leaf).is_some_and(|arg| arg == "help") {
        args[leaf] = "--help".into();
    }
    args
}

pub fn parse_string(value: &str) -> Result<String, String> {
    match value.strip_prefix("file://") {
        Some(path) => std::fs::read_to_string(path).map_err(|err| format!("read {path}: {err}")),
        None => Ok(value.to_string()),
    }
}

pub fn emit(global: &GlobalArgs, value: &Value) {
    print!("{}", render(global.output, value));
}

pub fn render(format: OutputFormat, value: &Value) -> String {
    match format {
        OutputFormat::Json => {
            let mut out = serde_json::to_string_pretty(value).expect("json value serializes");
            out.push('\n');
            out
        }
        OutputFormat::Text => {
            let mut out = String::new();
            render_text(value, &mut out);
            out
        }
        OutputFormat::Table => render_table(value),
    }
}
fn scalar_text(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::String(s) => s.clone(),
        Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    if !line.ends_with('\n') {
        out.push('\n');
    }
}

fn is_nested(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

fn render_text(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let scalars: Vec<String> = map
                .values()
                .filter(|v| !is_nested(v))
                .map(scalar_text)
                .collect();
            if !scalars.is_empty() {
                push_line(out, &scalars.join("\t"));
            }
            for nested in map.values().filter(|v| is_nested(v)) {
                render_text(nested, out);
            }
        }
        Value::Array(items) if items.iter().all(|v| !is_nested(v)) => {
            if !items.is_empty() {
                let line: Vec<String> = items.iter().map(scalar_text).collect();
                push_line(out, &line.join("\t"));
            }
        }
        Value::Array(items) => {
            for item in items {
                render_text(item, out);
            }
        }
        scalar => push_line(out, &scalar_text(scalar)),
    }
}

fn render_table(value: &Value) -> String {
    let (header, rows): (Option<Vec<String>>, Vec<Vec<String>>) = match value {
        Value::Object(map) => (
            None,
            map.iter()
                .map(|(key, v)| vec![key.clone(), scalar_text(v)])
                .collect(),
        ),
        Value::Array(items) if !items.is_empty() && items.iter().all(Value::is_object) => {
            let mut keys: Vec<String> = Vec::new();
            for item in items {
                for key in item.as_object().into_iter().flat_map(Map::keys) {
                    if !keys.contains(key) {
                        keys.push(key.clone());
                    }
                }
            }
            let rows = items
                .iter()
                .map(|item| {
                    keys.iter()
                        .map(|key| item.get(key).map(scalar_text).unwrap_or_default())
                        .collect()
                })
                .collect();
            (Some(keys), rows)
        }
        Value::Array(items) => (None, items.iter().map(|v| vec![scalar_text(v)]).collect()),
        scalar => (None, vec![vec![scalar_text(scalar)]]),
    };
    let columns = header
        .as_ref()
        .map(Vec::len)
        .or_else(|| rows.iter().map(Vec::len).max())
        .unwrap_or(0);
    if columns == 0 {
        return String::new();
    }
    let mut widths = vec![0usize; columns];
    for row in header.iter().chain(rows.iter()) {
        for (i, cell) in row.iter().enumerate() {
            for line in cell.lines() {
                widths[i] = widths[i].max(line.chars().count());
            }
        }
    }
    let border = {
        let parts: Vec<String> = widths.iter().map(|w| "-".repeat(w + 2)).collect();
        format!("+{}+\n", parts.join("+"))
    };
    let draw_row = |row: &Vec<String>, out: &mut String| {
        let cells: Vec<Vec<&str>> = row.iter().map(|cell| cell.lines().collect()).collect();
        let height = cells.iter().map(Vec::len).max().unwrap_or(0).max(1);
        for line in 0..height {
            out.push('|');
            for (i, width) in widths.iter().enumerate() {
                let text = cells
                    .get(i)
                    .and_then(|c| c.get(line))
                    .copied()
                    .unwrap_or("");
                let pad = width - text.chars().count();
                out.push_str(&format!(" {text}{} |", " ".repeat(pad)));
            }
            out.push('\n');
        }
    };
    let mut out = border.clone();
    if let Some(header) = &header {
        draw_row(header, &mut out);
        out.push_str(&border);
    }
    for row in &rows {
        draw_row(row, &mut out);
    }
    out.push_str(&border);
    out
}

pub fn run<C: Parser + std::fmt::Debug>(
    args: Vec<OsString>,
    execute: impl FnOnce(C) -> Result<(), CliError>,
) -> i32 {
    match parse::<C>(args) {
        Parsed::Exit(code) => code,
        Parsed::Run(cli) => match execute(cli) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{}", err.message());
                err.exit_code()
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Subcommand;

    #[derive(Parser, Debug)]
    #[command(name = "t", subcommand_required = true)]
    struct Cli {
        #[command(flatten)]
        global: GlobalArgs,
        #[command(subcommand)]
        command: Command,
    }

    #[derive(Subcommand, Debug)]
    enum Command {
        #[command(subcommand)]
        Thing(ThingCommand),
    }

    #[derive(Subcommand, Debug)]
    enum ThingCommand {
        GetThing(GetThingArgs),
    }

    #[derive(Args, Debug)]
    struct GetThingArgs {
        #[arg(long, value_name = "STRING", value_parser = parse_string)]
        name: Option<String>,
        #[arg(long, value_name = "INTEGER", value_parser = clap::value_parser!(u16))]
        port: Option<u16>,
        #[arg(long, overrides_with = "no_auth")]
        auth: bool,
        #[arg(long, overrides_with = "auth")]
        no_auth: bool,
    }

    fn argv(args: &[&str]) -> Vec<OsString> {
        std::iter::once("t")
            .chain(args.iter().copied())
            .map(OsString::from)
            .collect()
    }

    fn run_ok(args: &[&str]) -> GetThingArgs {
        match parse::<Cli>(argv(args)) {
            Parsed::Run(cli) => {
                let Command::Thing(ThingCommand::GetThing(args)) = cli.command;
                args
            }
            Parsed::Exit(code) => panic!("exited with {code}"),
        }
    }

    fn exit_code(args: &[&str]) -> i32 {
        match parse::<Cli>(argv(args)) {
            Parsed::Run(_) => panic!("expected exit"),
            Parsed::Exit(code) => code,
        }
    }

    #[test]
    fn rejects_positional_arguments() {
        assert_eq!(exit_code(&["thing", "get-thing", "x"]), EXIT_PARAM);
    }

    #[test]
    fn help_at_every_level() {
        assert_eq!(exit_code(&["help"]), 0);
        assert_eq!(exit_code(&["thing", "help"]), 0);
        assert_eq!(exit_code(&["thing", "get-thing", "help"]), 0);
    }

    #[test]
    fn negated_flag_overrides_the_flag() {
        let args = run_ok(&["thing", "get-thing", "--auth", "--no-auth"]);
        assert!(!args.auth);
        assert!(args.no_auth);
    }

    #[test]
    fn file_prefix_reads_value() {
        let path = std::env::temp_dir().join(format!("tuiw-grammar-{}", std::process::id()));
        std::fs::write(&path, "from file").unwrap();
        let value = format!("file://{}", path.display());
        let args = run_ok(&["thing", "get-thing", "--name", &value]);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(args.name.as_deref(), Some("from file"));
    }

    #[test]
    fn text_and_table_output() {
        let value = serde_json::json!({"A": "x", "Bb": 1});
        assert_eq!(render(OutputFormat::Text, &value), "x\t1\n");
        assert_eq!(
            render(OutputFormat::Table, &value),
            "+----+---+\n| A  | x |\n| Bb | 1 |\n+----+---+\n"
        );
    }
}
