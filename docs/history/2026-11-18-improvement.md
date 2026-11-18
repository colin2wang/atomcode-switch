# atomcode-switch 代码分析与优化建议
## 一、总体评价
代码结构清晰（`app/ops/io/i18n` 分层合理），v0.1.5~v0.1.7 的重构（`lib.rs` 拆分、跨天重置时间、窗口消息区）方向正确。主要问题集中在：**i18n 约定违规残留、配置健壮性、解析逻辑重复、获取流程竞态、版本/文档不同步**。以下按优先级列出。
---
## 二、高优先级（正确性 / 一致性）
### 1. i18n 强制约定被违反（3 处硬编码）
AGENTS.md 明确要求所有用户可见字符串走 `i18n.t*`，但：
- `ui.rs::handle_export` 中对话框标题硬编码 `"导出冲突"` —— 而 `dialog_overwrite_title` 这个 key 在两个语言文件里**已经定义却没用上**：
```rust
// 现状
tinyfiledialogs::message_box_yes_no("导出冲突", &msg, ...)
// 应改为
tinyfiledialogs::message_box_yes_no(&self.i18n.t0("dialog_overwrite_title"), &msg, ...)
```
- `fetch_info.rs` 中 `"（未获取到输出）"`、`"（无法执行 atomcode login: {}）"` 是硬编码中文，会流入文本框并参与解析失败提示。这些字符串在 spawn 线程前无法用 `self.i18n`（闭包移动问题），应在 `start_auto_update` 里先翻译好再 move 进线程。
### 2. 导出/导入文件名不一致
- 导出对话框默认文件名是 `config.yaml`（`handle_export`）；
- 设置里切换数据目录导入时读的是 `<dir>/atomcode-accounts.yaml`；
- 旧数据迁移也是 `config.yaml → atomcode-accounts.yaml`。
用户按默认名导出后，再指向该目录**无法导入**。建议导出默认名改为 `atomcode-accounts.yaml`。
### 3. 配置解析失败会静默丢数据
`config_io::load_config` 用 `unwrap_or_default()`：YAML 任何字段缺失/类型错（如旧文件缺 `auto_switch_rules`，该字段没有 `#[serde(default)]`）→ 整个配置回退为默认空配置，随后 `save_config` **覆盖原文件**，账号永久丢失。建议：
- 给 `auto_switch_rules` 等复合字段补 `#[serde(default)]`（`AutoSwitchRules` 实现 `Default`）；
- 解析失败时先备份原文件再回退：
```rust
match serde_yaml::from_str::<AppConfig>(&content) {
    Ok(c) => c,
    Err(e) => {
        let _ = std::fs::copy(&path, path.with_extension("yaml.bak"));
        eprintln!("config parse failed: {e}");
        AppConfig::default()
    }
}
```
### 4. 旧 `HH:MM` 格式在排序中被判为"最差"
`parse_reset_time_for_sort` 只支持新格式，旧 `HH:MM` 返回 `None` → `stale=true` → 固定排最后；而 `is_account_expired` / `format_reset_countdown` 都兼容旧格式。**三处解析逻辑应统一**（见第四节重构建议）。
### 5. 版本号与 CHANGE_HISTORY 不同步
`../../Cargo.toml` 已是 `0.1.8`，但 `CHANGE_HISTORY.md` 最高只到 `v0.1.7`。按 AGENTS.md 发布流程，升版本必须同一次改动中补条目 + `releases/v0.1.8.md`，否则 tag 时遗漏。
### 6. `status_fetched` 文案与 v0.1.7 行为矛盾
文案仍是"点击「解析并更新」完成更新"，但 v0.1.7 起获取完成后**自动解析**。且 `poll_auto_fetch` 的 `else` 分支（非 pending 路径）当前所有调用点都设置了 `auto_parse_pending = true`，实际是死分支。建议：更新两个语言文件里的 `status_fetched` 文案，或删除该分支。
### 7. 更新窗口成功消息"闪现不可见"
解析成功路径：`set_update_message(Success)` 后立刻 `show_manual_update = false`，绿色成功消息只渲染一帧；且 `set_update_message` 仅对 Error 写状态栏。要么接受现状（状态栏已有"导入成功: xxx"），要么成功后延迟关闭，否则 `Success` 分支形同虚设。
### 8. 自动获取的固定 3s + 1s recv 竞态
`sleep(3s)` 后 kill、`recv_timeout(1s)`：网络慢时输出被截断（解析失败），输出快时白等 3 秒。且"无法执行 atomcode login"的错误文本被当作**输出**填入文本框走解析流程，用户看到的是"未能识别出任何账号信息"而非真实原因。建议：
- 通道改 `Result<String, String>`，spawn 失败/超时直接走 `Err` → `set_update_message(Error)`，不进入解析；
- 等待改为轮询 + 更长上限（如 10s），读到 `套餐`/`Plan`/`resets` 标记或进程退出即提前结束：
```rust
let deadline = Instant::now() + Duration::from_secs(10);
while child.try_wait().map_or(false, |s| s.is_none()) && Instant::now() < deadline {
    thread::sleep(Duration::from_millis(100));
}
let _ = child.kill();
```
---
## 三、健壮性 / 潜在 panic
### 9. `raw[..5]` 字节切片可能 panic
`apply_parsed_fields` 中两处：
```rust
let hm = if raw.len() >= 5 && raw.as_bytes()[2] == b':' { raw[..5].to_string() } ...
```
`byte 2 == ':'` 只保证索引 3 是边界；若字节 3 起是 3 字节 UTF-8 字符（占 3/4/5），`raw[..5]` 不是字符边界 → **panic（release 下 abort）**。防御性写法：
```rust
let hm = match raw.get(..5) {
    Some(s) if s.as_bytes()[2] == b':' => s.to_string(),
    _ => raw,
};
```
### 10. `/login` 解析覆盖缺口
- **用户名匹配只认英文** `"logged in as "`：中文输出若措辞不同，会跳过账号匹配，把**其他账号**的套餐/用量写到当前激活账号上（数据串号）。建议补充中文用户名行匹配，或匹配失败时告警而非静默套用。
- **用量百分比只认 `"用量约"`**：纯英文输出（`Usage: 12%`）解析不出。建议增加 `Usage` 回退分支（与 `Plan:`/`resets` 同等处理）。
- 英文 `d /` 前向取数字可能误匹配套餐名中的 `d /`，可用行级正则/限定上下文缓解。
---
## 四、重构建议
### 11. 统一 reset_time 解析器（消除三份重复逻辑）
`is_account_expired`、`format_reset_countdown`、`parse_reset_time_for_sort` 各自解析一遍。抽一个纯函数：
```rust
pub(crate) fn parse_reset_datetime(
    reset_time: &str,
    now: chrono::DateTime<chrono::Local>,
) -> Option<chrono::DateTime<chrono::Local>> {
    let t = reset_time.trim();
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M") {
        return dt.and_local_timezone(chrono::Local).single();
    }
    // 兼容旧 HH:MM：结合今天日期，已过则顺延到明天
    if t.len() >= 5 && t.as_bytes()[2] == b':' {
        let h = t[..2].parse::<u32>().ok()?;
        let m = t[3..5].parse::<u32>().ok()?;
        let naive = now.date_naive().and_hms_opt(h, m, 0)?;
        let dt = naive.and_local_timezone(chrono::Local).single()?;
        return Some(if dt < now { dt + chrono::Duration::days(1) } else { dt });
    }
    None
}
```
三处调用点收敛后，过期判断、倒计时、排序（含旧格式不再误判 stale）行为自动一致。
### 12. 卡片排序的每帧 `accounts.clone()`
`update()` 每帧 `self.config.accounts.clone()`（含完整 auth token 字符串）。clone 存在的原因是 `render_account_card(&mut self, acc: &...)` 的借用冲突，不是无谓开销。要消除可采用**延迟动作**模式：渲染循环只收集 `enum CardAction { Switch(id), Delete(id), OpenUpdate }`，循环结束后统一用 `&mut self` 执行，渲染期间即可借用 `&self.config.accounts` 的引用排序，无需 clone。
### 13. 颜色 / 按钮样式收敛到 theme.rs
卡片里大量 Bootstrap 色值硬编码：`(13,110,253)`、`(220,255,220)`、`(40,167,69)`、`(220,53,69)`、`(33,37,41)`、`(108,117,125)` —— 与 `GITHUB_BLUE/GREEN/RED/TEXT_*` 常量**色值不同**（如 13,110,253 ≠ 9,105,218），同屏混用两套设计系统。建议全部收敛到 `theme.rs`，并补 `style_button_primary/danger/success` helper（当前 `rounding(4.0)+fill(...)` 三处重复）。
---
## 五、安全与性能
| 项 | 说明 | 建议 |
|---|---|---|
| 明文 token | `atomcode-accounts.yaml` 及导出文件含 `access_token/refresh_token` 明文 | 至少在 README 声明风险；可选：导出时脱敏开关、文件权限收紧、OS keyring 集成 |
| `update_message.clone()` 每帧 | 视口内每帧 clone String | 改为 match 借用，开销小但顺手 |
| `check_auto_switch` | 阈值判断 `>=` 与文案"超过"语义微差；切换目标若过期会弹自动更新窗口 | `delete_account`/自动切换等程序性切换可加参数跳过 `open_auto_update_window`，避免删除账号时意外弹窗 |
---
## 六、工程化建议
1. **单元测试**：`apply_parsed_fields`、`parse_reset_datetime`、`combine_reset_datetime` 都是纯函数，非常适合补 `#[cfg(test)]` 用中英文样例输出做回归测试（AGENTS 说"以 cargo check 为准"，加测试不冲突）。
2. **CI**：加 `cargo fmt --check` / `cargo clippy` / `cargo check` 的工作流，防止文档说的"check 通过"只靠本地自觉。
3. **文档同步**：
   - 文件树缺 `app.rc` 和 `../../atomcode.ico`（`build.rs`/`main.rs` 都依赖它们，AGENTS 维护规则要求同步）；
   - README Usage 部分仍写 "click **Auto-Fetch**"，v0.1.6 已更名 **Re-fetch**；按钮名 "Update Login Info" 与实际 "Update Info" 也不一致。
