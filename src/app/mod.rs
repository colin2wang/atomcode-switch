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
        };

        app.status_message = app.i18n.t0("status_ready");
        app.sync_active_account_status();
        app
    }
}
