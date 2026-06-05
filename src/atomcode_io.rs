use crate::models::AuthToml;
use std::fs;
use std::path::PathBuf;

/// 获取用户主目录
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .ok()
}

/// 获取 Atomcode 目录路径
pub fn get_atomcode_dir(custom_dir: &Option<String>) -> PathBuf {
    if let Some(dir) = custom_dir {
        PathBuf::from(dir)
    } else if let Some(home) = home_dir() {
        home.join(".atomcode")
    } else {
        PathBuf::from(".atomcode")
    }
}

/// 获取 auth.toml 文件路径
pub fn get_auth_file_path(custom_dir: &Option<String>) -> PathBuf {
    get_atomcode_dir(custom_dir).join("auth.toml")
}

/// 读取当前系统的 auth.toml
pub fn read_current_auth(custom_dir: &Option<String>) -> Option<AuthToml> {
    let path = get_auth_file_path(custom_dir);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            return toml::from_str(&content).ok();
        }
    }
    None
}

/// 写入 auth.toml
pub fn write_auth(custom_dir: &Option<String>, auth: &AuthToml) -> Result<(), String> {
    let path = get_auth_file_path(custom_dir);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    match toml::to_string(auth) {
        Ok(content) => fs::write(&path, content).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// 清空 auth.toml（删除文件）
pub fn clear_auth(custom_dir: &Option<String>) -> Result<(), String> {
    let path = get_auth_file_path(custom_dir);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}