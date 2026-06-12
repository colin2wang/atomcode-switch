//! 自动获取登录信息：执行 `atomcode login` 捕获输出并填入文本框
//!
//! 与手动粘贴共享同一个文本框 +「解析并更新」按钮，
//! 本模块只负责「自动获取」的流程控制。

use crate::app::AtomcodeSwitchApp;

impl AtomcodeSwitchApp {
    /// 启动自动获取：隐藏窗口执行 `atomcode login`，捕获输出填入文本框
    pub fn start_auto_update(&mut self) {
        use std::io::Read;
        use std::time::Duration;

        let (tx, rx) = std::sync::mpsc::channel();
        self.auto_update_rx = Some(rx);
        self.is_auto_updating = true;
        self.status_message = "正在获取登录信息...".to_string();

        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("atomcode");
            cmd.args(["login"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .stdin(std::process::Stdio::null());

            // Windows 下隐藏控制台窗口
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                cmd.creation_flags(CREATE_NO_WINDOW);
            }

            let output = match cmd.spawn() {
                Ok(mut child) => {
                    let stdout = child.stdout.take();
                    let stderr = child.stderr.take();

                    let (out_tx, out_rx) = std::sync::mpsc::channel();

                    std::thread::spawn(move || {
                        let mut buf = String::new();
                        if let Some(mut reader) = stdout {
                            let _ = reader.read_to_string(&mut buf);
                        }
                        let mut err_buf = String::new();
                        if let Some(mut reader) = stderr {
                            let _ = reader.read_to_string(&mut err_buf);
                        }
                        let combined = if err_buf.is_empty() {
                            buf
                        } else {
                            format!("{}\n{}", buf, err_buf)
                        };
                        let _ = out_tx.send(combined);
                    });

                    // 等待输出后终止（命令会进入交互模式）
                    std::thread::sleep(Duration::from_secs(3));
                    let _ = child.kill();
                    let _ = child.wait();

                    out_rx
                        .recv_timeout(Duration::from_secs(1))
                        .unwrap_or_else(|_| "（未获取到输出）".to_string())
                }
                Err(e) => {
                    format!("（无法执行 atomcode login: {}）", e)
                }
            };

            let _ = tx.send(output);
        });
    }

    /// 检查自动获取是否完成，若完成则将输出填入文本框
    pub fn poll_auto_fetch(&mut self) {
        if let Some(rx) = self.auto_update_rx.take() {
            match rx.try_recv() {
                Ok(output) => {
                    self.manual_update_text = output;
                    self.is_auto_updating = false;
                    self.status_message = "已获取登录信息，点击「解析并更新」完成更新".to_string();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.auto_update_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_auto_updating = false;
                    self.status_message = "自动获取失败: 进程异常退出".to_string();
                }
            }
        }
    }
}
