//! 国际化翻译 — 每门语言一个独立的 .rs 文件
//!
//! 新增语言步骤：
//! 1. 创建 src/i18n/lang/xx.rs，实现 `pub fn get(key: &str) -> &str`
//! 2. 在本文件添加 `pub mod xx;`
//! 3. 在 `Language` 枚举添加 `Xx` 变体
//! 4. 在 `lookup()` 中添加匹配臂

pub mod zh_cn;
pub mod en_us;

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    ZhCn,
    EnUs,
}

impl Language {
    /// 从持久化字符串解析
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "en" | "en_us" | "en-us" | "english" => Language::EnUs,
            _ => Language::ZhCn,
        }
    }

    /// 转为持久化字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::ZhCn => "zh_cn",
            Language::EnUs => "en_us",
        }
    }

    /// 获取该语言的字符串查找函数
    pub fn lookup(&self) -> fn(&str) -> &str {
        match self {
            Language::ZhCn => zh_cn::get,
            Language::EnUs => en_us::get,
        }
    }
}

/// 所有已定义的 i18n key。新增 key 时必须同时加入此表与两个语言文件。
#[cfg(test)]
pub const ALL_KEYS: &[&str] = &[
    "toolbar_settings", "toolbar_sync", "bottom_accounts", "status_normal", "status_error",
    "btn_export", "btn_clear_auth", "card_in_use", "card_activated", "card_update_info",
    "card_activate", "card_delete", "card_valid", "card_invalid", "card_expiring", "card_expired", "card_remaining_days",
    "card_usage_label", "card_reset_label", "card_reset_countdown", "card_reset_countdown_h",
    "card_reset_expired", "empty_no_accounts", "empty_hint", "empty_import", "sort_none",
    "sort_reset_time", "sort_window_remaining", "settings_title", "settings_export_dir",
    "settings_export_dir_desc", "settings_browse", "settings_export_dir_hint",
    "settings_atomcode_dir", "settings_atomcode_dir_desc", "settings_atomcode_hint", "settings_auto_switch",
    "settings_enable_auto_switch", "settings_usage_exceeds", "settings_auto_switch_to",
    "settings_auto_update_on_switch_expired", "btn_confirm", "btn_cancel", "update_title",
    "update_desc", "update_auto_fetch", "update_fetching", "update_cancel", "update_parse",
    "update_hint", "update_error_title", "update_error_hint", "update_parse_failed", "update_success_title",
    "delete_confirm_title", "delete_confirm_msg", "btn_delete", "dialog_select_data_dir",
    "dialog_select_atomcode_dir", "dialog_export_title", "dialog_switch_dir_title",
    "dialog_overwrite_title", "dialog_switch_dir_msg", "dialog_overwrite_msg", "dialog_none",
    "dialog_account_line", "status_ready", "status_save_failed", "status_config_load_failed",
    "status_auto_switched", "status_switched", "status_switch_failed", "status_imported",
    "status_auth_not_found", "status_account_deleted", "status_no_active_account",
    "status_active_account_not_found", "status_updated_from_paste", "status_updated_manual",
    "status_fetching", "status_fetched", "status_fetch_failed", "status_paste_empty",
    "status_no_active_import", "status_updated", "status_update_failed", "status_parse_no_fields",
    "status_auth_cleared", "status_clear_failed", "status_auto_fetch_cancelled",
    "status_imported_count", "status_kept_current", "status_exported", "status_kept_original",
    "status_settings_saved", "status_imported_hint", "account_plan_unknown", "fetch_no_output",
    "fetch_spawn_failed", "fetch_timeout",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keys_present_in_both_languages() {
        for key in ALL_KEYS {
            assert_ne!(zh_cn::get(key), *key, "zh_cn missing key: {key}");
            assert_ne!(en_us::get(key), *key, "en_us missing key: {key}");
        }
    }
}
