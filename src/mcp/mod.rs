mod capture;

use image::ImageFormat;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::*;
use rmcp::schemars;
use rmcp::service::ServiceExt;
use rmcp::{tool, tool_router};
use serde::Deserialize;

#[derive(Clone)]
pub struct CloverMcpServer;

impl CloverMcpServer {
    pub fn new() -> Self {
        Self
    }
}

// ── 参数定义 ──────────────────────────────────────────────

#[derive(Deserialize, schemars::JsonSchema)]
struct ScreenshotParams {
    /// 截图模式："active_window"（活动窗口）、"monitor"（指定显示器）、"all_monitors"（所有显示器）。
    /// 不指定时默认截取活动窗口；如果指定了 monitor_index 但未指定 mode，则截取指定显示器（向后兼容）。
    mode: Option<String>,
    /// 指定显示器索引，仅在 mode="monitor" 或未指定 mode 时使用。
    monitor_index: Option<u64>,
}

// ── 工具实现 ──────────────────────────────────────────────

#[tool_router(server_handler)]
impl CloverMcpServer {
    /// 截取屏幕截图，保存到文件并返回路径
    #[tool(
        name = "take_screenshot",
        description = "Capture a screenshot and return the saved file path. Modes: 'active_window' (default, captures the focused window), 'monitor' (requires monitor_index), 'all_monitors' (captures all monitors). If mode is omitted but monitor_index is provided, captures that specific monitor for backward compatibility."
    )]
    async fn take_screenshot(
        &self,
        Parameters(ScreenshotParams { mode, monitor_index }): Parameters<ScreenshotParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let captures = tokio::task::spawn_blocking(move || {
            match mode.as_deref() {
                Some("active_window") => {
                    capture::capture_active_window().map(|c| vec![c])
                }
                Some("monitor") => {
                    let index = monitor_index.ok_or("monitor_index is required when mode='monitor'")?;
                    capture::capture_monitor(index as usize).map(|c| vec![c])
                }
                Some("all_monitors") => {
                    capture::capture_all_monitors()
                }
                Some(other) => {
                    Err(format!("Unknown mode: {other}"))
                }
                None => {
                    // mode 未指定：monitor_index 有值则截取指定显示器（向后兼容），否则截取活动窗口
                    if let Some(index) = monitor_index {
                        capture::capture_monitor(index as usize).map(|c| vec![c])
                    } else {
                        capture::capture_active_window().map(|c| vec![c])
                    }
                }
            }
        })
        .await
        .map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Task join error: {e}"), None)
        })?
        .map_err(|e| rmcp::ErrorData::internal_error(e, None))?;

        // 保存目录 ~/.clover-viewer/tmp/
        let tmp_dir = dirs::home_dir()
            .ok_or_else(|| rmcp::ErrorData::internal_error("Cannot determine home directory", None))?
            .join(".clover-viewer")
            .join("tmp");
        std::fs::create_dir_all(&tmp_dir)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to create tmp dir: {e}"), None))?;

        let mut paths = Vec::new();
        for (i, cap) in captures.into_iter().enumerate() {
            let ts = {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let s = (now % 60) as u32;
                let m = ((now / 60) % 60) as u32;
                let h = ((now / 3600) % 24) as u32;
                let days_since_epoch = (now / 86400) as i64;
                let (y, mo, d) = days_to_ymd(days_since_epoch);
                format!("{y:04}{mo:02}{d:02}_{h:02}{m:02}{s:02}")
            };
            let filename = if i == 0 {
                format!("screenshot_{ts}.png")
            } else {
                format!("screenshot_{ts}_{i}.png")
            };
            let filepath = tmp_dir.join(&filename);

            let img = image::DynamicImage::ImageRgba8(cap.image);
            img.save_with_format(&filepath, ImageFormat::Png)
                .map_err(|e| rmcp::ErrorData::internal_error(format!("PNG save error: {e}"), None))?;

            paths.push(filepath.to_string_lossy().into_owned());
        }

        Ok(CallToolResult::success(vec![Content::text(paths.join("\n"))]))
    }
}

// ── 时间戳辅助 ────────────────────────────────────────────

/// 将 Unix 纪元以来的天数转为 (年, 月, 日)
fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ── 启动入口 ──────────────────────────────────────────────

pub fn run_mcp_server(_instance_guard: single_instance::SingleInstance) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let server = CloverMcpServer::new();
        let running = server
            .serve(rmcp::transport::io::stdio())
            .await
            .expect("Failed to start MCP server");
        running.waiting().await.expect("MCP server error");
    });
}

pub fn run_mcp_http_server(port: u16) {
    use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let service = StreamableHttpService::new(
            || Ok(CloverMcpServer::new()),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default(),
        );

        let app = axum::Router::new().nest_service("/mcp", service);

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .expect("Failed to bind TCP listener");

        println!("MCP HTTP server listening on http://127.0.0.1:{port}/mcp");

        axum::serve(listener, app)
            .await
            .expect("HTTP server error");
    });
}
