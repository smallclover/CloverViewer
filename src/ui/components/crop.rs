use eframe::epaint::StrokeKind;
use egui::{Color32, Context, LayerId, Order, Pos2, Rect, Stroke};
use crate::core::business::BusinessData;
use crate::model::state::ViewState;
use crate::utils::screenshot::capture_screen_area;

pub fn handle_crop_mode(ctx: &Context, state: &mut ViewState, data: &mut BusinessData) {
    if !state.crop_state.is_active {
        return;
    }

    // 设置鼠标样式为十字准星
    ctx.set_cursor_icon(egui::CursorIcon::Crosshair);

    let layer_id = LayerId::new(Order::Background, egui::Id::new("crop_bg_layer"));
    let painter = ctx.layer_painter(layer_id);

    // 获取偏移量
    let offset = state.crop_state.offset;

    // 绘制每个显示器的底图
    for monitor in &state.crop_state.monitor_textures {
        // 将绝对坐标转换为相对于窗口的坐标
        let rel_rect = monitor.rect.translate(-offset.to_vec2());

        painter.image(
            monitor.texture.id(),
            rel_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    let overlay_layer_id = LayerId::new(Order::Foreground, egui::Id::new("crop_overlay_layer"));
    let overlay_painter = ctx.layer_painter(overlay_layer_id);

    // 获取鼠标位置
    let mouse_pos = ctx.input(|i| i.pointer.interact_pos());

    // 确定当前鼠标所在的显示器
    let mut current_monitor_rect = Rect::ZERO;
    let mut found_monitor = false;

    if let Some(pos) = mouse_pos {
        for monitor in &state.crop_state.monitor_textures {
            let rel_rect = monitor.rect.translate(-offset.to_vec2());
            if rel_rect.contains(pos) {
                current_monitor_rect = rel_rect;
                found_monitor = true;
                break;
            }
        }
    }

    // 如果没找到（比如鼠标刚开始可能在边缘），默认取第一个或者保持 ZERO
    if !found_monitor && !state.crop_state.monitor_textures.is_empty() {
        current_monitor_rect = state.crop_state.monitor_textures[0].rect.translate(-offset.to_vec2());
    }

    // 绘制绿色边框（只在当前显示器）
    if found_monitor {
        overlay_painter.rect_stroke(
            current_monitor_rect,
            0.0,
            Stroke::new(4.0, Color32::GREEN),
            StrokeKind::Inside,
        );
    }

    // 绘制非当前显示器的遮罩
    for monitor in &state.crop_state.monitor_textures {
        let rel_rect = monitor.rect.translate(-offset.to_vec2());
        let is_current = if let Some(pos) = mouse_pos {
            rel_rect.contains(pos)
        } else {
            false
        };

        if !is_current {
            overlay_painter.rect_filled(
                rel_rect,
                0.0,
                Color32::from_black_alpha(150), //稍微深一点的灰色
            );
        }
    }


    let mut crop_rect = None;
    let mut should_exit = false;

    ctx.input(|i| {
        if i.pointer.primary_pressed() {
            state.crop_state.start = i.pointer.interact_pos();
            state.crop_state.current = i.pointer.interact_pos();
        }

        if i.pointer.primary_down() {
            if let Some(start) = state.crop_state.start {
                state.crop_state.current = i.pointer.interact_pos();
            }
        }

        if i.pointer.primary_released() {
            if let (Some(start), Some(end)) = (state.crop_state.start, state.crop_state.current) {
                let rect = Rect::from_two_pos(start, end);
                if rect.width() > 5.0 && rect.height() > 5.0 {
                    crop_rect = Some(rect);
                }
            }
            should_exit = true;
        }

        // 按 ESC 取消截图
        if i.key_pressed(egui::Key::Escape) {
            should_exit = true;
        }
    });

    if should_exit {
        state.crop_state.is_active = false;
        state.crop_state.start = None;
        state.crop_state.current = None;
        state.crop_state.monitor_textures.clear();

        // 恢复窗口状态
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
    }

    // 绘制当前选择区域（只在当前显示器内）
    if let (Some(start), Some(current)) = (state.crop_state.start, state.crop_state.current) {
        let rect = Rect::from_two_pos(start, current);

        // 绘制选区边框
        overlay_painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(2.0, Color32::WHITE),
            StrokeKind::Outside,
        );
    }

    // 重新实现遮罩逻辑：只在选区之外绘制遮罩（仅限于当前显示器）
    // 对于非当前显示器，上面已经绘制了全遮罩
    if let (Some(start), Some(current)) = (state.crop_state.start, state.crop_state.current) {
        let rect = Rect::from_two_pos(start, current);
        let screen_rect = current_monitor_rect; // 限制在当前显示器

        // 上
        overlay_painter.rect_filled(
            Rect::from_min_max(screen_rect.min, Pos2::new(screen_rect.max.x, rect.min.y.max(screen_rect.min.y))),
            0.0,
            Color32::from_black_alpha(100),
        );
        // 下
        overlay_painter.rect_filled(
            Rect::from_min_max(Pos2::new(screen_rect.min.x, rect.max.y.min(screen_rect.max.y)), screen_rect.max),
            0.0,
            Color32::from_black_alpha(100),
        );
        // 左
        overlay_painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(screen_rect.min.x, rect.min.y.max(screen_rect.min.y)),
                Pos2::new(rect.min.x.max(screen_rect.min.x), rect.max.y.min(screen_rect.max.y))
            ),
            0.0,
            Color32::from_black_alpha(100),
        );
        // 右
        overlay_painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(rect.max.x.min(screen_rect.max.x), rect.min.y.max(screen_rect.min.y)),
                Pos2::new(screen_rect.max.x, rect.max.y.min(screen_rect.max.y))
            ),
            0.0,
            Color32::from_black_alpha(100),
        );
    } else {
        // 如果没有选区，绘制当前显示器的全屏遮罩
         overlay_painter.rect_filled(
            current_monitor_rect,
            0.0,
            Color32::from_black_alpha(100),
        );
    }


    if let Some(rect) = crop_rect {
        // 第三步：裁剪并保存
        // 注意：这里的 rect 是基于 egui 窗口坐标的。
        // xcap 截图是基于屏幕绝对坐标的。
        // 如果窗口不是全屏或者有偏移，需要进行坐标转换。
        // 简单起见，假设用户是在全屏模式下或者我们截取的是整个屏幕然后裁剪。
        // 更准确的做法是获取窗口在屏幕上的位置。

        // 获取窗口位置
        let window_pos = ctx.input(|i| i.viewport().outer_rect).unwrap_or(Rect::ZERO).min;

        // 计算绝对坐标
        // 这里的 rect 是相对于窗口的，我们需要加上窗口的偏移量（offset）才能得到绝对坐标
        let abs_x = (window_pos.x + rect.min.x + offset.x) as i32;
        let abs_y = (window_pos.y + rect.min.y + offset.y) as i32;
        let width = rect.width() as u32;
        let height = rect.height() as u32;

        if let Some(image) = capture_screen_area(abs_x, abs_y, width, height) {
             let temp_dir = std::env::temp_dir();
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let path = temp_dir.join(format!("screenshot_crop_{}.png", timestamp));

            if let Err(e) = image.save(&path) {
                eprintln!("Failed to save screenshot: {}", e);
            } else {
                data.open_new_context(ctx.clone(), path);
            }
        }
    }
}
