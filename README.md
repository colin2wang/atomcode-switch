# AtomCode Switch

A desktop GUI tool for managing multiple AtomCode accounts and switching between them with ease.

## Links

[Change History](CHANGE_HISTORY.md) | 

## Features

- **Internationalization** – Built-in Chinese and English support. Toggle between languages instantly via the toolbar button (中/EN). Language preference is persisted across sessions.
- **Multi-Account Management** – Import, view, and manage multiple AtomCode accounts in one place.
- **One-Click Switching** – Switch between accounts instantly. The tool writes the selected account's `auth.toml` file to the AtomCode directory.
- **Usage Monitoring** – See each account's usage percentage, plan name, remaining days, and reset time at a glance.
- **Auto & Manual Info Update** – Click "Update Login Info" on the active account card to open the update dialog. Either paste `/login` output manually, or click "自动获取" to run `atomcode login` in a hidden window and auto-fill the output — both use the same "解析并更新" button to extract plan, usage, remaining days, and reset time.
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

Two methods, both accessed by clicking **Update Login Info** on the active account card:

**Auto-fetch** (recommended):
1. In the update dialog, click **Auto-Fetch**.
2. The tool runs `atomcode login` in a hidden window, captures the output (plan, usage, expiry), and fills it into the text editor.
3. Click **Parse & Update** to parse and save.

**Manual paste**:
1. In your terminal, run the AtomCode command `/login` and copy all of its output.
2. In the update dialog, paste the `/login` output into the text editor and click **Parse & Update**.
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

## Module Structure

| Module | Description |
|--------|-------------|
| [`i18n.rs`](src/i18n.rs) | Internationalization wrapper. Provides `I18n` struct with a function-pointer lookup per language, and `t0`–`t4` helper methods for parameterized string formatting with `{0}` `{1}` placeholders. |
| [`lang.rs`](src/lang.rs) + [`lang/zh_cn.rs`](src/lang/zh_cn.rs) / [`lang/en_us.rs`](src/lang/en_us.rs) | Language definitions. Each language is a standalone `.rs` file exporting `pub fn get(key: &str) -> &str` via a match expression. Adding a new language = one new file + one enum variant. |
| [`main.rs`](src/main.rs) | Entry point. Loads the window icon from `atomcode.ico`, configures viewport size and min constraints, then launches the eframe event loop. Windows subsystem attribute hides the console window in release builds. |
| [`app.rs`](src/app.rs) | Application state (`AtomcodeSwitchApp` struct) and constructor. Holds the `AppConfig`, settings dialog state, status message, delete confirmation state, and manual-update dialog fields (text, auto-fetch flag, channel receiver). |
| [`ui.rs`](src/ui.rs) | All UI rendering. Implements `eframe::App::update()` with top toolbar, bottom status bar, settings window, delete confirmation dialog, and the manual-update info window. Contains account card rendering with usage progress bar, plan info, dynamic reset countdown (`format_reset_countdown`), and the auto-fetch flow. |
| [`models.rs`](src/models.rs) | Data structures: `AuthToml` (auth.toml deserialization), `User` (user profile), `AppConfig` (managed accounts + auto-switch rules), `ManagedAccount` (per-account fields: plan, usage, reset time, remaining days, validity), and `AutoSwitchRules`. |
| [`account_ops.rs`](src/account_ops.rs) | Account CRUD operations + `/login` output parser. Includes: `sync_active_account_status` (read auth.toml to detect active account), `check_auto_switch` (periodic usage-based auto-switch), `switch_to_account` (write auth.toml + update config), `import_current_auth` (import from current auth.toml), `delete_account`, `parse_login_output_and_update` (parse `/login` text to extract plan, usage %, remaining days, and reset time — supports both Chinese and English output formats). |
| [`fetch_info.rs`](src/fetch_info.rs) | Auto-fetch flow control. Spawns a hidden `atomcode login` process (using `CREATE_NO_WINDOW` on Windows), captures its output via background threads, then provides the result to the manual-update text box. |
| [`atomcode_io.rs`](src/atomcode_io.rs) | Low-level auth.toml file I/O. `read_current_auth` / `write_auth` / `clear_auth` — handles custom directory resolution and TOML serialization. |
| [`config_io.rs`](src/config_io.rs) | Local config file management. Manages two files: `config.yaml` (next to the exe, stores custom data directory path) and `atomcode-accounts.yaml` (in the data dir, stores all account data, active ID, and auto-switch rules). Includes automatic migration from the old single-file layout. |
| [`theme.rs`](src/theme.rs) | GitHub-style light theme and Chinese font support. Provides color constants (`GITHUB_BG`, `GITHUB_BLUE`, etc.), color helper functions (`usage_color`, `progress_bar_color`), cross-platform Chinese font auto-discovery (Windows → msyh.ttc, macOS → PingFang.ttc, Linux → multiple distributions), and theme application via `setup_github_theme`. |
| [`build.rs`](build.rs) | Build script that embeds `atomcode.ico` into the Windows `.exe` via `embed-resource`, so the compiled binary has a proper file manager icon. |

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
