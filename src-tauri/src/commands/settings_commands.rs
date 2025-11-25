//! 应用设置命令
//! 负责应用程序配置的管理和存储，使用 State 模式

use tauri::{AppHandle, Manager};


/// 获取静默启动状态
#[tauri::command]
pub async fn is_silent_start_enabled(
    app: AppHandle,
) -> Result<bool, String> {
    crate::log_async_command!("is_silent_start_enabled", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();
        let settings = settings_manager.get_settings();
        Ok(settings.silent_start_enabled)
    })
}

/// 保存静默启动状态
#[tauri::command]
pub async fn save_silent_start_state(
    app: AppHandle,
    enabled: bool,
) -> Result<String, String> {
    crate::log_async_command!("save_silent_start_state", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();

        settings_manager.update_settings(|settings| {
            settings.silent_start_enabled = enabled;
        })?;

        let status_text = if enabled { "已启用" } else { "已禁用" };
        Ok(format!("静默启动{}", status_text))
    })
}

/// 获取所有应用设置
#[tauri::command]
pub async fn get_all_settings(
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    crate::log_async_command!("get_all_settings", async {
        let settings_manager = app.state::<crate::app_settings::AppSettingsManager>();
        let settings = settings_manager.get_settings();

        Ok(serde_json::json!({
            "system_tray_enabled": settings.system_tray_enabled,
            "silent_start_enabled": settings.silent_start_enabled
        }))
    })
}
