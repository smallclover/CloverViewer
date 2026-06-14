use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// 初始化日志系统
/// 在应用启动时调用一次
pub fn init_logging() {
    // 构建环境过滤器，默认显示 info 级别
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let is_mcp = std::env::args().any(|a| a == "--mcp");

    // 初始化 tracing-subscriber
    // MCP 模式下日志输出到 stderr，避免污染 stdout（MCP 用 stdout 做传输通道）
    if is_mcp {
        tracing_subscriber::registry()
            .with(fmt::layer().with_target(true).with_writer(std::io::stderr))
            .with(filter)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(fmt::layer().with_target(true))
            .with(filter)
            .init();
    }

    tracing::info!("CloverViewer logging initialized");
}
