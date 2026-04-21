use crate::i18n::lang::get_text;
use crate::model::{config::Config, window_state::WindowState};
use crate::os::current_platform;
use crate::ui::widgets::toast::ToastManager;
use egui::{Context, ViewportBuilder, ViewportCommand};
use std::{
    env,
    ffi::OsStr,
    path::PathBuf,
    sync::Arc,
};

const STARTUP_ARG: &str = "--startup";

pub struct LaunchOptions {
    pub start_in_background: bool,
    pub start_path: Option<PathBuf>,
}

pub fn parse_launch_options() -> LaunchOptions {
    let mut start_in_background = false;
    let mut start_path = None;

    for arg in env::args_os().skip(1) {
        if arg == OsStr::new(STARTUP_ARG) {
            start_in_background = true;
            continue;
        }

        if start_path.is_none() {
            start_path = Some(PathBuf::from(arg));
        }
    }

    LaunchOptions {
        start_in_background,
        start_path,
    }
}

pub fn configure_startup_viewport(
    viewport: ViewportBuilder,
    launch_options: &LaunchOptions,
) -> ViewportBuilder {
    if launch_options.start_in_background {
        viewport.with_visible(false)
    } else {
        viewport
    }
}

pub fn sync_launch_on_startup(config: &Config) {
    if config.launch_on_startup
        && let Err(err) = current_platform().set_launch_on_startup(true)
    {
        tracing::warn!("Failed to sync launch-on-startup registry entry: {}", err);
    }
}

pub fn apply_background_launch(
    ctx: &Context,
    window_state: &Arc<WindowState>,
    start_in_background: bool,
) {
    if !start_in_background {
        return;
    }

    if let Ok(mut visible) = window_state.visible.lock() {
        *visible = false;
    }

    ctx.send_viewport_cmd(ViewportCommand::Visible(false));
    current_platform().show_window_hide(window_state.hwnd_usize);
}

pub fn apply_launch_on_startup_setting(
    previous: &Config,
    next: &Config,
    toast_manager: &ToastManager,
) {
    if previous.launch_on_startup == next.launch_on_startup {
        return;
    }

    if let Err(err) = current_platform().set_launch_on_startup(next.launch_on_startup) {
        tracing::error!("Failed to update launch on startup: {}", err);
        let text = get_text(next.language);
        toast_manager.error(format!("{} {}", text.toast.launch_on_startup_failed, err));
    }
}