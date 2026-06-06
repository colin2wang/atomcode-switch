use crate::models::{AppConfig, ManagedAccount};
use crate::{atomcode_io, config_io};
use eframe::egui;
use std::time::{SystemTime, UNIX_EPOCH};

// ============ GitHub CSS 颜色常量 ============
const GITHUB_BG: egui::Color32 = egui::Color32::from_rgb(246, 248, 250);     // #f6f8fa -> 稍浅
const _GITHUB_CARD: egui::Color32 = egui::Color32::WHITE;                     // #ffffff
const GITHUB_BORDER: egui::Color32 = egui::Color32::from_rgb(208, 215, 222); // #d0d7de
const GITHUB_TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(31, 35, 40);   // #24292f -> 稍暗
const GITHUB_TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(101, 109, 118); // #6e7681
const _GITHUB_TEXT_TERTIARY: egui::Color32 = GITHUB_TEXT_SECONDARY;
const GITHUB_BLUE: egui::Color32 = egui::Color32::from_rgb(9, 105, 218);     // #0969da
const GITHUB_GREEN: egui::Color32 = egui::Color32::from_rgb(45, 164, 78);    // #2da44e
const GITHUB_RED: egui::Color32 = egui::Color32::from_rgb(207, 34, 46);      // #cf222e
const GITHUB_YELLOW: egui::Color32 = egui::Color32::from_rgb(212, 167, 44);  // #d4a72c
const _GITHUB_BTN_BG: egui::Color32 = egui::Color32::from_rgb(246, 248, 250); // #f6f8fa
const _GITHUB_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(243, 244, 246); // #f3f4f6
const GITHUB_AVATAR_INACTIVE: egui::Color32 = egui::Color32::from_rgb(175, 184, 193); // #afb8c1
const GITHUB_PROGRESS_BG: egui::Color32 = egui::Color32::from_rgb(233, 236, 239);
const _GITHUB_BLUE_HOVER: egui::Color32 = egui::Color32::from_rgb(8, 90, 186);
const _GITHUB_GREEN_HOVER: egui::Color32 = egui::Color32::from_rgb(36, 146, 67);

/// 设置对话框的临时状态
struct SettingsState {
    data_dir: String,
    atomcode_dir: String,
    auto_switch_enabled: bool,
    max_usage_percent: f32,
}

/// 主应用结构体
pub struct AtomcodeSwitchApp {
    config: AppConfig,
    settings_state: Option<SettingsState>,
    status_message: String,
    show_settings: bool,
    delete_confirm_id: Option<String>,
    last_auto_check: f64,
}

