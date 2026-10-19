//! 重新获取登录信息：执行 `atomcode login` 捕获输出并填入文本框
//!
//! 打开「更新账号信息」窗口时会自动触发一次，与手动粘贴共享
//! 同一个文本框 +「解析并更新」按钮，本模块只负责获取的流程控制。

use crate::app::AtomcodeSwitchApp;

impl AtomcodeSwitchApp {
    /// 启动自动获取：隐藏窗口执行 `atomcode login`，捕获输出填入文本框
    pub fn start_auto_update(&mut self) {
        use std::io::Read;
        use std::time::Duration;

        let (tx, rx) = std::sync::mpsc::channel();
        self.auto_update_rx = Some(rx);
        self.is_auto_updating = true;
        let msg = self.i18n.t0("status_fetching");
        self.status_message = msg.clone();
        self.set_update_message(crate::app::UpdateMessageKind::Info, msg);

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
                    if self.auto_parse_pending {
                        // 切换过期账号触发的自动获取：直接解析并更新
                        self.auto_parse_pending = false;
                        let text = self.manual_update_text.clone();
                        if text.trim().is_empty() {
                            let msg = self.i18n.t0("status_paste_empty");
                            self.set_update_message(crate::app::UpdateMessageKind::Error, msg);
                        } else if self.config.active_account_id.is_none() {
                            let msg = self.i18n.t0("status_no_active_import");
                            self.set_update_message(crate::app::UpdateMessageKind::Error, msg);
                        } else {
                            match self.parse_login_output_and_update(&text) {
                                Ok(()) => {
                                    self.import_current_auth();
                                    self.manual_update_text.clear();
                                    // 解析成功：记录结果消息后关闭窗口
                                    let msg = self.status_message.clone();
                                    self.set_update_message(crate::app::UpdateMessageKind::Success, msg);
                                    self.show_manual_update = false;
                                }
                                Err(e) => {
                                    // 解析失败：保留窗口与原始内容，在窗口内显示失败原因
                                    self.set_update_message(crate::app::UpdateMessageKind::Error, e);
                                }
                            }
                        }
                    } else {
                        let msg = self.i18n.t0("status_fetched");
                        self.status_message = msg.clone();
                        self.set_update_message(crate::app::UpdateMessageKind::Info, msg);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.auto_update_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.is_auto_updating = false;
                    let msg = self.i18n.t0("status_fetch_failed");
                    self.set_update_message(crate::app::UpdateMessageKind::Error, msg);
                }
            }
        }
    }
}
