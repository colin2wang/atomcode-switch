use crate::models::AuthToml;
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

/// 获取 Atomcode 目录路径
pub fn get_atomcode_dir(custom_dir: &Option<String>) -> PathBuf {
    if let Some(dir) = custom_dir {
        PathBuf::from(dir)
    } else {
        let base_dirs = BaseDirs::new().expect("无法获取系统主目录");
        base_dirs.home_dir().join(".atomcode")
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

/// 写入 auth.toml 到目标系统
pub fn write_auth(custom_dir: &Option<String>, auth: &AuthToml) -> Result<(), String> {
    let dir = get_atomcode_dir(custom_dir);
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    let path = dir.join("auth.toml");
    let content = toml::to_string(auth).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;

    Ok(())
}

/// 清空 auth.toml（删除文件）
pub fn clear_auth(custom_dir: &Option<String>) -> Result<(), String> {
    let path = get_auth_file_path(custom_dir);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
