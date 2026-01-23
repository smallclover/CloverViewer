// 仅在非 debug 模式（即 release）下应用 windows 子系统
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod utils;
mod ui;
mod image_loader;
mod navigator;
mod constants;
mod i18n;
mod config;

fn main() -> eframe::Result<()> {
    #[cfg(target_os = "windows")]
    app::run()
}