4. **日志**：目前无任何日志，解析失败只能靠 UI 提示。建议加轻量 `log` + 文件输出，便于排查 `/login` 格式变化。
---
## 七、次要细节清单
- `theme.rs` 里 `pub const _GITHUB_*` 下划线前缀对 pub 项无 unused 抑制作用，纯命名噪音，建议去前缀或删除。
- `I18n::t` 顺序替换占位符：若某参数本身含 `{1}` 会被后续替换注入，可一次遍历替换（低风险）。
- 语言切换后 `status_message` 被重置为"就绪"，丢上下文（可接受，仅提示）。
- 子视口未 `with_icon`，任务栏/标题栏图标与主窗口不一致。
- `import_current_auth` 新账号硬编码 `plan_name: "CodingPlan Lite"`、`remaining_days: 27`，建议改为中性默认（"未知/Unknown"）并提示用户执行 /login。
- 顶部工具栏与底部状态栏同时显示 `status_message`，信息重复（若刻意保留可忽略）。
- 连续点击"重新获取"会叠加后台线程/子进程（旧线程 3s 后自 kill，危害小），可在 `start_auto_update` 前检查 `is_auto_updating` 直接 return。
---
**建议处理顺序**：二.1/二.2/二.3（正确性）→ 二.4/三.9（统一解析 + panic 防御）→ 二.8/三.10（获取流程与解析覆盖）→ 四.11~13（重构）→ 其余按需。


