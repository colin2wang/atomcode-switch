use crate::models::AppConfig;
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

/// 获取配置文件路径（存放在用户主目录下 .atomcode-switch/config.yaml）
fn get_config_file_path() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        let config_dir = base_dirs.home_dir().join(".atomcode-switch");
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        config_dir.join("config.yaml")
    } else {
        PathBuf::from("config.yaml")
    }
}

/// 加载配置文件
pub fn load_config() -> AppConfig {
    let path = get_config_file_path();
    if let Ok(content) = fs::read_to_string(path) {
        serde_yaml::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

/// 保存配置文件
pub fn save_config(config: &AppConfig) {
    let path = get_config_file_path();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if let Ok(content) = serde_yaml::to_string(config) {
        let _ = fs::write(path, content);
    }
}
