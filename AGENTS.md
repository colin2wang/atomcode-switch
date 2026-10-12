# 项目指令（atomcode-switch）

Rust 2024 单 crate，eframe/egui 0.27 桌面 GUI，用于切换管理多个 AtomCode 账户（写 `auth.toml`）。工作目录为 Windows + Git Bash。

## 构建与验证

- 检查：`cargo check`（仓库**没有任何测试**，不要假设有测试可跑；以 `cargo check`/`cargo build` 通过为准）
- 打包：`build.ps1` → `cargo build --release` 后拷贝到 `dist/atomcode-switch.exe`
- release profile：`opt-level=z`、`lto`、`panic="abort"`、`strip`——panic 不可恢复，注意不要在 release 下依赖 `panic!` 之外的错误路径
- `build.rs` 用 `embed-resource` 嵌入 `app.rc` 窗口图标（仅 Windows），改图标需同步 `atomcode.ico`

## 关键约定

- **i18n 强制**：所有用户可见字符串必须走 `self.i18n.t0/t1/t2/t3/t4(key, …)`（`{0}` 占位符）；新增 key 必须**同时**写入 `src/i18n/lang/zh_cn.rs` 和 `en_us.rs` 两处
- **配置兼容**：`AppConfig`（持久化于 `atomcode-accounts.yaml`）新增字段必须加 `#[serde(default)]`，旧配置文件必须能继续加载
- **时间格式**：`reset_time` 为 `YYYY-MM-DD HH:MM`（本地时区），需兼容旧 `HH:MM` 格式；到期判断集中在 `is_account_expired()`，卡片展示（`format_reset_countdown`）与其保持一致
- `/login` 输出解析在 `src/ops/account_ops.rs`，需同时兼容中英文输出（`剩余`/`remaining`、`套餐`/`Plan` 等）

## 发布流程

1. `Cargo.toml` 升版本号
2. 写 `CHANGE_HISTORY.md` 条目 + `releases/vX.Y.Z.md`
3. git tag 形如 `Release v0.1.7`

## 维护规则

当项目结构、构建/测试命令、架构边界、开发约定或本文件记录的其他事实发生变化时，必须在同一次改动中同步更新本文件。
