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
    pub auto_update_rx: Option<std::sync::mpsc::Receiver<String>>,
    /// 自动获取完成后是否自动解析并更新（切换过期账号时触发）
    pub auto_parse_pending: bool,
    /// 「更新账号信息」窗口消息区展示的解析/获取结果（不论成败）
    pub update_message: Option<UpdateMessage>,
}

impl AtomcodeSwitchApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::app::theme::setup_github_theme(&cc.egui_ctx);
        crate::app::theme::setup_fonts(&cc.egui_ctx);

        let config = config_io::load_config();
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
            auto_parse_pending: false,
            update_message: None,
        };

        app.status_message = app.i18n.t0("status_ready");
        app.sync_active_account_status();
        app
    }

    /// 记录「更新账号信息」的结果消息：显示在窗口内消息区，失败同时写入主窗口状态栏
    ///
    /// 无论成功与否都记录；是否关闭窗口由调用方根据成功与否决定。
    pub fn set_update_message(&mut self, kind: UpdateMessageKind, text: String) {
        if kind == UpdateMessageKind::Error {
            self.status_message = self.i18n.t1("status_update_failed", &text);
        }
        self.update_message = Some(UpdateMessage { kind, text });
    }

    /// 清除「更新账号信息」窗口内的结果消息
    pub fn clear_update_message(&mut self) {
        self.update_message = None;
    }
}
