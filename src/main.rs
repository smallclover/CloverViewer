#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod feature;
mod i18n;
mod model;
mod os;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    // 初始化日志系统
    core::logging::init_logging();

    let _instance_guard = match single_instance::SingleInstance::new("CloverViewer") {
        Ok(instance) => {
            if !instance.is_single() {
                return Ok(());
            }
            Some(instance)
        }
        Err(err) => {
            tracing::error!("Failed to create single instance guard: {}", err);
            None
        }
    };
    #[cfg(target_os = "windows")]
    app::run()
}
