# AtomCode Switch

A desktop GUI tool for managing multiple AtomCode accounts and switching between them with ease.

![Screenshot](screenshot.png)

> **Note**: `screenshot.png` is not included in this repository. Replace with an actual screenshot or remove the line above.

## Features

- **Multi-Account Management** – Import, view, and manage multiple AtomCode accounts in one place.
- **One-Click Switching** – Switch between accounts instantly. The tool writes the selected account's `auth.toml` file to the AtomCode directory.
- **Usage Monitoring** – See each account's usage percentage, plan name, remaining days, and reset time at a glance.
- **Auto-Switch Rules** – Set a usage threshold. When the active account exceeds it, the tool automatically switches to the lowest-usage valid account.
- **Custom AtomCode Directory** – Supports custom `auth.toml` storage locations (useful for WSL or non-default setups).
- **Imported-Account Persistence** – Account data (including `auth.toml` content, usage stats, and plan info) is stored locally in a config file.
- **Slim Binary** – Optimized release build with minimal dependencies and LTO, producing a small executable.

## Configuration Files

| File | Path | Purpose |
|------|------|---------|
| Local Config | `<exe_dir>/config.yaml` | Stores custom data directory path (only created when non-default values are set) |
| Account Data | `<data_dir>/atomcode-accounts.yaml` | Stores managed accounts, active account ID, auto-switch rules, custom auth.toml directory path |
| Auth Data | `~/.atomcode/auth.toml` | AtomCode authentication token file (this is what the tool reads/writes to switch accounts) |

> Default `<data_dir>` is `~/.atomcode-switch`. You can customize it via the settings panel.

## Installation

### Prerequisites

- Rust toolchain (edition 2021)
- A Chinese font installed on your system (the app auto-discovers fonts on Windows, macOS, and Linux)

### Build from Source

```bash
git clone https://github.com/your-username/atomcode-switch.git
cd atomcode-switch
cargo build --release
```

The binary will be at `target/release/atomcode-switch.exe` (Windows) or `target/release/atomcode-switch` (macOS/Linux).

### Run

```bash
cargo run --release
```

## Usage Guide

### Importing an Account

1. Make sure you are logged into AtomCode (so `~/.atomcode/auth.toml` exists).
2. Click **📥 Sync Login Info** in the toolbar.
3. The account appears in the account list with its user info and plan details.

### Switching Accounts

Click the **Switch** button on any managed account card. The tool writes that account's `auth.toml` to the AtomCode directory, making it the active session for AtomCode.

### Deleting an Account

Click **Delete** on an account card. If the deleted account was the active one, the tool automatically switches to the first remaining account.

### Auto-Switch Rules

Enable auto-switch in **⚙ Settings**:
- Toggle **Auto-Switch** on.
- Set a **Usage Threshold (%)** (default: 95%).
- When the active account's usage exceeds the threshold, the tool automatically switches to the account with the lowest usage.

Auto-switch is checked every 30 seconds.

### Custom AtomCode Directory

If your AtomCode `auth.toml` is stored in a non-default location (e.g., inside a WSL distribution), specify the directory in **⚙ Settings** → **Custom AtomCode Directory**.

## Tech Stack

- **GUI**: [egui](https://github.com/emilk/egui) 0.27 / [eframe](https://github.com/emilk/egui/tree/master/eframe) (wgpu backend)
- **Serialization**: serde, serde_yaml, toml
- **Dialogs**: tinyfiledialogs
- **Icon**: ico (embedded via build script)
- **Build**: embed-resource (Windows .exe icon embedding)

## License

This project is provided for reference and personal use. See the `LICENSE` file for details (if applicable).

---

*Disclaimer: This tool is not officially affiliated with AtomCode. Use at your own risk.*
