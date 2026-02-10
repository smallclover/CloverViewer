use egui::{ColorImage, Context, Key, Modifiers, Pos2};
use xcap::Monitor;
use crate::core::business::BusinessData;
use crate::model::state::{ViewState, MonitorTexture};
use crate::utils::screenshot::capture_all_monitors;

pub fn handle_input_events(ctx: &Context, data: &mut BusinessData, state: &mut ViewState) {
    ctx.input(|i| {
        if i.key_pressed(Key::ArrowLeft) {
            data.prev_image(ctx.clone());
        }
        if i.key_pressed(Key::ArrowRight) {
            data.next_image(ctx.clone());
        }
    });

    if ctx.input_mut(|i| i.consume_key(Modifiers::ALT, Key::S)) {
        // Debug: Print monitor info based on mouse position
        let mouse_pos = ctx.input(|i| i.pointer.interact_pos());
        let window_rect = ctx.input(|i| i.viewport().outer_rect);

        if let (Some(pos), Some(w_rect)) = (mouse_pos, window_rect) {
            let abs_x = (w_rect.min.x + pos.x) as i32;
            let abs_y = (w_rect.min.y + pos.y) as i32;
            println!("DEBUG: Mouse Abs Pos: ({}, {})", abs_x, abs_y);

            if let Ok(monitors) = Monitor::all() {
                for monitor in monitors {
                    let mx = monitor.x().unwrap_or(0);
                    let my = monitor.y().unwrap_or(0);
                    let mw = monitor.width().unwrap_or(0);
                    let mh = monitor.height().unwrap_or(0);

                    if abs_x >= mx && abs_x < mx + mw as i32 && abs_y >= my && abs_y < my + mh as i32 {
                        println!("DEBUG: Mouse is on Monitor: {} (x:{}, y:{}, w:{}, h:{})",
                            monitor.name().unwrap_or_default(), mx, my, mw, mh);
                    }
                }
            }
        } else {
            println!("DEBUG: Could not determine mouse position relative to screen.");
        }

        if !state.crop_state.is_active {
            // 第一步：获取所有显示器的截图
            if let Some(monitors) = capture_all_monitors() {
                state.crop_state.monitor_textures.clear();

                // 计算所有显示器的最小坐标，作为偏移量
                let mut min_x = f32::MAX;
                let mut min_y = f32::MAX;

                for info in &monitors {
                    min_x = min_x.min(info.rect.min.x);
                    min_y = min_y.min(info.rect.min.y);
                }

                state.crop_state.offset = Pos2::new(min_x, min_y);

                for info in monitors {
                    let size = [info.image.width() as usize, info.image.height() as usize];
                    let pixels = info.image.as_flat_samples();
                    let color_image = ColorImage::from_rgba_unmultiplied(
                        size,
                        pixels.as_slice(),
                    );

                    let texture = ctx.load_texture(
                        format!("monitor_{}", info.id),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );

                    state.crop_state.monitor_textures.push(MonitorTexture {
                        id: info.id,
                        rect: info.rect,
                        texture,
                    });
                }

                state.crop_state.is_active = true;
                state.crop_state.start = None;
                state.crop_state.current = None;

                // 第二步：创建全屏交互层
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
            }
        }
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
