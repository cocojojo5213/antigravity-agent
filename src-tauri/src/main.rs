// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dirs;
use std::fs;
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::prelude::*;

// Modules
mod antigravity;
mod app_settings;
mod config_manager;
mod constants;
mod platform;
mod system_tray;
mod utils;
mod window;

mod commands;
mod db_monitor;
mod path_utils;
mod setup;
mod state;

// Re-export AppState for compatibility with other modules
pub use state::{AntigravityAccount, AppState, ProfileInfo};

// Use commands
use crate::commands::*;

/// 获取日志目录路径，与 state.rs 和 logging_commands.rs 保持一致
fn get_log_directory() -> PathBuf {
    if cfg!(windows) {
        // Windows: 优先使用 APPDATA 环境变量
        std::env::var_os("APPDATA")
            .map(|appdata| PathBuf::from(appdata).join(".antigravity-agent"))
            .or_else(|| {
                // 备用方案：通过用户主目录构建 AppData\Roaming 路径
                dirs::home_dir().map(|home| {
                    home.join("AppData")
                        .join("Roaming")
                        .join(".antigravity-agent")
                })
            })
            .or_else(|| {
                // 最后备用：使用系统标准配置目录
                dirs::config_dir().map(|config| config.join(".antigravity-agent"))
            })
            .unwrap_or_else(|| PathBuf::from(".antigravity-agent"))
            .join("logs")
    } else {
        // macOS/Linux: 使用标准配置目录
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".antigravity-agent")
            .join("logs")
    }
}

/// 初始化双层日志系统（控制台 + 文件）
fn init_tracing() -> WorkerGuard {
    // 创建日志目录
    let log_dir = get_log_directory();
    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("警告：无法创建日志目录 {}: {}", log_dir.display(), e);
    }

    // 设置文件 appender（滚动日志文件）
    let file_appender = rolling::daily(&log_dir, "antigravity-agent");
    let (non_blocking, guard) = non_blocking(file_appender);

    // 设置控制台和文件双层输出
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false)
                .compact()
                .with_ansi(true), // 控制台启用颜色
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_ansi(false) // 文件不使用颜色代码
                .json(), // 文件使用 JSON 格式，便于后续处理
        )
        .init();

    guard // 返回 guard 以防止日志缓冲区被过早清理
}

fn main() {
    // 初始化双层日志系统（控制台 + 文件）
    let _guard = init_tracing();

    tracing::info!(target: "app::startup", "🚀 启动 Antigravity Agent");
    tracing::info!(target: "app::startup", "📝 日志系统已初始化（控制台 + 文件）");
    tracing::info!(target: "app::startup", "📁 日志目录: {}", get_log_directory().display());

    // 记录系统启动信息
    crate::utils::tracing_config::log_system_info();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_http::init())
        .manage(AppState::default())
        .setup(|app| setup::init(app))
        .invoke_handler(tauri::generate_handler![
            backup_profile,
            restore_profile,
            get_recent_accounts,
            collect_backup_contents,
            restore_backup_files,
            delete_backup,
            clear_all_backups,
            // Antigravity 相关命令
            switch_antigravity_account,
            get_antigravity_accounts,
            get_current_antigravity_info,
            backup_antigravity_current_account,
            restore_antigravity_account,
            switch_to_antigravity_account,
            clear_all_antigravity_data,
            // 进程管理命令
            kill_antigravity,
            is_antigravity_running,
            list_antigravity_processes,
            start_antigravity,
            backup_and_restart_antigravity,
            // 平台支持命令
            get_platform_info,
            find_antigravity_installations,
            get_current_paths,
            // 数据库路径相关
            detect_antigravity_installation,
            // 可执行文件路径相关
            validate_antigravity_executable,
            detect_antigravity_executable,
            save_antigravity_executable,
            enable_system_tray,
            disable_system_tray,
            minimize_to_tray,
            restore_from_tray,
            get_system_tray_state,
            toggle_system_tray,
            update_tray_menu_command,
            is_silent_start_enabled,
            save_silent_start_state,
            get_all_settings,
            // 数据库监控命令
            is_database_monitoring_running,
            start_database_monitoring,
            stop_database_monitoring,
            get_log_info,
            clear_logs,
            decrypt_config_data,
            encrypt_config_data,
            write_text_file,
            write_frontend_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
