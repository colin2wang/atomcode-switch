pub mod theme;
pub mod ui;

use crate::i18n::I18n;
use crate::i18n::lang::Language;
use crate::io::config_io;
use crate::models::AppConfig;

/// 设置对话框的临时状态
pub struct SettingsState {
    pub data_dir: String,
    pub atomcode_dir: String,
    pub auto_switch_enabled: bool,
    pub max_usage_percent: f32,
    /// 切换时过期账号自动更新
    pub auto_update_on_switch_expired: bool,
    /// 浏览目录时确认要从中导入账号的目录；在「确定」时才真正导入
    pub pending_import_data_dir: Option<String>,
}

/// 「更新账号信息」窗口内消息区消息的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMessageKind {
    /// 进行中/普通提示（如「正在获取登录信息...」），无标题
    Info,
    /// 解析成功（绿色）
    Success,
    /// 解析/获取失败（红色，附重试提示）
    Error,
}

/// 「更新账号信息」窗口内消息区展示的一条消息
#[derive(Debug, Clone)]
pub struct UpdateMessage {
    pub kind: UpdateMessageKind,
    pub text: String,
}

/// 主应用结构体
pub struct AtomcodeSwitchApp {
    pub config: AppConfig,
    pub i18n: I18n,
    pub settings_state: Option<SettingsState>,
    pub status_message: String,
    pub show_settings: bool,
    pub delete_confirm_id: Option<String>,
    pub last_auto_check: f64,
    pub show_manual_update: bool,
    pub manual_update_text: String,
    pub is_auto_updating: bool,
    /// 获取结果通道：(generation, 输出或错误)。过期 generation 的结果会被丢弃。
    pub auto_update_rx: Option<std::sync::mpsc::Receiver<(u64, Result<String, String>)>>,
    /// 当前获取代号；关窗/取消时自增，用于作废后台线程的结果
    pub fetch_generation: u64,
    /// 自动获取完成后是否自动解析并更新（切换过期账号时触发）
    pub auto_parse_pending: bool,
    /// 「更新账号信息」窗口消息区展示的解析/获取结果（不论成败）
    pub update_message: Option<UpdateMessage>,
    /// 解析成功后延迟关闭窗口的时刻，让绿色成功消息有机会渲染
    pub close_update_after: Option<std::time::Instant>,
}

impl AtomcodeSwitchApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::app::theme::setup_github_theme(&cc.egui_ctx);
        crate::app::theme::setup_fonts(&cc.egui_ctx);

        let (config, config_load_error) = config_io::load_config();
        let lang = Language::from_str(&config.language);
        let i18n = I18n::load(lang);

        let mut app = Self {
            i18n,
            config,
            settings_state: None,
            status_message: String::new(),
            show_settings: false,
            delete_confirm_id: None,
            last_auto_check: 0.0,
            show_manual_update: false,
            manual_update_text: String::new(),
            is_auto_updating: false,
            auto_update_rx: None,
            fetch_generation: 0,
            auto_parse_pending: false,
            update_message: None,
            close_update_after: None,
        };

        app.status_message = match config_load_error {
            Some(e) => app.i18n.t1("status_config_load_failed", &e),
            None => app.i18n.t0("status_ready"),
        };
        app.sync_active_account_status();
        app
    }

    /// 记录「更新账号信息」的结果消息：显示在窗口内消息区，失败同时写入主窗口状态栏
    ///
    /// 无论成功与否都记录；是否关闭窗口由调用方根据成功与否决定。
    pub fn set_update_message(&mut self, kind: UpdateMessageKind, text: String) {
        if kind == UpdateMessageKind::Error {
            self.status_message = self.i18n.t1("status_update_failed", &text);
            // 任何错误都必须保留窗口供重试：取消可能残留的延迟关窗定时器，
            // 否则下一帧 poll_update_window_close 会关窗并清掉这条错误消息。
            self.close_update_after = None;
            self.auto_parse_pending = false;
        }
        self.update_message = Some(UpdateMessage { kind, text });
    }

    /// 解析失败：在窗口内显示明确的「解析失败」提示，保留窗口与输入框原文供重试。
    pub fn mark_parse_failed(&mut self, detail: String) {
        let text = self.i18n.t1("update_parse_failed", &detail);
        self.close_update_after = None;
        self.auto_parse_pending = false;
        self.set_update_message(UpdateMessageKind::Error, text);
    }

    /// 清除「更新账号信息」窗口内的结果消息
    pub fn clear_update_message(&mut self) {
        self.update_message = None;
    }

    /// 统一关闭更新窗口并清理全部获取状态：自增 generation 使后台结果作废，
    /// 防止关闭后仍解析落盘。
    pub fn close_update_window(&mut self) {
        self.fetch_generation = self.fetch_generation.wrapping_add(1);
        self.show_manual_update = false;
        self.auto_update_rx = None;
        self.is_auto_updating = false;
        self.auto_parse_pending = false;
        self.close_update_after = None;
        self.clear_update_message();
    }

    /// 请求延迟关闭更新窗口（成功消息渲染一小段时间后再关）。
    /// 同时自增 generation，避免后台旧结果再次触发解析。
    pub fn schedule_update_window_close(&mut self, delay: std::time::Duration) {
        self.fetch_generation = self.fetch_generation.wrapping_add(1);
        self.auto_update_rx = None;
        self.is_auto_updating = false;
        self.auto_parse_pending = false;
        self.close_update_after = Some(std::time::Instant::now() + delay);
    }

    /// 仅取消正在进行的获取（保留窗口与文本框内容），使后台结果作废。
    pub fn cancel_auto_fetch(&mut self) {
        self.fetch_generation = self.fetch_generation.wrapping_add(1);
        self.auto_update_rx = None;
        self.is_auto_updating = false;
        self.auto_parse_pending = false;
        self.close_update_after = None;
    }

    /// 为一次新的获取重置窗口状态（不关闭窗口），使上一个后台结果作废。
    pub fn reset_for_new_fetch(&mut self) {
        self.fetch_generation = self.fetch_generation.wrapping_add(1);
        self.auto_update_rx = None;
        self.is_auto_updating = false;
        self.close_update_after = None;
    }

    /// 处理延迟关闭：到点后真正关闭窗口。
    pub fn poll_update_window_close(&mut self) {
        let due = self
            .close_update_after
            .map(|t| std::time::Instant::now() >= t)
            .unwrap_or(false);
        if due {
            self.close_update_window();
        }
    }

    /// 保存配置；失败时在状态栏显示本地化错误并返回 false。
    /// 调用方仅在返回 true 时写入「成功」状态消息，避免错误信息被覆盖。
    pub fn persist_config(&mut self) -> bool {
        match config_io::save_config(&self.config) {
            Ok(()) => true,
            Err(e) => {
                self.status_message = self.i18n.t1("status_save_failed", &e);
                false
            }
        }
    }
}
