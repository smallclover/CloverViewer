#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

mod app;
mod core;
mod feature;
mod i18n;
mod mcp;
mod model;
mod os;
mod ui;
mod utils;

fn main() {
    // 初始化日志系统
    core::logging::init_logging();

    // MCP server 模式：--mcp 参数在 single-instance 检查之前处理
    if std::env::args().any(|a| a == "--mcp") {
        let instance_guard = match single_instance::SingleInstance::new("CloverViewer_MCP") {
            Ok(instance) => {
                if !instance.is_single() {
                    eprintln!("Another MCP server instance is already running, exiting.");
                    return;
                }
                instance
            }
            Err(err) => {
                eprintln!("Failed to create single instance guard: {err}");
                return;
            }
        };
        mcp::run_mcp_server(instance_guard);
        return;
    }

    // MCP HTTP server 模式
    if std::env::args().any(|a| a == "--mcp-http") {
        let port = std::env::args()
            .collect::<Vec<_>>()
            .windows(2)
            .find_map(|w| if w[0] == "--port" { w[1].parse::<u16>().ok() } else { None })
            .unwrap_or(3000);
        // 探测端口是否已被占用
        if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            eprintln!("Port {port} is already in use, another MCP HTTP server may be running. Exiting.");
            return;
        }
        mcp::run_mcp_http_server(port);
        return;
    }

    // GUI 模式：隐藏控制台窗口（仅 release 构建）
    #[cfg(all(not(debug_assertions), target_os = "windows"))]
    hide_console_window();

    let _instance_guard = match single_instance::SingleInstance::new("CloverViewer") {
        Ok(instance) => {
            if !instance.is_single() {
                return;
            }
            Some(instance)
        }
        Err(err) => {
            tracing::error!("Failed to create single instance guard: {}", err);
            None
        }
    };
    #[cfg(target_os = "windows")]
    app::run().expect("Failed to run application");
}

/// 隐藏控制台窗口（release 模式下使用）
/// 因为 windows_subsystem 改为 "console" 以支持 MCP stdio，
/// GUI 模式需要手动隐藏控制台窗口
#[cfg(all(not(debug_assertions), target_os = "windows"))]
fn hide_console_window() {
    use windows::Win32::System::Console::GetConsoleWindow;
    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};

    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_invalid() {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
}
