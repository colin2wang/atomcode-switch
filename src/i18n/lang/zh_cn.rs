//! 简体中文翻译

pub fn get(key: &str) -> &str {
    match key {
        // ======== 工具栏 ========
        "toolbar_settings" => "设置",
        "toolbar_sync" => "同步登录信息",

        // ======== 底部栏 ========
        "bottom_accounts" => "{0} 个账户",
        "status_normal" => "正常",
        "status_error" => "异常",
        "btn_export" => "导出",
        "btn_clear_auth" => "清空登录",

        // ======== 卡片 ========
        "card_in_use" => "· 当前使用中",
        "card_activated" => "已激活",
        "card_update_info" => "更新信息",
        "card_activate" => "激活",
        "card_delete" => "删除",
        "card_valid" => "● 正常",
        "card_invalid" => "● 异常",
        "card_remaining_days" => "剩余 {0} 天",
        "card_usage_label" => "当前时间窗口用量",
        "card_reset_label" => "重置: {0}",
        "card_reset_countdown" => "重置: {0}（还有 {1} 分 {2} 秒）",
        "card_reset_countdown_h" => "重置: {0}（还有 {1} 小时 {2} 分 {3} 秒）",

        // ======== 空状态 ========
        "empty_no_accounts" => "暂无账号",
        "empty_hint" => "点击顶部 \"导入\" 按钮添加当前系统账号",
        "empty_import" => "导入当前系统账号",

        // ======== 设置窗口 ========
        "settings_title" => "设置",
        "settings_export_dir" => "导出目录",
        "settings_export_dir_desc" => "自定义账号数据存储目录（默认为 ~/.atomcode-switch）",
        "settings_browse" => "浏览…",
        "settings_export_dir_hint" => "默认: ~/.atomcode-switch",
        "settings_atomcode_dir" => "Atomcode 目录",
        "settings_atomcode_dir_desc" => "自定义 auth.toml 保存目录（默认为 ~/.atomcode）",
        "settings_atomcode_hint" => "默认: ~/.atomcode",
        "settings_auto_switch" => "自动切换规则",
        "settings_enable_auto_switch" => "启用自动切换",
        "settings_usage_exceeds" => "用量超过",
        "settings_auto_switch_to" => "时自动切换至用量最低的账号",
        "btn_confirm" => "确定",
        "btn_cancel" => "取消",

        // ======== 更新信息对话框 ========
        "update_title" => "更新账号信息",
        "update_desc" => "可将下方文本框清空，点击「自动获取」自动填入：",
        "update_auto_fetch" => "自动获取",
        "update_fetching" => "获取中...",
        "update_cancel" => "取消",
        "update_parse" => "解析并更新",
        "update_hint" => "在此粘贴 /login 的输出，或点击上方「自动获取」自动填入...",

        // ======== 删除确认对话框 ========
        "delete_confirm_title" => "确认删除",
        "delete_confirm_msg" => "确定要删除账号 \"{0}\" 吗？此操作不可撤销。",
        "btn_delete" => "删除",

        // ======== 对话框标题（tinyfiledialogs）=======
        "dialog_select_data_dir" => "选择数据目录",
        "dialog_select_atomcode_dir" => "选择 Atomcode 目录",
        "dialog_export_title" => "导出配置",
        "dialog_switch_dir_title" => "切换导出目录",
        "dialog_overwrite_title" => "导出冲突",
        "dialog_switch_dir_msg" => "目标目录已有 {0} 个账号：\n{1}\n\n当前程序有 {2} 个账号：\n{3}\n\n是否用目标目录的账号替换当前账号？",
        "dialog_overwrite_msg" => "目标文件已有 {0} 个账号：\n{1}\n\n即将导出 {2} 个账号：\n{3}\n\n是否覆盖？",
        "dialog_none" => "（无）",

        // ======== 状态消息 ========
        "status_ready" => "就绪",
        "status_auto_switched" => "自动切换至账号: {0}",
        "status_switched" => "已切换至: {0}",
        "status_switch_failed" => "切换失败: {0}",
        "status_imported" => "导入成功: {0}",
        "status_auth_not_found" => "未找到当前的 auth.toml 文件",
        "status_account_deleted" => "账号已删除",
        "status_no_active_account" => "没有激活的账号",
        "status_active_account_not_found" => "未找到当前激活的账号数据",
        "status_updated_from_paste" => "已从粘贴信息更新账号: {0}",
        "status_updated_manual" => "账号信息已更新（手动粘贴）",
        "status_fetching" => "正在获取登录信息...",
        "status_fetched" => "已获取登录信息，点击「解析并更新」完成更新",
        "status_fetch_failed" => "自动获取失败: 进程异常退出",
        "status_paste_empty" => "粘贴内容为空",
        "status_no_active_import" => "没有激活的账号，请先导入账号",
        "status_updated" => "账号信息已更新",
        "status_update_failed" => "更新失败: {0}",
        "status_auth_cleared" => "已清空当前登录",
        "status_clear_failed" => "清空失败: {0}",
        "status_auto_fetch_cancelled" => "已取消自动获取",
        "status_imported_count" => "已导入 {0} 个账号",
        "status_kept_current" => "已保留当前账号",
        "status_exported" => "导出成功",
        "status_kept_original" => "已保留原文件",
        "status_settings_saved" => "设置已保存",

        // 找不到 key 时原样返回
        _ => key,
    }
}
