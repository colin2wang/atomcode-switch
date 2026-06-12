//! UI 渲染：update() 入口 + 所有面板和对话框

use crate::app::{AtomcodeSwitchApp, SettingsState};
use crate::models::AppConfig;
use crate::{atomcode_io, config_io, theme};
use eframe::egui;
use theme::*;

impl eframe::App for AtomcodeSwitchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_time = ctx.input(|i| i.time);
        self.check_auto_switch(current_time);
        self.poll_auto_fetch();

        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        // ============ 顶部工具栏 ============
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

                    // 右侧按钮组
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(12.0);

                        // 设置
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("设置").size(13.0))
                                    .rounding(4.0),
                            )
                            .clicked()
                        {
                            self.show_settings = !self.show_settings;
                        }

                        // 同步登录信息
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("同步登录信息").size(13.0),
                                )
                                .rounding(4.0),
                            )
                            .clicked()
                        {
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

                    let count = self.config.accounts.len();
                    ui.label(
                        egui::RichText::new(format!("{} 个账户", count))
                            .size(12.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );

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
                            .color(if active_valid { GITHUB_GREEN } else { GITHUB_RED }),
                    );
                    ui.label(
                        egui::RichText::new(if active_valid { "正常" } else { "异常" })
                            .size(12.0)
                            .color(if active_valid { GITHUB_GREEN } else { GITHUB_RED }),
                    );

                    ui.separator();

                    // 导出按钮
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("导出").size(12.0))
                                .rounding(4.0),
                        )
                        .clicked()
                    {
                        self.handle_export();
                    }

                    // 清空登录按钮
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new("清空登录").size(12.0))
                                .rounding(4.0),
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

        // ============ 设置面板 ============
        if self.show_settings {
            self.render_settings_window(ctx);
        }

        // ============ 中央内容区 ============
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().rect_filled(ui.max_rect(), 0.0, GITHUB_BG);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            // 当前使用中的账号指示器
                            self.render_active_indicator(ui);

                            // 账号卡片列表
                            let accounts = self.config.accounts.clone();
                            let active_id = self.config.active_account_id.clone();

                            for acc in &accounts {
                                let is_active = active_id.as_deref() == Some(acc.id.as_str());
                                self.render_account_card(ui, acc, is_active);
                            }

                            // 空状态提示
                            if accounts.is_empty() {
                                self.render_empty_state(ui);
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
                                        egui::RichText::new("删除")
                                            .size(13.0)
                                            .color(egui::Color32::WHITE),
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

        // ============ 更新信息对话框 ============
        if self.show_manual_update {
            let mut open = true;

            egui::Window::new("更新账号信息")
                .open(&mut open)
                .resizable(true)
                .collapsible(false)
                .default_pos([150.0, 120.0])
                .default_size([520.0, 380.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("可将下方文本框清空，点击「自动获取」自动填入：")
                                .size(12.0)
                                .color(GITHUB_TEXT_SECONDARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.is_auto_updating {
                                ui.label(
                                    egui::RichText::new("获取中...")
                                        .size(13.0)
                                        .color(GITHUB_BLUE)
                                        .strong(),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("取消").size(12.0),
                                        )
                                        .rounding(4.0)
                                        .fill(GITHUB_RED),
                                    )
                                    .clicked()
                                {
                                    self.is_auto_updating = false;
                                    self.auto_update_rx = None;
                                    self.status_message = "已取消自动获取".to_string();
                                }
                            } else {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("自动获取")
                                                .size(13.0)
                                                .color(egui::Color32::WHITE),
                                        )
                                        .rounding(4.0)
                                        .fill(GITHUB_BLUE),
                                    )
                                    .clicked()
                                {
                                    self.manual_update_text.clear();
                                    self.start_auto_update();
                                }
                            }
                        });
                    });

                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut self.manual_update_text)
                                    .hint_text("在此粘贴 /login 的输出，或点击上方「自动获取」自动填入...")
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );
                        });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("解析并更新")
                                        .size(13.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .rounding(4.0)
                                .fill(GITHUB_BLUE),
                            )
                            .clicked()
                        {
                            let text = std::mem::take(&mut self.manual_update_text);
                            if text.trim().is_empty() {
                                self.status_message = "粘贴内容为空".to_string();
                            } else if self.config.active_account_id.is_none() {
                                self.status_message = "没有激活的账号，请先导入账号".to_string();
                            } else {
                                match self.parse_login_output_and_update(&text) {
                                    Ok(()) => {
                                        self.status_message =
                                            "账号信息已更新".to_string();
                                    }
                                    Err(e) => {
                                        self.status_message = format!("更新失败: {}", e);
                                    }
                                }
                            }
                            self.show_manual_update = false;
                        }

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("取消").clicked() {
                                    self.show_manual_update = false;
                                }
                            },
                        );
                    });
                });

            if !open {
                self.show_manual_update = false;
            }
        }
    }
}

