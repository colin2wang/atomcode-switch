use super::{atomic_write_text, backup_file};
use crate::models::AppConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 获取用户主目录：Windows 下优先 USERPROFILE，避免 Git Bash 的 POSIX 风格 HOME
/// 被 `PathBuf::from` 解析成当前盘根下的错误路径。
fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    let value = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME"));
    #[cfg(not(windows))]
    let value = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"));
    value.map(PathBuf::from).ok()
}

/// 本地配置（保存在可执行程序同目录下的 config.yaml）
/// 存储数据目录路径等本地设施配置。只在修改默认值时创建此文件。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LocalConfig {
    /// 自定义数据目录路径（默认 ~/.atomcode-switch）
    pub custom_data_dir: Option<String>,
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
        match serde_yaml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("local config parse failed ({}): {e}", path.display());
                let _ = backup_file(&path);
                LocalConfig::default()
            }
        }
    } else {
        LocalConfig::default()
    }
}

/// 保存本地配置
fn save_local_config(config: &LocalConfig) {
    let path = get_local_config_path();
    let result = if config.custom_data_dir.is_some() {
        match serde_yaml::to_string(config) {
            Ok(content) => atomic_write_text(&path, &content),
            Err(e) => Err(e.to_string()),
        }
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())
    } else {
        Ok(())
    };
    if let Err(e) = result {
        eprintln!("save local config failed ({}): {e}", path.display());
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
    } else if let Some(home) = home_dir() {
        home.join(".atomcode-switch")
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
    let old_path = if let Some(home) = home_dir() {
        home.join(".atomcode-switch").join("config.yaml")
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
            // 尝试解析旧文件内容并写入新位置（原子写）
            if serde_yaml::from_str::<AppConfig>(&content).is_ok() {
                if atomic_write_text(&new_path, &content).is_ok() {
                    let _ = fs::remove_file(&old_path);
                }
            } else {
                eprintln!(
                    "legacy config parse failed ({}), original kept",
                    old_path.display()
                );
                let _ = backup_file(&old_path);
            }
        }
    }
}

/// 加载账号配置，返回 `(配置, 加载错误)`。
///
/// 解析失败时备份原文件并回退到默认配置，避免后续保存静默清空账号数据；
/// 错误信息交由 UI 显示，便于用户发现数据异常。
pub fn load_config() -> (AppConfig, Option<String>) {
    // 首次加载时自动迁移旧数据
    migrate_old_config_if_needed();

    let path = get_accounts_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        match serde_yaml::from_str(&content) {
            Ok(config) => (config, None),
            Err(e) => {
                eprintln!("accounts config parse failed ({}): {e}", path.display());
                // 备份损坏文件，便于用户人工恢复后再决定是否覆盖
                let _ = backup_file(&path);
                (AppConfig::default(), Some(e.to_string()))
            }
        }
    } else {
        (AppConfig::default(), None)
    }
}

/// 保存账号配置。返回写入错误，调用方需显式处理，避免静默丢数据。
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_accounts_file_path();
    let content = serde_yaml::to_string(config).map_err(|e| e.to_string())?;
    atomic_write_text(&path, &content)
}