读完了文档里附的全部源码（`app/`、`ops/`、`io/`、`i18n/`、`models.rs`、`main.rs`、构建脚本）。整体判断：**这是一个模块拆分相当干净、约定文档写得比多数小项目都认真的小型 GUI 工具**——`lib.rs` 薄入口、按职责分目录、i18n 与配置兼容都有显式规约、`panic=abort` 的 release 约束也写进了 `../../AGENTS.md`。但正因为约束写得清楚，对照代码能发现几处"规约没被代码兜住"的地方，其中有几条会**静默改错数据或直接丢账号**。

以下按修复优先级排列，行号对应文档中的文件。

---

## 一、P0：正确性与数据安全，建议优先修

### 1. 配置解析失败 → 静默清空全部账号，且写入非原子

`config_io.rs:2549` / `:2563`：

```rust
serde_yaml::from_str(&content).unwrap_or_default()   // 解析失败 = 空配置
let _ = fs::write(&path, content);                    // 写失败也吞掉
```

`AppConfig`（`models.rs:2667`）只有 `language` / `auto_update_on_switch_expired` / `sort_mode` 带 `#[serde(default)]`，`accounts`、`auto_switch_rules` 及 `ManagedAccount` 的字段靠"人肉保证"兼容。任何旧版 YAML 缺字段、手工编辑出错、或未来加字段忘标 default，都会导致**整份反序列化失败 → 回退到空配置 → 用户下一次点设置/切换就 `save_config` 把空配置写回原文件**，账号和 token 一次性没了。`migrate_old_config_if_needed()`（`:2527`）同样只在"能解析成 AppConfig"时才迁移，否则旧文件既不迁也不提示，数据被静默遗弃。

