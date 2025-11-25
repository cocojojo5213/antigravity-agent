//! 数据库监控模块 - 简化版本：newData, oldData, diff

use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

// 数据差异结构
#[derive(Debug, Clone, Serialize)]
pub struct DataDiff {
    pub has_changes: bool,
    pub changed_fields: Vec<String>,
    pub summary: String,
}

// 数据库监控器
pub struct DatabaseMonitor {
    app_handle: AppHandle,
    last_data: Arc<Mutex<Option<Value>>>,
    is_running: Arc<Mutex<bool>>,
}

impl DatabaseMonitor {
    /// 创建新的数据库监控器
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            last_data: Arc::new(Mutex::new(None)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// 启动数据库监控
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔧 启动数据库自动监控（简化版）");

        let last_data = self.last_data.clone();
        let is_running = self.is_running.clone();
        let app_handle = self.app_handle.clone();

        // 标记监控为运行状态
        *is_running.lock().await = true;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3)); // 3秒间隔，更敏感

            loop {
                interval.tick().await;

                // 检查监控是否还在运行
                let running = is_running.lock().await;
                if !*running {
                    info!("⏹️ 数据库监控已停止");
                    break;
                }
                drop(running);

                // 获取当前完整数据
                match Self::get_complete_data().await {
                    Ok(new_data) => {
                        let mut last = last_data.lock().await;

                        // 检查是否有数据变化
                        if let Some(ref old_data) = *last {
                            // 分析差异
                            let diff = Self::analyze_diff(old_data, &new_data);

                            if diff.has_changes {
                                info!("📢 检测到数据库变化: {}", diff.summary);

                                // 构建简化的事件数据：newData, oldData, diff
                                let event_data = serde_json::json!({
                                    "newData": new_data,
                                    "oldData": old_data,
                                    "diff": diff
                                });

                                // 推送事件到前端
                                if let Err(e) = app_handle.emit("database-changed", &event_data) {
                                    error!("❌ 推送数据库变化事件失败: {}", e);
                                } else {
                                    info!("✅ 数据库变化事件推送成功");
                                }
                            }
                        }

                        *last = Some(new_data);
                    }
                    Err(e) => {
                        warn!("⚠️ 获取完整数据失败: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止数据库监控
    pub async fn stop_monitoring(&self) {
        info!("⏹️ 停止数据库自动监控");
        *self.is_running.lock().await = false;
    }

    /// 获取完整数据库数据
    async fn get_complete_data() -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // 检测数据库路径
        let db_path = if cfg!(windows) {
            dirs::home_dir()
                .unwrap_or_default()
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")
        } else {
            dirs::config_dir()
                .unwrap_or_default()
                .join("Antigravity")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")
        };

        let mut complete_data = serde_json::Map::new();

        if db_path.exists() {
            let conn = rusqlite::Connection::open(&db_path)?;
            
            // 查询所有数据（完整的ItemTable）
            let mut stmt = conn.prepare("SELECT key, value FROM ItemTable ORDER BY key")?;
            
            let rows: Vec<(String, String)> = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?.collect::<Result<Vec<_>, _>>()?;

            // 构建完整数据对象
            for (key, value) in rows {
                // 尝试解析为JSON，如果失败则保持原始字符串
                let json_value: Value = match serde_json::from_str(&value) {
                    Ok(parsed) => parsed,
                    Err(_) => Value::String(value.clone()),
                };
                
                complete_data.insert(key, json_value);
            }
        }

        Ok(Value::Object(complete_data))
    }

    /// 分析两个数据之间的差异
    fn analyze_diff(old: &Value, new: &Value) -> DataDiff {
        let mut changed_fields = Vec::new();

        // 比较数据
        match (old, new) {
            (Value::Object(old_obj), Value::Object(new_obj)) => {
                // 检查新增的字段
                for key in new_obj.keys() {
                    match old_obj.get(key) {
                        Some(old_value) => {
                            if old_value != new_obj.get(key).unwrap() {
                                changed_fields.push(format!("{}: changed", key));
                            }
                        }
                        None => {
                            changed_fields.push(format!("{}: added", key));
                        }
                    }
                }

                // 检查删除的字段
                for key in old_obj.keys() {
                    if !new_obj.contains_key(key) {
                        changed_fields.push(format!("{}: removed", key));
                    }
                }
            }
            (Value::Null, Value::Object(_)) => {
                changed_fields.push("data: added".to_string());
            }
            (Value::Object(_), Value::Null) => {
                changed_fields.push("data: removed".to_string());
            }
            (Value::Null, Value::Null) => {
                // 都没有数据，无变化
            }
            _ => {
                changed_fields.push("data: structure_changed".to_string());
            }
        }

        let has_changes = !changed_fields.is_empty();
        let summary = if has_changes {
            format!("{} fields changed", changed_fields.len())
        } else {
            "No changes".to_string()
        };

        DataDiff {
            has_changes,
            changed_fields,
            summary,
        }
    }
}
