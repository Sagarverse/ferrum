# ⚒️ Ferrum

> **Iron-Forged, Zero-Trust Continuous Data Protection & Automated Recovery Engine**

![Ferrum](https://img.shields.io/badge/Ferrum-Iron_Forged-E8913A?style=for-the-badge&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Electron](https://img.shields.io/badge/Electron-47848F?style=for-the-badge&logo=electron&logoColor=white)
![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)
![License](https://img.shields.io/badge/License-MIT-E8913A?style=for-the-badge)

**Ferrum** (Latin for *iron*) is an enterprise-grade, lightweight data protection platform built in Rust — providing real-time file system monitoring, AES-256-GCM chunked encryption, deduplication, multi-target backup repository routing (including Google Drive folder integration), anti-ransomware protection, and instantaneous version recovery.

> *Iron rusts. Ferrum doesn't.* 🔥

---

## ✨ Key Features

- **⚒️ Dual-Engine Architecture**: High-performance **Rust Core Daemon** (`Axum` + `Tokio`) paired with a sleek **Electron + React** desktop UI.
- **🛡️ Real-Time File System Watcher**: Continuously monitors protected directories for file changes, creations, and deletions with debounced batch processing.
- **⚡ Dynamic Repository Routing & Customization**:
  - Assign specific backup repositories to individual folders.
  - Full toggle controls to turn repositories ON/OFF.
  - External repository support (e.g. Google Drive synced folders, external drives, NAS).
- **🔒 Zero-Trust Encryption & Deduplication**:
  - Content-defined chunking with SHA-256 manifest verification.
  - AES-256-GCM data encryption at rest.
  - Automatic cross-file chunk deduplication to optimize storage usage.
- **🗑️ Smart Deletion & Sync**: Synchronized data removal — deleting a protected file cleans up corresponding backups while maintaining version history integrity.
- **💻 macOS Menu Bar & Background Execution**: Minimizes directly into the macOS status bar for uninterrupted background file protection.
- **🤖 Built-in AI Assistant**: Interactive assistant for executing backup policies, querying protection status, and managing files via natural language.

---

## 🏗️ Architecture Overview

```
                    ┌──────────────────────────────┐
                    │   Electron + React Desktop   │
                    │   (IPC Bridge / REST API)     │
                    └──────────────┬───────────────┘
                                   │
                    ┌──────────────▼───────────────┐
                    │   Rust Core Engine (:19270)   │
                    │   Axum HTTP + Tokio Runtime   │
                    └──┬──────┬──────┬──────┬──────┘
                       │      │      │      │
                  ┌────▼─┐ ┌─▼────┐ ┌▼────┐ ┌▼─────────┐
                  │Watch │ │Crypt │ │ DB  │ │ Security │
                  │FS    │ │AES   │ │SQLite│ │ Monitor  │
                  └──────┘ └──────┘ └─────┘ └──────────┘
                                │
                  ┌─────────────▼────────────────┐
                  │   Backup Repositories        │
                  │   Local · NAS · Google Drive  │
                  └──────────────────────────────┘
```

---

## 🛠️ Quick Start

### Prerequisites

- **Node.js** (v18+) & `npm`
- **Rust** (Cargo 1.70+)

### 1. Clone the Repository

```bash
git clone https://github.com/Sagarverse/ferrum.git
cd ferrum
```

### 2. Build the Rust Backend Core

```bash
cd core
cargo build --release
```

### 3. Install & Run the Desktop App

```bash
cd ../ui
npm install
npm run app
```

---

## 📖 Usage Guide

| Action | How |
|--------|-----|
| **Protect a folder** | Navigate to **Protected Data** → click **+ Add Folder** |
| **Configure repositories** | Go to **Storage** → manage local, NAS, or Google Drive targets |
| **Background mode** | Close the window — Ferrum runs silently from the macOS menu bar |
| **Restore files** | Access **Versions & Recovery** for point-in-time restores |
| **AI commands** | Use the **Assistant** — try "protect my Documents folder" |

---

## 🧰 Tech Stack

| Layer | Technology |
|-------|-----------|
| Core Engine | Rust, Axum, Tokio, SQLite |
| Encryption | AES-256-GCM, SHA-256 |
| Desktop UI | Electron, React, TypeScript |
| Styling | Custom CSS Design System |
| Build | Cargo, Vite, npm |

---

## 📄 License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for details.

---

<p align="center">
  <strong>Iron rusts. Ferrum doesn't.</strong><br/>
  Made with ⚒️ by <a href="https://github.com/Sagarverse">Sagarverse</a>
</p>