同时 `fs::write` 是直接覆盖，写一半断电/崩溃会留下截断的 YAML，下次启动又触发上面的清空逻辑。

**建议**
- `AppConfig` / `ManagedAccount` / `AutoSwitchRules` 加**结构体级** `#[serde(default)]` 并实现 `Default`，比逐字段加更不容易漏；
- 解析失败时把原文件重命名为 `.bak`、在 UI 明确报错，**不要 `unwrap_or_default()`**；
- 保存走 `写 .tmp → fs::rename` 原子替换，保留一份 `.bak`。`save_config` 返回 `Result<(), String>`，别再吞错误。

### 2. 关掉更新窗口后，后台任务仍会悄悄改账号并落盘

`ui.rs:1124`、`:1251` 的取消 / 关闭处理只做了：

```rust
self.show_manual_update = false;
self.clear_update_message();
```

没清 `auto_update_rx`、`is_auto_updating`、`auto_parse_pending`。而 `poll_auto_fetch()`（`fetch_info.rs:3238`）在 `update()` 里**每帧无条件执行**：3 秒后输出到达，若 `auto_parse_pending` 仍为 true，就直接 `parse_login_output_and_update` → `save_config` → `import_current_auth`，窗口早已不存在，用户只看到主状态栏莫名变了一次。`update_cancel` 按钮（`:1161`）同样只丢 rx 不清 pending，后台线程照跑。

**建议**：抽一个统一的 `close_update_window()` 清干净全部状态；给每次 fetch 分配递增的 `generation: u64`，后台 `tx.send((gen, output))`，poll 时 `gen != current_gen` 直接丢弃。取消/关窗即 `current_gen += 1`，旧结果永不生效。

### 3. 解析成功后的自动同步，会撤销"登录用户 ≠ 当前账号"的账号切换

`account_ops.rs:2961-3002`：当 `/login` 输出里的用户名匹配到**另一个**已存账号时，代码把 `active_account_id` 改成匹配账号，但**没有把该账号的 `auth.toml` 写到磁盘**。紧接着 UI 成功路径 `ui.rs:1101` / `fetch_info.rs:3257` 调用 `import_current_auth()`，它读磁盘上仍是原账号的 auth，并无条件执行 `self.config.active_account_id = Some(id)`（`account_ops.rs:2889`）——刚切过去的账号又被切回来了。这个功能是 v0.1.6 引入自动同步后才失效的，属于两个改动叠加出的回归。

顺带一提：成功路径里 `status_message` 被 `import_current_auth()` 覆盖成"导入成功: xxx"，窗口内绿色 Success 消息（`:1104`）显示的是同步结果而非"账号信息已更新"，语义也串了。

**建议**：匹配分支成功时同步 `atomcode_io::write_auth(matched)`，或明确改成"检测到用户不一致 → 提示用户手动切换"，不要静默改 active；成功消息单独构造，别复用 `status_message`。

### 4. 自动切换会在两个账号之间每 30 秒来回抖动

`account_ops.rs:2766-2801` 的判定只有「当前 active 用量 ≥ 阈值」，然后从**其余账号**里挑用量最低的就切，没有要求目标账号低于阈值，也没有滞回/冷却。当所有账号都超阈值时：切到最低 → 30 秒后发现新 active 仍超阈值 → 挑"除它之外最低"（正是刚切走的那个）→ 再切回去……每次都 `write_auth` + `save_config`。

**建议**：加 `best_acc.usage_percent < threshold` 前置条件，或要求 `best < active - hysteresis`（如 5%），并给 `last_auto_check` 加切换冷却。

### 5. `/login` 解析把垃圾重置时间当作"解析成功"，违反"零字段不落盘"约定

