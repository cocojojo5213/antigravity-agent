use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

use crate::config_manager::ConfigManager;

/// 应用程序设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 是否启用系统托盘
    pub system_tray_enabled: bool,
    /// 是否启用静默启动（启动时最小化到托盘或后台）
    pub silent_start_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            system_tray_enabled: false, // 默认不启用，避免打扰用户
            silent_start_enabled: false, // 默认不启用静默启动，让用户看到应用界面
        }
    }
}

/// 应用程序设置管理器
pub struct AppSettingsManager {
    settings: Mutex<AppSettings>,
    config_path: PathBuf,
}

impl AppSettingsManager {
    /// 创建新的设置管理器
    pub fn new(app_handle: &AppHandle) -> Self {
        let config_path = match ConfigManager::new() {
            Ok(manager) => manager.app_settings_file(),
            Err(_) => {
                // 如果 ConfigManager 初始化失败，尝试使用 Tauri 的配置目录
                app_handle.path().app_config_dir().unwrap_or(PathBuf::from(".")).join("app_settings.json")
            }
        };
        
        // 尝试加载现有设置
        let settings = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => AppSettings::default(),
            }
        } else {
            AppSettings::default()
        };

        Self {
            settings: Mutex::new(settings),
            config_path,
        }
    }

    /// 获取当前设置的副本
    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    /// 更新设置
    pub fn update_settings<F>(&self, update_fn: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppSettings),
    {
        let mut settings = self.settings.lock().unwrap();
        update_fn(&mut settings);
        
        // 保存到文件
        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("序列化设置失败: {}", e))?;
            
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        
        fs::write(&self.config_path, json)
            .map_err(|e| format!("写入设置文件失败: {}", e))?;
            
        Ok(())
    }
}
