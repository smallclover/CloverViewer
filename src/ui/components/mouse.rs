use egui::{ColorImage, Context, Key, Modifiers, Pos2};
use xcap::Monitor;
use crate::core::business::BusinessData;
use crate::model::state::{ViewState, MonitorTexture};
use crate::utils::screenshot::capture_all_monitors;

pub fn handle_input_events(ctx: &Context, data: &mut BusinessData) {
    ctx.input(|i| {
        if i.key_pressed(Key::ArrowLeft) {
            data.prev_image(ctx.clone());
        }
        if i.key_pressed(Key::ArrowRight) {
            data.next_image(ctx.clone());
        }
    });
    
    if let Some(path) = ctx.input(|i| {
        i.raw
            .dropped_files
            .first()
            .and_then(|f| f.path.clone())
    }) {
        data.handle_dropped_file(ctx.clone(), path);
    }

    let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
    data.update_zoom(scroll_delta);
}
