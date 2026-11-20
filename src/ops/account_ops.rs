//! 账号操作：同步、切换、导入、删除、自动切换、/login 解析

use crate::app::AtomcodeSwitchApp;
use crate::io::atomcode_io;
use crate::models::ManagedAccount;
use crate::ops::login_parser::parse_login_fields;
use crate::ops::time_utils::parse_reset_datetime;

impl AtomcodeSwitchApp {
    /// 同步当前激活账号的状态（从 auth.toml 读取）
    pub fn sync_active_account_status(&mut self) {
        if let Some(auth) = atomcode_io::read_current_auth(&self.config.custom_atomcode_dir) {
            let id = auth.user.id.clone();
            if let Some(acc) = self.config.accounts.iter_mut().find(|a| a.id == id) {
                acc.auth_data = auth;
                self.config.active_account_id = Some(id);
            }
        }
    }

    /// 检查是否需要自动切换
    pub fn check_auto_switch(&mut self, current_time: f64) {
        if current_time - self.last_auto_check < 30.0 {
            return;
        }
        self.last_auto_check = current_time;

        if !self.config.auto_switch_rules.enabled {
            return;
        }

        let threshold = self.config.auto_switch_rules.max_usage_percent;

        if let Some(active_id) = &self.config.active_account_id {
            if let Some(active_acc) = self.config.accounts.iter().find(|a| &a.id == active_id) {
                let active_usage = active_acc.usage_percent;
                if active_usage >= threshold {
                    // 目标必须低于阈值，且与当前账号有足够差距（滞回），
                    // 避免所有账号都超阈值时在两者之间来回抖动。
                    const HYSTERESIS: f32 = 5.0;
                    let mut best_acc: Option<&ManagedAccount> = None;
                    for acc in &self.config.accounts {
                        if acc.is_valid
                            && acc.id != *active_id
                            && acc.info_status == crate::models::AccountInfoStatus::Fresh
                            && acc.usage_percent < threshold
                            && active_usage - acc.usage_percent >= HYSTERESIS
                        {
                            let better = best_acc
                                .map(|b| acc.usage_percent < b.usage_percent)
                                .unwrap_or(true);
                            if better {
                                best_acc = Some(acc);
                            }
                        }
                    }

                    if let Some(acc_to_switch) = best_acc {
                        let id_to_switch = acc_to_switch.id.clone();
                        let name = acc_to_switch.auth_data.user.name.clone();
                        // 自动切换属于程序性切换，不打开更新窗口
                        self.switch_to_account_silent(&id_to_switch);
                        self.status_message = self.i18n.t1("status_auto_switched", &name);
                    }
                }
            }
        }
    }

    /// 判断账号信息是否已过期（重置时间已过且未解析出新数据）
    /// 与倒计时、排序共用 `parse_reset_datetime` 单一解析入口，兼容新旧格式。
    pub fn is_account_expired(&self, acc: &ManagedAccount) -> bool {
        let now = chrono::Local::now();
        parse_reset_datetime(&acc.reset_time, now)
            .map(|target| target < now)
            .unwrap_or(false)
    }

    /// 切换到账号；若开启了「过期自动更新」且目标账号信息已过期，
    /// 自动打开更新信息窗口并触发自动获取 + 解析更新
    pub fn switch_to_account(&mut self, id: &str) {
        self.switch_to_account_impl(id, true);
    }

    /// 程序性切换（自动切换、删除后补位等）：不打开更新窗口，避免意外弹窗
    pub fn switch_to_account_silent(&mut self, id: &str) {
        self.switch_to_account_impl(id, false);
    }

    fn switch_to_account_impl(&mut self, id: &str, open_update_on_expired: bool) {
        if let Some(acc) = self.config.accounts.iter().find(|a| a.id == id) {
            let expired = self.is_account_expired(acc);
            match atomcode_io::write_auth(&self.config.custom_atomcode_dir, &acc.auth_data) {
                Ok(_) => {
                    self.config.active_account_id = Some(id.to_string());
                    let name = acc.auth_data.user.name.clone();
                    // `acc` 至此不再使用，借用释放后可调用 &mut self 方法
                    if self.persist_config() {
                        self.status_message = self.i18n.t1("status_switched", &name);
                        if open_update_on_expired
                            && self.config.auto_update_on_switch_expired
                            && expired
                        {
                            self.open_auto_update_window();
                        }
                    }
                }
                Err(e) => {
                    self.status_message = self.i18n.t1("status_switch_failed", &e);
                }
            }
        }
    }

    /// 打开更新信息窗口并自动获取，获取完成后自动解析更新
    pub fn open_auto_update_window(&mut self) {
        self.reset_for_new_fetch();
        self.show_manual_update = true;
        self.manual_update_text.clear();
        self.auto_parse_pending = true;
        self.clear_update_message();
        self.start_auto_update();
    }

    /// 从当前 auth.toml 导入账号
    pub fn import_current_auth(&mut self) {
        if let Some(auth) = atomcode_io::read_current_auth(&self.config.custom_atomcode_dir) {
            let id = auth.user.id.clone();
            let name = auth.user.name.clone();

            if let Some(acc) = self.config.accounts.iter_mut().find(|a| a.id == id) {
                acc.auth_data = auth;
                acc.last_updated = Self::current_time_str();
            } else {
                self.config.accounts.push(ManagedAccount {
                    id: id.clone(),
                    plan_name: self.i18n.t0("account_plan_unknown"),
                    usage_percent: 0.0,
                    reset_time: Self::default_reset_time(),
                    is_valid: true,
                    auth_data: auth,
                    remaining_days: 0,
                    last_updated: Self::current_time_str(),
                    info_status: crate::models::AccountInfoStatus::Unknown,
                });
            }

            self.config.active_account_id = Some(id);
            if self.persist_config() {
                self.status_message = self.i18n.t1("status_imported_hint", &name);
            }
        } else {
            self.status_message = self.i18n.t0("status_auth_not_found");
        }
    }

