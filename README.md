# tuiw

[![Crates.io](https://img.shields.io/crates/v/tuiw.svg)](https://crates.io/crates/tuiw)
[![Documentation](https://docs.rs/tuiw/badge.svg)](https://docs.rs/tuiw)
[![License](https://img.shields.io/crates/l/tuiw.svg)](https://github.com/conao3/rust-tuiw/blob/master/LICENSE)

![rust-tuiw](./.github/img/thumbnail.png)

TUI applications wrapper with tmux for headless operation.

## Overview

rust-tuiw is a simple CLI wrapper around tmux that enables headless interaction with TUI (Terminal User Interface) applications. It provides an easy-to-use interface for programmatic control and automation of interactive terminal applications.

## Getting Started

### Prerequisites

- Rust toolchain (1.91.1 or later)
- tmux
- NixOS users: Use `nix develop` for development environment

### Installation

Install via crates.io.

```bash
cargo install tuiw
```

or install via nixpkgs.

```bash
nix shell github:conao3/rust-tuiw
```

or install from source.

```bash
git clone https://github.com/conao3/rust-tuiw.git
cd rust-tuiw
cargo build --release
./target/release/tuiw --help
```

### Basic Usage

#### 1. Create a Session

Create a new TUI session by specifying the command to run:

```bash
tuiw create "bash"
```

This will output a session ID (8 character hex format):
```
2a638ef5
```

You can also specify a working directory:

```bash
tuiw create "vim" --cwd /path/to/project
```

#### 2. List Sessions

View all active tuiw sessions:

```bash
tuiw list
```

Output (tab-separated: ID, command, working directory):
```
2a638ef5	bash	/home/user
f3b91a2c	vim	/path/to/project
```

#### 3. Send Keys

Send keyboard input to a session. By default, Enter is sent after the keys:

```bash
tuiw send 2a638ef5 "echo hello"
```

To send keys without Enter, use the `-n` flag:

```bash
tuiw send -n 2a638ef5 "i"
tuiw send -n 2a638ef5 "Escape"
```

#### 4. View Screen

View the current screen content with colors (default):

```bash
tuiw view 2a638ef5
```

To view without colors, use the `--no-color` flag:

```bash
tuiw view --no-color 2a638ef5
```

#### 5. Close Session

Terminate a session:

```bash
tuiw close 2a638ef5
```

### Example: Automating vim

```bash
# Create a vim session
SESSION_ID=$(tuiw create "vim")

# Open a file
tuiw send $SESSION_ID ":e test.txt"

# Enter insert mode and type
tuiw send -n $SESSION_ID "i"
tuiw send $SESSION_ID "Hello, World!"

# Save and quit
tuiw send -n $SESSION_ID "Escape"
tuiw send $SESSION_ID ":wq"

# Close session
tuiw close $SESSION_ID
```

### Troubleshooting

**Session not found:**
- Verify the session ID is correct using `tuiw list`
- Check if the tmux session still exists with `tmux list-sessions`

**Keys not being sent:**
- By default, Enter is sent after each command. Use `-n` to suppress it
- Special keys like Escape should be sent with `-n` flag

**tmux not found:**
- Ensure tmux is installed and in PATH
- On NixOS, use `nix develop` or run with `steam-run tuiw`

## Architecture

tuiw is a simple CLI wrapper around tmux. Each tuiw session corresponds to a tmux session with the name pattern `tuiw-{session_id}`.

### How it works

```
┌─────────────────────────────────────┐
│         tuiw CLI                    │
│  (Command line interface)           │
└──────────────┬──────────────────────┘
               │
               │ Direct tmux commands
               │
┌──────────────▼──────────────────────┐
│        tmux Sessions                │
│   ┌──────────────────────┐          │
│   │  tuiw-2a638ef5       │          │
│   │  ┌────────────────┐  │          │
│   │  │   TUI App      │  │          │
│   │  │   (e.g., vim)  │  │          │
│   │  └────────────────┘  │          │
│   └──────────────────────┘          │
│   ┌──────────────────────┐          │
│   │  tuiw-f3b91a2c       │          │
│   │  ┌────────────────┐  │          │
│   │  │   TUI App      │  │          │
│   │  │   (e.g., bash) │  │          │
│   │  └────────────────┘  │          │
│   └──────────────────────┘          │
└─────────────────────────────────────┘
```

### Session Management

- Session IDs are 8-character hexadecimal strings (derived from UUID)
- tmux session names follow the pattern `tuiw-{session_id}`
- Session metadata (command, cwd) is stored in `~/.config/tuiw/session.json`
- Stale sessions (where tmux session no longer exists) are automatically cleaned up when running `create` or `list` commands
- Sessions persist until explicitly closed or tmux server is stopped
- Multiple TUI applications can run simultaneously in separate sessions

## Use Cases

### Primary: Claude Code Automation

Claude Code currently lacks a headless interaction mode. By wrapping Claude Code with rust-tuiw, you can:
- Programmatically send commands
- Monitor output changes
- Automate coding workflows

### General TUI Automation

Since rust-tuiw works with any TUI application, it enables automation for:
- All coding agents (not just Claude Code)
- Interactive CLI tools
- Terminal-based development environments
- Any ncurses/TUI application

## Technology Stack

- **Language**: Rust (edition 2024)
- **Async Runtime**: tokio (minimal features)
- **CLI Parsing**: clap
- **Process Management**: tmux

## Development

See `Makefile` for available commands:
- `make build` - Build the project
- `make run` - Run the application
- `make test` - Run tests
- `make check` - Run cargo check
- `make clippy` - Run clippy lints
- `make fmt` - Format code

## License

Apache-2.0