`account_ops.rs:3114-3149`：只要某行 `contains("重置于")`，`split(...).nth(1)` 必然拿到内容；若后面不是合法 `HH:MM`，`hm` 就是整段残文，`combine_reset_datetime()`（`:2926`）里 `parse()` 失败静默退化成 `h=0, m=0`，生成 `"明天 00:00 残文..."`，却仍然 `parsed_any = true` 并落盘。英文 `resets` 分支同样。结果是：不完整/错误的输出被当成有效数据，账号显示一个永不过期的怪字符串。函数注释还承诺"返回 false 时不修改 acc 的任何字段"，当前实现是先就地改、边改边计数，无法回滚。

**建议**：严格校验 `0<=h<24 && 0<=m<60`；把解析结果先收进一个 `ParsedFields { plan: Option<String>, ... }`，确认至少一个字段有效后再一次性 apply——这样"零字段不落盘"才是真的事务性，也顺带让这段纯逻辑可单测。另补纯英文 `Usage: 12%` 的用量匹配（现在只认中文"用量约"）。

---

## 二、P1：崩溃风险、约定一致性与跨平台兼容

### 6. `panic = "abort"` 下仍有一串 `unwrap` / `expect` / `assert`

`../../AGENTS.md` 自己写了"release 下 panic 不可恢复"，但代码里：

| 位置 | 触发条件 |
|---|---|
| `main.rs:2605-2610` | ICO 损坏、无条目、解码失败 → 启动即 abort，无任何提示 |
| `ui.rs:1061` | `assert!(class == ViewportClass::Immediate)`，防御性断言在 release 同样生效 |
| `account_ops.rs:2816`、`ui.rs:1883` | `NaiveDateTime::and_local_timezone(Local).unwrap()`，DST 跳变时本地时间不存在 → `LocalResult::None` → panic |

**建议**：图标解析失败降级为无图标启动并返回可读错误；`assert!` 改 `debug_assert!`；时区转换用 `.single()` / `.earliest()` 显式分支。全仓扫一遍 `unwrap/expect`，release 路径只保留不可能失败的不变量。

### 7. "是否过期"实际上有三套实现，排序那套不兼容旧格式

`../../AGENTS.md` 要求"到期判断集中在 `is_account_expired()`，卡片展示与其保持一致"：

- `is_account_expired()`（`account_ops.rs:2804`）— 带 `Local` 时区；
- `format_reset_countdown()`（`ui.rs:1876`）— 带时区，另算倒计时；
- `sort_accounts()`（`ui.rs:1929-1953`）— 自己用 `parse_reset_time_for_sort` + `now.naive_local()` 判 `stale`。

第三套对旧 `HH:MM` 配置一律返回 `None → stale=true`，导致**兼容旧格式"的账号在排序模式里永远被排到最后**；且 naive 与带时区的口径在边界时刻可能不一致。

**建议**：抽 `parse_reset_datetime(&str) -> Option<DateTime<Local>>`（内部兼容新旧两种格式）作为唯一入口，`is_account_expired()` 唯一判过期，倒计时、排序、切换过期自动更新三处全部复用它。这一条能同时消掉 3 个潜在 bug 面。

### 8. `atomcode login` 输出捕获存在管道背压，靠 3 秒强杀兜底

`fetch_info.rs:3202-3226` 在**一个线程里先 `read_to_string(stdout)`、再 `read_to_string(stderr)`**。若子进程先写满 stderr 管道，会阻塞在写 stderr、stdout 不关闭，读线程就卡住；外层靠固定 `sleep(3s)` + `child.kill()` 兜底，虽不至于永久挂起，但输出可能被截断，且即使进程 0.2 秒就输出完也要等满 3 秒。

**建议**：stdout / stderr 各开一个读取线程；用 `try_wait()` 检测进程自然退出后立即收尾；超时改为可配置，kill 前留短暂 flush 窗口。另外 `Command::new("atomcode")` 走 PATH 且隐藏窗口执行，建议改用配置里的 AtomCode 目录解析绝对路径，避免 PATH 劫持。

### 9. Windows + Git Bash 下 `HOME` 优先会把 `~/.atomcode` 指错

`atomcode_io.rs:2355`、`config_io.rs:2423` 都是 `HOME` 优先于 `USERPROFILE`。Git Bash 里 `HOME` 常是 POSIX 风格 `/c/Users/xxx`，`PathBuf::from` 在 Windows 上会被解析成当前盘根下的 `c\Users\xxx`，于是**读不到 AtomCode CLI 实际写的 `%USERPROFILE%\.atomcode\auth.toml`**——恰好命中项目声明的工作环境。

