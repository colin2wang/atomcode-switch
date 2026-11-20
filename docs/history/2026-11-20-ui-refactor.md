# 2026-11-20 UI 重构方案（核对与优化版）

> **实施状态：已完成。** 下文 P1 #1–#5、P2 #6–#9、第 2 节四级分色、P3 文档发布均已落地；
> 变更记录在 `../../CHANGE_HISTORY.md` 与 `releases/v0.1.9.md`。按项目约定，编译/测试由维护者自行执行。
>
> 本文基于 `2026-11-18-improvement.md` 的历史分析，与 v0.1.9 实际源码逐条核对后修订。
> 历史问题中的原子写、时间解析统一、事务性解析、generation 防竞态、`info_status` 等均已落地；下文只列仍未闭合或本次新增的改动。所有行号对应核对时的 v0.1.9 源码。

---

## 0. 核对勘误（原稿需修正的 5 处）

| # | 原稿表述 | 核对结果 | 修正 |
|---|---------|---------|------|
| A | 建议 #6：`3600` 硬编码「两处」 | `ui.rs` 实际 **3 处**：`is_expiring_soon` 第 1258 行、`format_reset_countdown` 第 1287 与 1292 行 | 常量化时 3 处全部替换 |
| B | 建议 #2：目标文件损坏 → 确认后「静默清空当前账号」 | 仅当文件是**合法 YAML 但内容为空/缺字段**（如 `{}`）时，`load_accounts_from` 返回 `Some(AppConfig::default())`，保存阶段（第 1142 行）才清空；**完全损坏的 YAML 返回 `None`**，当前账号实际保留，但全程没有任何错误提示 | 两种失败都要覆盖：改 `Result` + 明确报错并中止导入，不清空账号 |
| C | 建议 #7：补充 `剩余`/`Usage`/`到期时间` 早退关键词 | 当前早退在「套餐」首字段即 `break`，且 `flush = if exited { 500ms } else { 0 }`（`fetch_info.rs:122`）——早退时进程未退出，flush 为 0，可能只拿到套餐行。**补充关键词会加剧截断** | 早退条件收敛为「末字段（重置时间）出现」或「字段齐备」；任何 break 都保留 flush 窗口；新关键词仅用于齐备判定 |
| D | 代码块：`const EXPIRING_SOON_SECS: i64 = 3600;` 与 `diff: u64` 比较 | `diff < EXPIRING_SOON_SECS` 类型不匹配，**无法编译** | 改为 `u64`，或比较 `diff_secs`（i64） |
| E | 建议 #5：「缺标记就静默套用」 | 现象成立，但中文输出可能没有 `logged in as `，一刀切报错会误伤 | 「有标记但匹配不到」→ Err；「无标记」→ 维持写激活账号，并补中文用户名标记识别以减少漏判 |

---

## 1. 修改建议（修订后）

### P1：正确性 / 一致性

