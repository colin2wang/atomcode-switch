# Change History

## v0.1.2 (2026-06-12)

### Added
- **Manual Update via /login Paste** — Click the 📝 更新信息 button next to the active user indicator to open a paste dialog. Paste the output of `/login` to automatically update plan name, usage percentage, remaining days, and reset time.
- **Module Refactoring** — Split the monolithic `app.rs` (1471 lines) into four focused modules:
  - `theme.rs` — Color constants, font loading, GitHub theme setup
  - `account_ops.rs` — Account CRUD, auto-switch, /login parsing
  - `ui.rs` — All UI rendering (toolbar, panels, cards, dialogs)
  - `app.rs` — Struct definition and constructor only (~50 lines)

### Fixed
- **Visible Update Button** — The "📝 更新信息" button now has a visible border and background instead of `frame(false)`, making it clearly clickable

## v0.1.1 (2026-06-06)

### Added
- **Window Icon** — `atomcode.ico` is embedded as both the .exe file icon (via `build.rs` + `embed-resource`) and the program window icon (decoded by the `ico` crate and passed to `ViewportBuilder::with_icon()`)
- **Version in Title Bar** — The main window title shows `AtomCode Switch v0.1.1`, with the version number automatically read from `Cargo.toml`
- **Export Conflict Prompt** — When exporting a config to an existing file, a dialog lists the accounts in the existing file versus those being exported, asking whether to overwrite or keep the original (default: keep)
- **CHANGE_HISTORY.md** — Change history document

### Optimizations
- **Dependency Slimming**:
  - Replaced `image` (heavyweight) → `ico` (~30KB), used only for ICO decoding
  - Replaced `rfd` (includes async runtime) → `tinyfiledialogs` (C bindings)
  - `eframe`/`egui` set to `default-features = false`, retaining only `default_fonts` + `wgpu`
- **Release Profile** (`[profile.release]`):
  - `opt-level = "z"` — optimize for size
  - `lto = true` + `codegen-units = 1` — maximum link-time optimization
  - `strip = true` — strip debug symbols
  - `panic = "abort"` — remove unwind tables
- Updated `README.md` Tech Stack and build instructions

## v0.1.0 (2026-06-05)

### Added
- **Multi-Account Management** — Import, view, and manage multiple AtomCode accounts
- **One-Click Switching** — Write the selected account's `auth.toml` to the AtomCode directory for instant switching
- **Usage Monitoring** — Display each account's usage percentage, plan name, remaining days, and reset time
- **Auto-Switch Rules** — Set a usage threshold; when exceeded, automatically switch to the lowest-usage valid account (checked every 30 seconds)
- **Custom AtomCode Directory** — Support non-default `auth.toml` storage locations (e.g., WSL)
- **Account Persistence** — Account data stored locally in `atomcode-accounts.yaml`
- **GitHub Theme UI** — GitHub-inspired color scheme

### Tech Stack
- GUI: eframe 0.27 / egui 0.27
- Serialization: serde + serde_yaml + toml
- File Dialogs: rfd
- Image Decoding: image