**建议**：Windows 下优先 `USERPROFILE`，或直接引入 `dirs` / `home` crate；对自定义目录做 `canonicalize()` + 可写校验，别存未校验的相对路径。

### 10. 导出默认文件名 `config.yaml` 会覆盖程序自己的本地配置

`ui.rs:1820` 的 `save_file_dialog(..., "config.yaml")` 与 `config_io.rs:2455` 的 `get_exe_dir()/config.yaml` 同名同目录。导出到 dist 目录后，`load_local_config()` 会把这份完整 `AppConfig` 反序列化成 `LocalConfig { custom_data_dir }`；serde 默认忽略未知字段、`Option` 缺失即 `None`，**自定义数据目录设置被静默清空**，下次启动找不到 `atomcode-accounts.yaml`。

**建议**：导出改名 `atomcode-accounts-export.yaml`；对话框明确提示"包含 access_token / refresh_token 明文"；提供"仅导出账号元数据"选项。同时 `atomcode-accounts.yaml` 与 `auth.toml` 建议设 `0o600`（Unix）并原子写（`atomcode_io.rs:2397` 目前也是直接 `fs::write`，外部 AtomCode 可能读到半个 toml）。

### 11. i18n 强制约定有漏网，且没有机制保证中英词条对齐

- `ui.rs:1853` 硬编码 `"导出冲突"`，而 `dialog_overwrite_title` 在中英两个语言文件里都定义好了却没人用；
- `fetch_info.rs:3226/3229` 的 `"（未获取到输出）"`、`"（无法执行 atomcode login: {}）"` 是中文硬编码，会填进文本框并参与解析，英文界面下也显示中文；
- `ui.rs:1663/1831` 的 `"  - {}（{}）"` 全角括号同样是硬编码格式。

两个语言文件的 `_ => key` 回退意味着**漏加 key 不报错，直接把 `status_xxx` 显示给用户**。

**建议**：补齐上述 key；把语言表改成编译期可比对的结构（或至少加一个遍历 key 集合的 `#[cfg(test)]` 断言），让"同时写入两处"从人工约定变成自动校验。`I18n::t()`（`:2306`）用顺序 `replace` 还有个小坑：若参数值本身含 `{1}`，会被后续替换误伤，建议一次性解析模板。

### 12. 导入时硬编码的"假数据"会被自动切换当成最健康账号

`account_ops.rs:2877-2886` 新账号固定 `usage_percent: 0.0`、`is_valid: true`、`remaining_days: 27`、`plan: "CodingPlan Lite"`。这些信息全是编的，却直接参与第 4 条的自动切换选择——刚导入、信息完全未知的账号会被判定为"用量最低的有效账号"优先切换。

**建议**：引入显式状态（如 `info_status: Unknown / Fresh / Expired`）或 `usage_percent: Option<f32>`；`Unknown` 账号不参与自动切换，并引导用户走一次更新流程。`is_valid` 与"信息新鲜度"是两个概念，不该混用。

---

## 三、P2：性能、架构与工程化

