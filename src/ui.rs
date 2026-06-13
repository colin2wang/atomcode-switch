//! UI 渲染：update() 入口 + 所有面板和对话框

use crate::app::{AtomcodeSwitchApp, SettingsState};
use crate::models::AppConfig;
use crate::{atomcode_io, config_io, theme};
use chrono::Timelike;
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

                        // 语言切换
                        let current_lang = self.i18n.language();
                        let lang_label = match current_lang {
                            crate::lang::Language::ZhCn => "EN",
                            crate::lang::Language::EnUs => "中",
                        };
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new(lang_label).size(13.0))
                                    .rounding(4.0),
                            )
                            .clicked()
                        {
                            let new_lang = match current_lang {
                                crate::lang::Language::ZhCn => crate::lang::Language::EnUs,
                                crate::lang::Language::EnUs => crate::lang::Language::ZhCn,
                            };
                            self.i18n = crate::i18n::I18n::load(new_lang);
                            self.config.language = new_lang.as_str().to_string();
                            config_io::save_config(&self.config);
                            self.status_message = self.i18n.t0("status_ready");
                        }

                        // 设置
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new(self.i18n.t0("toolbar_settings")).size(13.0))
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
                                    egui::RichText::new(self.i18n.t0("toolbar_sync")).size(13.0),
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
                        egui::RichText::new(self.i18n.t1("bottom_accounts", &count.to_string()))
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
                        egui::RichText::new(if active_valid { self.i18n.t0("status_normal") } else { self.i18n.t0("status_error") })
                            .size(12.0)
                            .color(if active_valid { GITHUB_GREEN } else { GITHUB_RED }),
                    );

                    ui.separator();

                    // 导出按钮
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(self.i18n.t0("btn_export")).size(12.0))
                                .rounding(4.0),
                        )
                        .clicked()
                    {
                        self.handle_export();
                    }

                    // 清空登录按钮
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(self.i18n.t0("btn_clear_auth")).size(12.0))
                                .rounding(4.0),
                        )
                        .clicked()
                    {
                        match atomcode_io::clear_auth(&self.config.custom_atomcode_dir) {
                            Ok(_) => {
                                self.config.active_account_id = None;
                                config_io::save_config(&self.config);
                                self.status_message = self.i18n.t0("status_auth_cleared");
                            }
                            Err(e) => {
                                self.status_message = self.i18n.t1("status_clear_failed", &e);
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

            egui::Window::new(self.i18n.t0("delete_confirm_title"))
                .collapsible(false)
                .resizable(false)
                .default_size([320.0, 120.0])
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(self.i18n.t1("delete_confirm_msg", &account_name));
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.t0("btn_cancel")).clicked() {
                            self.delete_confirm_id = None;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(self.i18n.t0("btn_delete"))
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

            egui::Window::new(self.i18n.t0("update_title"))
                .open(&mut open)
                .resizable(true)
                .collapsible(false)
                .default_pos([150.0, 120.0])
                .default_size([520.0, 380.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(self.i18n.t0("update_desc"))
                                .size(12.0)
                                .color(GITHUB_TEXT_SECONDARY),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if self.is_auto_updating {
                                ui.label(
                                    egui::RichText::new(self.i18n.t0("update_fetching"))
                                        .size(13.0)
                                        .color(GITHUB_BLUE)
                                        .strong(),
                                );
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(self.i18n.t0("update_cancel")).size(12.0),
                                        )
                                        .rounding(4.0)
                                        .fill(GITHUB_RED),
                                    )
                                    .clicked()
                                {
                                    self.is_auto_updating = false;
                                    self.auto_update_rx = None;
                                    self.status_message = self.i18n.t0("status_auto_fetch_cancelled");
                                }
                            } else {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(self.i18n.t0("update_auto_fetch"))
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
                                    .hint_text(self.i18n.t0("update_hint"))
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );
                        });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        let has_text = !self.manual_update_text.trim().is_empty();
                        if ui
                            .add_enabled(
                                has_text,
                                egui::Button::new(
                                    egui::RichText::new(self.i18n.t0("update_parse"))
                                        .size(13.0)
                                        .color(egui::Color32::WHITE),
                                )
                                .rounding(4.0)
                                .fill(if has_text { GITHUB_BLUE } else { GITHUB_BORDER }),
                            )
                            .clicked()
                        {
                            let text = std::mem::take(&mut self.manual_update_text);
                            if text.trim().is_empty() {
                                self.status_message = self.i18n.t0("status_paste_empty");
                            } else if self.config.active_account_id.is_none() {
                                self.status_message = self.i18n.t0("status_no_active_import");
                            } else {
                                match self.parse_login_output_and_update(&text) {
                                    Ok(()) => {
                                        self.status_message = self.i18n.t0("status_updated");
                                    }
                                    Err(e) => {
                                        self.status_message = self.i18n.t1("status_update_failed", &e);
                                    }
                                }
                            }
                            self.show_manual_update = false;
                        }

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button(self.i18n.t0("btn_cancel")).clicked() {
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
                        egui::RichText::new(self.i18n.t0("card_in_use"))
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
                                    egui::RichText::new(self.i18n.t0("card_valid"))
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(40, 167, 69)),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(self.i18n.t0("card_invalid"))
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
                                    egui::RichText::new(self.i18n.t0("card_activate"))
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
                                        egui::RichText::new(self.i18n.t0("card_activated"))
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
                                            egui::RichText::new(self.i18n.t0("card_update_info"))
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
                                        egui::RichText::new(self.i18n.t0("card_delete"))
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
                            egui::RichText::new(self.i18n.t1("card_remaining_days", &acc.remaining_days.to_string()))
                                .size(12.0)
                                .color(egui::Color32::from_rgb(108, 117, 125)),
                        );
                    });
                });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.i18n.t0("card_usage_label"))
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
                        egui::RichText::new(self.format_reset_countdown(&acc.reset_time))
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
                        egui::RichText::new(self.i18n.t0("empty_no_accounts"))
                            .size(18.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(self.i18n.t0("empty_hint"))
                            .size(13.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );
                    ui.add_space(16.0);
                    let import_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(self.i18n.t0("empty_import"))
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

        egui::Window::new(self.i18n.t0("settings_title"))
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
                        egui::RichText::new(self.i18n.t0("settings_export_dir")).size(14.0).strong(),
                    );
                    ui.label(
                        egui::RichText::new(self.i18n.t0("settings_export_dir_desc"))
                            .size(12.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.data_dir)
                                .desired_width(300.0)
                                .hint_text(self.i18n.t0("settings_export_dir_hint")),
                        );
                        if ui.button(self.i18n.t0("settings_browse")).clicked() {
                            if let Some(dir) =
                                tinyfiledialogs::select_folder_dialog(&self.i18n.t0("dialog_select_data_dir"), &state.data_dir)
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

                                    let none_label = self.i18n.t0("dialog_none");
                                    let existing_list = if existing_names.is_empty() { none_label.clone() } else { existing_names.join("\n") };
                                    let current_list = if current_names.is_empty() { none_label } else { current_names.join("\n") };
                                    let msg = self.i18n.t4(
                                        "dialog_switch_dir_msg",
                                        &existing_accounts.len().to_string(),
                                        &existing_list,
                                        &self.config.accounts.len().to_string(),
                                        &current_list,
                                    );

                                    let should_import = matches!(
                                        tinyfiledialogs::message_box_yes_no(
                                            &self.i18n.t0("dialog_switch_dir_title"),
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
                                            self.status_message = self.i18n.t1("status_imported_count", &self.config.accounts.len().to_string());
                                        }
                                    } else {
                                        self.status_message = self.i18n.t0("status_kept_current");
                                    }
                                }
                                state.data_dir = dir;
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // Atomcode 目录
                    ui.label(
                        egui::RichText::new(self.i18n.t0("settings_atomcode_dir")).size(14.0).strong(),
                    );
                    ui.label(
                        egui::RichText::new(self.i18n.t0("settings_atomcode_dir_desc"))
                            .size(12.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.atomcode_dir)
                                .desired_width(300.0)
                                .hint_text(self.i18n.t0("settings_atomcode_hint")),
                        );
                        if ui.button(self.i18n.t0("settings_browse")).clicked() {
                            if let Some(path) = tinyfiledialogs::select_folder_dialog(
                                &self.i18n.t0("dialog_select_atomcode_dir"),
                                &state.atomcode_dir,
                            ) {
                                state.atomcode_dir = path;
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // 自动切换规则
                    ui.label(
                        egui::RichText::new(self.i18n.t0("settings_auto_switch")).size(14.0).strong(),
                    );
                    ui.add_space(4.0);
                    ui.checkbox(&mut state.auto_switch_enabled, self.i18n.t0("settings_enable_auto_switch"));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(self.i18n.t0("settings_usage_exceeds"));
                        ui.add(
                            egui::DragValue::new(&mut state.max_usage_percent)
                                .clamp_range(50.0..=100.0)
                                .suffix("%")
                                .speed(0.5),
                        );
                        ui.label(self.i18n.t0("settings_auto_switch_to"));
                    });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(self.i18n.t0("btn_confirm")).clicked() {
                                save = true;
                            }
                            if ui.button(self.i18n.t0("btn_cancel")).clicked() {
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
                self.status_message = self.i18n.t0("status_settings_saved");
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
        if let Some(path_str) = tinyfiledialogs::save_file_dialog(&self.i18n.t0("dialog_export_title"), "config.yaml") {
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

                let none_label = self.i18n.t0("dialog_none");
                let existing_list = if existing_names.is_empty() { none_label.clone() } else { existing_names.join("\n") };
                let new_list = if new_names.is_empty() { none_label } else { new_names.join("\n") };
                let msg = self.i18n.t4(
                    "dialog_overwrite_msg",
                    &existing_accounts.len().to_string(),
                    &existing_list,
                    &self.config.accounts.len().to_string(),
                    &new_list,
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
                    self.status_message = self.i18n.t0("status_exported");
                }
            } else {
                self.status_message = self.i18n.t0("status_kept_original");
            }
        }
    }

    /// 从重置时间实时计算倒计时，支持 YYYY-MM-DD HH:MM 和旧 HH:MM 格式
    fn format_reset_countdown(&self, reset_time: &str) -> String {
        let trimmed = reset_time.trim();
        let now = chrono::Local::now();

        // 尝试解析 YYYY-MM-DD HH:MM 格式
        if trimmed.len() >= 16 && trimmed.as_bytes()[4] == b'-' && trimmed.as_bytes()[7] == b'-' && trimmed.as_bytes()[10] == b' ' && trimmed.as_bytes()[13] == b':' {
            if let Ok(target_dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M") {
                let target = target_dt.and_local_timezone(chrono::Local).unwrap();
                let diff_secs = (target - now).num_seconds();
                if diff_secs < 0 {
                    return self.i18n.t1("card_reset_label", trimmed);
                }
                let diff = diff_secs as u64;
                let minutes = diff / 60;
                let seconds = diff % 60;
                if diff >= 3600 {
                    return self.i18n.t4("card_reset_countdown_h", trimmed, &(diff / 3600).to_string(), &(minutes % 60).to_string(), &seconds.to_string());
                } else {
                    return self.i18n.t3("card_reset_countdown", trimmed, &minutes.to_string(), &seconds.to_string());
                }
            }
        }

        // 兼容旧的 HH:MM 格式
        if trimmed.len() >= 5 && trimmed.as_bytes()[2] == b':' {
            let (h_str, m_str) = trimmed.split_at(2);
            let m_str = &m_str[1..];
            if let Ok(target_h) = h_str.parse::<u32>() {
                if let Ok(target_m) = m_str.parse::<u32>() {
                    let target_secs = target_h * 3600 + target_m * 60;
                    let now_secs = now.hour() * 3600 + now.minute() * 60 + now.second();
                    let diff = if target_secs > now_secs {
                        target_secs - now_secs
                    } else {
                        target_secs + 86400 - now_secs
                    };
                    let minutes = diff / 60;
                    let seconds = diff % 60;
                    if diff >= 3600 {
                        return self.i18n.t4("card_reset_countdown_h", trimmed, &(diff / 3600).to_string(), &(minutes % 60).to_string(), &seconds.to_string());
                    } else {
                        return self.i18n.t3("card_reset_countdown", trimmed, &minutes.to_string(), &seconds.to_string());
                    }
                }
            }
        }

        self.i18n.t1("card_reset_label", reset_time)
    }
}
