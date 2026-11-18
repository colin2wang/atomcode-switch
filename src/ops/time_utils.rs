//! 重置时间解析的纯函数集合。
//!
//! 统一 `YYYY-MM-DD HH:MM` 新格式与旧 `HH:MM` 格式的解析，
//! 供过期判断、倒计时、排序共用，避免多处口径不一致。

use chrono::{DateTime, Local, LocalResult, NaiveDateTime, Timelike};

/// 严格解析 `HH:MM`，校验小时/分钟范围，返回 `(hour, minute)`。
/// 使用 `str::get` 做字符边界切片，避免多字节 UTF-8 触发 panic。
pub(crate) fn parse_hhmm(raw: &str) -> Option<(u32, u32)> {
    let t = raw.trim();
    if t.len() < 5 || t.as_bytes().get(2) != Some(&b':') {
        return None;
    }
    let h_str = t.get(..2)?;
    let m_str = t.get(3..5)?;
    if !h_str.bytes().all(|b| b.is_ascii_digit()) || !m_str.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let hour: u32 = h_str.parse().ok()?;
    let minute: u32 = m_str.parse().ok()?;
    if hour < 24 && minute < 60 {
        Some((hour, minute))
    } else {
        None
    }
}

/// 将本地 naive 时间转为 `DateTime<Local>`；DST 跳变时取最早有效匹配，避免 panic。
fn to_local(naive: NaiveDateTime) -> Option<DateTime<Local>> {
    match naive.and_local_timezone(Local) {
        LocalResult::Single(dt) => Some(dt),
        LocalResult::Ambiguous(earliest, _latest) => Some(earliest),
        LocalResult::None => None,
    }
}

/// 统一解析重置时间：
/// - 新格式 `YYYY-MM-DD HH:MM`：直接解析为本地时间；
/// - 旧格式 `HH:MM`：结合 `now` 的日期，若今天该时刻已过则顺延到明天。
pub(crate) fn parse_reset_datetime(
    reset_time: &str,
    now: DateTime<Local>,
) -> Option<DateTime<Local>> {
    let t = reset_time.trim();
    if let Ok(naive) = NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M") {
        return to_local(naive);
    }
    let (hour, minute) = parse_hhmm(t)?;
    let naive = now.date_naive().and_hms_opt(hour, minute, 0)?;
    let target = to_local(naive)?;
    if target < now {
        let tomorrow = (now.date_naive() + chrono::Duration::days(1)).and_hms_opt(hour, minute, 0)?;
        to_local(tomorrow)
    } else {
        Some(target)
    }
}

/// 根据 `HH:MM` 与当前时间组合出 `YYYY-MM-DD HH:MM`；
/// 若今天的该时刻已过，则顺延到明天。解析失败返回 `None`。
pub(crate) fn combine_reset_datetime(hm: &str, now: DateTime<Local>) -> Option<String> {
    let (hour, minute) = parse_hhmm(hm)?;
    let today = now.date_naive();
    let target_secs = hour * 3600 + minute * 60;
    let now_secs = now.hour() * 3600 + now.minute() * 60 + now.second();
    let date = if target_secs > now_secs {
        today
    } else {
        today + chrono::Duration::days(1)
    };
    Some(format!(
        "{} {:02}:{:02}",
        date.format("%Y-%m-%d"),
        hour,
        minute
    ))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use super::*;

    fn fixed_now() -> DateTime<Local> {
        chrono::Local
            .with_ymd_and_hms(2026, 11, 18, 10, 30, 0)
            .single()
            .unwrap()
    }

    #[test]
    fn parse_hhmm_valid_and_invalid() {
        assert_eq!(parse_hhmm("09:05"), Some((9, 5)));
        assert_eq!(parse_hhmm(" 23:59 rest"), Some((23, 59)));
        assert_eq!(parse_hhmm("24:00"), None);
        assert_eq!(parse_hhmm("12:60"), None);
        assert_eq!(parse_hhmm("9:05"), None);
        // 多字节字符不会在非字符边界 panic
        assert_eq!(parse_hhmm("中:05"), None);
    }

    #[test]
    fn parse_reset_datetime_new_format() {
        let dt = parse_reset_datetime("2026-11-18 12:00", fixed_now()).unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2026-11-18 12:00");
    }

    #[test]
    fn parse_reset_datetime_legacy_rolls_over() {
        // 10:30 已过 -> 顺延到明天
        let dt = parse_reset_datetime("09:00", fixed_now()).unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2026-11-19 09:00");
        // 11:00 未到 -> 今天
        let dt = parse_reset_datetime("11:00", fixed_now()).unwrap();
        assert_eq!(dt.format("%Y-%m-%d %H:%M").to_string(), "2026-11-18 11:00");
    }

    #[test]
    fn parse_reset_datetime_garbage_returns_none() {
        assert!(parse_reset_datetime("not a time", fixed_now()).is_none());
        assert!(parse_reset_datetime("", fixed_now()).is_none());
    }

    #[test]
    fn combine_reset_datetime_strict() {
        assert_eq!(
            combine_reset_datetime("09:00", fixed_now()),
            Some("2026-11-19 09:00".to_string())
        );
        assert_eq!(combine_reset_datetime("99:99", fixed_now()), None);
    }
}