| # | 位置 | 问题与建议 |
|---|---|---|
| 13 | `ui.rs:980` | 每帧 `self.config.accounts.clone()`，每个账号含完整 `AuthToml`（access/refresh token）。改为克隆索引/`Vec<&ManagedAccount>` 排序，或仅在 config 变更时重排并缓存顺序；配合 `request_repaint_after(1s)`，能省掉大量无谓的敏感串拷贝。 |
| 14 | `theme.rs:563-578` vs `:679-693` | 顶部定义了一套颜色常量，`setup_github_theme` 又重新 `let` 一套局部变量，`_GITHUB_BTN_HOVER=(243,244,246)` 与 `github_btn_hover=(238,242,246)` 已经不一致。统一引用常量；`usage_color` 与 `progress_bar_color` 逻辑重复，可参数化合并。 |
| 15 | `ui.rs:1651-1707` | 设置窗口的"浏览目录"回调里内联了 60 行：读已有配置、拼账号列表、弹冲突框、**直接改 `self.config.accounts`**。用户随后点"取消"，内存已被替换但不落盘，状态不一致。浏览应只改 `state.data_dir`，导入动作统一放到"确定"里；与 `handle_export`（`:1819`）重复的冲突提示逻辑抽成公共函数。 |
| 16 | `ui.rs:1104`、`:3260` | 成功时先 `set_update_message(Success, ...)` 再 `show_manual_update = false`，绿色成功区**永远没有机会渲染**。要么延迟 1~2 秒关窗，要么把成功结果直接写主状态栏并删掉这段死 UI。 |
| 17 | `fetch_info.rs:3270` | `auto_parse_pending` 在三处入口都被设为 true，`else` 分支的 `status_fetched`（"点击解析并更新完成更新"）事实上不可达，文案还与 v0.1.7 的自动解析行为矛盾，建议删除。 |
| 18 | 整体 | `AtomcodeSwitchApp` 同时持有 config、i18n、全部对话框字段和后台 channel，业务方法靠 `impl` 分散在 `app/`、`ops/` 三个文件里拼接；`config_io` 又是直接读写真实路径的全局函数，**纯逻辑无法脱离文件系统单测**。建议把解析/时间/排序/迁移抽成不依赖 App 的纯函数层，IO 通过 `&Path` 或 trait 注入。 |
| 19 | `AGENTS.md:64` | 明确写着"仓库没有任何测试"——但这套代码里最有价值、最容易回归的恰恰是纯函数：`/login` 解析、新旧时间格式兼容、三处过期判定一致性、中英词条对齐、排序、配置迁移。**这应该是第一个补 `#[cfg(test)]` 的项目**，用 `cargo test` 把这些约定固化成断言；按 `../../AGENTS.md` 的维护规则，测试命令变化也要同步回该文件。 |

---

## 四、文档与工程事实已经漂移

这些不直接影响运行，但会让下一位维护者（或 AI 助手）按错误前提改代码，且直接违反 `../../AGENTS.md` 的"维护规则"：

- **版本**：`../../Cargo.toml` 已是 `0.1.8`，`CHANGE_HISTORY.md` 最新条目停在 `v0.1.7`，发布流程里的示例 tag 也是 `Release v0.1.7`；`releases/vX.Y.Z.md` 目录在文件树里根本不存在。
- **README**：`Auto & Manual Info Update` 和 Usage Guide 仍写"点 Auto-Fetch → 再点 Parse & Update"，与 v0.1.6/0.1.7 的**打开即自动获取、获取完自动解析、成功自动关窗、按钮改名 Re-fetch** 完全不符；`Links` 行尾还留了个孤立 `|`。
- **文件树**：漏了 `app.rc`（`../../build.rs` 引用）、`atomcode.ico`（`main.rs` 用 `include_bytes!` 编译期引用，缺了全平台编译不过）、`releases/`；`.gitignore` 里 `.atomcode/` 与 README 的 `~/.atomcode` 也容易混淆。
- **构建脚本**：`../../build.rs` 无条件调用 `embed_resource::compile("app.rc", NONE)`，缺 `cargo:rerun-if-changed=app.rc` / `atomcode.ico`，也没加 `#[cfg(windows)]` 保护；`build.ps1` 硬编码 `.exe`，与 README 的 macOS/Linux 构建说明不一致。
- **小语义**：`sort_none` 词条叫"不排序"，实现（`ui.rs:1941`）其实是按账号名排序。

---

## 建议的落地顺序

1. **先保数据**：配置结构体级 `serde default` + 解析失败备份报错 + 原子写（第 1 条），顺手把 `write_auth` 也改原子写。
2. **再修状态机**：统一关窗/取消 + fetch generation 令牌（第 2 条），修自动切换抖动与解析事务性（第 4、5 条），修解析后 import 撤销切换（第 3 条）。
3. **然后收敛约定**：抽 `parse_reset_datetime` 单一过期判定（第 7 条）、清掉 release 里的 panic 面（第 6 条）、补 i18n 漏网与词条对齐断言（第 11 条）。
4. **最后工程化**：补 `#[cfg(test)]` 覆盖纯逻辑并把测试命令写回 `../../AGENTS.md`；同步 README / CHANGE_HISTORY / 文件树 / 版本号；再做性能与架构拆分（第 13–18 条）。

---

以上是对所附文档内全部源码的**静态阅读分析，未实际编译运行**（文档是快照，不是可运行仓库）。如果你把仓库或关键几个 `.rs` 单独给我，我可以进一步做 `cargo check` 验证，或者直接把第 1、2、5、7 条改成可提交的补丁，并补上对应的单元测试。