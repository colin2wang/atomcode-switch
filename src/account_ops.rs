//! 账号操作：同步、切换、导入、删除、自动切换、/login 解析

use crate::models::ManagedAccount;
use crate::{atomcode_io, config_io};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::AtomcodeSwitchApp;

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
                if active_acc.usage_percent >= threshold {
                    let mut best_acc: Option<&ManagedAccount> = None;
                    for acc in &self.config.accounts {
                        if acc.is_valid && acc.id != *active_id {
                            if best_acc.is_none()
                                || acc.usage_percent < best_acc.unwrap().usage_percent
                            {
                                best_acc = Some(acc);
                            }
                        }
                    }

                    if let Some(acc_to_switch) = best_acc {
                        let id_to_switch = acc_to_switch.id.clone();
                        let name = acc_to_switch.auth_data.user.name.clone();
                        self.switch_to_account(&id_to_switch);
                        self.status_message = self.i18n.t1("status_auto_switched", &name);
                    }
                }
            }
        }
    }

    /// 写入 auth.toml 并切换激活账号
    pub fn switch_to_account(&mut self, id: &str) {
        if let Some(acc) = self.config.accounts.iter().find(|a| a.id == id) {
            match atomcode_io::write_auth(&self.config.custom_atomcode_dir, &acc.auth_data) {
                Ok(_) => {
                    self.config.active_account_id = Some(id.to_string());
                    config_io::save_config(&self.config);
                    self.status_message = self.i18n.t1("status_switched", &acc.auth_data.user.name);
                }
                Err(e) => {
                    self.status_message = self.i18n.t1("status_switch_failed", &e);
                }
            }
        }
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
                    plan_name: "CodingPlan Lite".to_string(),
                    usage_percent: 0.0,
                    reset_time: Self::default_reset_time(),
                    is_valid: true,
                    auth_data: auth,
                    remaining_days: 27,
                    last_updated: Self::current_time_str(),
                });
            }

            self.config.active_account_id = Some(id);
            config_io::save_config(&self.config);
            self.status_message = self.i18n.t1("status_imported", &name);
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
                self.switch_to_account(&new_id);
            }
        }
        config_io::save_config(&self.config);
        self.status_message = self.i18n.t0("status_account_deleted");
    }

    // ---- 时间工具 ----

    /// 获取当前本地时间字符串 (HH:MM)
    pub fn current_time_str() -> String {
        chrono::Local::now().format("%H:%M").to_string()
    }

    /// 生成默认重置时间（当前时间 + 1小时），仅返回 HH:MM 格式
    pub fn default_reset_time() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let now_mins = (secs / 60) % 1440;
        let reset_mins = (now_mins + 60) % 1440;
        format!("{:02}:{:02}", reset_mins / 60, reset_mins % 60)
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
                    a.auth_data.user.name == login_name
                        || a.auth_data.user.username == login_name
                }) {
                    self.config.active_account_id =
                        Some(self.config.accounts[matched_idx].id.clone());
                    let new_acc = &mut self.config.accounts[matched_idx];
                    new_acc.last_updated = Self::current_time_str();
                    new_acc.is_valid = true;
                    Self::apply_parsed_fields(new_acc, text);
                    config_io::save_config(&self.config);
                    self.status_message = self.i18n.t1("status_updated_from_paste", login_name);
                    return Ok(());
                }
            }
        }

        let acc = &mut self.config.accounts[active_idx];
        Self::apply_parsed_fields(acc, text);
        acc.last_updated = Self::current_time_str();
        acc.is_valid = true;

        config_io::save_config(&self.config);
        self.status_message = self.i18n.t0("status_updated_manual");
        Ok(())
    }

    /// 将解析出的字段应用到 ManagedAccount（纯借用辅助方法）
    /// 同时支持中文和英文格式的 /login 输出
    pub fn apply_parsed_fields(acc: &mut ManagedAccount, text: &str) {
        // ---- 套餐名 ----
        // 中文：套餐：CodingPlan Lite · expires ...
        // 英文：Plan: CodingPlan Lite  ·  expires ...
        if let Some(start) = text.find("套餐：") {
            let after = &text[start + "套餐：".len()..];
            if let Some(end) = after.find("·") {
                acc.plan_name = after[..end].trim().to_string();
            } else if let Some(end) = after.find('\n') {
                acc.plan_name = after[..end].trim().to_string();
            }
        } else if let Some(start) = text.find("Plan:") {
            let after = &text[start + "Plan:".len()..];
            if let Some(end) = after.find("·") {
                acc.plan_name = after[..end].trim().to_string();
            } else if let Some(end) = after.find('\n') {
                acc.plan_name = after[..end].trim().to_string();
            }
        }

        // ---- 剩余天数 ----
        // 中文：剩余 19 天
        // 英文：expires ... (19d / 30d remaining)
        let mut days_found = false;
        if let Some(pos) = text.find("剩余 ") {
            let rest = &text[pos + "剩余 ".len()..];
            let days_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !days_str.is_empty() {
                if let Ok(days) = days_str.parse::<u32>() {
                    acc.remaining_days = days;
                    days_found = true;
                }
            }
        }
        if !days_found {
            // 英文：找 "d /" 前面的数字
            if let Some(pos) = text.find("d /") {
                let before = &text[..pos];
                let digits: String = before
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if !digits.is_empty() {
                    if let Ok(days) = digits.parse::<u32>() {
                        acc.remaining_days = days;
                    }
                }
            }
        }

        // ---- 用量百分比 ----
        // 中文：用量约 0%
        // 英文：Usage: 当前时间窗口用量约 0%
        if let Some(pos) = text.find("用量约") {
            let rest = &text[pos + "用量约".len()..];
            let num_str: String = rest
                .chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if !num_str.is_empty() {
                if let Ok(pct) = num_str.parse::<f32>() {
                    acc.usage_percent = pct;
                }
            }
        }

        // ---- 重置时间 ----
        // 中文：重置于 21:04 (in ...)
        // 英文：resets 21:04 (in ...)
        let mut reset_found = false;
        for line in text.lines() {
            if line.contains("重置于") {
                if let Some(after) = line.split("重置于").nth(1) {
                    let raw = after.trim().to_string();
                    // 提取 HH:MM 部分（前5个字符）
                    acc.reset_time = if raw.len() >= 5 && raw.as_bytes()[2] == b':' {
                        raw[..5].to_string()
                    } else {
                        raw
                    };
                }
                reset_found = true;
                break;
            }
        }
        if !reset_found {
            for line in text.lines() {
                if line.contains("resets ") || line.contains("resets\t") {
                    if let Some(after) = line.split("resets").nth(1) {
                        let raw = after.trim().to_string();
                        // 提取 HH:MM 部分（前5个字符）
                        acc.reset_time = if raw.len() >= 5 && raw.as_bytes()[2] == b':' {
                            raw[..5].to_string()
                        } else {
                            raw
                        };
                    }
                    break;
                }
            }
        }
    }
}
