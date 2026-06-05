use crate::models::AppConfig;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 本地配置（保存在可执行程序同目录下的 config.yaml）
/// 存储数据目录路径等本地设施配置。只在修改默认值时创建此文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    /// 自定义数据目录路径（默认 ~/.atomcode-switch）
    pub custom_data_dir: Option<String>,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            custom_data_dir: None,
        }
    }
}

/// 获取可执行程序所在目录
fn get_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 获取本地配置文件路径（可执行程序同目录下的 config.yaml）
fn get_local_config_path() -> PathBuf {
    get_exe_dir().join("config.yaml")
}

/// 加载本地配置
fn load_local_config() -> LocalConfig {
    let path = get_local_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        LocalConfig::default()
    }
}

/// 保存本地配置
fn save_local_config(config: &LocalConfig) {
    let path = get_local_config_path();
    if config.custom_data_dir.is_some() {
        if let Ok(content) = serde_yaml::to_string(config) {
            let _ = fs::write(&path, content);
        }
    } else {
        let _ = fs::remove_file(&path);
    }
}

/// 获取当前自定义数据目录
pub fn get_custom_data_dir() -> Option<String> {
    load_local_config().custom_data_dir
}

/// 设置自定义数据目录
pub fn set_custom_data_dir(dir: Option<String>) {
    let mut local = load_local_config();
    local.custom_data_dir = dir;
    save_local_config(&local);
}

/// 获取数据目录路径
fn get_data_dir_path() -> PathBuf {
    let local = load_local_config();
    if let Some(dir) = &local.custom_data_dir {
        PathBuf::from(dir)
    } else if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.home_dir().join(".atomcode-switch")
    } else {
        PathBuf::from(".atomcode-switch")
    }
}

/// 获取账号信息文件路径（数据目录下的 atomcode-accounts.yaml）
fn get_accounts_file_path() -> PathBuf {
    get_data_dir_path().join("atomcode-accounts.yaml")
}

/// 从旧位置迁移数据（~/.atomcode-switch/config.yaml → atomcode-accounts.yaml）
fn migrate_old_config_if_needed() {
    let old_path = if let Some(base_dirs) = BaseDirs::new() {
        base_dirs.home_dir().join(".atomcode-switch").join("config.yaml")
    } else {
        return;
    };

    let new_path = get_accounts_file_path();
    if old_path.exists() && !new_path.exists() {
        if let Ok(content) = fs::read_to_string(&old_path) {
            if content.trim().is_empty() {
                // 旧文件是空的，直接删除，不迁移
                let _ = fs::remove_file(&old_path);
                return;
            }
            // 尝试解析旧文件内容并写入新位置
            if serde_yaml::from_str::<AppConfig>(&content).is_ok() {
                if fs::write(&new_path, &content).is_ok() {
                    let _ = fs::remove_file(&old_path);
                }
            }
        }
    }
}

/// 加载账号配置
pub fn load_config() -> AppConfig {
    // 首次加载时自动迁移旧数据
    migrate_old_config_if_needed();

    let path = get_accounts_file_path();
    // 确保数据目录存在
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if let Ok(content) = fs::read_to_string(&path) {
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

/// 保存账号配置
pub fn save_config(config: &AppConfig) {
    let path = get_accounts_file_path();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if let Ok(content) = serde_yaml::to_string(config) {
        let _ = fs::write(&path, content);
    }
}