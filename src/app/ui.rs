//! UI 渲染：update() 入口 + 所有面板和对话框

use crate::app::theme;
use crate::app::{AtomcodeSwitchApp, SettingsState};
use crate::io::{atomic_write_text, atomcode_io, config_io};
use crate::models::AppConfig;
use crate::ops::time_utils::parse_reset_datetime;
use eframe::egui;
use theme::*;

/// 「即将过期」阈值：重置时间在 1 小时内（expiry 判断与倒计时共用）
const EXPIRING_SOON_SECS: u64 = 3600;

/// 卡片渲染期间收集的延迟动作：渲染阶段只借用账号引用，循环结束后统一执行
enum CardAction {
    Switch(String),
    Delete(String),
    OpenUpdate,
}

/// 账号卡片的视觉状态，统一驱动徽标、卡片底色/边框与进度条的语义配色。
/// 与"是否激活"正交：未激活是常态，不应被误判为过期。
#[derive(Clone, Copy, PartialEq, Eq)]
enum AccountVisualState {
    Invalid,
    Expired,
    Expiring,
    Normal,
}

impl AccountVisualState {
    /// 状态方块（徽标）配色：返回 (前景/描边色, 底色)。
    /// 按 code plan 时间窗口有效性分级，见 theme.rs 的 STATUS_* 常量。
    fn badge_colors(self) -> (egui::Color32, egui::Color32) {
        match self {
            Self::Normal => (STATUS_OK_FG, STATUS_OK_BG),
            Self::Expiring => (STATUS_EXPIRING_FG, STATUS_EXPIRING_BG),
            Self::Expired => (STATUS_EXPIRED_FG, STATUS_EXPIRED_BG),
            Self::Invalid => (STATUS_INVALID_FG, STATUS_INVALID_BG),
        }
    }

    /// 各状态对应的 i18n key
    fn badge_key(self) -> &'static str {
        match self {
            Self::Normal => "card_valid",
            Self::Expiring => "card_expiring",
            Self::Expired => "card_expired",
            Self::Invalid => "card_invalid",
        }
    }

    /// 卡片底色与边框色：统一白底，按主题给边框上色。
    /// 选中的激活账号用红色边框（覆盖状态色）；未选中账号按状态着色：
    /// 正常=绿、即将过期=蓝、已过期=黄、已失效=红。
    fn card_colors(self, is_active: bool) -> (egui::Color32, egui::Color32) {
        let stroke = if is_active {
            GITHUB_RED
        } else {
            match self {
                Self::Normal => STATUS_OK_FG,
                Self::Expiring => STATUS_EXPIRING_FG,
                Self::Expired => STATUS_EXPIRED_FG,
                Self::Invalid => STATUS_INVALID_FG,
            }
        };
        (egui::Color32::WHITE, stroke)
    }

    /// 进度条填充色：异常/过期卡片改用语义色，正常卡片仍按用量百分比着色。
    fn progress_color(self, usage_percent: f32) -> egui::Color32 {
        match self {
            Self::Invalid => STATUS_INVALID_FG,
            Self::Expired => STATUS_EXPIRED_FG,
            Self::Expiring => STATUS_EXPIRING_FG,
            Self::Normal => theme::progress_bar_color(usage_percent),
        }
    }
}

/// 状态方块徽标：底色矩形 + 同色描边 + 文字，直接用 painter 绘制，
/// 避免禁用态按钮样式覆盖自定义填充色
fn status_badge(ui: &mut egui::Ui, text: impl Into<String>, fg: egui::Color32, bg: egui::Color32) {
    let font = egui::FontId::proportional(11.0);
    let galley = ui.painter().layout_no_wrap(text.into(), font, fg);
    let size = galley.size() + egui::vec2(12.0, 8.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, bg);
    painter.rect(
        rect,
        4.0,
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(1.0_f32, fg),
    );
    painter.galley(
        egui::pos2(rect.left() + 6.0, rect.center().y - galley.size().y / 2.0),
        galley,
        fg,
    );
}

