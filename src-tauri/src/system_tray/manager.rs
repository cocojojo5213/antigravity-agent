use tauri::{AppHandle, Manager};

use crate::app_settings::AppSettingsManager;

/// 系统托盘管理器
pub struct SystemTrayManager;

impl SystemTrayManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self
    }

    /// 启用系统托盘
    pub fn enable(&self, app_handle: &AppHandle) -> Result<(), String> {
        // 1. 更新设置
        let settings_manager = app_handle.state::<AppSettingsManager>();
        settings_manager
            .update_settings(|s| s.system_tray_enabled = true)
            .map_err(|e| e.to_string())?;

        // 2. 检查是否已存在托盘
        if let Some(app_tray) = app_handle.tray_by_id("main") {
            tracing::info!("显示现有托盘");
            app_tray.set_visible(true).map_err(|e| {
                tracing::error!("显示托盘图标失败: {e}");
                e.to_string()
            })?;
        } else {
            // 创建新的托盘
            crate::system_tray::create_tray_with_return(app_handle)?;
            tracing::info!("系统托盘已创建");
        }

        Ok(())
    }

    /// 禁用系统托盘
    pub fn disable(&self, app_handle: &AppHandle) -> Result<(), String> {
        // 1. 更新设置
        let settings_manager = app_handle.state::<AppSettingsManager>();
        settings_manager
            .update_settings(|s| s.system_tray_enabled = false)
            .map_err(|e| e.to_string())?;

        // 2. 隐藏托盘
        if let Some(app_tray) = app_handle.tray_by_id("main") {
            app_tray.set_visible(false).map_err(|e| {
                tracing::error!("隐藏托盘图标失败: {e}");
                e.to_string()
            })?;
            tracing::info!("托盘图标已隐藏");
        }

        Ok(())
    }

    /// 切换系统托盘状态
    pub fn toggle(&self, app_handle: &AppHandle) -> Result<bool, String> {
        let settings_manager = app_handle.state::<AppSettingsManager>();
        let is_enabled = settings_manager.get_settings().system_tray_enabled;

        if is_enabled {
            self.disable(app_handle)?;
            Ok(false)
        } else {
            self.enable(app_handle)?;
            Ok(true)
        }
    }

    /// 检查系统托盘是否应启用（基于设置）
    pub fn is_enabled_setting(&self, app_handle: &AppHandle) -> bool {
        app_handle
            .state::<AppSettingsManager>()
            .get_settings()
            .system_tray_enabled
    }

    /// 最小化窗口到托盘
    pub fn minimize_to_tray(&self, app_handle: &AppHandle) -> Result<(), String> {
        if let Some(window) = app_handle.get_webview_window("main") {
            window.hide().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// 从托盘恢复窗口
    pub fn restore_from_tray(&self, app_handle: &AppHandle) -> Result<(), String> {
        if let Some(window) = app_handle.get_webview_window("main") {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
