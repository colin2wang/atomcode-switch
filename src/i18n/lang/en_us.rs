//! English translation

pub fn get(key: &str) -> &str {
    match key {
        // ======== Toolbar ========
        "toolbar_settings" => "Settings",
        "toolbar_sync" => "Sync Login Info",

        // ======== Bottom Bar ========
        "bottom_accounts" => "{0} accounts",
        "status_normal" => "Normal",
        "status_error" => "Error",
        "btn_export" => "Export",
        "btn_clear_auth" => "Clear Auth",

        // ======== Card ========
        "card_in_use" => "· Currently in use",
        "card_activated" => "Activated",
        "card_update_info" => "Update Info",
        "card_activate" => "Activate",
        "card_delete" => "Delete",
        "card_valid" => "● Valid",
        "card_invalid" => "● Invalid",
        "card_remaining_days" => "{0} days remaining",
        "card_usage_label" => "Current window usage",
        "card_reset_label" => "Reset: {0}",
        "card_reset_countdown" => "Reset: {0} (in {1}m {2}s)",
        "card_reset_countdown_h" => "Reset: {0} (in {1}h {2}m {3}s)",
        "card_reset_expired" => "Reset: {0} (please update)",

        // ======== Empty State ========
        "empty_no_accounts" => "No accounts",
        "empty_hint" => "Click \"Import\" to add current system account",
        "empty_import" => "Import Current Account",

        // ======== Settings Window ========
        "settings_title" => "Settings",
        "settings_export_dir" => "Export Directory",
        "settings_export_dir_desc" => "Custom data directory (default ~/.atomcode-switch)",
        "settings_browse" => "Browse…",
        "settings_export_dir_hint" => "Default: ~/.atomcode-switch",
        "settings_atomcode_dir" => "Atomcode Directory",
        "settings_atomcode_dir_desc" => "Custom auth.toml directory (default ~/.atomcode)",
        "settings_atomcode_hint" => "Default: ~/.atomcode",
        "settings_auto_switch" => "Auto-Switch Rules",
        "settings_enable_auto_switch" => "Enable auto-switch",
        "settings_usage_exceeds" => "Switch when usage exceeds",
        "settings_auto_switch_to" => "to the lowest-usage account",
        "btn_confirm" => "OK",
        "btn_cancel" => "Cancel",

        // ======== Update Dialog ========
        "update_title" => "Update Account Info",
        "update_desc" => "Clear the text area and click \"Auto Fetch\" to fill automatically:",
        "update_auto_fetch" => "Auto Fetch",
        "update_fetching" => "Fetching...",
        "update_cancel" => "Cancel",
        "update_parse" => "Parse & Update",
        "update_hint" => "Paste /login output here, or click \"Auto Fetch\" above...",

        // ======== Delete Confirmation ========
        "delete_confirm_title" => "Confirm Delete",
        "delete_confirm_msg" => "Are you sure to delete account \"{0}\"? This cannot be undone.",
        "btn_delete" => "Delete",

        // ======== Dialog Titles (tinyfiledialogs) ========
        "dialog_select_data_dir" => "Select data directory",
        "dialog_select_atomcode_dir" => "Select Atomcode directory",
        "dialog_export_title" => "Export config",
        "dialog_switch_dir_title" => "Switch export directory",
        "dialog_overwrite_title" => "Export conflict",
        "dialog_switch_dir_msg" => "Target directory has {0} accounts:\n{1}\n\nCurrent app has {2} accounts:\n{3}\n\nReplace current accounts with target directory ones?",
        "dialog_overwrite_msg" => "Target file has {0} accounts:\n{1}\n\nExporting {2} accounts:\n{3}\n\nOverwrite?",
        "dialog_none" => "(none)",

        // ======== Status Messages ========
        "status_ready" => "Ready",
        "status_auto_switched" => "Auto-switched to: {0}",
        "status_switched" => "Switched to: {0}",
        "status_switch_failed" => "Switch failed: {0}",
        "status_imported" => "Imported: {0}",
        "status_auth_not_found" => "Auth file not found",
        "status_account_deleted" => "Account deleted",
        "status_no_active_account" => "No active account",
        "status_active_account_not_found" => "Active account data not found",
        "status_updated_from_paste" => "Updated from paste: {0}",
        "status_updated_manual" => "Account info updated (manual paste)",
        "status_fetching" => "Fetching login info...",
        "status_fetched" => "Login info fetched, click \"Parse & Update\" to apply",
        "status_fetch_failed" => "Auto-fetch failed: process exited abnormally",
        "status_paste_empty" => "Paste content is empty",
        "status_no_active_import" => "No active account, please import one first",
        "status_updated" => "Account info updated",
        "status_update_failed" => "Update failed: {0}",
        "status_auth_cleared" => "Auth cleared",
        "status_clear_failed" => "Clear failed: {0}",
        "status_auto_fetch_cancelled" => "Auto-fetch cancelled",
        "status_imported_count" => "Imported {0} accounts",
        "status_kept_current" => "Kept current accounts",
        "status_exported" => "Export succeeded",
        "status_kept_original" => "Kept original file",
        "status_settings_saved" => "Settings saved",

        // fallback
        _ => key,
    }
}