impl eframe::App for AtomcodeSwitchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_time = ctx.input(|i| i.time);
        self.check_auto_switch(current_time);
        self.poll_auto_fetch();
        self.poll_update_window_close();

        // 关闭倒计时期间提高重绘频率，让剩余秒数更准确；平时保持 1 秒
        let repaint_after = if self.close_update_after.is_some() {
            std::time::Duration::from_millis(250)
        } else {
            std::time::Duration::from_secs(1)
        };
        ctx.request_repaint_after(repaint_after);

        // ============ 顶部工具栏 ============
        egui::TopBottomPanel::top("toolbar")
            .exact_height(44.0)
            .show(ctx, |ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height()),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // 左侧：状态消息
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(&self.status_message)
                                .size(13.0)
                                .color(GITHUB_TEXT_SECONDARY),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);

                            // 语言切换
                            let current_lang = self.i18n.language();
                            let lang_label = match current_lang {
                                crate::i18n::lang::Language::ZhCn => "EN",
                                crate::i18n::lang::Language::EnUs => "中",
                            };
                            if ui
                                .add(style_button_borderless(egui::Button::new(
                                    egui::RichText::new(lang_label)
                                        .size(13.0)
                                        .color(GITHUB_TEXT_PRIMARY),
                                )))
                                .clicked()
                            {
                                let new_lang = match current_lang {
                                    crate::i18n::lang::Language::ZhCn => {
                                        crate::i18n::lang::Language::EnUs
                                    }
                                    crate::i18n::lang::Language::EnUs => {
                                        crate::i18n::lang::Language::ZhCn
                                    }
                                };
                                self.i18n = crate::i18n::I18n::load(new_lang);
                                self.config.language = new_lang.as_str().to_string();
                                if self.persist_config() {
                                    self.status_message = self.i18n.t0("status_ready");
                                }
                            }

                            // 设置
                            if ui
                                .add(style_button_borderless(egui::Button::new(
                                    egui::RichText::new(self.i18n.t0("toolbar_settings"))
                                        .size(13.0)
                                        .color(GITHUB_TEXT_PRIMARY),
                                )))
                                .clicked()
                            {
                                self.show_settings = !self.show_settings;
                            }

                            // 同步登录信息
                            if ui
                                .add(style_button_borderless(egui::Button::new(
                                    egui::RichText::new(self.i18n.t0("toolbar_sync"))
                                        .size(13.0)
                                        .color(GITHUB_TEXT_PRIMARY),
                                )))
                                .clicked()
                            {
                                self.import_current_auth();
                            }

                            ui.add_space(4.0);
                        });
                    },
                );
            });

        // ============ 底部状态栏 ============
        egui::TopBottomPanel::bottom("bottom_panel")
            .exact_height(38.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);

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
                            .color(if active_valid {
                                GITHUB_GREEN
                            } else {
                                GITHUB_RED
                            }),
                    );
                    ui.label(
                        egui::RichText::new(if active_valid {
                            self.i18n.t0("status_normal")
                        } else {
                            self.i18n.t0("status_error")
                        })
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
                        .add(style_button_borderless(egui::Button::new(
                            egui::RichText::new(self.i18n.t0("btn_export"))
                                .size(12.0)
                                .color(GITHUB_TEXT_PRIMARY),
                        )))
                        .clicked()
                    {
                        self.handle_export();
                    }

                    // 清空登录按钮
                    if ui
                        .add(style_button_borderless(egui::Button::new(
                            egui::RichText::new(self.i18n.t0("btn_clear_auth"))
                                .size(12.0)
                                .color(GITHUB_TEXT_PRIMARY),
                        )))
                        .clicked()
                    {
                        match atomcode_io::clear_auth(&self.config.custom_atomcode_dir) {
                            Ok(_) => {
                                self.config.active_account_id = None;
                                if self.persist_config() {
                                    self.status_message = self.i18n.t0("status_auth_cleared");
                                }
                            }
                            Err(e) => {
                                self.status_message = self.i18n.t1("status_clear_failed", &e);
                            }
                        }
                    }

                    // 右侧状态消息
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
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
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.vertical(|ui| {
                            // 当前使用中的账号指示器 + 排序按钮（同一行）
                            self.render_active_indicator(ui);

                            // 账号卡片列表（按当前排序方式排列）。
                            // 仅借用账号引用，避免每帧 clone 完整 auth token；
                            // 渲染期间收集动作，循环结束后统一执行。
                            let mut actions: Vec<CardAction> = Vec::new();
                            {
                                let mut accounts: Vec<&crate::models::ManagedAccount> =
                                    self.config.accounts.iter().collect();
                                self.sort_accounts(&mut accounts);
                                let active_id = self.config.active_account_id.clone();
                                for acc in accounts {
                                    let is_active = active_id.as_deref() == Some(acc.id.as_str());
                                    let action = std::cell::Cell::new(None::<CardAction>);
                                    self.render_account_card(ui, acc, is_active, &action);
                                    if let Some(a) = action.take() {
                                        actions.push(a);
                                    }
                                }
                            }

                            // 空状态提示
                            if self.config.accounts.is_empty() {
                                self.render_empty_state(ui);
                            }

                            // 账号引用借用已结束，统一执行渲染期间收集的动作
                            for action in actions {
                                match action {
                                    CardAction::Switch(id) => self.switch_to_account(&id),
                                    CardAction::Delete(id) => self.delete_confirm_id = Some(id),
                                    CardAction::OpenUpdate => {
                                        self.reset_for_new_fetch();
                                        self.show_manual_update = true;
                                        self.manual_update_text.clear();
                                        self.auto_parse_pending = true;
                                        self.clear_update_message();
                                        self.start_auto_update();
                                    }
                                }
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

        // ============ 更新信息对话框（独立子窗口，可浮动于主窗口之外） ============
        if self.show_manual_update {
            self.show_manual_update_viewport(ctx);
        }
    }
}

impl AtomcodeSwitchApp {
    /// 「更新账号信息」独立子视口：作为真正的系统窗口浮动于主窗口之外，
    /// 共享主窗口的 egui Context（字体/主题自动继承）。
    fn show_manual_update_viewport(&mut self, ctx: &egui::Context) {
        let title = self.i18n.t0("update_title");
        let mut viewport = egui::ViewportBuilder::default()
            .with_title(title.clone())
            .with_inner_size([520.0, 380.0])
            .with_min_inner_size([420.0, 300.0]);
        if let Some(icon) = &self.icon {
            viewport = viewport.with_icon(icon.clone());
        }

        ctx.show_viewport_immediate(
            egui::ViewportId(egui::Id::new("manual_update_window")),
            viewport,
            |ctx, class| {
                debug_assert!(class == egui::ViewportClass::Immediate);

                // 底部按钮栏：白底 + 顶部分隔线，上下留白对称，按钮垂直居中
                egui::TopBottomPanel::bottom("manual_update_buttons")
                    .frame(
                        egui::Frame::none()
                            .fill(GITHUB_CARD)
                            .stroke(egui::Stroke::new(1.0_f32, GITHUB_BORDER))
                            .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal_centered(|ui| {
                            if let Some(deadline) = self.close_update_after {
                                // 解析成功后处于延迟关闭：主按钮显示关闭倒计时
                                let remaining =
                                    deadline.saturating_duration_since(std::time::Instant::now());
                                let secs = (remaining.as_secs_f64().ceil() as u64).max(1);
                                let label = self.i18n.t1("update_closing", &secs.to_string());
                                ui.add_enabled(
                                    false,
                                    egui::Button::new(
                                        egui::RichText::new(label)
                                            .size(13.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .rounding(4.0)
                                    .fill(STATUS_OK_FG),
                                );
                            } else {
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
                                        .fill(if has_text {
                                            GITHUB_BLUE
                                        } else {
                                            GITHUB_BORDER
                                        }),
                                    )
                                    .clicked()
                                {
                                    let text = self.manual_update_text.clone();
                                    if text.trim().is_empty() {
                                        let msg = self.i18n.t0("status_paste_empty");
                                        self.set_update_message(
                                            crate::app::UpdateMessageKind::Error,
                                            msg,
                                        );
                                    } else if self.config.active_account_id.is_none() {
                                        let msg = self.i18n.t0("status_no_active_import");
                                        self.set_update_message(
                                            crate::app::UpdateMessageKind::Error,
                                            msg,
                                        );
                                    } else {
                                        match self.parse_login_output_and_update(&text) {
                                            Ok(()) => {
                                                // 仅解析成功才关闭窗口：记录结果消息后再关
                                                self.import_current_auth();
                                                self.manual_update_text.clear();
                                                let msg = self.status_message.clone();
                                                self.set_update_message(
                                                    crate::app::UpdateMessageKind::Success,
                                                    msg,
                                                );
                                                self.schedule_update_window_close(
                                                    std::time::Duration::from_millis(3000),
                                                );
                                            }
                                            Err(e) => {
                                                // 解析失败：窗口与原始内容保留，消息区显示「解析失败」
                                                self.mark_parse_failed(e);
                                            }
                                        }
                                    }
                                }
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(self.i18n.t0("btn_cancel"))
                                                    .size(13.0)
                                                    .color(GITHUB_TEXT_PRIMARY),
                                            )
                                            .rounding(4.0)
                                            .fill(GITHUB_CARD)
                                            .stroke(egui::Stroke::new(1.0_f32, GITHUB_BORDER)),
                                        )
                                        .clicked()
                                    {
                                        self.close_update_window();
                                    }
                                },
                            );
                        });
                    });

                // 中间内容区：描述行 + 占满剩余空间的文本框
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(12.0))
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(self.i18n.t0("update_desc"))
                                    .size(12.0)
                                    .color(GITHUB_TEXT_SECONDARY),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if self.is_auto_updating {
                                        ui.label(
                                            egui::RichText::new(self.i18n.t0("update_fetching"))
                                                .size(13.0)
                                                .color(GITHUB_BLUE)
                                                .strong(),
                                        );
                                        if ui
                                            .add(style_button_danger(egui::Button::new(
                                                egui::RichText::new(self.i18n.t0("update_cancel"))
                                                    .size(12.0),
                                            )))
                                            .clicked()
                                        {
                                            self.cancel_auto_fetch();
                                            self.status_message =
                                                self.i18n.t0("status_auto_fetch_cancelled");
                                        }
                                    } else {
                                        if ui
                                            .add(style_button_primary(egui::Button::new(
                                                egui::RichText::new(
                                                    self.i18n.t0("update_auto_fetch"),
                                                )
                                                .size(13.0)
                                                .color(egui::Color32::WHITE),
                                            )))
                                            .clicked()
                                        {
                                            self.manual_update_text.clear();
                                            // 重新获取成功后自动解析更新
                                            self.reset_for_new_fetch();
                                            self.clear_update_message();
                                            self.auto_parse_pending = true;
                                            self.start_auto_update();
                                        }
                                    }
                                },
                            );
                        });

                        ui.add_space(8.0);

                        // 消息区：解析/获取的结果（成功或失败）都显示在窗口内
                        if let Some(msg) = self.update_message.clone() {
                            use crate::app::UpdateMessageKind::{Error, Info, Success};
                            let (title, accent, bg) = match msg.kind {
                                Info => (String::new(), GITHUB_BLUE, MSG_INFO_BG),
                                Success => (
                                    self.i18n.t0("update_success_title"),
                                    GITHUB_GREEN,
                                    MSG_SUCCESS_BG,
                                ),
                                Error => {
                                    (self.i18n.t0("update_error_title"), GITHUB_RED, MSG_ERROR_BG)
                                }
                            };

                            egui::Frame::none()
                                .fill(bg)
                                .stroke(egui::Stroke::new(1.0_f32, accent))
                                .rounding(4.0)
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    if !title.is_empty() {
                                        ui.label(
                                            egui::RichText::new(title)
                                                .size(12.0)
                                                .strong()
                                                .color(accent),
                                        );
                                    }
                                    ui.label(
                                        egui::RichText::new(&msg.text).size(12.0).color(accent),
                                    );
                                    if msg.kind == Error {
                                        ui.label(
                                            egui::RichText::new(self.i18n.t0("update_error_hint"))
                                                .size(11.0)
                                                .color(GITHUB_TEXT_SECONDARY),
                                        );
                                    }
                                });

                            ui.add_space(8.0);
                        }

                        // 文本框占据剩余全部空间，按钮行贴底
                        egui::ScrollArea::vertical()
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::TextEdit::multiline(&mut self.manual_update_text)
                                        .hint_text(self.i18n.t0("update_hint"))
                                        .code_editor()
                                        .desired_width(f32::INFINITY),
                                );
                            });
                    });

                // 点击系统标题栏 X 时关闭子窗口
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.close_update_window();
                }
            },
        );
    }
}

