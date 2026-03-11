use eframe::egui::{Context, Key};
use crate::{
    core::business::ViewerState,
    model::window_state::WindowState,
    model::config::get_context_config,
    os::window::show_window_hide,
};

pub fn handle_input_events(ctx: &Context, viewer: &mut ViewerState, window_state: &WindowState) {

    if ctx.input(|i| i.viewport().close_requested()){
        let config = get_context_config(ctx);
        let aq = window_state.allow_quit.lock().unwrap();
        let mut vis = window_state.visible.lock().unwrap();
        if config.minimize_on_close && !*aq {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            *vis = false;
            show_window_hide(window_state.hwnd_isize);
        }else{
        }
    }

    if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
        viewer.prev_image(ctx.clone());
    }
    if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
        viewer.next_image(ctx.clone());
    }

    if let Some(path) = ctx.input(|i| {
        i.raw
            .dropped_files
            .first()
            .and_then(|f| f.path.clone())
    }) {
        viewer.handle_dropped_file(ctx.clone(), path);
    }

    let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
    viewer.update_zoom(scroll_delta);
}
