# tuiw

[![Crates.io](https://img.shields.io/crates/v/tuiw.svg)](https://crates.io/crates/tuiw)
[![Documentation](https://docs.rs/tuiw/badge.svg)](https://docs.rs/tuiw)
[![License](https://img.shields.io/crates/l/tuiw.svg)](https://github.com/conao3/rust-tuiw/blob/master/LICENSE)

![rust-tuiw](./.github/img/thumbnail.png)

TUI applications wrapper with tmux for headless operation.

## Overview

rust-tuiw enables headless interaction with TUI (Terminal User Interface) applications by wrapping them with tmux. This allows for programmatic control and automation of interactive terminal applications.

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

#### 0. Start the Daemon

First, start the daemon in the background:

```bash
tuiw daemon > /tmp/tuiw-daemon.log 2>&1 &
```

Or run it in the foreground for debugging:

```bash
tuiw daemon
```

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

View all active sessions:

```bash
tuiw list
```

Output (tab-separated):
```
2a638ef5	bash	/home/user
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

View the current screen content:

```bash
tuiw view 2a638ef5
```

#### 5. Check Status

Check if a session is running:

```bash
tuiw status 2a638ef5
```

#### 6. Close Session

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

### SSE for Real-time Monitoring

Subscribe to screen changes via Server-Sent Events:

```bash
curl -N http://127.0.0.1:50051/sse/2a638ef5
```

This streams output whenever the screen content changes.

### GraphQL API

The daemon exposes a GraphQL API at `http://127.0.0.1:50051/graphql`:

```bash
curl -X POST http://127.0.0.1:50051/graphql \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { listSessions { id command cwd } }"
  }'
```

### Troubleshooting

**Error: daemon is not running:**
- Start the daemon with `tuiw daemon > /tmp/tuiw-daemon.log 2>&1 &`
- Check if the daemon is running with `ps aux | grep "tuiw daemon"`

**Daemon not starting:**
- Check if port 50051 is available
- Ensure tmux is installed and in PATH

**Session not found:**
- The daemon is stateless and sessions are lost on restart
- Verify the session ID is correct using `tuiw list`

**Keys not being sent:**
- Ensure the session is still running with `tuiw status`
- By default, Enter is sent after each command. Use `-n` to suppress it
- Special keys like Escape should be sent with `-n` flag

## Architecture

### Daemon/Client Model

The application operates in two modes:
- **Daemon**: GraphQL server that manages tmux sessions and TUI applications
- **Client**: CLI interface that communicates with the daemon via GraphQL

You must start the daemon manually with `tuiw daemon` before using client commands.

### Components

```
┌─────────────────────────────────────────────────────────┐
│                        Client                           │
│                     (GraphQL Client)                    │
└──────────────────────┬──────────────────────────────────┘
                       │ GraphQL over HTTP
                       │ SSE for subscriptions
┌──────────────────────▼──────────────────────────────────┐
│                       Daemon                            │
│                   (GraphQL Server)                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │          Session Management (in-memory)         │   │
│  │  - Session ID generation                        │   │
│  │  - CWD tracking                                 │   │
│  │  - Multiple session support                     │   │
│  └─────────────────────┬───────────────────────────┘   │
│                        │                                │
│  ┌─────────────────────▼───────────────────────────┐   │
│  │           Tmux Wrapper                          │   │
│  │  - Session creation                             │   │
│  │  - Key input injection                          │   │
│  │  - Output capture (capture-pane)                │   │
│  │  - Change detection for SSE                     │   │
│  └─────────────────────┬───────────────────────────┘   │
└────────────────────────┼───────────────────────────────┘
                         │
                ┌────────▼────────┐
                │  Tmux Sessions  │
                │   ┌──────────┐  │
                │   │   TUI    │  │
                │   │   App    │  │
                │   └──────────┘  │
                └─────────────────┘
```

### GraphQL API

#### Mutations
- `createSession(command: String!, cwd: String!): SessionID`
- `sendKeys(sessionId: SessionID!, keys: String!): Boolean`
- `closeSession(sessionId: SessionID!): Boolean`

#### Queries
- `sessions: [Session!]!`
- `sessionCapture(sessionId: SessionID!): String!`
- `sessionStatus(sessionId: SessionID!): SessionStatus!`

#### Subscriptions
- `screenChanges(sessionId: SessionID!): String!` (via SSE)

### Session Management

Each session maintains:
- Unique session ID (8 character hex string derived from UUID)
- Working directory (cwd) from client invocation location
- Tmux session reference
- Output buffer and change detection state

Multiple TUI applications can run simultaneously, each in its own session.

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
- **Async Runtime**: tokio
- **GraphQL**: async-graphql
- **HTTP Server**: axum
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