// ============ 辅助渲染方法 ============

impl AtomcodeSwitchApp {
    /// 渲染激活账号指示器
    fn render_active_indicator(&mut self, ui: &mut egui::Ui) {
        if let Some(active_id) = &self.config.active_account_id {
            if let Some(active_acc) = self.config.accounts.iter().find(|a| &a.id == active_id) {
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
    }

    /// 渲染账号卡片
    fn render_account_card(&mut self, ui: &mut egui::Ui, acc: &crate::models::ManagedAccount, is_active: bool) {
        egui::Frame::default()
            .fill(egui::Color32::WHITE)
            .rounding(8.0)
            .stroke(egui::Stroke::new(
                1.0,
                if is_active { GITHUB_BLUE } else { GITHUB_BORDER },
            ))
            .inner_margin(egui::Margin::symmetric(20.0, 16.0))
            .outer_margin(egui::Margin::same(12.0))
            .show(ui, |ui| {
                // ---- 卡片顶部 ----
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                    let avatar_color = if is_active { GITHUB_BLUE } else { GITHUB_AVATAR_INACTIVE };
                    ui.painter().circle_filled(rect.center(), 18.0, avatar_color);
                    let initial = acc.auth_data.user.name.chars().next().unwrap_or('?');
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        initial.to_string(),
                        egui::FontId::proportional(16.0),
                        egui::Color32::WHITE,
                    );

                    ui.add_space(8.0);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&acc.auth_data.user.name)
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(33, 37, 41))
                                    .strong(),
                            );
                            ui.add_space(4.0);
                            if acc.is_valid {
                                ui.label(
                                    egui::RichText::new("\u{25CF} 正常")
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(40, 167, 69)),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("\u{25CF} 异常")
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(220, 53, 69)),
                                );
                            }
                        });
                        ui.label(
                            egui::RichText::new(&acc.auth_data.user.id)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(108, 117, 125)),
                        );
                    });

                    // 右侧操作区
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !is_active {
                            let switch_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("激活")
                                        .size(13.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .rounding(4.0)
                                .fill(egui::Color32::from_rgb(13, 110, 253)),
                            );
                            if switch_btn.clicked() {
                                self.switch_to_account(&acc.id);
                            }
                        } else {
                            ui.horizontal(|ui| {
                                ui.add_enabled(
                                    false,
                                    egui::Button::new(
                                        egui::RichText::new("已激活")
                                            .size(13.0)
                                            .color(GITHUB_GREEN),
                                    )
                                    .rounding(4.0)
                                    .fill(egui::Color32::from_rgb(220, 255, 220))
                                    .stroke(egui::Stroke::new(1.0, GITHUB_GREEN)),
                                );
                                ui.add_space(6.0);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("更新信息")
                                                .size(12.0)
                                                .color(GITHUB_BLUE),
                                        )
                                        .rounding(4.0)
                                        .fill(egui::Color32::WHITE)
                                        .stroke(egui::Stroke::new(1.0, GITHUB_BORDER)),
                                    )
                                    .clicked()
                                {
                                    self.show_manual_update = true;
                                    self.manual_update_text.clear();
                                }
                            });
                        }

                        if !is_active {
                            ui.add_space(4.0);
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("删除")
                                            .size(13.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .rounding(4.0)
                                    .fill(GITHUB_RED),
                                )
                                .clicked()
                            {
                                self.delete_confirm_id = Some(acc.id.clone());
                            }
                        }
                    });
                });

                ui.add_space(12.0);

                // ---- 卡片中部 ----
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}", acc.plan_name))
                            .size(13.0)
                            .color(GITHUB_TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("剩余 {} 天", acc.remaining_days))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(108, 117, 125)),
                        );
                    });
                });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("当前时间窗口用量")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(108, 117, 125)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{:.0}%", acc.usage_percent))
                                .size(12.0)
                                .color(theme::usage_color(acc.usage_percent))
                                .strong(),
                        );
                    });
                });

                // 进度条
                let progress = acc.usage_percent / 100.0;
                let bar_height = 8.0;
                let available_width = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(available_width, bar_height + 4.0),
                    egui::Sense::hover(),
                );
                let bar_rect = egui::Rect::from_min_max(
                    rect.min + egui::vec2(0.0, 2.0),
                    rect.max - egui::vec2(0.0, 2.0),
                );
                ui.painter().rect_filled(bar_rect, 4.0, GITHUB_PROGRESS_BG);
                let fill_width = bar_rect.width() * progress.min(1.0);
                if fill_width > 0.0 {
                    let fill_rect = egui::Rect::from_min_max(
                        bar_rect.min,
                        egui::pos2(bar_rect.min.x + fill_width, bar_rect.max.y),
                    );
                    ui.painter()
                        .rect_filled(fill_rect, 4.0, theme::progress_bar_color(acc.usage_percent));
                }

                ui.add_space(6.0);

                // ---- 卡片底部 ----
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("重置: {}", acc.reset_time))
                            .size(11.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", acc.last_updated))
                                .size(11.0)
                                .color(egui::Color32::from_rgb(108, 117, 125)),
                        );
                    });
                });
            });
    }

    /// 渲染空状态提示
    fn render_empty_state(&mut self, ui: &mut egui::Ui) {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
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
                    egui::RichText::new("导入当前系统账号")
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

    /// 渲染设置窗口
    fn render_settings_window(&mut self, ctx: &egui::Context) {
        if self.settings_state.is_none() {
            self.settings_state = Some(SettingsState {
                data_dir: config_io::get_custom_data_dir().unwrap_or_default(),
                atomcode_dir: self.config.custom_atomcode_dir.clone().unwrap_or_default(),
                auto_switch_enabled: self.config.auto_switch_rules.enabled,
                max_usage_percent: self.config.auto_switch_rules.max_usage_percent,
            });
        }

        let mut show_settings = true;
        let mut save = false;
        let mut cancel = false;

        egui::Window::new("设置")
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
                        egui::RichText::new("导出目录").size(14.0).strong(),
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
                            if let Some(dir) =
                                tinyfiledialogs::select_folder_dialog("选择数据目录", &state.data_dir)
                            {
                                // inline: 检查目标目录是否已有 atomcode-accounts.yaml
                                let accounts_path = std::path::Path::new(&dir)
                                    .join("atomcode-accounts.yaml");
                                if accounts_path.exists() {
                                    let existing_accounts = std::fs::read_to_string(&accounts_path)
                                        .ok()
                                        .and_then(|c| serde_yaml::from_str::<AppConfig>(&c).ok())
                                        .map(|c| c.accounts)
                                        .unwrap_or_default();

                                    let existing_names: Vec<String> = existing_accounts
                                        .iter()
                                        .map(|a| format!("  - {}（{}）", a.auth_data.user.name, a.auth_data.user.username))
                                        .collect();
                                    let current_names: Vec<String> = self
                                        .config
                                        .accounts
                                        .iter()
                                        .map(|a| format!("  - {}（{}）", a.auth_data.user.name, a.auth_data.user.username))
                                        .collect();

                                    let msg = format!(
                                        "目标目录已有 {} 个账号：\n{}\n\n当前程序有 {} 个账号：\n{}\n\n是否用目标目录的账号替换当前账号？",
                                        existing_accounts.len(),
                                        if existing_names.is_empty() { "  （无）".into() } else { existing_names.join("\n") },
                                        self.config.accounts.len(),
                                        if current_names.is_empty() { "  （无）".into() } else { current_names.join("\n") },
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
                                        if let Some(config) = std::fs::read_to_string(&accounts_path)
                                            .ok()
                                            .and_then(|c| serde_yaml::from_str::<AppConfig>(&c).ok())
                                        {
                                            self.config.accounts = config.accounts;
                                            self.config.active_account_id = config.active_account_id;
                                            self.config.auto_switch_rules = config.auto_switch_rules;
                                            self.status_message = format!("已导入 {} 个账号", self.config.accounts.len());
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
                        egui::RichText::new("Atomcode 目录").size(14.0).strong(),
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
                        egui::RichText::new("自动切换规则").size(14.0).strong(),
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

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("确定").clicked() {
                                save = true;
                            }
                            if ui.button("取消").clicked() {
                                cancel = true;
                            }
                        });
                    });
                }
            });

        if save {
            if let Some(ref state) = self.settings_state {
                config_io::set_custom_data_dir(
                    if state.data_dir.is_empty() {
                        None
                    } else {
                        Some(state.data_dir.clone())
                    },
                );
                self.config.custom_atomcode_dir = if state.atomcode_dir.is_empty() {
                    None
                } else {
                    Some(state.atomcode_dir.clone())
                };
                self.config.auto_switch_rules.enabled = state.auto_switch_enabled;
                self.config.auto_switch_rules.max_usage_percent = state.max_usage_percent;
                config_io::save_config(&self.config);
                self.status_message = "设置已保存".to_string();
            }
            self.settings_state = None;
            self.show_settings = false;
        } else if cancel {
            self.settings_state = None;
            self.show_settings = false;
        } else {
            self.show_settings = show_settings;
            if !show_settings {
                self.settings_state = None;
            }
        }
    }

    // ---- 内部辅助 ----

    /// 处理导出按钮逻辑
    fn handle_export(&mut self) {
        if let Some(path_str) = tinyfiledialogs::save_file_dialog("导出配置", "config.yaml") {
            let path = std::path::Path::new(&path_str);
            let should_overwrite = if path.exists() {
                let existing_accounts = std::fs::read_to_string(path)
                    .ok()
                    .and_then(|c| serde_yaml::from_str::<AppConfig>(&c).ok())
                    .map(|c| c.accounts)
                    .unwrap_or_default();

                let existing_names: Vec<String> = existing_accounts
                    .iter()
                    .map(|a| format!("  - {}（{}）", a.auth_data.user.name, a.auth_data.user.username))
                    .collect();
                let new_names: Vec<String> = self
                    .config
                    .accounts
                    .iter()
                    .map(|a| format!("  - {}（{}）", a.auth_data.user.name, a.auth_data.user.username))
                    .collect();

                let msg = format!(
                    "目标文件已有 {} 个账号：\n{}\n\n即将导出 {} 个账号：\n{}\n\n是否覆盖？",
                    existing_accounts.len(),
                    if existing_names.is_empty() { "  （无）".into() } else { existing_names.join("\n") },
                    self.config.accounts.len(),
                    if new_names.is_empty() { "  （无）".into() } else { new_names.join("\n") },
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
}
