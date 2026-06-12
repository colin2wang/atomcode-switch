# AtomCode Switch

A desktop GUI tool for managing multiple AtomCode accounts and switching between them with ease.

## Links

[Change History](CHANGE_HISTORY.md) | 

## Features

- **Multi-Account Management** – Import, view, and manage multiple AtomCode accounts in one place.
- **One-Click Switching** – Switch between accounts instantly. The tool writes the selected account's `auth.toml` file to the AtomCode directory.
- **Usage Monitoring** – See each account's usage percentage, plan name, remaining days, and reset time at a glance.
- **Auto & Manual Info Update** – Click "更新信息" on the active account card to open the update dialog. Either paste `/login` output manually, or click "自动获取" to run `atomcode login` in a hidden window and auto-fill the output — both use the same "解析并更新" button to extract plan, usage, remaining days, and reset time.
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

- Rust toolchain (edition 2024)
- A Chinese font installed on your system (the app auto-discovers fonts on Windows, macOS, and Linux)

### Build from Source

```bash
git clone https://github.com/colin2wang/atomcode-switch.git
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
2. Click **Sync Login Info** in the toolbar.
3. The account appears in the account list with its user info and plan details.

### Updating Account Info

Two methods, both accessed by clicking **更新信息** on the active account card:

**Auto-fetch** (recommended):
1. In the update dialog, click **自动获取**.
2. The tool runs `atomcode login` in a hidden window, captures the output (plan, usage, expiry), and fills it into the text editor.
3. Click **解析并更新** to parse and save.

**Manual paste**:
1. In your terminal, run the AtomCode command `/login` and copy all of its output.
2. In the update dialog, paste the `/login` output into the text editor and click **解析并更新**.
3. The app automatically extracts your plan name, usage percentage, remaining days, and reset time.

### Switching Accounts

Click the **Switch** button on any managed account card. The tool writes that account's `auth.toml` to the AtomCode directory, making it the active session for AtomCode.

### Deleting an Account

Click **Delete** on an account card. If the deleted account was the active one, the tool automatically switches to the first remaining account.

### Auto-Switch Rules

Enable auto-switch in **Settings**:
- Toggle **Auto-Switch** on.
- Set a **Usage Threshold (%)** (default: 95%).
- When the active account's usage exceeds the threshold, the tool automatically switches to the account with the lowest usage.

Auto-switch is checked every 30 seconds.

### Custom AtomCode Directory

If your AtomCode `auth.toml` is stored in a non-default location (e.g., inside a WSL distribution), specify the directory in **Settings** → **Custom AtomCode Directory**.

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
