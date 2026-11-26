# ⚡ pnuke

<div align="center">

**The sub-millisecond port killer and listener cockpit for developers.**

[![CI](https://github.com/almostordinary/pnuke/actions/workflows/ci.yml/badge.svg)](https://github.com/almostordinary/pnuke/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-red.svg)](https://crates.io/crates/pnuke)

<br />

```
$ pnuke 3000
✔ Nuked 'node' (PID: 14205) on port 3000 in 0.82ms
```

</div>

---

## 💥 The Problem

Every developer has encountered this:
```
Error: listen EADDRINUSE: address already in use :::3000
```

To fix it, you usually have to remember messy, platform-specific bash runes:
```bash
# macOS / Linux
lsof -ti :3000 | xargs kill -9

# Windows
netstat -ano | findstr :3000
taskkill /F /PID 12345
```

Or you install heavy npm packages that pull in 15 dependencies and take 400ms just to boot Node.js.

**`pnuke` solves this once and for all.** Written in pure Rust, it finds the locking process and terminates it in **under 2 milliseconds** with zero runtime dependencies.

---

## ⚡ Features

* **Blazingly Fast:** Executes in sub-milliseconds with a tiny **~1.8MB** static binary.
* **Interactive TUI Cockpit:** Run `pnuke` without arguments to launch a real-time, searchable terminal dashboard powered by [Ratatui](https://ratatui.rs).
* **Multi-Port Support:** Nuke multiple ports simultaneously: `pnuke 3000 8080 5432`.
* **Graceful & Force Modes:** Sends `SIGTERM` first with a 100ms graceful exit window, or instant `SIGKILL` with `-f` / `--force`.
* **Agent & Automation Friendly:** `--json` output flag allows seamless integration into CI pipelines, scripts, and AI agents.
* **Cross-Platform:** Works natively on macOS, Linux, and Windows.

---

## 🕹️ Interactive TUI Cockpit

Run `pnuke` with no arguments to launch the interactive cockpit:

```bash
pnuke
```

```
┌ Active Listening Ports ────────────────────────────────────────────────────────┐
│ PORT     PROTO    PID        PROCESS NAME             ADDRESS                  │
│ ▶ 3000   TCP      14205      node                     127.0.0.1:3000           │
│   5037   TCP      32913      adb                      127.0.0.1:5037           │
│   8000   TCP      42280      php8.5                   127.0.0.1:8000           │
│   5432   TCP      9102       postgres                 0.0.0.0:5432             │
└────────────────────────────────────────────────────────────────────────────────┘
 [↑/↓] Navigate   [Enter/k] Nuke Port   [/] Filter   [r] Refresh   [q] Quit
```

* **Navigate:** Use `↑` / `↓` or `j` / `k`.
* **Search / Filter:** Press `/` and type to filter by process name or port number in real time.
* **Nuke:** Press `Enter` or `x` on any port to terminate it instantly.
* **Refresh:** Press `r` to rescan listening sockets.

---

## 🚀 Installation

### Using Cargo
```bash
cargo install pnuke
```

### From GitHub Releases (Pre-built Binaries)
Download the pre-compiled binary for macOS (Apple Silicon / Intel), Linux (x86_64 / arm64), or Windows from [GitHub Releases](https://github.com/almostordinary/pnuke/releases).

```bash
# macOS / Linux one-liner
curl -fsSL https://raw.githubusercontent.com/almostordinary/pnuke/main/install.sh | sh
```

*(npm / npx and Homebrew packages coming soon!)*

---

## 📖 CLI Usage

```
Usage: pnuke [OPTIONS] [PORTS]...

Arguments:
  [PORTS]...  Port(s) to inspect or nuke (e.g. 3000, 8080, 5432)

Options:
  -f, --force        Force immediate termination (SIGKILL) without grace period
  -c, --check        Check and inspect what is running without terminating
  -l, --list         Non-interactively list all active listening ports
  -i, --interactive  Explicitly launch the interactive terminal UI
      --json         Output results as machine-readable JSON
  -q, --quiet        Suppress all informational output except errors
  -h, --help         Print help
  -V, --version      Print version
```

### Examples

#### 1. Instant Kill
```bash
pnuke 3000
# Output: ✔ Nuked 'node' (PID: 14205) on port 3000 in 0.82ms
```

#### 2. Nuke Multiple Ports at Once
```bash
pnuke 3000 8080 5432
```

#### 3. Inspect / Dry-Run (Check without killing)
```bash
pnuke -c 3000
# Output: ● Port 3000 is occupied by 'node' (PID: 14205, Protocol: TCP)
```

#### 4. Non-Interactive List
```bash
pnuke -l
```

#### 5. Machine-Readable JSON Output
```bash
pnuke 3000 --json
```
```json
[
  {
    "port": 3000,
    "pid": 14205,
    "process_name": "node",
    "success": true,
    "message": "Terminated successfully",
    "duration_ms": 0.82
  }
]
```

---

## 🏎️ Benchmark

| Tool | Language | Startup + Kill Time | Binary Size | Runtime Dependency |
| :--- | :--- | :--- | :--- | :--- |
| **`pnuke`** | **Rust** 🦀 | **~10ms** | **~1.8 MB** | **None (Zero)** |
| `kill-port` | Node.js | ~420ms | ~15 MB (node_modules) | Node.js runtime |
| `lsof + kill` | Shell / Bash | ~45ms | N/A | POSIX shell tools |

---

## 🛡️ License

Dual-licensed under either:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

<div align="center">
  <sub>Engineered with precision by <a href="https://github.com/almostordinary">Almost Ordinary</a> • Curated by <a href="https://github.com/wellparth">@wellparth</a></sub>
</div>
