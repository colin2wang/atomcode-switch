//! `/login` 输出解析的纯函数集合。
//!
//! 仅做文本解析，不触碰任何账号状态：先收集到 `ParsedFields`，
//! 由调用方确认至少识别出一项后再事务性应用，保证「零字段不落盘」。

use crate::ops::time_utils::combine_reset_datetime;
use chrono::{DateTime, Local};

/// 从 `/login` 输出中解析出的字段；每个字段仅在严格校验成功后才为 `Some`。
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ParsedFields {
    pub plan_name: Option<String>,
    pub usage_percent: Option<f32>,
    pub reset_time: Option<String>,
    pub remaining_days: Option<u32>,
}

impl ParsedFields {
    /// 一个字段都没识别出来时返回 true（调用方应据此拒绝写盘）。
    pub fn is_empty(&self) -> bool {
        self.plan_name.is_none()
            && self.usage_percent.is_none()
            && self.reset_time.is_none()
            && self.remaining_days.is_none()
    }
}

/// 截取 `marker` 之后到终止符之间的文本，作为套餐名。
///
/// 终止符：`·` 分隔符、全角/半角左括号、到期时间关键词、换行——
/// 兼容 `套餐：X · …`、`套餐：X    到期时间 …（剩余 …）` 等多种输出格式。
fn plan_after(text: &str, marker: &str) -> Option<String> {
    let start = text.find(marker)? + marker.len();
    let after = &text[start..];
    let end = ['·', '（', '(']
        .iter()
        .filter_map(|m| after.find(*m))
        .chain(after.find("到期时间").into_iter())
        .chain(after.find("expires").into_iter())
        .min()
        .unwrap_or(after.len());
    let value = after[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// 从「剩余 19 天」或「Usage: 12%」这类前缀后提取数字。
fn number_after<'a>(text: &'a str, marker: &str, allow_dot: bool) -> Option<&'a str> {
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let value: String = rest
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit() || (allow_dot && *c == '.'))
        .collect();
    if value.is_empty() {
        None
    } else {
        // 返回在原串中的切片，避免额外分配
        let skipped = rest.len() - rest.trim_start().len();
        Some(&rest[skipped..skipped + value.len()])
    }
}

/// 从英文「19d / 30d」中提取剩余天数：取 `d /` 前紧邻的数字。
fn remaining_days_en(text: &str) -> Option<u32> {
    for line in text.lines() {
        if let Some(pos) = line.find("d /") {
            let before = &line[..pos];
            let digits: String = before
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if !digits.is_empty() {
                if let Ok(days) = digits.chars().rev().collect::<String>().parse::<u32>() {
                    return Some(days);
                }
            }
        }
    }
    None
}

/// 从单行中按关键词提取重置时间（严格校验 HH:MM，失败返回 None）。
fn reset_time_in_line(line: &str, keyword: &str, now: DateTime<Local>) -> Option<String> {
    let after = line.split(keyword).nth(1)?.trim();
    combine_reset_datetime(after, now)
}

/// 解析 `/login` 输出。纯函数：不修改任何状态，不产生 IO。
pub(crate) fn parse_login_fields(text: &str, now: DateTime<Local>) -> ParsedFields {
    let mut fields = ParsedFields::default();

    // 套餐名：中文优先，英文兜底
    fields.plan_name = plan_after(text, "套餐：").or_else(|| plan_after(text, "Plan:"));

    // 剩余天数：中文「剩余 19 天」，英文「19d / 30d」
    fields.remaining_days = number_after(text, "剩余 ", false)
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| remaining_days_en(text));

    // 用量百分比：中文「用量约 12%」，英文「Usage: 12%」
    fields.usage_percent = number_after(text, "用量约", true)
        .or_else(|| number_after(text, "Usage:", true))
        .or_else(|| number_after(text, "usage:", true))
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|p| *p >= 0.0 && *p <= 100.0);

    // 重置时间：中文「重置于」，英文「resets 」/「resets\t」
    for line in text.lines() {
        if let Some(value) = reset_time_in_line(line, "重置于", now) {
            fields.reset_time = Some(value);
            break;
        }
    }
    if fields.reset_time.is_none() {
        for line in text.lines() {
            let found = reset_time_in_line(line, "resets ", now)
                .or_else(|| reset_time_in_line(line, "resets\t", now));
            if let Some(value) = found {
                fields.reset_time = Some(value);
                break;
            }
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Local> {
        chrono::Local
            .with_ymd_and_hms(2026, 11, 18, 10, 30, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn parse_chinese_login_output() {
        let text = "已登录\n套餐：CodingPlan Lite · expires 2026-12-31\n剩余 19 天\n当前时间窗口用量约 12%\n重置于 21:04 (in 10h 34m)";
        let f = parse_login_fields(text, now());
        assert_eq!(f.plan_name.as_deref(), Some("CodingPlan Lite"));
        assert_eq!(f.remaining_days, Some(19));
        assert_eq!(f.usage_percent, Some(12.0));
        assert_eq!(f.reset_time.as_deref(), Some("2026-11-18 21:04"));
        assert!(!f.is_empty());
    }

    #[test]
    fn parse_english_login_output() {
        let text = "Logged in\nPlan: Pro Plan · expires soon\n19d / 30d remaining\nUsage: 45%\nresets 21:04 (in 10h)";
        let f = parse_login_fields(text, now());
        assert_eq!(f.plan_name.as_deref(), Some("Pro Plan"));
        assert_eq!(f.remaining_days, Some(19));
        assert_eq!(f.usage_percent, Some(45.0));
        assert_eq!(f.reset_time.as_deref(), Some("2026-11-18 21:04"));
    }

    #[test]
    fn parse_pure_english_usage_without_chinese() {
        let f = parse_login_fields("Usage: 12%", now());
        assert_eq!(f.usage_percent, Some(12.0));
    }

    #[test]
    fn garbage_reset_time_not_accepted() {
        let f = parse_login_fields("重置于 garbage text", now());
        assert!(f.reset_time.is_none());
        assert!(f.is_empty());
    }

    #[test]
    fn out_of_range_usage_rejected() {
        assert!(parse_login_fields("用量约 150%", now())
            .usage_percent
            .is_none());
    }

    #[test]
    fn parse_real_login_output_with_space_separators() {
        let text = "  AtomCode CodingPlan 配置：\n\n  ✓ already logged in as JW(hzjwang)\n  ✓ CodingPlan Lite-体验版 生效\n  ✓ 已添加 1 个账号 · 2 个模型：\n      • AtomGit  ·  qwen3.8-27b  （默认）\n      • AtomGit  ·  glm5.3-flash\n  ✓ 视觉预处理器 → AtomGit · qwen3.8-27b  （自动检测）\n  ✓ CodingPlan 状态：\n      套餐：CodingPlan Lite-体验版    到期时间 2026-11-25（剩余 5d / 共 7d）\n      用量：当前时间窗口用量约 23%    重置于 14:17（9m 40s 后）\n\n  Starting AtomCode...\n";
        let f = parse_login_fields(text, now());
        assert_eq!(
            f.plan_name.as_deref(),
            Some("CodingPlan Lite-体验版"),
            "plan={:?}",
            f.plan_name
        );
        assert_eq!(f.remaining_days, Some(5), "days={:?}", f.remaining_days);
        assert_eq!(f.usage_percent, Some(23.0), "usage={:?}", f.usage_percent);
        assert_eq!(
            f.reset_time.as_deref(),
            Some("2026-11-18 14:17"),
            "reset={:?}",
            f.reset_time
        );
        assert!(!f.is_empty());
    }
}
