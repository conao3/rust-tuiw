use std::ffi::OsString;

use clap::error::ErrorKind;
use clap::parser::ValueSource;
use clap::{ArgAction, ArgMatches, Args, CommandFactory, Parser, ValueEnum};
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
    /// JMESPath query applied to the output
    #[arg(long, global = true, value_name = "JMESPATH")]
    pub query: Option<String>,
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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonKind {
    Input,
    Output,
}

#[derive(Args, Debug, Clone)]
pub struct InputArgs {
    /// Read all parameters from JSON (keys are PascalCase parameter names)
    #[arg(long, value_name = "JSON", value_parser = parse_string)]
    pub cli_input_json: Option<String>,
    /// Print an input or output skeleton instead of running the command
    #[arg(
        long,
        value_enum,
        value_name = "TYPE",
        num_args = 0..=1,
        default_missing_value = "input"
    )]
    pub generate_cli_skeleton: Option<SkeletonKind>,
}

pub trait CliSpec: Parser + std::fmt::Debug {
    fn global(&self) -> &GlobalArgs;
    fn input(&self) -> &InputArgs;
    fn output_skeleton(&self) -> Value;
}

pub enum Parsed<C> {
    Run(C, Option<Value>),
    Exit(i32),
}

const INPUT_ARG_IDS: [&str; 2] = ["cli_input_json", "generate_cli_skeleton"];

pub fn parse<C: CliSpec>(args: Vec<OsString>) -> Parsed<C> {
    let args = normalize_help(args);
    let debug = args.iter().any(|arg| arg == "--debug");
    if debug {
        eprintln!("[debug] argv: {args:?}");
    }
    let matches = match C::command().try_get_matches_from(&args) {
        Ok(matches) => matches,
        Err(err) => return Parsed::Exit(print_clap_error(err)),
    };
    let matches = match merge_cli_input_json::<C>(&args, matches, debug) {
        Ok(matches) => matches,
        Err(err) => {
            eprintln!("{}", err.message());
            return Parsed::Exit(err.exit_code());
        }
    };
    let cli = match C::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(err) => return Parsed::Exit(print_clap_error(err)),
    };
    if debug {
        eprintln!("[debug] parsed: {cli:?}");
    }
    if let Some(query) = &cli.global().query
        && let Err(err) = jmespath::compile(query)
    {
        eprintln!("invalid --query: {err}");
        return Parsed::Exit(EXIT_PARAM);
    }
    let skeleton = cli.input().generate_cli_skeleton.map(|kind| match kind {
        SkeletonKind::Input => input_skeleton::<C>(&matches),
        SkeletonKind::Output => cli.output_skeleton(),
    });
    Parsed::Run(cli, skeleton)
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
        if arg == "--output" || arg == "--query" {
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

fn leaf_matches<C: CommandFactory>(matches: &ArgMatches) -> Option<(clap::Command, &ArgMatches)> {
    let (command, command_matches) = matches.subcommand()?;
    let (subcommand, leaf) = command_matches.subcommand()?;
    let definition = C::command()
        .find_subcommand(command)?
        .find_subcommand(subcommand)?
        .clone();
    Some((definition, leaf))
}

struct Param<'a> {
    key: String,
    arg: &'a clap::Arg,
    negation: Option<&'a clap::Arg>,
}

fn params(definition: &clap::Command) -> Vec<Param<'_>> {
    let args: Vec<&clap::Arg> = definition
        .get_arguments()
        .filter(|arg| arg.get_long().is_some() && !arg.is_global_set())
        .filter(|arg| !INPUT_ARG_IDS.contains(&arg.get_id().as_str()))
        .filter(|arg| !matches!(arg.get_id().as_str(), "help" | "version"))
        .collect();
    let long_of = |arg: &clap::Arg| arg.get_long().unwrap_or_default().to_string();
    args.iter()
        .filter(|arg| {
            let long = long_of(arg);
            !long
                .strip_prefix("no-")
                .is_some_and(|base| args.iter().any(|other| long_of(other) == base))
        })
        .map(|arg| {
            let long = long_of(arg);
            let negation = args
                .iter()
                .find(|other| long_of(other) == format!("no-{long}"))
                .copied();
            Param {
                key: pascal_case(&long),
                arg,
                negation,
            }
        })
        .collect()
}

pub fn pascal_case(kebab: &str) -> String {
    kebab
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

fn is_flag(arg: &clap::Arg) -> bool {
    matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse)
}

fn is_list(arg: &clap::Arg) -> bool {
    arg.get_num_args()
        .is_some_and(|range| range.max_values() > 1)
}

fn is_integer(arg: &clap::Arg) -> bool {
    arg.get_value_names()
        .is_some_and(|names| names.iter().any(|name| name == "INTEGER"))
}

fn input_skeleton<C: CommandFactory>(matches: &ArgMatches) -> Value {
    let Some((definition, _)) = leaf_matches::<C>(matches) else {
        return Value::Object(Map::new());
    };
    let map = params(&definition)
        .into_iter()
        .map(|param| {
            let scalar = if is_flag(param.arg) {
                Value::Bool(true)
            } else if is_integer(param.arg) {
                Value::from(0)
            } else {
                Value::String(String::new())
            };
            let value = if is_list(param.arg) {
                Value::Array(vec![scalar])
            } else {
                scalar
            };
            (param.key, value)
        })
        .collect();
    Value::Object(map)
}

