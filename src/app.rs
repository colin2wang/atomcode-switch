//! 主应用定义 — struct、new() 构造器
//!
//! 功能代码拆分到同级模块：
//! - [`theme`] — 颜色常量、字体、样式
//! - [`account_ops`] — 账号 CRUD、自动切换、/login 解析
//! - [`ui`] — 所有 UI 渲染（update、面板、对话框）

use crate::{config_io, models::AppConfig};

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
        crate::theme::setup_github_theme(&cc.egui_ctx);
        crate::theme::setup_fonts(&cc.egui_ctx);

        let mut app = Self {
            config: config_io::load_config(),
            settings_state: None,
            status_message: "就绪".to_string(),
            show_settings: false,
            delete_confirm_id: None,
            last_auto_check: 0.0,
            show_manual_update: false,
            manual_update_text: String::new(),
            is_auto_updating: false,
            auto_update_rx: None,
        };

        app.sync_active_account_status();
        app
    }
}
