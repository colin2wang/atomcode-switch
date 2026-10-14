use serde::{Deserialize, Serialize};

// --- auth.toml 对应的数据结构 ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthToml {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub created_at: u64,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
}

// --- atomcode-accounts.yaml 对应的数据结构 ---

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            auto_switch_rules: AutoSwitchRules {
                enabled: false,
                max_usage_percent: 95.0,
            },
            language: default_language(),
            auto_update_on_switch_expired: false,
            sort_mode: SortMode::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitchRules {
    pub enabled: bool,
    pub max_usage_percent: f32,
}
