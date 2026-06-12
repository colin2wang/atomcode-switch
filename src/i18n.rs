//! 国际化字符串查找 — 代理到 lang 模块的各语言 .rs 文件
//!
//! 用法：self.i18n.t0("key")  /  self.i18n.t1("key", &arg)  /  ...

use crate::lang::Language;

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

    /// 获取字符串（支持 {0} {1} 占位符替换）
    pub fn t(&self, key: &str, args: &[&str]) -> String {
        let template = (self.lookup)(key);
        if args.is_empty() {
            return template.to_string();
        }
        let mut result = template.to_string();
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
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
