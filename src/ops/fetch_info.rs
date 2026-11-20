//! 重新获取登录信息：执行 `atomcode login` 捕获输出并填入文本框
//!
//! 打开「更新账号信息」窗口时会自动触发一次，与手动粘贴共享
//! 同一个文本框 +「解析并更新」按钮，本模块只负责获取的流程控制。

use crate::app::{AtomcodeSwitchApp, UpdateMessageKind};
use std::io::Read;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// 获取超时上限：`atomcode login` 会进入交互模式，需主动终止。
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
/// 进程退出后留给输出读取线程的 flush 窗口。
const FLUSH_WINDOW: Duration = Duration::from_millis(500);
/// 轮询进程状态的间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(100);
/// 成功消息展示时长。
const SUCCESS_DISPLAY: Duration = Duration::from_millis(1500);

impl AtomcodeSwitchApp {
    /// 启动自动获取：隐藏窗口执行 `atomcode login`，捕获输出填入文本框。
    /// 已在获取中时直接返回，避免连续点击叠加后台线程与子进程。
    pub fn start_auto_update(&mut self) {
        if self.is_auto_updating {
            return;
        }

        let fetch_gen = self.fetch_generation.wrapping_add(1);
        self.fetch_generation = fetch_gen;
        let (tx, rx) = mpsc::channel();
        self.auto_update_rx = Some(rx);
        self.is_auto_updating = true;

        // 「正在获取」提示只显示在更新窗口内的消息区（气泡）；
        // 不写主窗口状态栏，避免关窗后残留。
        let fetching_msg = self.i18n.t0("status_fetching");
        self.set_update_message(UpdateMessageKind::Info, fetching_msg);

        // 错误文案在主线程先本地化，再 move 进后台线程（I18n 不是 Send）
        let no_output_msg = self.i18n.t0("fetch_no_output");
        let timeout_msg = self.i18n.t0("fetch_timeout");

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

        match cmd.spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                // stdout / stderr 各用一个读取线程，避免单线程顺序读取造成管道背压
                let (out_tx, out_rx) = mpsc::channel::<String>();
                let (err_tx, err_rx) = mpsc::channel::<String>();

                if let Some(mut reader) = stdout {
                    let tx = out_tx.clone();
                    std::thread::spawn(move || {
                        let mut buf = String::new();
                        let _ = reader.read_to_string(&mut buf);
                        let _ = tx.send(buf);
                    });
                }
                if let Some(mut reader) = stderr {
                    std::thread::spawn(move || {
                        let mut buf = String::new();
                        let _ = reader.read_to_string(&mut buf);
                        let _ = err_tx.send(buf);
                    });
                }
                drop(out_tx);

                std::thread::spawn(move || {
                    let deadline = Instant::now() + FETCH_TIMEOUT;
                    let mut early_out = String::new();
                    let mut exited = false;
                    let mut timed_out = false;

                    loop {
                        // 读到关键字段即提前结束等待
                        if let Ok(chunk) = out_rx.try_recv() {
                            early_out.push_str(&chunk);
                        }
                        // 读到关键字段即提前结束等待；关键词覆盖中英文套餐/用量/
                        // 剩余/到期/重置等标记，避免纯英文或含「剩余」的输出要多等。
                        // 注意：stdout 由独立读取线程持续读取，早退后仍会给 flush
                        // 窗口收齐剩余输出，因此不会因早退丢字段。
                        if early_out.contains("套餐")
                            || early_out.contains("Plan:")
                            || early_out.contains("剩余")
                            || early_out.contains("Usage")
                            || early_out.contains("usage")
                            || early_out.contains("到期时间")
                            || early_out.contains("resets")
                            || early_out.contains("重置于")
                        {
                            break;
                        }
                        match child.try_wait() {
                            Ok(Some(_)) => {
                                exited = true;
                                break;
                            }
                            Ok(None) => {}
                            Err(_) => break,
                        }
                        if Instant::now() >= deadline {
                            timed_out = true;
                            break;
                        }
                        std::thread::sleep(POLL_INTERVAL);
                    }

                    // 终止交互进程（若仍在运行）
                    if !exited {
                        let _ = child.kill();
                    }
                    let _ = child.wait();

                    // 始终给读取线程一个 flush 窗口：即使因关键词早退或被超时 kill，
                    // stdout/stderr 读取线程仍可能正在把剩余输出写入通道，flush=0 会截断输出。
                    let flush = FLUSH_WINDOW;
                    let err_buf = err_rx.recv_timeout(flush).unwrap_or_default();
                    let out_buf = out_rx.recv_timeout(flush).unwrap_or_default();
                    early_out.push_str(&out_buf);
                    // 合并 stdout/stderr：任何非空控制台输出都作为「已获取信息」
                    // 返回，以便填入更新窗口输入框供用户查看。
                    let combined = if err_buf.trim().is_empty() {
                        early_out
                    } else {
                        format!("{}\n{}", early_out, err_buf)
                    };
                    let result = if combined.trim().is_empty() {
                        if timed_out {
                            Err(timeout_msg)
                        } else {
                            Err(no_output_msg)
                        }
                    } else {
                        Ok(combined)
                    };
                    let _ = tx.send((fetch_gen, result));
                });
            }
            Err(e) => {
                // spawn 失败直接走 Err，不进入解析流程
                let msg = self.i18n.t1("fetch_spawn_failed", &e.to_string());
                let _ = tx.send((fetch_gen, Err(msg)));
            }
        }
    }

    /// 检查自动获取是否完成，若完成则处理输出。
    /// generation 不匹配的结果一律丢弃，避免关窗/取消后的旧结果仍解析落盘。
    pub fn poll_auto_fetch(&mut self) {
        let rx = match self.auto_update_rx.take() {
            Some(rx) => rx,
            None => return,
        };
        match rx.try_recv() {
            Ok((fetch_gen, result)) => {
                if fetch_gen != self.fetch_generation {
                    // 过期结果：RX 已取出，直接丢弃
                    return;
                }
                self.is_auto_updating = false;
                match result {
                    Err(e) => {
                        // 获取失败：错误信息显示在消息区，保留输入框内容不动
                        self.set_update_message(UpdateMessageKind::Error, e);
                    }
                    Ok(output) => {
                        self.manual_update_text = output;
                        if self.auto_parse_pending {
                            self.auto_parse_pending = false;
                            let text = self.manual_update_text.clone();
                            if text.trim().is_empty() {
                                let msg = self.i18n.t0("status_paste_empty");
                                self.mark_parse_failed(msg);
                            } else if self.config.active_account_id.is_none() {
                                let msg = self.i18n.t0("status_no_active_import");
                                self.mark_parse_failed(msg);
                            } else {
                                match self.parse_login_output_and_update(&text) {
                                    Ok(()) => {
                                        self.import_current_auth();
                                        // 保留获取到的控制台输出，便于在输入框内查看
                                        let msg = self.i18n.t0("status_updated");
                                        self.set_update_message(UpdateMessageKind::Success, msg);
                                        self.schedule_update_window_close(SUCCESS_DISPLAY);
                                    }
                                    Err(e) => {
                                        self.mark_parse_failed(e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.auto_update_rx = Some(rx);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.is_auto_updating = false;
                let msg = self.i18n.t0("status_fetch_failed");
                self.set_update_message(UpdateMessageKind::Error, msg);
            }
        }
    }
}
