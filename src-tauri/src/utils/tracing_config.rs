//! Tracing 配置模块
//! 提供统一的结构化日志配置和初始化

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt::MakeWriter};
use std::path::Path;
use std::io::{Write, BufWriter};
use std::fs::OpenOptions;
use std::sync::Mutex;
use super::log_sanitizer::LogSanitizer;

/// 每日日志文件 appender，生成格式: antigravity-agent.2025-11-23.log
struct DailyLogFileAppender {
    writer: Mutex<BufWriter<std::fs::File>>,
    log_dir: std::path::PathBuf,
    file_prefix: String,
    current_date: Mutex<String>,
}

impl DailyLogFileAppender {
    fn new(log_dir: &Path, file_prefix: &str) -> Self {
        let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let file_path = log_dir.join(format!("{}.{}.log", file_prefix, current_date));

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .expect("Failed to create log file");

        Self {
            writer: Mutex::new(BufWriter::new(file)),
            log_dir: log_dir.to_path_buf(),
            file_prefix: file_prefix.to_string(),
            current_date: Mutex::new(current_date),
        }
    }

    fn check_and_rotate(&self) -> std::io::Result<()> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let mut current_date = self.current_date.lock().unwrap();

        if *current_date != today {
            // 需要轮转文件
            let file_path = self.log_dir.join(format!("{}.{}.log", self.file_prefix, today));

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&file_path)?;

            *current_date = today.clone();
            let mut writer = self.writer.lock().unwrap();
            *writer = BufWriter::new(file);
        }
        Ok(())
    }
}

impl Write for DailyLogFileAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Err(e) = self.check_and_rotate() {
            eprintln!("日志轮转失败: {}", e);
        }

        // 对输出内容进行脱敏处理
        let content = String::from_utf8_lossy(buf);
        let sanitizer = LogSanitizer::new();
        let sanitized_content = sanitizer.sanitize(&content);

        let mut writer = self.writer.lock().unwrap();
        writer.write(sanitized_content.as_bytes())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()
    }
}

impl<'a> MakeWriter<'a> for DailyLogFileAppender {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        // 创建一个新的 writer 实例，使用相同的配置
        DailyLogFileAppender::new(&self.log_dir, &self.file_prefix)
    }
}

/// 初始化 Tracing 日志系统
///
/// 设置日志输出到文件和控制台，支持环境变量配置
///
/// # 参数
/// * `config_dir` - 配置目录路径
///
/// # 返回
/// Result<(), Box<dyn std::error::Error>>
///
/// # 示例
/// ```rust
/// use std::path::PathBuf;
/// init_tracing(&PathBuf::from("/config/dir")).expect("Failed to init tracing");
/// ```
pub fn init_tracing(config_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // AppState 已经把 config_dir 设为 %APPDATA%/.antigravity-agent，直接在其下创建 logs
    let log_dir = config_dir.join("logs");
    std::fs::create_dir_all(&log_dir)?;

    // 文件 appender (按日滚动，使用正确的文件名格式)
    let file_appender = DailyLogFileAppender::new(&log_dir, "antigravity-agent");

    // 控制台 appender (开发时使用)
    let (console_non_blocking, _console_guard) = tracing_appender::non_blocking(std::io::stdout());

    // 环境过滤器，默认 info 级别
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new("antigravity_agent=info,tower_http=debug")
        });

    // 组合多个输出目标
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)        // 文件输出不使用颜色
                .json()                 // 使用 JSON 格式便于结构化分析
                .with_current_span(false) // 在 JSON 中不重复显示 span
                .with_target(true)      // 显示模块路径
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(console_non_blocking)
                .compact()              // 控制台使用紧凑格式
                .with_target(false)     // 控制台不显示模块路径
                .with_thread_ids(false) // 控制台不显示线程ID
        )
        .init();

    Ok(())
}


/// 记录系统启动信息
pub fn log_system_info() {
    tracing::info!("🚀 启动 Antigravity Agent v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "🖥️ 系统信息: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    tracing::info!("📁 配置目录已初始化");
    tracing::info!("📁 Tracing 日志系统已启用");
}

/// 记录数据库操作
pub fn log_database_operation(operation: &str, table: Option<&str>, success: bool) {
    match (table, success) {
        (Some(table), true) => {
            tracing::info!(
                operation = operation,
                table = table,
                "🗄️ 数据库操作成功"
            );
        }
        (Some(table), false) => {
            tracing::error!(
                operation = operation,
                table = table,
                "❌ 数据库操作失败"
            );
        }
        (None, true) => {
            tracing::info!(
                operation = operation,
                "🗄️ 数据库操作成功"
            );
        }
        (None, false) => {
            tracing::error!(
                operation = operation,
                "❌ 数据库操作失败"
            );
        }
    }
}
