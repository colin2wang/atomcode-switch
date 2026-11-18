# 项目指令（atomcode-switch）

Rust 2024 单 crate，eframe/egui 0.27 桌面 GUI，用于切换管理多个 AtomCode 账户（写 `auth.toml`）。工作目录为 Windows + Git Bash。

## 构建与验证

- 检查：`cargo check`；单元测试：`cargo test`（含解析、时间、配置兼容、i18n key 对齐）；CI 另跑 `cargo fmt --all -- --check` 与 `cargo clippy --all-targets -- -D warnings`
- 打包：`build.ps1` → `cargo build --release` 后拷贝到 `dist/atomcode-switch.exe`
- release profile：`opt-level=z`、`lto`、`panic="abort"`、`strip`——panic 不可恢复，注意不要在 release 下依赖 `panic!` 之外的错误路径；已刻意消除 ICO 解码 `unwrap`、`assert!`、字节切片与 `and_local_timezone(Local).unwrap()` 等可触发 panic 的路径
- `build.rs` 用 `embed-resource` 嵌入 `app.rc` 窗口图标（仅 Windows，带 `#[cfg(windows)]` 保护），改图标需同步 `atomcode.ico`

## 关键约定

- **i18n 强制**：所有用户可见字符串必须走 `self.i18n.t0/t1/t2/t3/t4(key, …)`（`{0}` 占位符）；新增 key 必须**同时**写入 `src/i18n/lang/zh_cn.rs`、`en_us.rs`，并加入 `src/i18n/lang/mod.rs::ALL_KEYS`（`all_keys_present_in_both_languages` 测试会校验三处一致）
- **配置兼容**：`AppConfig`/`ManagedAccount`/`AutoSwitchRules` 等所有可持久化结构体均带结构体级 `#[serde(default)]` 且实现 `Default`，旧配置文件必须能继续加载；解析失败先备份 `*.bak` 再回退默认并向状态栏报错
- **持久化**：配置与 `auth.toml` 保存统一走 `src/io/mod.rs::atomic_write_text`（`.tmp` → 备份 `.bak` → `fs::rename`），`save_config` 返回 `Result`，调用方用 `AtomcodeSwitchApp::persist_config()` 处理错误
- **时间格式**：`reset_time` 为 `YYYY-MM-DD HH:MM`（本地时区），需兼容旧 `HH:MM` 格式；唯一解析入口是 `src/ops/time_utils.rs::parse_reset_datetime`，过期判断（`is_account_expired`）、卡片倒计时（`format_reset_countdown`）与排序全部复用它
- **`/login` 输出解析**：纯函数在 `src/ops/login_parser.rs`，先收集 `ParsedFields` 再事务性应用（零字段不落盘）；需同时兼容中英文输出（`剩余`/`remaining`、`套餐`/`Plan`、`用量约`/`Usage:`）
- **信息新鲜度**：`ManagedAccount::info_status` 默认 `Unknown`；新导入账号不参与自动切换，只有 `/login` 解析成功后才置为 `Fresh`
- **自动切换**：候选账号必须低于阈值且与当前账号有至少 5% 差距（滞回），程序性切换不弹更新窗口
- **更新窗口消息/关闭语义**：解析结果（成功或失败）统一通过 `App::update_message`（`UpdateMessage { kind: UpdateMessageKind::{Info, Success, Error}, text }`，用 `set_update_message`/`clear_update_message`）显示在「更新账号信息」窗口内的消息区，Error 为红色并附重试提示、Success 为绿色、Info（如「正在获取…」）为蓝色无标题；**只有解析成功才延迟关闭窗口**，失败时窗口与文本框原始内容必须保留。关窗统一调用 `close_update_window()` / `cancel_auto_fetch()`，使 `fetch_generation` 自增作废后台结果；获取通道传 `(generation, Result<String, String>)`，错误走消息区而非解析流程
- **导出**：账号导出默认文件名为 `atomcode-accounts.yaml`，不与本地 `config.yaml` 混淆

## 发布流程

1. `Cargo.toml` 升版本号
2. 写 `CHANGE_HISTORY.md` 条目 + `releases/vX.Y.Z.md`
3. git tag 形如 `Release v0.1.8`

## 维护规则

当项目结构、构建/测试命令、架构边界、开发约定或本文件记录的其他事实发生变化时，必须在同一次改动中同步更新本文件。
