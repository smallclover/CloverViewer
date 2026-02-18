use egui::{Context, Key};
use crate::core::business::BusinessData;

pub fn handle_input_events(ctx: &Context, data: &mut BusinessData) {
    if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
        data.prev_image(ctx.clone());
    }
    if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
        data.next_image(ctx.clone());
    }
    
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
