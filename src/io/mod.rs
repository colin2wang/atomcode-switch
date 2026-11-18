pub mod atomcode_io;
pub mod config_io;

use std::fs;
use std::path::{Path, PathBuf};

/// 为已存在的文件保留一份 `.bak` 备份（best-effort，备份失败不阻断主写入）。
pub(crate) fn backup_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let mut backup = path.as_os_str().to_os_string();
    backup.push(".bak");
    fs::copy(path, PathBuf::from(backup))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// 原子写入文本文件：先写同目录临时文件，备份原文件，再替换目标。
///
/// 写入失败时原文件保持不变；Windows 下 `fs::rename` 无法覆盖已存在目标，
/// 因此先移除目标再重命名（备份仍保留）。
pub(crate) fn atomic_write_text(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let tmp_path = {
        let mut tmp = path.as_os_str().to_os_string();
        tmp.push(".tmp");
        PathBuf::from(tmp)
    };

    fs::write(&tmp_path, content).map_err(|e| e.to_string())?;

    if let Err(e) = backup_file(path) {
        eprintln!("backup {} failed: {e}", path.display());
    }

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp_path);
            Err(e.to_string())
        }
    }
}
