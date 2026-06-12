//! 国际化翻译 — 每门语言一个独立的 .rs 文件
//!
//! 新增语言步骤：
//! 1. 创建 src/lang/xx.rs，实现 `pub fn get(key: &str) -> &str`
//! 2. 在本文件添加 `pub mod xx;`
//! 3. 在 `Language` 枚举添加 `Xx` 变体
//! 4. 在 `lookup()` 中添加匹配臂

pub mod zh_cn;
pub mod en_us;

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    ZhCn,
    EnUs,
}

impl Language {
    /// 从持久化字符串解析
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "en" | "en_us" | "en-us" | "english" => Language::EnUs,
            _ => Language::ZhCn,
        }
    }

    /// 转为持久化字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::ZhCn => "zh_cn",
            Language::EnUs => "en_us",
        }
    }

    /// 获取该语言的字符串查找函数
    pub fn lookup(&self) -> fn(&str) -> &str {
        match self {
            Language::ZhCn => zh_cn::get,
            Language::EnUs => en_us::get,
        }
    }
}