impl AtomcodeSwitchApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置 GitHub 主题和中文字体
        Self::setup_github_theme(&cc.egui_ctx);
        Self::setup_fonts(&cc.egui_ctx);

        let mut app = Self {
            config: config_io::load_config(),
            settings_state: None,
            status_message: "就绪".to_string(),
            show_settings: false,
            delete_confirm_id: None,
            last_auto_check: 0.0,
        };

        // 启动时同步当前系统账号状态
        app.sync_active_account_status();
        app
    }

    /// GitHub CSS 风格主题
    fn setup_github_theme(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // GitHub 颜色变量
        let github_bg = egui::Color32::from_rgb(246, 248, 250);
        let github_card = egui::Color32::from_rgb(255, 255, 255);
        let github_border = egui::Color32::from_rgb(208, 215, 222);
        let github_text_primary = egui::Color32::from_rgb(31, 35, 40);
        let _github_text_secondary = egui::Color32::from_rgb(101, 109, 118);
        let github_blue = egui::Color32::from_rgb(9, 105, 218);
        let _github_blue_hover = egui::Color32::from_rgb(8, 90, 186);
        let github_blue_active = egui::Color32::from_rgb(7, 78, 160);
        let _github_green = egui::Color32::from_rgb(45, 164, 78);
        let github_red = egui::Color32::from_rgb(207, 34, 46);
        let github_yellow = egui::Color32::from_rgb(212, 167, 44);
        let github_btn_bg = egui::Color32::from_rgb(246, 248, 250);
        let github_btn_hover = egui::Color32::from_rgb(238, 242, 246);
        let github_btn_active = egui::Color32::from_rgb(220, 226, 232);
        let github_btn_text = egui::Color32::from_rgb(31, 35, 40);
        let github_btn_border = egui::Color32::from_rgb(208, 215, 222);
        let github_input_bg = egui::Color32::from_rgb(255, 255, 255);
        let github_selection = egui::Color32::from_rgba_premultiplied(8, 138, 237, 40);

        let rounding = egui::Rounding::same(6.0);

        // Visuals
        style.visuals = egui::Visuals {
            dark_mode: false,
            window_fill: github_card,
            panel_fill: github_bg,
            faint_bg_color: github_bg,
            extreme_bg_color: github_card,
            code_bg_color: egui::Color32::from_rgb(246, 248, 250),
            warn_fg_color: github_yellow,
            error_fg_color: github_red,
            hyperlink_color: github_blue,
            selection: egui::style::Selection {
                bg_fill: github_selection,
                stroke: egui::Stroke::NONE,
            },
            window_stroke: egui::Stroke::new(1.0, github_border),
            window_rounding: egui::Rounding::same(8.0),
            window_shadow: egui::epaint::Shadow {
                offset: [0.0, 4.0].into(),
                blur: 12.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(24),
            },
            popup_shadow: egui::epaint::Shadow {
                offset: [0.0, 8.0].into(),
                blur: 24.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(32),
            },
            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: github_input_bg,
                    weak_bg_fill: github_bg,
                    bg_stroke: egui::Stroke::new(1.0, github_border),
                    rounding,
                    fg_stroke: egui::Stroke::new(1.0, github_text_primary),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: github_btn_bg,
                    weak_bg_fill: github_btn_bg,
                    bg_stroke: egui::Stroke::new(1.0, github_btn_border),
                    rounding,
                    fg_stroke: egui::Stroke::new(1.0, github_btn_text),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: github_btn_hover,
                    weak_bg_fill: github_btn_hover,
                    bg_stroke: egui::Stroke::new(1.0, github_blue),
                    rounding,
                    fg_stroke: egui::Stroke::new(1.5, github_blue),
                    expansion: 1.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: github_btn_active,
                    weak_bg_fill: github_btn_active,
                    bg_stroke: egui::Stroke::new(1.0, github_blue_active),
                    rounding,
                    fg_stroke: egui::Stroke::new(2.0, github_blue_active),
                    expansion: 1.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: github_btn_hover,
                    weak_bg_fill: github_btn_hover,
                    bg_stroke: egui::Stroke::new(1.0, github_blue),
                    rounding,
                    fg_stroke: egui::Stroke::new(1.5, github_blue),
                    expansion: 0.0,
                },
            },
            slider_trailing_fill: true,
            striped: false,
            ..style.visuals
        };

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 5.0);
        style.spacing.indent = 16.0;
        style.spacing.menu_margin = egui::Margin::symmetric(16.0, 12.0);
        style.spacing.window_margin = egui::Margin::same(8.0);

        ctx.set_style(style);
    }

    /// 设置中文字体支持
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 尝试加载系统中文字体
        let chinese_font_data = Self::load_chinese_font();

        if let Some(font_data) = chinese_font_data {
            // 添加中文字体
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            // 将中文字体插入到字体族的最前面，确保中文优先使用此字体
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese_font".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese_font".to_owned());
        }

        ctx.set_fonts(fonts);
    }

    /// 加载系统中文字体（优先微软雅黑）
    fn load_chinese_font() -> Option<Vec<u8>> {
        // 按优先级尝试加载中文字体
        let font_paths = [
            // Windows 微软雅黑（首选）
            "/usr/share/fonts/truetype/chinese/NotoSansSC[wght].ttf",
            "/usr/share/fonts/truetype/chinese/SarasaMonoSC-Regular.ttf",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/noto-serif-sc/NotoSerifSC-Regular.ttf",
            "/usr/share/fonts/truetype/lxgw-wenkai/LXGWWenKai-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "C:\\Windows\\Fonts\\msyh.ttc",        // Windows 微软雅黑
            "C:\\Windows\\Fonts\\simhei.ttf",       // Windows 黑体
            "/System/Library/Fonts/PingFang.ttc",   // macOS 苹方
        ];

        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                return Some(data);
            }
        }

        // 尝试通过 fc-list 查找中文字体（在 Linux 上）
        if let Ok(output) = std::process::Command::new("fc-match")
            .arg("-f")
            .arg("%{file}")
            .arg("NotoSansSC")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    if let Ok(data) = std::fs::read(&path) {
                        return Some(data);
                    }
                }
            }
        }

        None
    }

    /// 同步当前激活账号的状态
    fn sync_active_account_status(&mut self) {
        if let Some(auth) = atomcode_io::read_current_auth(&self.config.custom_atomcode_dir) {
            let id = auth.user.id.clone();
            if let Some(acc) = self.config.accounts.iter_mut().find(|a| a.id == id) {
                acc.auth_data = auth;
                self.config.active_account_id = Some(id);
            }
        }
    }

    /// 检查是否需要自动切换
    fn check_auto_switch(&mut self, current_time: f64) {
        // 每30秒检查一次，避免频繁检查
        if current_time - self.last_auto_check < 30.0 {
            return;
        }
        self.last_auto_check = current_time;

        if !self.config.auto_switch_rules.enabled {
            return;
        }

        let threshold = self.config.auto_switch_rules.max_usage_percent;

        if let Some(active_id) = &self.config.active_account_id {
            if let Some(active_acc) = self.config.accounts.iter().find(|a| &a.id == active_id) {
                if active_acc.usage_percent >= threshold {
                    // 寻找用量最低的有效账号
                    let mut best_acc: Option<&ManagedAccount> = None;
                    for acc in &self.config.accounts {
                        if acc.is_valid && acc.id != *active_id {
                            if best_acc.is_none()
                                || acc.usage_percent < best_acc.unwrap().usage_percent
                            {
                                best_acc = Some(acc);
                            }
                        }
                    }

                    if let Some(acc_to_switch) = best_acc {
                        let id_to_switch = acc_to_switch.id.clone();
                        let name = acc_to_switch.auth_data.user.name.clone();
                        self.switch_to_account(&id_to_switch);
                        self.status_message = format!("自动切换至账号: {}", name);
                    }
                }
            }
        }
    }

    /// 切换到指定账号
    fn switch_to_account(&mut self, id: &str) {
        if let Some(acc) = self.config.accounts.iter().find(|a| a.id == id) {
            match atomcode_io::write_auth(&self.config.custom_atomcode_dir, &acc.auth_data) {
                Ok(_) => {
                    self.config.active_account_id = Some(id.to_string());
                    config_io::save_config(&self.config);
                    self.status_message = format!("已切换至: {}", acc.auth_data.user.name);
                }
                Err(e) => {
                    self.status_message = format!("切换失败: {}", e);
                }
            }
        }
    }

    /// 导入当前系统账号
    fn import_current_auth(&mut self) {
        if let Some(auth) = atomcode_io::read_current_auth(&self.config.custom_atomcode_dir) {
            let id = auth.user.id.clone();
            let name = auth.user.name.clone();

            if let Some(acc) = self.config.accounts.iter_mut().find(|a| a.id == id) {
                acc.auth_data = auth;
                acc.last_updated = Self::current_time_str();
            } else {
                self.config.accounts.push(ManagedAccount {
                    id: id.clone(),
                    plan_name: "CodingPlan Lite".to_string(),
                    usage_percent: 0.0,
                    reset_time: Self::default_reset_time(),
                    is_valid: true,
                    auth_data: auth,
                    remaining_days: 27,
                    last_updated: Self::current_time_str(),
                });
            }

            self.config.active_account_id = Some(id);
            config_io::save_config(&self.config);
            self.status_message = format!("导入成功: {}", name);
        } else {
            self.status_message = "未找到当前的 auth.toml 文件".to_string();
        }
    }

    /// 删除指定账号
    fn delete_account(&mut self, id: &str) {
        self.config.accounts.retain(|a| a.id != id);
        if self.config.active_account_id.as_deref() == Some(id) {
            self.config.active_account_id = self.config.accounts.first().map(|a| a.id.clone());
            // 如果删除的是当前激活账号，需要切换到其他账号
            let new_active_id = self.config.active_account_id.clone();
            if let Some(new_id) = new_active_id {
                self.switch_to_account(&new_id);
            }
        }
        config_io::save_config(&self.config);
        self.status_message = "账号已删除".to_string();
    }

    /// 获取当前时间字符串 (HH:MM)
    fn current_time_str() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mins = (secs / 60) % 1440;
        format!("{:02}:{:02}", mins / 60, mins % 60)
    }

    /// 生成默认重置时间（当前时间 + 1小时）
    fn default_reset_time() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let now_mins = (secs / 60) % 1440;
        let reset_mins = (now_mins + 60) % 1440;
        format!(
            "{:02}:{:02} (60m 0s)",
            reset_mins / 60,
            reset_mins % 60
        )
    }

    /// 获取用量百分比对应的颜色
    fn usage_color(percent: f32) -> egui::Color32 {
        if percent < 50.0 {
            GITHUB_GREEN // 绿色
        } else if percent < 80.0 {
            GITHUB_YELLOW // 黄色
        } else {
            GITHUB_RED // 红色
        }
    }

    /// 获取进度条填充颜色
    fn progress_bar_color(percent: f32) -> egui::Color32 {
        if percent < 50.0 {
            GITHUB_BLUE // 蓝色
        } else if percent < 80.0 {
            GITHUB_YELLOW // 黄色
        } else {
            GITHUB_RED // 红色
        }
    }
}