| # | 位置 | 问题 | 落地要点 |
|---|------|------|---------|
| 1 | `ui.rs::handle_export` | 导出用 `std::fs::write` 直接覆盖（非原子），且 `let _ = ...` 吞掉写失败，失败时状态栏仍显示「导出成功」 | 改用 `crate::io::atomic_write_text`（已是 `pub(crate)`，可直接调用）；`serde_yaml::to_string` 的 `Err` 也要一并处理（当前同样被 `if let Ok` 静默吞掉）；新增 `status_export_failed` key（三处同步：zh_cn / en_us / ALL_KEYS），失败走 `status_message` 报错，保留 `status_exported` 用于成功 |
| 2 | `ui.rs::load_accounts_from` | `.ok()` 静默吞掉 YAML 解析错误（见勘误 B） | 签名改 `Result<AppConfig, String>`；**3 个调用点**都要改：设置浏览（约 1022 行，冲突框）、设置确定（约 1142 行，真正导入）、导出冲突检查（约 1217 行）；解析失败弹错误并中止导入，不清空账号 |
| 3 | `zh_cn.rs` / `en_us.rs` | `status_fetched`（"点击「解析并更新」完成更新"）是死键——全仓仅 3 处定义（`zh_cn.rs:110`、`en_us.rs:116`、`ALL_KEYS:128`），无任何调用；v0.1.7 起获取后自动解析 | 从两个语言文件与 `ALL_KEYS` **三处同时删除** |
| 4 | `ui.rs::show_manual_update_viewport` | 子视口 `ViewportBuilder::default()` 未调用 `.with_icon()`，任务栏/标题栏图标与主窗口不一致 | `main.rs:25` 已构造 `Arc<IconData>`；把 `AtomcodeSwitchApp::new(cc)` 改为 `new(cc, icon.clone())`（`App` 增 `icon: Option<Arc<egui::IconData>>` 字段），子视口 `.with_icon(self.icon.clone())`；主视口逻辑不变 |
| 5 | `account_ops.rs::parse_login_output_and_update` | 用户名匹配只认 `logged in as `；有标记但匹配不到任何账号、或完全无标记时，套餐/用量静默套到当前激活账号（潜在串号） | 先用 `parse_login_fields` 纯解析；**有标记但匹配不到** → 返回 Err（新 key `status_login_user_unmatched`，`{0}` 带出识别到的用户名），不写盘；匹配到非激活账号仍先写 `auth.toml` 再切 active（现有逻辑保留）；补中文标记 `已登录为` / `登录为` / `当前用户`；无标记时维持写激活账号，并在文档注明该假设 |

### P2：健壮性 / 可维护性

| # | 位置 | 问题 | 落地要点 |
|---|------|------|---------|
| 6 | `ui.rs` | `3600` 硬编码实际 3 处（见勘误 A），口径必须一致 | `const EXPIRING_SOON_SECS: u64 = 3600;`，`is_expiring_soon` 侧比较 `i64` 时用 `EXPIRING_SOON_SECS as i64` 或直接用 `diff_secs` |
| 7 | `fetch_info.rs` | 早退关键词与 flush 缺陷（见勘误 C） | 早退触发收敛为「重置时间标记 `resets`/`重置于` 出现」或「全部关键字段齐备」；`flush` 改为固定 `FLUSH_WINDOW`（kill/wait 后管道已关闭，读线程会立即结束），不再因早退丢输出；`剩余`/`Usage`/`到期时间` 仅作为齐备判定信号 |
| 8 | `zh_cn.rs` `sort_none` | 文案"不排序"，实现实际按名称排序（`ui.rs:1319`） | 改文案为「按名称 / By name」，与 `models.rs::SortMode::None` 的注释一致；**不建议**改为保持导入顺序（会改变已发布语义） |
| 9 | `theme.rs` | 状态方块用 Material 橙红（`E65100/F44336`），主色用 GitHub 绿蓝，同屏两套色板 | 与本次四级分色一并统一到 GitHub palette：danger `#cf222e`、severe `#9a6700` |

### P3：文档 / 发布

10. **版本漂移仍未闭合**：`../../Cargo.toml` 已是 `0.1.9`，但 `CHANGE_HISTORY.md` 与 `releases/` 最高只到 v0.1.8。按 AGENTS.md 发布流程，需补 v0.1.9 条目 + `releases/v0.1.9.md`（含本次方块分色 + 建议 #1–#9 的修复），git tag `Release v0.1.9`。

---

## 2. 账号方块按 code plan 时间窗口有效性分色

当前状态只有三级：`Normal` 是纯绿文字（无底色）、`Expired`/`Expiring` 共用一套橙色、`Invalid` 用红色。细化为**四级独立配色**。

### 状态映射