fn merge_cli_input_json<C: CommandFactory>(
    args: &[OsString],
    matches: ArgMatches,
    debug: bool,
) -> Result<ArgMatches, CliError> {
    let Some((definition, leaf)) = leaf_matches::<C>(&matches) else {
        return Ok(matches);
    };
    let Some(json) = leaf.get_one::<String>("cli_input_json") else {
        return Ok(matches);
    };
    let input: Map<String, Value> = serde_json::from_str(json)
        .map_err(|err| CliError::Param(format!("invalid --cli-input-json: {err}")))?;
    let params = params(&definition);
    let given = |arg: &clap::Arg| {
        leaf.value_source(arg.get_id().as_str()) == Some(ValueSource::CommandLine)
    };
    let mut extra: Vec<OsString> = Vec::new();
    for (key, value) in &input {
        let param = params
            .iter()
            .find(|param| &param.key == key)
            .ok_or_else(|| {
                CliError::Param(format!("unknown parameter in --cli-input-json: {key}"))
            })?;
        if given(param.arg) || param.negation.is_some_and(given) {
            continue;
        }
        let long = param.arg.get_long().unwrap_or_default();
        let invalid = || CliError::Param(format!("invalid value for {key} in --cli-input-json"));
        if is_flag(param.arg) {
            match (value, param.negation) {
                (Value::Bool(true), _) => extra.push(format!("--{long}").into()),
                (Value::Bool(false), Some(negation)) => {
                    extra.push(format!("--{}", negation.get_long().unwrap_or_default()).into())
                }
                (Value::Bool(false), None) => {}
                _ => return Err(invalid()),
            }
        } else if is_list(param.arg) {
            let Value::Array(items) = value else {
                return Err(invalid());
            };
            extra.push(format!("--{long}").into());
            for item in items {
                extra.push(scalar_token(item).ok_or_else(invalid)?.into());
            }
        } else {
            let token = scalar_token(value).ok_or_else(invalid)?;
            extra.push(format!("--{long}={token}").into());
        }
    }
    if debug {
        eprintln!("[debug] cli-input-json parameters: {extra:?}");
    }
    if extra.is_empty() {
        return Ok(matches);
    }
    let merged: Vec<OsString> = args.iter().cloned().chain(extra).collect();
    C::command()
        .try_get_matches_from(merged)
        .map_err(|err| CliError::Param(err.to_string().trim_end().to_string()))
}

fn scalar_token(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

pub fn parse_string(value: &str) -> Result<String, String> {
    match value.strip_prefix("file://") {
        Some(path) => std::fs::read_to_string(path).map_err(|err| format!("read {path}: {err}")),
        None => Ok(value.to_string()),
    }
}

pub fn emit(global: &GlobalArgs, value: &Value) -> Result<(), CliError> {
    let value = match &global.query {
        Some(query) => {
            let expr = jmespath::compile(query)
                .map_err(|err| CliError::Param(format!("invalid --query: {err}")))?;
            let found = expr
                .search(value)
                .map_err(|err| CliError::Param(format!("--query failed: {err}")))?;
            serde_json::to_value(&*found).map_err(|err| CliError::Other(err.to_string()))?
        }
        None => value.clone(),
    };
    print!("{}", render(global.output, &value));
    Ok(())
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

pub fn run<C: CliSpec>(
    args: Vec<OsString>,
    execute: impl FnOnce(C) -> Result<(), CliError>,
) -> i32 {
    match parse::<C>(args) {
        Parsed::Exit(code) => code,
        Parsed::Run(_, Some(skeleton)) => {
            print!("{}", render(OutputFormat::Json, &skeleton));
            0
        }
        Parsed::Run(cli, None) => match execute(cli) {
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
        #[command(flatten)]
        input: InputArgs,
    }

    impl CliSpec for Cli {
        fn global(&self) -> &GlobalArgs {
            &self.global
        }
        fn input(&self) -> &InputArgs {
            let Command::Thing(ThingCommand::GetThing(args)) = &self.command;
            &args.input
        }
        fn output_skeleton(&self) -> Value {
            serde_json::json!({"Name": ""})
        }
    }

    fn argv(args: &[&str]) -> Vec<OsString> {
        std::iter::once("t")
            .chain(args.iter().copied())
            .map(OsString::from)
            .collect()
    }

    fn run_ok(args: &[&str]) -> (GetThingArgs, Option<Value>) {
        match parse::<Cli>(argv(args)) {
            Parsed::Run(cli, skeleton) => {
                let Command::Thing(ThingCommand::GetThing(args)) = cli.command;
                (args, skeleton)
            }
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
    fn input_skeleton_pairs_flags() {
        let (_, skeleton) = run_ok(&["thing", "get-thing", "--generate-cli-skeleton"]);
        assert_eq!(
            skeleton.unwrap(),
            serde_json::json!({"Name": "", "Port": 0, "Auth": true})
        );
    }

    #[test]
    fn cli_input_json_yields_to_explicit_parameters() {
        let (args, _) = run_ok(&[
            "thing",
            "get-thing",
            "--port",
            "2",
            "--cli-input-json",
            r#"{"Name": "a", "Port": 1, "Auth": false}"#,
        ]);
        assert_eq!(args.name.as_deref(), Some("a"));
        assert_eq!(args.port, Some(2));
        assert!(args.no_auth);
    }

    #[test]
    fn cli_input_json_rejects_unknown_keys() {
        assert_eq!(
            exit_code(&["thing", "get-thing", "--cli-input-json", r#"{"X": 1}"#]),
            EXIT_PARAM
        );
    }

    #[test]
    fn invalid_query_is_parameter_error() {
        assert_eq!(
            exit_code(&["--query", "[", "thing", "get-thing"]),
            EXIT_PARAM
        );
    }

    #[test]
    fn file_prefix_reads_value() {
        let path = std::env::temp_dir().join(format!("tuiw-grammar-{}", std::process::id()));
        std::fs::write(&path, "from file").unwrap();
        let value = format!("file://{}", path.display());
        let (args, _) = run_ok(&["thing", "get-thing", "--name", &value]);
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
