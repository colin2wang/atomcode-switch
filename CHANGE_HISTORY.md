# Change History

## v0.1.5 (2026-06-13)

### Fixed
- **Reset Time Cross-Day Calculation** — `reset_time` now stores `YYYY-MM-DD HH:MM` instead of `HH:MM`, fixing incorrect countdown intervals when the reset time crosses midnight. Old `HH:MM` config values are handled gracefully as a fallback.

### Changed
- **`current_time_str()`** — Returns `YYYY-MM-DD HH:MM` format (was `HH:MM`), providing full timestamp for `last_updated`.
- **`default_reset_time()`** — Returns `YYYY-MM-DD HH:MM` format, combining current date with target time.
- **New `combine_reset_datetime()`** — Helper that combines today's date with a parsed `HH:MM` value; automatically advances to tomorrow if the time has already passed today.
- **`format_reset_countdown()`** — Now parses `YYYY-MM-DD HH:MM` and computes the exact `target - now` diff instead of guessing "next day" by adding 86400s. Falls back to legacy `HH:MM` parsing for old configs.

## v0.1.4 (2026-06-12)

### Added
- **Internationalization (i18n) Support** — Users can now switch between Chinese and English via a toolbar button (中/EN). All UI labels, buttons, status messages, and dialog titles are translated instantly.
- **`lang/` Module** — New `src/lang.rs` + `src/lang/zh_cn.rs` + `src/lang/en_us.rs`. Each language is a standalone `.rs` file with a `pub fn get(key) -> &str` match function. Adding a new language requires only creating a new file and registering it in `lang.rs`.
- **`i18n.rs` Module** — `I18n` struct using a function-pointer-based lookup (`fn(&str) -> &str`), with `t0`–`t4` helpers for parameterized strings with `{0}` `{1}` placeholders.
- **Language Persistence** — Language choice is saved to `atomcode-accounts.yaml` (`language` field in `AppConfig`) and restored on next launch.

### Changed
- **All user-facing strings externalized** — Previously hardcoded Chinese/English strings in `ui.rs`, `account_ops.rs`, and `fetch_info.rs` replaced with `self.i18n.t0/t1/t3/t4()` calls.
- **`format_reset_countdown` converted to method** — Now uses `self.i18n` to produce localized countdown text (e.g., `重置: 21:04（还有 23 小时 35 分 59 秒）` vs `Reset: 21:04 (in 23h 35m 59s)`).
- **`AppConfig` now includes `language` field** — Serialized as `language: "zh_cn"` or `"en_us"` with serde default for backward compatibility.
- **"解析并更新" button now disabled when empty** — Prevents accidental click with no input.

### Removed
- **YAML-based translation files** — `languages/i18n.yaml.zh_cn` and `languages/i18n.yaml.en_us` deleted in favor of native `.rs` files.

## v0.1.3 (2026-06-12)

### Added
- **Auto-Fetch from `atomcode login`** — New "自动获取" button in the update dialog runs `atomcode login` in a hidden window, captures the output, and fills it into the text editor. The user then clicks "解析并更新" to parse, the same as manual paste.
- **`fetch_info` Module** — Extracted auto-fetch logic (`start_auto_update`, `poll_auto_fetch`) into a dedicated `fetch_info.rs` module.
- **English /login Output Support** — The `/login` parser now handles both Chinese and English output formats (e.g., `Plan:` / `套餐：`, `d /` / `剩余`, `resets` / `重置于`).
- **`chrono` Dependency** — Added for correct local timezone handling.

### Changed
- **"更新信息" Button Relocated** — Moved from the top active-indicator bar into the active account's card (right next to "✓ 已激活"). Only visible on the active card.
- **Streamlined Update Dialog** — Auto-fetch and manual paste now share the same text box and the same "解析并更新" button, providing a unified workflow.
- **Local Timezone** — `current_time_str()` now uses `chrono::Local::now()` instead of manually computing from UTC epoch, fixing incorrect display times in non-UTC timezones.

### Removed
- **"刷新" Toolbar Button** — Removed

### Added
- **Manual Update via /login Paste** — Click the "更新信息" button next to the active user indicator to open a paste dialog. Paste the output of `/login` to automatically update plan name, usage percentage, remaining days, and reset time.
- **Module Refactoring** — Split the monolithic `app.rs` (1471 lines) into four focused modules:
  - `theme.rs` — Color constants, font loading, GitHub theme setup
  - `account_ops.rs` — Account CRUD, auto-switch, /login parsing
  - `ui.rs` — All UI rendering (toolbar, panels, cards, dialogs)
  - `app.rs` — Struct definition and constructor only (~50 lines)

### Fixed
- **Visible Update Button** — The "更新信息" button now has a visible border and background instead of `frame(false)`, making it clearly clickable

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