impl eframe::App for AtomcodeSwitchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_time = ctx.input(|i| i.time);
        self.check_auto_switch(current_time);

        // 持续刷新UI（用于动画和实时更新）
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // ============ 顶部工具栏（功能按钮） ============
        egui::TopBottomPanel::top("toolbar")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 左侧：状态消息
                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .size(12.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );

                    // 右侧按钮组（右对齐）
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);

                        // ⚙ 设置
                        let settings_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("⚙ 设置").size(13.0),
                            )
                            .frame(false),
                        );
                        if settings_btn.clicked() {
                            self.show_settings = !self.show_settings;
                        }

                        // 🔄 刷新
                        let refresh_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("🔄 刷新").size(13.0),
                            )
                            .frame(false),
                        );
                        if refresh_btn.clicked() {
                            self.sync_active_account_status();
                            self.status_message = "已刷新".to_string();
                        }

                        // 📥 同步登录信息
                        let sync_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("📥 同步登录信息").size(13.0),
                            )
                            .frame(false),
                        );
                        if sync_btn.clicked() {
                            self.import_current_auth();
                        }

                        ui.add_space(12.0);
                    });
                });
            });

        // ============ 底部状态栏 ============
        egui::TopBottomPanel::bottom("bottom_panel")
            .exact_height(36.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);

                    // 账户数量
                    let count = self.config.accounts.len();
                    ui.label(
                        egui::RichText::new(format!("{} 个账户", count))
                            .size(12.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );

                    // 状态指示
                    let active_valid = self
                        .config
                        .active_account_id
                        .as_ref()
                        .and_then(|id| self.config.accounts.iter().find(|a| &a.id == id))
                        .map(|a| a.is_valid)
                        .unwrap_or(false);

                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("\u{25CF}")
                            .size(10.0)
                            .color(if active_valid {
                                GITHUB_GREEN
                            } else {
                                GITHUB_RED
                            }),
                    );
                    ui.label(
                        egui::RichText::new(if active_valid { "正常" } else { "异常" })
                            .size(12.0)
                            .color(if active_valid {
                                GITHUB_GREEN
                            } else {
                                GITHUB_RED
                            }),
                    );

                    ui.separator();

                    // 导出按钮
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("\u{2B06} 导出").size(12.0),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        if let Some(path_str) = tinyfiledialogs::save_file_dialog(
                            "导出配置",
                            "config.yaml",
                        ) {
                            let path = std::path::Path::new(&path_str);
                            let should_overwrite = if path.exists() {
                                // 读取现有文件中的账号
                                let existing_accounts = std::fs::read_to_string(path)
                                    .ok()
                                    .and_then(|c| serde_yaml::from_str::<AppConfig>(&c).ok())
                                    .map(|c| c.accounts)
                                    .unwrap_or_default();

                                let existing_names: Vec<String> = existing_accounts
                                    .iter()
                                    .map(|a| {
                                        format!(
                                            "  - {}（{}）",
                                            a.auth_data.user.name, a.auth_data.user.username
                                        )
                                    })
                                    .collect();
                                let new_names: Vec<String> = self
                                    .config
                                    .accounts
                                    .iter()
                                    .map(|a| {
                                        format!(
                                            "  - {}（{}）",
                                            a.auth_data.user.name, a.auth_data.user.username
                                        )
                                    })
                                    .collect();

                                let msg = format!(
                                    "目标文件已有 {} 个账号：\n{}\n\n即将导出 {} 个账号：\n{}\n\n是否覆盖？",
                                    existing_accounts.len(),
                                    if existing_names.is_empty() {
                                        "  （无）".to_string()
                                    } else {
                                        existing_names.join("\n")
                                    },
                                    self.config.accounts.len(),
                                    if new_names.is_empty() {
                                        "  （无）".to_string()
                                    } else {
                                        new_names.join("\n")
                                    },
                                );

                                matches!(
                                    tinyfiledialogs::message_box_yes_no(
                                        "导出冲突",
                                        &msg,
                                        tinyfiledialogs::MessageBoxIcon::Question,
                                        tinyfiledialogs::YesNo::No,
                                    ),
                                    tinyfiledialogs::YesNo::Yes
                                )
                            } else {
                                true
                            };

                            if should_overwrite {
                                if let Ok(content) = serde_yaml::to_string(&self.config) {
                                    let _ = std::fs::write(path, content);
                                    self.status_message = "导出成功".to_string();
                                }
                            } else {
                                self.status_message = "已保留原文件".to_string();
                            }
                        }
                    }

                    // 清空登录按钮
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("\u{1F504} 清空登录").size(12.0),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        match atomcode_io::clear_auth(&self.config.custom_atomcode_dir) {
                            Ok(_) => {
                                self.config.active_account_id = None;
                                config_io::save_config(&self.config);
                                self.status_message = "已清空当前登录".to_string();
                            }
                            Err(e) => {
                                self.status_message = format!("清空失败: {}", e);
                            }
                        }
                    }

                    // 右侧状态消息
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(&self.status_message)
                                .size(12.0)
                                .color(GITHUB_TEXT_SECONDARY),
                        );
                    });
                });

            });

        // ============ 设置面板（浮动窗口） ============
        if self.show_settings {
            // 初始化设置状态（首次打开时）
            if self.settings_state.is_none() {
                self.settings_state = Some(SettingsState {
                    data_dir: config_io::get_custom_data_dir().unwrap_or_default(),
                    atomcode_dir: self.config.custom_atomcode_dir.clone().unwrap_or_default(),
                    auto_switch_enabled: self.config.auto_switch_rules.enabled,
                    max_usage_percent: self.config.auto_switch_rules.max_usage_percent,
                });
            }

            let mut show_settings = true;
            let mut save_settings = false;
            let mut cancel_settings = false;

            egui::Window::new("\u{2699} 设置")
                .open(&mut show_settings)
                .resizable(false)
                .collapsible(false)
                .default_pos([200.0, 150.0])
                .default_size([420.0, 280.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);

                    if let Some(ref mut state) = self.settings_state {
                        // 导出目录
                        ui.label(
                            egui::RichText::new("导出目录")
                                .size(14.0)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("自定义账号数据存储目录（默认为 ~/.atomcode-switch）")
                                .size(12.0)
                                .color(GITHUB_TEXT_SECONDARY),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut state.data_dir)
                                    .desired_width(300.0)
                                    .hint_text("默认: ~/.atomcode-switch"),
                            );
                            if ui.button("浏览…").clicked() {
                                if let Some(dir) = tinyfiledialogs::select_folder_dialog(
                                    "选择数据目录",
                                    &state.data_dir,
                                ) {
                                    // 检查目标目录是否已有 atomcode-accounts.yaml
                                    let accounts_path = std::path::Path::new(&dir)
                                        .join("atomcode-accounts.yaml");
                                    if accounts_path.exists() {
                                        // 读取现有文件中的账号
                                        let existing_accounts = std::fs::read_to_string(&accounts_path)
                                            .ok()
                                            .and_then(|c| serde_yaml::from_str::<AppConfig>(&c).ok())
                                            .map(|c| c.accounts)
                                            .unwrap_or_default();

                                        let existing_names: Vec<String> = existing_accounts
                                            .iter()
                                            .map(|a| {
                                                format!(
                                                    "  - {}（{}）",
                                                    a.auth_data.user.name, a.auth_data.user.username
                                                )
                                            })
                                            .collect();
                                        let current_names: Vec<String> = self
                                            .config
                                            .accounts
                                            .iter()
                                            .map(|a| {
                                                format!(
                                                    "  - {}（{}）",
                                                    a.auth_data.user.name, a.auth_data.user.username
                                                )
                                            })
                                            .collect();

                                        let msg = format!(
                                            "目标目录已有 {} 个账号：\n{}\n\n当前程序有 {} 个账号：\n{}\n\n是否用目标目录的账号替换当前账号？",
                                            existing_accounts.len(),
                                            if existing_names.is_empty() {
                                                "  （无）".to_string()
                                            } else {
                                                existing_names.join("\n")
                                            },
                                            self.config.accounts.len(),
                                            if current_names.is_empty() {
                                                "  （无）".to_string()
                                            } else {
                                                current_names.join("\n")
                                            },
                                        );

                                        let should_import = matches!(
                                            tinyfiledialogs::message_box_yes_no(
                                                "切换导出目录",
                                                &msg,
                                                tinyfiledialogs::MessageBoxIcon::Question,
                                                tinyfiledialogs::YesNo::No,
                                            ),
                                            tinyfiledialogs::YesNo::Yes
                                        );

                                        if should_import {
                                            // 读取文件内容导入账号
                                            if let Some(config) = std::fs::read_to_string(&accounts_path)
                                                .ok()
                                                .and_then(|c| serde_yaml::from_str::<AppConfig>(&c).ok())
                                            {
                                                self.config.accounts = config.accounts;
                                                self.config.active_account_id = config.active_account_id;
                                                self.config.auto_switch_rules = config.auto_switch_rules;
                                                self.status_message = format!(
                                                    "已导入 {} 个账号",
                                                    self.config.accounts.len()
                                                );
                                            }
                                        } else {
                                            self.status_message = "已保留当前账号".to_string();
                                        }
                                    }
                                    state.data_dir = dir;
                                }
                            }
                        });

                        ui.add_space(12.0);

                        // Atomcode 目录
                        ui.label(
                            egui::RichText::new("Atomcode 目录")
                                .size(14.0)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("自定义 auth.toml 保存目录（默认为 ~/.atomcode）")
                                .size(12.0)
                                .color(GITHUB_TEXT_SECONDARY),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut state.atomcode_dir)
                                    .desired_width(300.0)
                                    .hint_text("默认: ~/.atomcode"),
                            );
                            if ui.button("浏览…").clicked() {
                                if let Some(path) = tinyfiledialogs::select_folder_dialog(
                                    "选择 Atomcode 目录",
                                    &state.atomcode_dir,
                                ) {
                                    state.atomcode_dir = path;
                                }
                            }
                        });

                        ui.add_space(12.0);

                        // 自动切换规则
                        ui.label(
                            egui::RichText::new("自动切换规则")
                                .size(14.0)
                                .strong(),
                        );
                        ui.add_space(4.0);
                        ui.checkbox(&mut state.auto_switch_enabled, "启用自动切换");
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("用量超过");
                            ui.add(
                                egui::DragValue::new(&mut state.max_usage_percent)
                                    .clamp_range(50.0..=100.0)
                                    .suffix("%")
                                    .speed(0.5),
                            );
                            ui.label("时自动切换至用量最低的账号");
                        });

                        ui.add_space(16.0);

                        // 底部按钮
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("确定").clicked() {
                                    save_settings = true;
                                }
                                if ui.button("取消").clicked() {
                                    cancel_settings = true;
                                }
                            });
                        });
                    }
                });

            if save_settings {
                if let Some(ref state) = self.settings_state {
                    // 保存导出目录
                    config_io::set_custom_data_dir(
                        if state.data_dir.is_empty() { None } else { Some(state.data_dir.clone()) }
                    );
                    // 保存 Atomcode 目录
                    self.config.custom_atomcode_dir =
                        if state.atomcode_dir.is_empty() { None } else { Some(state.atomcode_dir.clone()) };
                    // 保存自动切换规则
                    self.config.auto_switch_rules.enabled = state.auto_switch_enabled;
                    self.config.auto_switch_rules.max_usage_percent = state.max_usage_percent;
                    // 保存配置
                    config_io::save_config(&self.config);
                    self.status_message = "设置已保存".to_string();
                }
                self.settings_state = None;
                self.show_settings = false;
            } else if cancel_settings {
                self.settings_state = None;
                self.show_settings = false;
            } else {
                self.show_settings = show_settings;
                if !show_settings {
                    self.settings_state = None;
                }
            }
        }

        // ============ 中央内容区 ============
        egui::CentralPanel::default().show(ctx, |ui| {
            // 设置背景色
            let bg_color = GITHUB_BG;
            ui.painter().rect_filled(ui.max_rect(), 0.0, bg_color);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            // 当前使用中的账号指示器
                            if let Some(active_id) = &self.config.active_account_id {
                                if let Some(active_acc) =
                                    self.config.accounts.iter().find(|a| &a.id == active_id)
                                {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("\u{25CF}")
                                                .size(14.0)
                                                .color(GITHUB_GREEN),
                                        );
                                        ui.label(
                                            egui::RichText::new(&active_acc.auth_data.user.name)
                                                .size(15.0)
                                                .color(GITHUB_TEXT_PRIMARY)
                                                .strong(),
                                        );
                                        ui.label(
                                            egui::RichText::new("\u{00B7} 当前使用中")
                                                .size(13.0)
                                                .color(GITHUB_TEXT_SECONDARY),
                                        );
                                    });
                                    ui.add_space(8.0);
                                }
                            }

                            // 账号卡片列表
                            let accounts = self.config.accounts.clone();
                            let active_id = self.config.active_account_id.clone();

                            for acc in &accounts {
                                let is_active = active_id.as_deref() == Some(acc.id.as_str());

                                // 卡片容器
                                egui::Frame::default()
                                    .fill(egui::Color32::WHITE)
                                    .rounding(8.0)
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        if is_active {
                                            GITHUB_BLUE
                                        } else {
                                            GITHUB_BORDER
                                        },
                                    ))
                                    .inner_margin(egui::Margin::symmetric(20.0, 16.0))
                                    .outer_margin(egui::Margin::same(12.0))
                                    .show(ui, |ui| {
                                        // ---- 卡片顶部：头像 + 名称 + 状态 ----
                                        ui.horizontal(|ui| {
                                            // 头像圆形
                                            let (rect, _response) = ui.allocate_exact_size(
                                                egui::vec2(36.0, 36.0),
                                                egui::Sense::hover(),
                                            );
                                            let avatar_color = if is_active {
                                                GITHUB_BLUE
                                            } else {
                                                GITHUB_AVATAR_INACTIVE
                                            };
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                18.0,
                                                avatar_color,
                                            );
                                            // 头像首字母
                                            let initial = acc
                                                .auth_data
                                                .user
                                                .name
                                                .chars()
                                                .next()
                                                .unwrap_or('?');
                                            ui.painter().text(
                                                rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                initial.to_string(),
                                                egui::FontId::proportional(16.0),
                                                egui::Color32::WHITE,
                                            );

                                            ui.add_space(8.0);

                                            // 名称和ID
                                            ui.vertical(|ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(
                                                            &acc.auth_data.user.name,
                                                        )
                                                        .size(15.0)
                                                        .color(egui::Color32::from_rgb(
                                                            33, 37, 41,
                                                        ))
                                                        .strong(),
                                                    );
                                                    ui.add_space(4.0);
                                                    // 状态标签
                                                    if acc.is_valid {
                                                        ui.label(
                                                            egui::RichText::new("\u{25CF} 正常")
                                                                .size(11.0)
                                                                .color(egui::Color32::from_rgb(
                                                                    40, 167, 69,
                                                                )),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            egui::RichText::new("\u{25CF} 异常")
                                                                .size(11.0)
                                                                .color(egui::Color32::from_rgb(
                                                                    220, 53, 69,
                                                                )),
                                                        );
                                                    }
                                                });
                                                ui.label(
                                                    egui::RichText::new(&acc.auth_data.user.id)
                                                        .size(11.0)
                                                        .color(egui::Color32::from_rgb(
                                                            108, 117, 125,
                                                        )),
                                                );
                                            });

                                            // 右侧操作区
                                            ui.with_layout(
                                                egui::Layout::right_to_left(
                                                    egui::Align::Center,
                                                ),
                                                |ui| {
                                                    if !is_active {
                                                        // 切换按钮
                                                        let switch_btn = ui.add(
                                                            egui::Button::new(
                                                                egui::RichText::new(
                                                                    "切换至此账号",
                                                                )
                                                                .size(13.0)
                                                                .color(egui::Color32::WHITE),
                                                            )
                                                            .rounding(4.0)
                                                            .fill(egui::Color32::from_rgb(
                                                                13, 110, 253,
                                                            )),
                                                        );
                                                        if switch_btn.clicked() {
                                                            self.switch_to_account(&acc.id);
                                                        }
                                                    } else {
                                                        // 已激活标签
                                                        ui.label(
                                                            egui::RichText::new("\u{2713} 已激活")
                                                                .size(13.0)
                                                                .color(egui::Color32::from_rgb(
                                                                    13, 110, 253,
                                                                ))
                                                                .strong(),
                                                        );
                                                    }

                                                    // 删除按钮
                                                    if !is_active {
                                                        ui.add_space(4.0);
                                                        let del_btn = ui.add(
                                                            egui::Button::new(
                                                                egui::RichText::new("\u{1F5D1}")
                                                                    .size(14.0),
                                                            )
                                                            .rounding(4.0)
                                                            .frame(false),
                                                        );
                                                        if del_btn.clicked() {
                                                            self.delete_confirm_id =
                                                                Some(acc.id.clone());
                                                        }
                                                    }
                                                },
                                            );
                                        });

                                        ui.add_space(12.0);

                                        // ---- 卡片中部：Plan 信息 + 进度条 ----
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "\u{1F525} {}",
                                                    acc.plan_name
                                                ))
                                                .size(13.0)
                                                .color(GITHUB_TEXT_PRIMARY)
                                                .strong(),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(
                                                    egui::Align::Center,
                                                ),
                                                |ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "剩余 {} 天",
                                                            acc.remaining_days
                                                        ))
                                                        .size(12.0)
                                                        .color(egui::Color32::from_rgb(
                                                            108, 117, 125,
                                                        )),
                                                    );
                                                },
                                            );
                                        });

                                        ui.add_space(6.0);

                                        // 用量标签
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new("当前时间窗口用量")
                                                    .size(12.0)
                                                    .color(egui::Color32::from_rgb(
                                                        108, 117, 125,
                                                    )),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(
                                                    egui::Align::Center,
                                                ),
                                                |ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "{:.0}%",
                                                            acc.usage_percent
                                                        ))
                                                        .size(12.0)
                                                        .color(Self::usage_color(
                                                            acc.usage_percent,
                                                        ))
                                                        .strong(),
                                                    );
                                                },
                                            );
                                        });

                                        // 自定义进度条
                                        let progress = acc.usage_percent / 100.0;
                                        let bar_height = 8.0;
                                        let available_width = ui.available_width();
                                        let (rect, _response) = ui.allocate_exact_size(
                                            egui::vec2(available_width, bar_height + 4.0),
                                            egui::Sense::hover(),
                                        );
                                        let bar_rect = egui::Rect::from_min_max(
                                            rect.min + egui::vec2(0.0, 2.0),
                                            rect.max - egui::vec2(0.0, 2.0),
                                        );

                                        // 背景条
                                        ui.painter().rect_filled(
                                            bar_rect,
                                            4.0,
                                            GITHUB_PROGRESS_BG,
                                        );

                                        // 填充条
                                        let fill_width = bar_rect.width() * progress.min(1.0);
                                        if fill_width > 0.0 {
                                            let fill_rect = egui::Rect::from_min_max(
                                                bar_rect.min,
                                                egui::pos2(
                                                    bar_rect.min.x + fill_width,
                                                    bar_rect.max.y,
                                                ),
                                            );
                                            ui.painter().rect_filled(
                                                fill_rect,
                                                4.0,
                                                Self::progress_bar_color(acc.usage_percent),
                                            );
                                        }

                                        ui.add_space(6.0);

                                        // ---- 卡片底部：重置时间 + 更新时间 ----
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "重置: {}",
                                                    acc.reset_time
                                                ))
                                                .size(11.0)
                                                .color(GITHUB_TEXT_SECONDARY),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(
                                                    egui::Align::Center,
                                                ),
                                                |ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "\u{1F552} {}",
                                                            acc.last_updated
                                                        ))
                                                        .size(11.0)
                                                        .color(egui::Color32::from_rgb(
                                                            108, 117, 125,
                                                        )),
                                                    );
                                                },
                                            );
                                        });
                                    });
                            }

                            // 空状态提示
                            if accounts.is_empty() {
                                ui.add_space(40.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("\u{1F4E5}")
                                            .size(48.0),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new("暂无账号")
                                            .size(18.0)
                                            .color(GITHUB_TEXT_SECONDARY),
                                    );
                                    ui.add_space(8.0);
                                    ui.label(
                                        egui::RichText::new("点击顶部 \"导入\" 按钮添加当前系统账号")
                                            .size(13.0)
                                            .color(GITHUB_TEXT_SECONDARY),
                                    );
                                    ui.add_space(16.0);
                                    let import_btn = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("\u{1F4E5} 导入当前系统账号")
                                                .size(14.0)
                                                .color(egui::Color32::WHITE),
                                        )
                                        .rounding(6.0)
                                        .fill(GITHUB_BLUE),
                                    );
                                    if import_btn.clicked() {
                                        self.import_current_auth();
                                    }
                                });
                            }
                        });
                    });
                });
        });

        // ============ 删除确认对话框 ============
        if self.delete_confirm_id.is_some() {
            let confirm_id = self.delete_confirm_id.clone();
            let account_name = confirm_id
                .as_ref()
                .and_then(|id| self.config.accounts.iter().find(|a| &a.id == id))
                .map(|a| a.auth_data.user.name.clone())
                .unwrap_or_default();

            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .default_size([320.0, 120.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(format!(
                        "确定要删除账号 \"{}\" 吗？此操作不可撤销。",
                        account_name
                    ));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            self.delete_confirm_id = None;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("删除").size(13.0).color(egui::Color32::WHITE),
                                    )
                                    .fill(GITHUB_RED),
                                )
                                .clicked()
                            {
                                if let Some(id) = self.delete_confirm_id.clone() {
                                    self.delete_account(&id);
                                }
                                self.delete_confirm_id = None;
                            }
                        });
                    });
                });
        }
    }
}