| 状态 | 判定条件 | 徽标底色 | 徽标字色/描边 | 语义 |
|------|---------|---------|--------------|------|
| **有效** `Normal` | 窗口有效且距重置 > 1h | 浅绿 `#DAFBE1` | 深绿 `#1A7F37` | 时间窗口充足 |
| **即将过期** `Expiring` | 1 小时内重置 | 浅黄 `#FFF8C5` | 琥珀 `#9A6700` | 即将刷新，可留意 |
| **已过期** `Expired` | 重置时间已过 | 浅橙 `#FFF3E0` | 橙红 `#BC4C00`（按 #9 统一 GitHub palette） | 需执行 /login 更新 |
| **已失效** `Invalid` | `is_valid == false` | 浅红 `#FFEBEE` | 危险红 `#CF222E`（按 #9 统一 GitHub palette） | 功能不可用 |

### 1. `theme.rs` — 新增两对常量（其余保留）

```rust
// 状态方块配色：按 code plan 时间窗口有效性四级分色。
// 有效=绿（窗口充足）→ 即将过期=琥珀 → 已过期=橙（需更新）→ 已失效=红。
pub const STATUS_OK_BG: egui::Color32 = egui::Color32::from_rgb(0xDA, 0xFB, 0xE1);
pub const STATUS_OK_FG: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x7F, 0x37);
pub const STATUS_EXPIRING_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xF8, 0xC5);
pub const STATUS_EXPIRING_FG: egui::Color32 = egui::Color32::from_rgb(0x9A, 0x67, 0x00);
```

> 顺带清理（已实施）：删除了不再被引用的 `CARD_VALID`；`STATUS_WARN_*` 更名为 `STATUS_EXPIRED_*` 以准确表达"已过期"语义，并统一到 GitHub palette。

### 2. `ui.rs` — `AccountVisualState` 增加徽标配色与文案映射

```rust
impl AccountVisualState {
    /// 状态方块（徽标）配色：返回 (前景/描边色, 底色)。
    /// 按 code plan 时间窗口有效性分级，见 theme.rs 的 STATUS_* 常量。
    fn badge_colors(self) -> (egui::Color32, egui::Color32) {
        match self {
            Self::Normal   => (STATUS_OK_FG,       STATUS_OK_BG),
            Self::Expiring => (STATUS_EXPIRING_FG, STATUS_EXPIRING_BG),
            Self::Expired  => (STATUS_EXPIRED_FG,  STATUS_EXPIRED_BG),
            Self::Invalid  => (STATUS_INVALID_FG,  STATUS_INVALID_BG),
        }
    }
    /// 各状态对应的 i18n key
    fn badge_key(self) -> &'static str {
        match self {
            Self::Normal   => "card_valid",
            Self::Expiring => "card_expiring",
            Self::Expired  => "card_expired",
            Self::Invalid  => "card_invalid",
        }
    }
}
```

> 注意元组顺序：`badge_colors` 是 `(fg, bg)`，而现有 `card_colors` 是 `(bg, stroke)`。建议统一顺序或在方法名/注释中明确，避免调用点误用。

### 3. `ui.rs::render_account_card` — 徽标渲染统一为一个分支

替换原第 732–761 行的四分支 `match visual_state`（Normal 走纯文字、其余三个手写 `status_badge`）：

```rust
let (badge_fg, badge_bg) = visual_state.badge_colors();
status_badge(ui, self.i18n.t0(visual_state.badge_key()), badge_fg, badge_bg);
```

`status_badge`（`ui.rs:54`）本身无需改动——已用 `fg` 同时画描边和文字、用 `bg` 画底色，天然满足"底色 + 内部字体配色"联动。

### 4. 同源配色对齐（推荐一并做，避免同屏两种"黄"）

```rust
impl AccountVisualState {
    fn card_colors(self, is_active: bool) -> (egui::Color32, egui::Color32) {
        match self {
            Self::Invalid  => (CARD_EXPIRED_BG, STATUS_INVALID_FG),
            Self::Expired  => (CARD_EXPIRED_BG, STATUS_EXPIRED_FG),
            Self::Expiring => (CARD_EXPIRED_BG, STATUS_EXPIRING_FG), // 边框改琥珀
            Self::Normal   => (egui::Color32::WHITE,
                if is_active { GITHUB_RED } else { GITHUB_BORDER }),
        }
    }
    fn progress_color(self, usage_percent: f32) -> egui::Color32 {
        match self {
            Self::Invalid              => STATUS_INVALID_FG,
            Self::Expired              => STATUS_EXPIRED_FG,
            Self::Expiring             => STATUS_EXPIRING_FG, // 进度条改琥珀
            Self::Normal               => theme::progress_bar_color(usage_percent),
        }
    }
}
```

