//! 系统托盘管理模块
//!
//! 使用 Tauri 2.9 内置 API 实现后端控制托盘，前端通过命令更新菜单

pub mod manager;
pub mod tray;

// Re-export the main structs for convenience
pub use manager::SystemTrayManager;
pub use tray::{create_tray_with_return, update_tray_menu};
