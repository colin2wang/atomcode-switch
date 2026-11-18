pub mod lang;

use self::lang::Language;

/// 国际化字符串查找器
pub struct I18n {
    lang: Language,
    lookup: fn(&str) -> &str,
}

impl I18n {
    /// 加载指定语言的翻译
    pub fn load(lang: Language) -> Self {
        let lookup = lang.lookup();
        I18n { lang, lookup }
    }

    /// 获取当前语言
    pub fn language(&self) -> Language {
        self.lang
    }

    /// 获取原始字符串（不含占位符替换），找不到时返回 key 本身
    #[allow(dead_code)]
    pub fn raw(&self, key: &str) -> String {
        (self.lookup)(key).to_string()
    }

    /// 获取字符串（支持 {0} {1} 占位符替换）。
    /// 单次遍历模板：参数值本身若含 `{N}` 不会被后续替换误伤。
    pub fn t(&self, key: &str, args: &[&str]) -> String {
        let template = (self.lookup)(key);
        if args.is_empty() || !template.contains('{') {
            return template.to_string();
        }
        let bytes = template.as_bytes();
        let mut result = String::with_capacity(template.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'{' && i + 2 < bytes.len() && bytes[i + 2] == b'}' {
                if let Some(digit) = (bytes[i + 1] as char).to_digit(10) {
                    let idx = digit as usize;
                    if let Some(arg) = args.get(idx) {
                        result.push_str(arg);
                        i += 3;
                        continue;
                    }
                }
            }
            // 按字符边界推进，兼容多字节 UTF-8
            match template.get(i..).and_then(|s| s.chars().next()) {
                Some(ch) => {
                    result.push(ch);
                    i += ch.len_utf8();
                }
                None => break,
            }
        }
        result
    }

    /// 无参数的快捷方法
    pub fn t0(&self, key: &str) -> String {
        (self.lookup)(key).to_string()
    }

    /// 1 个参数的快捷方法
    pub fn t1(&self, key: &str, a0: &str) -> String {
        self.t(key, &[a0])
    }

    /// 2 个参数的快捷方法
    #[allow(dead_code)]
    pub fn t2(&self, key: &str, a0: &str, a1: &str) -> String {
        self.t(key, &[a0, a1])
    }

    /// 3 个参数的快捷方法
    pub fn t3(&self, key: &str, a0: &str, a1: &str, a2: &str) -> String {
        self.t(key, &[a0, a1, a2])
    }

    /// 4 个参数的快捷方法
    pub fn t4(&self, key: &str, a0: &str, a1: &str, a2: &str, a3: &str) -> String {
        self.t(key, &[a0, a1, a2, a3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_replacement_single_pass() {
        let i18n = I18n::load(Language::ZhCn);
        // 参数本身含 {1}，不应被后续替换误伤
        let result = i18n.t("card_reset_countdown", &["T", "{1}s", "3"]);
        assert!(result.contains("{1}s"), "参数中的占位符被二次替换: {result}");
    }

    #[test]
    fn unknown_key_returns_key() {
        let i18n = I18n::load(Language::EnUs);
        assert_eq!(i18n.t0("definitely_missing_key"), "definitely_missing_key");
    }
}
