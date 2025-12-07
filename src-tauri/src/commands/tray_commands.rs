use crate::system_tray::{update_tray_menu, SystemTrayManager};
use tauri::Manager;

/// 启用系统托盘
#[tauri::command]
pub async fn enable_system_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.enable(&app)?;

    Ok("系统托盘已启用".to_string())
}

/// 禁用系统托盘
#[tauri::command]
pub async fn disable_system_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.disable(&app)?;

    Ok("系统托盘已禁用".to_string())
}

/// 切换系统托盘状态
#[tauri::command]
pub async fn toggle_system_tray(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let system_tray = app.state::<SystemTrayManager>();
    let enabled = system_tray.toggle(&app)?;

    Ok(serde_json::json!({
        "enabled": enabled,
        "message": if enabled { "系统托盘已启用" } else { "系统托盘已禁用" }
    }))
}

/// 获取系统托盘状态
#[tauri::command]
pub async fn get_system_tray_state(app: tauri::AppHandle) -> Result<bool, String> {
    let system_tray = app.state::<SystemTrayManager>();
    Ok(system_tray.is_enabled_setting(&app))
}

/// 更新托盘菜单（新增命令，供前端调用）
#[tauri::command]
pub async fn update_tray_menu_command(
    app: tauri::AppHandle,
    accounts: Vec<String>,
) -> Result<String, String> {
    update_tray_menu(&app, accounts)?;
    Ok("托盘菜单已更新".to_string())
}


/// 最小化到托盘
#[tauri::command]
pub async fn minimize_to_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.minimize_to_tray(&app)?;
    Ok("已最小化到托盘".to_string())
}

/// 从托盘恢复
#[tauri::command]
pub async fn restore_from_tray(app: tauri::AppHandle) -> Result<String, String> {
    let system_tray = app.state::<SystemTrayManager>();
    system_tray.restore_from_tray(&app)?;
    Ok("已恢复窗口".to_string())
}

