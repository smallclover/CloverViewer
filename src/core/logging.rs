use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// 初始化日志系统
/// 在应用启动时调用一次
pub fn init_logging() {
    // 构建环境过滤器，默认显示 info 级别
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 初始化 tracing-subscriber
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true))
        .with(filter)
        .init();

    tracing::info!("CloverViewer logging initialized");
}
