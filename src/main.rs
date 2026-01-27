// 仅在非 debug 模式（即 release）下应用 windows 子系统
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod utils;
mod ui;
mod image_loader;
mod navigator;
mod constants;
mod i18n;
mod config;

fn init_log() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_writer(std::io::stdout) // 显式指定输出到 stdout
        .init();
}

fn main() -> eframe::Result<()> {
    init_log();
    tracing::info!("app start");
    #[cfg(target_os = "windows")]
    app::run()
}
