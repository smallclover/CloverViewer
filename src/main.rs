#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod feature;
mod i18n;
mod model;
mod ui;
mod utils;
mod os;

fn main() -> eframe::Result<()> {
    // 初始化日志系统
    core::logging::init_logging();

    let instance = single_instance::SingleInstance::new("CloverViewer").unwrap();
    if !instance.is_single() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    app::run()
}