    /// 删除指定账号
    pub fn delete_account(&mut self, id: &str) {
        self.config.accounts.retain(|a| a.id != id);
        if self.config.active_account_id.as_deref() == Some(id) {
            self.config.active_account_id = self.config.accounts.first().map(|a| a.id.clone());
            let new_active_id = self.config.active_account_id.clone();
            if let Some(new_id) = new_active_id {
                self.switch_to_account_silent(&new_id);
            }
        }
        if self.persist_config() {
            self.status_message = self.i18n.t0("status_account_deleted");
        }
    }

    // ---- 时间工具 ----

    /// 获取当前本地时间字符串 (YYYY-MM-DD HH:MM)
    pub fn current_time_str() -> String {
        chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
    }

    /// 生成默认重置时间（当前时间 + 1小时），返回 YYYY-MM-DD HH:MM 格式
    pub fn default_reset_time() -> String {
        let target = chrono::Local::now() + chrono::Duration::hours(1);
        target.format("%Y-%m-%d %H:%M").to_string()
    }

    // ---- /login 输出解析 ----

    /// 从 /login 输出文本中解析信息并更新当前激活账号
    pub fn parse_login_output_and_update(&mut self, text: &str) -> Result<(), String> {
        let active_id = self
            .config
            .active_account_id
            .clone()
            .ok_or_else(|| self.i18n.t0("status_no_active_account"))?;

        let active_idx = self
            .config
            .accounts
            .iter()
            .position(|a| a.id == active_id)
            .ok_or_else(|| self.i18n.t0("status_active_account_not_found"))?;

        // 提取用户名并尝试匹配
        if let Some(pos) = text.find("logged in as ") {
            let rest = &text[pos + "logged in as ".len()..];
            let paren_end = rest.find(|c: char| c.is_ascii_whitespace() || c == '。' || c == '\n');
            let username_section = if let Some(e) = paren_end {
                &rest[..e]
            } else {
                rest.trim_end()
            };

            let login_name = if let Some(paren) = username_section.find('(') {
                username_section[..paren].trim()
            } else {
                username_section.trim()
            };

            let current_name = &self.config.accounts[active_idx].auth_data.user.name;
            if current_name != login_name {
                if let Some(matched_idx) = self.config.accounts.iter().position(|a| {
                    a.auth_data.user.name == login_name || a.auth_data.user.username == login_name
                }) {
                    // 先解析：一个字段都没识别出来时不写入任何数据，避免「假成功」
                    let parsed = {
                        let acc = &mut self.config.accounts[matched_idx];
                        Self::apply_parsed_fields(acc, text)
                    };
                    if !parsed {
                        return Err(self.i18n.t0("status_parse_no_fields"));
                    }

                    let matched_id = self.config.accounts[matched_idx].id.clone();
                    // 登录用户与当前激活账号不一致：把匹配账号的 auth 写入磁盘，
                    // 否则调用方随后的 import_current_auth() 会读回旧账号并撤销本次切换。
                    let matched_auth = self.config.accounts[matched_idx].auth_data.clone();
                    if let Err(e) =
                        atomcode_io::write_auth(&self.config.custom_atomcode_dir, &matched_auth)
                    {
                        return Err(e);
                    }
                    self.config.active_account_id = Some(matched_id);
                    {
                        let new_acc = &mut self.config.accounts[matched_idx];
                        new_acc.last_updated = Self::current_time_str();
                        new_acc.is_valid = true;
                    }
                    if !self.persist_config() {
                        return Err(self.status_message.clone());
                    }
                    self.status_message = self.i18n.t1("status_updated_from_paste", login_name);
                    return Ok(());
                }
            }
        }

        let parsed = {
            let acc = &mut self.config.accounts[active_idx];
            Self::apply_parsed_fields(acc, text)
        };
        if !parsed {
            return Err(self.i18n.t0("status_parse_no_fields"));
        }

        {
            let acc = &mut self.config.accounts[active_idx];
            acc.last_updated = Self::current_time_str();
            acc.is_valid = true;
        }

        if !self.persist_config() {
            return Err(self.status_message.clone());
        }
        self.status_message = self.i18n.t0("status_updated_manual");
        Ok(())
    }

    /// 将解析出的字段事务性地应用到 ManagedAccount。
    ///
    /// 解析逻辑在 `login_parser` 纯函数中完成；只有至少识别出一个字段时才写入，
    /// 返回 false 时 `acc` 的任何字段都不变。同时支持中文和英文的 /login 输出。
    pub fn apply_parsed_fields(acc: &mut ManagedAccount, text: &str) -> bool {
        let fields = parse_login_fields(text, chrono::Local::now());
        if fields.is_empty() {
            return false;
        }
        if let Some(plan) = fields.plan_name {
            acc.plan_name = plan;
        }
        if let Some(usage) = fields.usage_percent {
            acc.usage_percent = usage;
        }
        if let Some(reset) = fields.reset_time {
            acc.reset_time = reset;
        }
        if let Some(days) = fields.remaining_days {
            acc.remaining_days = days;
        }
        acc.info_status = crate::models::AccountInfoStatus::Fresh;
        true
    }
}
