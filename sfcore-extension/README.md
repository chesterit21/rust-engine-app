# AI Dev Agent - VS Code Extension

Local LLM-powered development assistant dengan React UI dan Rust backend integration.

## Features

- 💬 **Chat Interface**: Modern React-based chat UI
- 📎 **File Context**: Add files ke context untuk AI analysis
- 🔍 **Search Mode**: Toggle antara normal chat dan search mode
- ⚡ **Fast Transport**: UDS (Unix Domain Socket) untuk low-latency, HTTP fallback
- 🔒 **Local First**: Semua processing di local machine

## Installation

```bash
# Install dependencies
npm install

# Build extension
npm run build

# Watch mode (development)
npm run watch
```

## Development

1. Open folder in VS Code
2. Press `F5` untuk launch Extension Development Host
3. Extension akan aktif di window baru

## Architecture

```
sfcore-extension/
├── src/
│   ├── extension/          # Extension Host (TypeScript)
│   │   ├── commands/       # Command handlers
│   │   ├── providers/      # Webview, Context, Completion providers
│   │   ├── services/       # LLM, Context, File, State services
│   │   ├── transport/      # UDS + HTTP transport layer
│   │   └── utils/          # Logger, Config, Helpers
│   ├── webview/            # React UI
│   │   ├── components/     # ChatPanel, FileContext, ModeSelector
│   │   ├── hooks/          # useChat, useFileContext, useVSCode
│   │   └── styles/         # CSS styles
│   └── shared/             # Shared types & protocol
├── media/                  # Icons & static assets
└── dist/                   # Build output
```

## Configuration

Settings bisa diakses via VS Code Settings:

| Setting | Default | Description |
|---------|---------|-------------|
| `aiDevAgent.transport.type` | `auto` | Transport type (auto/uds/http) |
| `aiDevAgent.transport.uds.socketPath` | `/tmp/llm-server.sock` | UDS socket path |
| `aiDevAgent.transport.http.baseUrl` | `http://localhost:8080` | HTTP server URL |

## Commands

| Command | Description |
|---------|-------------|
| `aiDevAgent.openChat` | Open AI Dev Agent chat panel |
| `aiDevAgent.addFileToContext` | Add file to AI context |
| `aiDevAgent.clearContext` | Clear all files from context |

## License

MIT
