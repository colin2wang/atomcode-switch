use serde::{Deserialize, Serialize};

// --- auth.toml 对应的数据结构 ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AuthToml {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub created_at: u64,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct User {
    pub id: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
}

// --- atomcode-accounts.yaml 对应的数据结构 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// 自定义 auth.toml 保存目录（默认是 ~/.atomcode）
    pub custom_atomcode_dir: Option<String>,
    /// 当前激活的账户 ID
    pub active_account_id: Option<String>,
    /// 保存的账户列表
    pub accounts: Vec<ManagedAccount>,
    /// 自动切换规则设置
    pub auto_switch_rules: AutoSwitchRules,
    /// 语言设置
    #[serde(default = "default_language")]
    pub language: String,
    /// 切换账号时若目标账号信息已过期，自动打开更新窗口并自动更新
    #[serde(default)]
    pub auto_update_on_switch_expired: bool,
    /// 卡片列表排序方式
    #[serde(default)]
    pub sort_mode: SortMode,
}

/// 卡片列表排序方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortMode {
    /// 不排序（按名称排列）
    #[default]
    None,
    /// 按重置时间（主键），窗口余量（次键）
    ResetTime,
    /// 按窗口余量（主键），重置时间（次键）
    WindowRemaining,
}

fn default_language() -> String {
    "zh_cn".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            custom_atomcode_dir: None,
            active_account_id: None,
            accounts: vec![],
            auto_switch_rules: AutoSwitchRules::default(),
            language: default_language(),
            auto_update_on_switch_expired: false,
            sort_mode: SortMode::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ManagedAccount {
    pub id: String,
    pub auth_data: AuthToml,
    pub plan_name: String,
    pub usage_percent: f32,
    pub reset_time: String,
    pub is_valid: bool,
    /// 剩余天数
    pub remaining_days: u32,
    /// 上次更新时间
    pub last_updated: String,
    /// 账号信息新鲜度：新导入但未跑过 /login 的账号为 Unknown，
    /// 不参与自动切换，避免假数据（usage=0）被当成最健康账号。
    #[serde(default)]
    pub info_status: AccountInfoStatus,
}

/// 账号信息新鲜度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AccountInfoStatus {
    /// 信息未知（刚导入，尚未解析过 /login 输出）
    #[default]
    Unknown,
    /// 已通过 /login 解析刷新过
    Fresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoSwitchRules {
    pub enabled: bool,
    pub max_usage_percent: f32,
}

impl Default for AutoSwitchRules {
    fn default() -> Self {
        Self {
            enabled: false,
            max_usage_percent: 95.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_config_missing_fields_loads_with_defaults() {
        // 模拟旧版 YAML：缺少 accounts / auto_switch_rules / language 等字段
        let yaml = "custom_atomcode_dir: null\nactive_account_id: null\n";
        let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.accounts.is_empty());
        assert_eq!(config.language, "zh_cn");
        assert!(!config.auto_update_on_switch_expired);
        assert_eq!(config.sort_mode, SortMode::None);
        assert_eq!(config.auto_switch_rules.max_usage_percent, 95.0);
    }

    #[test]
    fn account_missing_new_field_defaults_to_unknown() {
        let yaml = "id: abc\nplan_name: Old\nusage_percent: 10.0\nreset_time: \"09:00\"\nis_valid: true\nremaining_days: 5\nlast_updated: x\nauth_data:\n  access_token: a\n  refresh_token: r\n  token_type: Bearer\n  expires_in: 1\n  created_at: 1\n  user:\n    id: u\n    username: n\n    name: n\n    email: e\n    avatar_url: \"\"\n";
        let account: ManagedAccount = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(account.info_status, AccountInfoStatus::Unknown);
        assert_eq!(account.reset_time, "09:00");
    }
}
