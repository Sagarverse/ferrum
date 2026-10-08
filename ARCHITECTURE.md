# Metallic Mini Architecture & Development Guide

## Project Structure

```
Metallic/
├── core/                       # Rust Backend Core Engine
│   ├── Cargo.toml
│   └── src/
│       ├── api/                # Axum HTTP server & REST endpoint handlers
│       ├── db/                 # SQLite database & migrations
│       ├── crypto/             # AES-256-GCM encryption & integrity hashing
│       ├── repository/         # Chunking, deduplication & multi-repo storage engine
│       ├── security/           # Anomaly & ransomware detector
│       └── watcher/            # File system event listener & debouncer
└── ui/                         # Frontend Desktop App
    ├── electron/               # Electron main process & IPC bridges
    ├── src/
    │   ├── components/         # Reusable UI components & layouts
    │   ├── pages/              # Dashboard pages (Home, Storage, ProtectedData, etc.)
    │   └── utils/              # API bridge & state helpers
    └── index.html
```

## Building for Production

To build a production bundle for macOS:

```bash
# 1. Compile release binary for Rust
cd core
cargo build --release

# 2. Package Electron Application
cd ../ui
npm run build
```