`format_reset_countdown` 底部倒计时文字同步（注意 `EXPIRING_SOON_SECS` 为 `u64`，与 `i64` 的 `diff_secs` 比较需显式转换）：

```rust
const EXPIRING_SOON_SECS: u64 = 3600;

if diff_secs < 0 {
    return (self.i18n.t1("card_reset_expired", reset_time.trim()), STATUS_EXPIRED_FG);
}
let color = if diff_secs < EXPIRING_SOON_SECS as i64 {
    STATUS_EXPIRING_FG
} else {
    GITHUB_TEXT_SECONDARY
};
```

### 5. 可选微调

- **去掉徽标内的 `●` 前缀**：有底色徽标后圆点冗余，可将 `card_valid` 等四个 key 的值改为纯文字（不影响 ALL_KEYS 测试，它只校验 key 存在性）。
- **旧格式说明**：legacy `HH:MM` 经 `parse_reset_datetime` 顺延后永远指向未来，因此**永远不会进入 `Expired`**（只可能 `Expiring`），这是 v0.1.5 设计使然，符合预期，无需处理。
- **单元测试**：`AccountVisualState` 是 `ui.rs` 私有枚举，同文件加 `#[cfg(test)]` 即可覆盖（同时校验底色与前景色互不相同）：

```rust
#[cfg(test)]
mod badge_tests {
    use super::*;
    #[test]
    fn badge_colors_distinct_per_state() {
        let states = [
            AccountVisualState::Normal,
            AccountVisualState::Expiring,
            AccountVisualState::Expired,
            AccountVisualState::Invalid,
        ];
        for i in 0..states.len() {
            for j in i + 1..states.len() {
                assert_ne!(states[i].badge_colors().1, states[j].badge_colors().1,
                    "状态 {} 与 {} 底色相同", states[i].badge_key(), states[j].badge_key());
                assert_ne!(states[i].badge_colors().0, states[j].badge_colors().0,
                    "状态 {} 与 {} 字色相同", states[i].badge_key(), states[j].badge_key());
            }
        }
    }
}
```

### 6. 发布同步

本次改动按 AGENTS.md 流程需要：`../../CHANGE_HISTORY.md` 增加 v0.1.9 条目（含徽标分色 + 建议 #1–#9 的修复）、新建 `releases/v0.1.9.md`、git tag `Release v0.1.9`。

---

## 3. 建议落地顺序与验证步骤

**顺序**：P1 #1/#2/#3（数据安全与 i18n 死键）→ P1 #5（串号防护）→ P2 #6/#7（阈值与捕获完整性）→ 第 2 节四级分色（含 P2 #8/#9）→ P1 #4（子视口图标）→ P3 文档发布。

**验证步骤（由维护者自行执行，本文不代为运行）**：

1. `cargo check`
2. `cargo test`（应覆盖 i18n key 对齐、新增的 `badge_colors_distinct_per_state`、既有解析/时间/配置兼容测试）
3. `cargo fmt --all -- --check`
4. `cargo clippy --all-targets -- -D warnings`
5. 手工验收：
   - 中英切换下四种徽标（正常/即将过期/已过期/已失效）文案与配色正确；
   - 导出到已存在文件并制造写失败（如只读目录），状态栏显示 `status_export_failed` 而非成功；
   - 切换数据目录分别指向「合法但空 YAML」与「完全损坏 YAML」，均应报错且当前账号不被清空；
   - 子视口标题栏/任务栏图标与主窗口一致；
   - `atomcode login` 输出包含纯英文 `Usage:` 时能完整解析用量与重置时间（验证早退不再截断）。
