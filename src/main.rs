#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod core;
mod i18n;
mod model;
mod ui;
mod utils;
mod os;
mod state;

fn main() -> eframe::Result<()> {
    let instance = single_instance::SingleInstance::new("CloverViewer").unwrap();
    if !instance.is_single() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    app::run()
}