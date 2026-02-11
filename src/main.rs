// 仅在非 debug 模式（即 release）下应用 windows 子系统
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod utils;
mod ui;
mod core;
mod model;
mod i18n;
mod screenshot;

fn main() -> eframe::Result<()> {
    let instance = single_instance::SingleInstance::new("CloverViewer").unwrap();
    if !instance.is_single() {
        // 如果已经有一个实例在运行，则直接退出
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    app::run()
}
