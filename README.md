# rust-tuiw

TUI applications wrapper with tmux for headless operation.

## Overview

rust-tuiw enables headless interaction with TUI (Terminal User Interface) applications by wrapping them with tmux. This allows for programmatic control and automation of interactive terminal applications.

## Architecture

### Daemon/Client Model

The application operates in two modes:
- **Daemon**: GraphQL server that manages tmux sessions and TUI applications
- **Client**: CLI interface that communicates with the daemon via GraphQL

On first invocation, if no daemon is running, the process automatically starts as a daemon. Subsequent invocations act as clients.

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
- `listSessions: [Session!]!`
- `getOutput(sessionId: SessionID!): String!`
- `getSessionStatus(sessionId: SessionID!): SessionStatus!`

#### Subscriptions
- `screenChanges(sessionId: SessionID!): String!` (via SSE)

### Session Management

Each session maintains:
- Unique session ID
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