// ============ 辅助渲染方法 ============

impl AtomcodeSwitchApp {
    /// 渲染激活账号指示器 + 排序按钮（同一行）
    fn render_active_indicator(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(active_id) = &self.config.active_account_id {
                if let Some(active_acc) = self.config.accounts.iter().find(|a| &a.id == active_id) {
                    ui.label(
                        egui::RichText::new("\u{25CF}")
                            .size(14.0)
                            .color(GITHUB_GREEN),
                    );
                    ui.label(
                        egui::RichText::new(&active_acc.auth_data.user.name)
                            .size(16.0)
                            .color(GITHUB_TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(self.i18n.t0("card_in_use"))
                            .size(13.0)
                            .color(GITHUB_TEXT_SECONDARY),
                    );
                }
            }

            ui.add_space(16.0);
            let sort_mode = self.config.sort_mode;
            let (none_text, reset_text, remaining_text) = (
                self.i18n.t0("sort_none"),
                self.i18n.t0("sort_reset_time"),
                self.i18n.t0("sort_window_remaining"),
            );
            if ui
                .selectable_label(
                    sort_mode == crate::models::SortMode::None,
                    egui::RichText::new(&none_text).size(13.0).color(
                        if sort_mode == crate::models::SortMode::None {
                            GITHUB_BLUE
                        } else {
                            GITHUB_TEXT_PRIMARY
                        },
                    ),
                )
                .clicked()
            {
                self.config.sort_mode = crate::models::SortMode::None;
                let _ = self.persist_config();
            }
            if ui
                .selectable_label(
                    sort_mode == crate::models::SortMode::WindowRemaining,
                    egui::RichText::new(&remaining_text).size(13.0).color(
                        if sort_mode == crate::models::SortMode::WindowRemaining {
                            GITHUB_BLUE
                        } else {
                            GITHUB_TEXT_PRIMARY
                        },
                    ),
                )
                .clicked()
            {
                self.config.sort_mode = crate::models::SortMode::WindowRemaining;
                let _ = self.persist_config();
            }
            if ui
                .selectable_label(
                    sort_mode == crate::models::SortMode::ResetTime,
                    egui::RichText::new(&reset_text).size(13.0).color(
                        if sort_mode == crate::models::SortMode::ResetTime {
                            GITHUB_BLUE
                        } else {
                            GITHUB_TEXT_PRIMARY
                        },
                    ),
                )
                .clicked()
            {
                self.config.sort_mode = crate::models::SortMode::ResetTime;
                let _ = self.persist_config();
            }
        });
        ui.add_space(12.0);
    }

    /// 渲染账号卡片
    fn render_account_card(
        &self,
        ui: &mut egui::Ui,
        acc: &crate::models::ManagedAccount,
        is_active: bool,
        action: &std::cell::Cell<Option<CardAction>>,
    ) {
        let visual_state = if !acc.is_valid {
            AccountVisualState::Invalid
        } else if self.is_account_expired(acc) {
            AccountVisualState::Expired
        } else if self.is_expiring_soon(acc) {
            AccountVisualState::Expiring
        } else {
            AccountVisualState::Normal
        };
        let (card_bg, card_stroke) = visual_state.card_colors(is_active);

        egui::Frame::default()
            .fill(card_bg)
            .rounding(10.0)
            .stroke(egui::Stroke::new(1.0_f32, card_stroke))
            .inner_margin(egui::Margin::symmetric(20.0, 16.0))
            .outer_margin(egui::Margin::symmetric(12.0, 8.0))
            .shadow(egui::epaint::Shadow {
                offset: [0.0, 2.0].into(),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(16),
            })
            .show(ui, |ui| {
                // ---- 卡片顶部 ----
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
                    let avatar_color = if is_active {
                        GITHUB_BLUE
                    } else {
                        GITHUB_AVATAR_INACTIVE
                    };
                    ui.painter()
                        .circle_filled(rect.center(), 18.0, avatar_color);
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
                                    .color(CARD_NAME_TEXT)
                                    .strong(),
                            );
                            ui.add_space(4.0);
                            // 四种状态统一徽标化，配色由 badge_colors 决定
                            let (badge_fg, badge_bg) = visual_state.badge_colors();
                            status_badge(
                                ui,
                                self.i18n.t0(visual_state.badge_key()),
                                badge_fg,
                                badge_bg,
                            );
                        });
                        ui.label(
                            egui::RichText::new(&acc.auth_data.user.id)
                                .size(11.0)
                                .color(CARD_SECONDARY_TEXT),
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
                                .fill(CARD_SWITCH_BLUE),
                            );
                            if switch_btn.clicked() {
                                action.set(Some(CardAction::Switch(acc.id.clone())));
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
                                    .fill(CARD_ACTIVATED_BG)
                                    .stroke(egui::Stroke::new(1.0_f32, GITHUB_GREEN)),
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
                                        .stroke(egui::Stroke::new(1.0_f32, GITHUB_BORDER)),
                                    )
                                    .clicked()
                                {
                                    action.set(Some(CardAction::OpenUpdate));
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
                                action.set(Some(CardAction::Delete(acc.id.clone())));
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
                            egui::RichText::new(
                                self.i18n
                                    .t1("card_remaining_days", &acc.remaining_days.to_string()),
                            )
                            .size(12.0)
                            .color(CARD_SECONDARY_TEXT),
                        );
                    });
                });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(self.i18n.t0("card_usage_label"))
                            .size(12.0)
                            .color(CARD_SECONDARY_TEXT),
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
                let bar_height = 6.0;
                let available_width = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(available_width, bar_height + 4.0),
                    egui::Sense::hover(),
                );
                let bar_rect = egui::Rect::from_min_max(
                    rect.min + egui::vec2(0.0, 2.0),
                    rect.max - egui::vec2(0.0, 2.0),
                );
                ui.painter().rect_filled(bar_rect, 3.0, GITHUB_PROGRESS_BG);
                let fill_width = bar_rect.width() * progress.min(1.0);
                if fill_width > 0.0 {
                    let fill_rect = egui::Rect::from_min_max(
                        bar_rect.min,
                        egui::pos2(bar_rect.min.x + fill_width, bar_rect.max.y),
                    );
                    let fill_color = visual_state.progress_color(acc.usage_percent);
                    ui.painter().rect_filled(fill_rect, 3.0, fill_color);
                }

                ui.add_space(6.0);

                // ---- 卡片底部 ----
                ui.horizontal(|ui| {
                    let (reset_text, reset_color) = self.format_reset_countdown(&acc.reset_time);
                    ui.label(
                        egui::RichText::new(reset_text)
                            .size(11.0)
                            .color(reset_color),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", acc.last_updated))
                                .size(11.0)
                                .color(CARD_SECONDARY_TEXT),
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
                auto_update_on_switch_expired: self.config.auto_update_on_switch_expired,
                pending_import_data_dir: None,
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
            .default_size([440.0, 300.0])
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::WHITE)
                    .rounding(12.0)
                    .inner_margin(egui::Margin::symmetric(20.0, 16.0))
                    .stroke(egui::Stroke::new(1.0_f32, GITHUB_BORDER))
                    .shadow(egui::epaint::Shadow {
                        offset: [0.0, 8.0].into(),
                        blur: 24.0,
                        spread: 0.0,
                        color: egui::Color32::from_black_alpha(32),
                    }),
            )
            .show(ctx, |ui| {
                ui.add_space(8.0);

                if let Some(ref mut state) = self.settings_state {
                    // 导出目录
                    ui.label(
                        egui::RichText::new(self.i18n.t0("settings_export_dir"))
                            .size(14.0)
                            .strong(),
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
                            if let Some(dir) = tinyfiledialogs::select_folder_dialog(
                                &self.i18n.t0("dialog_select_data_dir"),
                                &state.data_dir,
                            ) {
                                // 浏览只更新临时目录；实际导入在「确定」时执行。
                                // 若目标目录已有数据，先弹出冲突确认。
                                let accounts_path =
                                    std::path::Path::new(&dir).join("atomcode-accounts.yaml");
                                let mut confirmed_import = true;
                                if accounts_path.exists() {
                                    match Self::load_accounts_from(&accounts_path) {
                                        Ok(existing) => {
                                            let existing_accounts = existing.accounts.as_slice();
                                            let msg = self.i18n.t4(
                                                "dialog_switch_dir_msg",
                                                &existing_accounts.len().to_string(),
                                                &Self::format_account_lines(
                                                    &self.i18n,
                                                    existing_accounts,
                                                ),
                                                &self.config.accounts.len().to_string(),
                                                &Self::format_account_lines(
                                                    &self.i18n,
                                                    &self.config.accounts,
                                                ),
                                            );
                                            confirmed_import = matches!(
                                                tinyfiledialogs::message_box_yes_no(
                                                    &self.i18n.t0("dialog_switch_dir_title"),
                                                    &msg,
                                                    tinyfiledialogs::MessageBoxIcon::Question,
                                                    tinyfiledialogs::YesNo::No,
                                                ),
                                                tinyfiledialogs::YesNo::Yes
                                            );
                                        }
                                        Err(e) => {
                                            let msg =
                                                self.i18n.t1("status_accounts_file_invalid", &e);
                                            self.status_message = msg.clone();
                                            tinyfiledialogs::message_box_ok(
                                                &self.i18n.t0("dialog_import_failed_title"),
                                                &msg,
                                                tinyfiledialogs::MessageBoxIcon::Error,
                                            );
                                            // 文件损坏：中止导入，不设置 pending_import_data_dir
                                            confirmed_import = false;
                                        }
                                    }
                                }
                                if confirmed_import {
                                    let accounts_path =
                                        std::path::Path::new(&dir).join("atomcode-accounts.yaml");
                                    if accounts_path.exists() {
                                        state.pending_import_data_dir = Some(dir.clone());
                                    }
                                    state.data_dir = dir;
                                }
                            }
                        }
                    });

                    ui.add_space(12.0);

                    // Atomcode 目录
                    ui.label(
                        egui::RichText::new(self.i18n.t0("settings_atomcode_dir"))
                            .size(14.0)
                            .strong(),
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
                        egui::RichText::new(self.i18n.t0("settings_auto_switch"))
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.checkbox(
                        &mut state.auto_switch_enabled,
                        self.i18n.t0("settings_enable_auto_switch"),
                    );
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

                    ui.add_space(12.0);

                    // 切换时过期自动更新
                    ui.checkbox(
                        &mut state.auto_update_on_switch_expired,
                        self.i18n.t0("settings_auto_update_on_switch_expired"),
                    );

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
            if let Some(state) = self.settings_state.take() {
                let mut import_failed = false;
                // 「确定」时才真正导入目标目录中的账号
                if let Some(dir) = state.pending_import_data_dir {
                    let accounts_path = std::path::Path::new(&dir).join("atomcode-accounts.yaml");
                    match Self::load_accounts_from(&accounts_path) {
                        Ok(config) => {
                            self.config.accounts = config.accounts;
                            self.config.active_account_id = config.active_account_id;
                            self.config.auto_switch_rules = config.auto_switch_rules;
                        }
                        Err(e) => {
                            let msg = self.i18n.t1("status_accounts_file_invalid", &e);
                            tinyfiledialogs::message_box_ok(
                                &self.i18n.t0("dialog_import_failed_title"),
                                &msg,
                                tinyfiledialogs::MessageBoxIcon::Error,
                            );
                            self.status_message = msg;
                            // 文件损坏：保留当前账号，不清空
                            import_failed = true;
                        }
                    }
                }
                config_io::set_custom_data_dir(if state.data_dir.is_empty() {
                    None
                } else {
                    Some(state.data_dir.clone())
                });
                self.config.custom_atomcode_dir = if state.atomcode_dir.is_empty() {
                    None
                } else {
                    Some(state.atomcode_dir.clone())
                };
                self.config.auto_switch_rules.enabled = state.auto_switch_enabled;
                self.config.auto_switch_rules.max_usage_percent = state.max_usage_percent;
                self.config.auto_update_on_switch_expired = state.auto_update_on_switch_expired;
                if self.persist_config() && !import_failed {
                    self.status_message = self.i18n.t0("status_settings_saved");
                }
            }
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

    /// 把账号列表格式化为可显示的多行文本
    fn format_account_lines(
        i18n: &crate::i18n::I18n,
        accounts: &[crate::models::ManagedAccount],
    ) -> String {
        if accounts.is_empty() {
            return i18n.t0("dialog_none");
        }
        accounts
            .iter()
            .map(|a| {
                i18n.t2(
                    "dialog_account_line",
                    &a.auth_data.user.name,
                    &a.auth_data.user.username,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 读取指定 YAML 文件中的账号配置；读取或解析失败时返回错误，由调用方决定如何处理。
    fn load_accounts_from(path: &std::path::Path) -> Result<AppConfig, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str::<AppConfig>(&content).map_err(|e| e.to_string())
    }

    /// 处理导出按钮逻辑
    fn handle_export(&mut self) {
        // 默认文件名与数据目录下的账号文件一致，避免用户按默认名导出后
        // 在切换数据目录时找不到可导入的文件；同时不与本地 config.yaml 混淆。
        if let Some(path_str) = tinyfiledialogs::save_file_dialog(
            &self.i18n.t0("dialog_export_title"),
            "atomcode-accounts.yaml",
        ) {
            let path = std::path::Path::new(&path_str);
            let should_overwrite = if path.exists() {
                let existing = match Self::load_accounts_from(path) {
                    Ok(config) => Some(config),
                    Err(e) => {
                        // 目标文件损坏：提示但继续走覆盖确认（原子写会先备份原文件）
                        self.status_message = self.i18n.t1("status_accounts_file_invalid", &e);
                        None
                    }
                };
                let existing_accounts = existing
                    .as_ref()
                    .map(|c| c.accounts.as_slice())
                    .unwrap_or(&[]);
                let msg = self.i18n.t4(
                    "dialog_overwrite_msg",
                    &existing_accounts.len().to_string(),
                    &Self::format_account_lines(&self.i18n, existing_accounts),
                    &self.config.accounts.len().to_string(),
                    &Self::format_account_lines(&self.i18n, &self.config.accounts),
                );

                matches!(
                    tinyfiledialogs::message_box_yes_no(
                        &self.i18n.t0("dialog_overwrite_title"),
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
                let result = serde_yaml::to_string(&self.config)
                    .map_err(|e| e.to_string())
                    .and_then(|content| atomic_write_text(path, &content));
                match result {
                    Ok(()) => self.status_message = self.i18n.t0("status_exported"),
                    Err(e) => self.status_message = self.i18n.t1("status_export_failed", &e),
                }
            } else {
                self.status_message = self.i18n.t0("status_kept_original");
            }
        }
    }

    /// 是否即将过期：未过期且将在 1 小时内重置（复用 `parse_reset_datetime` 单一解析入口）
    fn is_expiring_soon(&self, acc: &crate::models::ManagedAccount) -> bool {
        let now = chrono::Local::now();
        parse_reset_datetime(&acc.reset_time, now)
            .map(|target| {
                target >= now && (target - now).num_seconds() < EXPIRING_SOON_SECS as i64
            })
            .unwrap_or(false)
    }

    /// 从重置时间实时计算倒计时，返回 (文本, 颜色)。
    /// 与过期判断、排序共用 `parse_reset_datetime` 单一解析入口。
    fn format_reset_countdown(&self, reset_time: &str) -> (String, egui::Color32) {
        let now = chrono::Local::now();
        let target = match parse_reset_datetime(reset_time, now) {
            Some(t) => t,
            None => {
                return (
                    self.i18n.t1("card_reset_label", reset_time),
                    GITHUB_TEXT_SECONDARY,
                );
            }
        };
        let diff_secs = (target - now).num_seconds();
        if diff_secs < 0 {
            // 已过期：黄色主题提醒
            return (
                self.i18n.t1("card_reset_expired", reset_time.trim()),
                STATUS_EXPIRED_FG,
            );
        }
        let diff = diff_secs as u64;
        let minutes = diff / 60;
        let seconds = diff % 60;
        // 即将过期（1 小时内）用蓝色主题提醒
        let color = if diff < EXPIRING_SOON_SECS {
            STATUS_EXPIRING_FG
        } else {
            GITHUB_TEXT_SECONDARY
        };
        let text = if diff >= EXPIRING_SOON_SECS {
            self.i18n.t4(
                "card_reset_countdown_h",
                reset_time.trim(),
                &(diff / 3600).to_string(),
                &(minutes % 60).to_string(),
                &seconds.to_string(),
            )
        } else {
            self.i18n.t3(
                "card_reset_countdown",
                reset_time.trim(),
                &minutes.to_string(),
                &seconds.to_string(),
            )
        };
        (text, color)
    }

    /// 按当前配置排序账号卡片：激活账号永远第一；主键为所选方式，另一项作次级键；
    /// 超时未更新的账号视为最差情况，固定排在最后
    fn sort_accounts(&self, accounts: &mut [&crate::models::ManagedAccount]) {
        use crate::models::SortMode;
        let active_id = self.config.active_account_id.clone();
        let now = chrono::Local::now();
        accounts.sort_by(|a, b| {
            // 不排序：不区分激活/超时，直接按名称排列
            if self.config.sort_mode == SortMode::None {
                return a.auth_data.user.name.cmp(&b.auth_data.user.name);
            }
            // 激活账号永远排第一
            let is_active_a = active_id.as_deref() == Some(a.id.as_str());
            let is_active_b = active_id.as_deref() == Some(b.id.as_str());
            if is_active_a != is_active_b {
                return if is_active_a {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            let reset_a = parse_reset_datetime(&a.reset_time, now);
            let reset_b = parse_reset_datetime(&b.reset_time, now);
            let stale_a = reset_a.map(|t| t < now).unwrap_or(true);
            let stale_b = reset_b.map(|t| t < now).unwrap_or(true);
            // 超时未更新的排最后（其余照常排序）
            if stale_a != stale_b {
                return stale_a.cmp(&stale_b);
            }
            let remaining_a = 100.0 - a.usage_percent;
            let remaining_b = 100.0 - b.usage_percent;
            match self.config.sort_mode {
                SortMode::ResetTime => reset_a.cmp(&reset_b).then(
                    remaining_b
                        .partial_cmp(&remaining_a)
                        .unwrap_or(std::cmp::Ordering::Equal),
                ),
                SortMode::WindowRemaining => remaining_b
                    .partial_cmp(&remaining_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(reset_a.cmp(&reset_b)),
                // None 已在上方提前按名称排序返回，不会走到这里
                SortMode::None => std::cmp::Ordering::Equal,
            }
        });
    }
}

#[cfg(test)]
mod badge_tests {
    use super::*;

    #[test]
    fn badge_colors_distinct_per_state() {
        let states = [
            AccountVisualState::Normal,
            AccountVisualState::Expiring,
            AccountVisualState::Expired,
            AccountVisualState::Invalid,
        ];
        for i in 0..states.len() {
            for j in i + 1..states.len() {
                assert_ne!(
                    states[i].badge_colors().1,
                    states[j].badge_colors().1,
                    "状态 {} 与 {} 底色相同",
                    states[i].badge_key(),
                    states[j].badge_key()
                );
                assert_ne!(
                    states[i].badge_colors().0,
                    states[j].badge_colors().0,
                    "状态 {} 与 {} 字色相同",
                    states[i].badge_key(),
                    states[j].badge_key()
                );
            }
        }
    }
}
