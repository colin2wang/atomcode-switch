//! GitHub 主题色彩常量、中文字体加载与样式设置
//! 均为纯函数，不依赖 app 结构体

use eframe::egui;

// ============ GitHub CSS 颜色常量 ============
pub const GITHUB_BG: egui::Color32 = egui::Color32::from_rgb(246, 248, 250);
pub const GITHUB_CARD: egui::Color32 = egui::Color32::WHITE;
pub const GITHUB_BORDER: egui::Color32 = egui::Color32::from_rgb(208, 215, 222);
pub const GITHUB_TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(31, 35, 40);
pub const GITHUB_TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(101, 109, 118);
pub const GITHUB_BLUE: egui::Color32 = egui::Color32::from_rgb(9, 105, 218);
pub const GITHUB_BLUE_ACTIVE: egui::Color32 = egui::Color32::from_rgb(7, 78, 160);
pub const GITHUB_GREEN: egui::Color32 = egui::Color32::from_rgb(45, 164, 78);
pub const GITHUB_RED: egui::Color32 = egui::Color32::from_rgb(207, 34, 46);
pub const GITHUB_YELLOW: egui::Color32 = egui::Color32::from_rgb(212, 167, 44);
pub const GITHUB_BTN_BG: egui::Color32 = egui::Color32::from_rgb(246, 248, 250);
pub const GITHUB_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(238, 242, 246);
pub const GITHUB_BTN_ACTIVE: egui::Color32 = egui::Color32::from_rgb(220, 226, 232);
pub const GITHUB_AVATAR_INACTIVE: egui::Color32 = egui::Color32::from_rgb(175, 184, 193);
pub const GITHUB_PROGRESS_BG: egui::Color32 = egui::Color32::from_rgb(233, 236, 239);

// 卡片专用语义色（Bootstrap 风格，收敛硬编码）
pub const CARD_NAME_TEXT: egui::Color32 = egui::Color32::from_rgb(33, 37, 41);
pub const CARD_SECONDARY_TEXT: egui::Color32 = egui::Color32::from_rgb(108, 117, 125);
pub const CARD_SWITCH_BLUE: egui::Color32 = egui::Color32::from_rgb(13, 110, 253);
pub const CARD_ACTIVATED_BG: egui::Color32 = egui::Color32::from_rgb(220, 255, 220);
/// 过期/失效卡片的弱化底色（极淡灰白），边框用语义色强调，不影响正常卡片
pub const CARD_EXPIRED_BG: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xFA, 0xFA);
pub const MSG_INFO_BG: egui::Color32 = egui::Color32::from_rgb(233, 244, 255);
pub const MSG_SUCCESS_BG: egui::Color32 = egui::Color32::from_rgb(233, 250, 233);
pub const MSG_ERROR_BG: egui::Color32 = egui::Color32::from_rgb(255, 238, 238);

// 状态方块配色：按 code plan 时间窗口有效性四级分色，统一到 GitHub palette。
// 有效=绿（窗口充足）→ 即将过期=琥珀(severe) → 已过期=橙 → 已失效=红(danger)。
pub const STATUS_OK_BG: egui::Color32 = egui::Color32::from_rgb(0xDA, 0xFB, 0xE1);
pub const STATUS_OK_FG: egui::Color32 = egui::Color32::from_rgb(0x1A, 0x7F, 0x37);
pub const STATUS_EXPIRING_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xF8, 0xC5);
pub const STATUS_EXPIRING_FG: egui::Color32 = egui::Color32::from_rgb(0x9A, 0x67, 0x00);
pub const STATUS_EXPIRED_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xF3, 0xE0);
pub const STATUS_EXPIRED_FG: egui::Color32 = egui::Color32::from_rgb(0xBC, 0x4C, 0x00);
pub const STATUS_INVALID_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xEB, 0xEE);
pub const STATUS_INVALID_FG: egui::Color32 = egui::Color32::from_rgb(0xCF, 0x22, 0x2E);

/// 获取用量百分比对应的颜色
pub fn usage_color(percent: f32) -> egui::Color32 {
    if percent < 50.0 {
        GITHUB_GREEN
    } else if percent < 80.0 {
        GITHUB_YELLOW
    } else {
        GITHUB_RED
    }
}

/// 获取进度条填充颜色
pub fn progress_bar_color(percent: f32) -> egui::Color32 {
    if percent < 50.0 {
        GITHUB_BLUE
    } else if percent < 80.0 {
        GITHUB_YELLOW
    } else {
        GITHUB_RED
    }
}

/// 为现有按钮应用无边框样式
pub fn style_button_borderless(button: egui::Button<'_>) -> egui::Button<'_> {
    button
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::NONE)
}

/// 主操作按钮（蓝色实心白字）
pub fn style_button_primary(button: egui::Button<'_>) -> egui::Button<'_> {
    button.rounding(4.0).fill(GITHUB_BLUE)
}

/// 危险操作按钮（红色实心白字）
pub fn style_button_danger(button: egui::Button<'_>) -> egui::Button<'_> {
    button.rounding(4.0).fill(GITHUB_RED)
}

/// 加载系统中文字体（优先微软雅黑）
pub fn load_chinese_font() -> Option<Vec<u8>> {
    let font_paths = [
        "/usr/share/fonts/truetype/chinese/NotoSansSC[wght].ttf",
        "/usr/share/fonts/truetype/chinese/SarasaMonoSC-Regular.ttf",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/noto-serif-sc/NotoSerifSC-Regular.ttf",
        "/usr/share/fonts/truetype/lxgw-wenkai/LXGWWenKai-Regular.ttf",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "/System/Library/Fonts/PingFang.ttc",
    ];

    for path in &font_paths {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }

    // 尝试通过 fc-list 查找中文字体（Linux）
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

/// 设置中文字体支持
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(font_data) = load_chinese_font() {
        fonts.font_data.insert(
            "chinese_font".to_owned(),
            egui::FontData::from_owned(font_data),
        );

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

/// 设置 GitHub CSS 风格主题
pub fn setup_github_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    let github_bg = GITHUB_BG;
    let github_card = GITHUB_CARD;
    let github_border = GITHUB_BORDER;
    let github_text_primary = GITHUB_TEXT_PRIMARY;
    let github_blue = GITHUB_BLUE;
    let github_blue_active = GITHUB_BLUE_ACTIVE;
    let github_red = GITHUB_RED;
    let github_yellow = GITHUB_YELLOW;
    let github_btn_bg = GITHUB_BTN_BG;
    let github_btn_hover = GITHUB_BTN_HOVER;
    let github_btn_active = GITHUB_BTN_ACTIVE;
    let github_btn_text = GITHUB_TEXT_PRIMARY;
    let github_btn_border = GITHUB_BORDER;
    let github_input_bg = GITHUB_CARD;
    let github_selection = egui::Color32::from_rgba_premultiplied(8, 138, 237, 40);

    let rounding = egui::Rounding::same(6.0);

    style.visuals = egui::Visuals {
        dark_mode: false,
        window_fill: github_card,
        panel_fill: github_bg,
        faint_bg_color: github_bg,
        extreme_bg_color: github_card,
        code_bg_color: GITHUB_BG,
        warn_fg_color: github_yellow,
        error_fg_color: github_red,
        hyperlink_color: github_blue,
        selection: egui::style::Selection {
            bg_fill: github_selection,
            stroke: egui::Stroke::NONE,
        },
        window_stroke: egui::Stroke::new(1.0_f32, github_border),
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
                bg_stroke: egui::Stroke::new(1.0_f32, github_border),
                rounding,
                fg_stroke: egui::Stroke::new(1.0_f32, github_text_primary),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: github_btn_bg,
                weak_bg_fill: github_btn_bg,
                bg_stroke: egui::Stroke::new(1.0_f32, github_btn_border),
                rounding,
                fg_stroke: egui::Stroke::new(1.0_f32, github_btn_text),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: github_btn_hover,
                weak_bg_fill: github_btn_hover,
                bg_stroke: egui::Stroke::new(1.0_f32, github_blue),
                rounding,
                fg_stroke: egui::Stroke::new(1.5_f32, github_blue),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: github_btn_active,
                weak_bg_fill: github_btn_active,
                bg_stroke: egui::Stroke::new(1.0_f32, github_blue_active),
                rounding,
                fg_stroke: egui::Stroke::new(2.0_f32, github_blue_active),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: github_btn_hover,
                weak_bg_fill: github_btn_hover,
                bg_stroke: egui::Stroke::new(1.0_f32, github_blue),
                rounding,
                fg_stroke: egui::Stroke::new(1.5_f32, github_blue),
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
