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

    // 使用 Foreground 层，确保覆盖在应用原本的 UI 之上（原本的 UI 通常在 Middle 层）
    let layer_id = LayerId::new(Order::Foreground, egui::Id::new("crop_bg_layer"));
    let painter = ctx.layer_painter(layer_id);

    // 绘制全屏黑色背景，防止应用内容裸露
    // 使用窗口大小绘制背景，坐标相对于窗口内容区域 (0,0)
    let window_inner_rect = ctx.input(|i| i.viewport().inner_rect).unwrap_or(Rect::ZERO);
    let bg_rect = Rect::from_min_size(Pos2::ZERO, window_inner_rect.size());
    painter.rect_filled(bg_rect, 0.0, Color32::BLACK);

    // 获取窗口位置和缩放比例
    let window_rect = ctx.input(|i| i.viewport().outer_rect).unwrap_or(Rect::ZERO);
    let window_pos = window_rect.min;
    let pixels_per_point = ctx.pixels_per_point();

    // 绘制每个显示器的底图
    for monitor in &state.crop_state.monitor_textures {
        let tex_size = monitor.texture.size_vec2();
        let rect_w = monitor.rect.width();

        // 判断 monitor.rect 是物理坐标还是逻辑坐标
        // 如果 rect 宽度接近纹理宽度（像素），则是物理坐标，需要除以 pixels_per_point
        // 否则认为是逻辑坐标，不需要缩放
        let scale_factor = if (rect_w - tex_size.x).abs() < 1.0 {
            pixels_per_point
        } else {
            1.0
        };

        // 将物理坐标转换为逻辑坐标
        let rect_logical = Rect::from_min_max(
            Pos2::new(monitor.rect.min.x / scale_factor, monitor.rect.min.y / scale_factor),
            Pos2::new(monitor.rect.max.x / scale_factor, monitor.rect.max.y / scale_factor),
        );

        // 将绝对坐标转换为相对于窗口的坐标
        let rel_rect = rect_logical.translate(-window_pos.to_vec2());

        painter.image(
            monitor.texture.id(),
            rel_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    // 使用 Tooltip 层，确保覆盖在截图层之上
    let overlay_layer_id = LayerId::new(Order::Tooltip, egui::Id::new("crop_overlay_layer"));
    let overlay_painter = ctx.layer_painter(overlay_layer_id);

    // 获取鼠标位置
    let mouse_pos = ctx.input(|i| i.pointer.interact_pos());

    // 确定当前鼠标所在的显示器
    let mut current_monitor_rect = Rect::ZERO;
    let mut found_monitor = false;

    if let Some(pos) = mouse_pos {
        for monitor in &state.crop_state.monitor_textures {
            let tex_size = monitor.texture.size_vec2();
            let rect_w = monitor.rect.width();
            let scale_factor = if (rect_w - tex_size.x).abs() < 1.0 {
                pixels_per_point
            } else {
                1.0
            };

            let rect_logical = Rect::from_min_max(
                Pos2::new(monitor.rect.min.x / scale_factor, monitor.rect.min.y / scale_factor),
                Pos2::new(monitor.rect.max.x / scale_factor, monitor.rect.max.y / scale_factor),
            );
            let rel_rect = rect_logical.translate(-window_pos.to_vec2());

            if rel_rect.contains(pos) {
                current_monitor_rect = rel_rect;
                found_monitor = true;
                break;
            }
        }
    }

    // 如果没找到（比如鼠标刚开始可能在边缘），默认取第一个或者保持 ZERO
    if !found_monitor && !state.crop_state.monitor_textures.is_empty() {
        let monitor = &state.crop_state.monitor_textures[0];
        let tex_size = monitor.texture.size_vec2();
        let rect_w = monitor.rect.width();
        let scale_factor = if (rect_w - tex_size.x).abs() < 1.0 {
            pixels_per_point
        } else {
            1.0
        };
        let rect_logical = Rect::from_min_max(
            Pos2::new(monitor.rect.min.x / scale_factor, monitor.rect.min.y / scale_factor),
            Pos2::new(monitor.rect.max.x / scale_factor, monitor.rect.max.y / scale_factor),
        );
        current_monitor_rect = rect_logical.translate(-window_pos.to_vec2());
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
        let tex_size = monitor.texture.size_vec2();
        let rect_w = monitor.rect.width();
        let scale_factor = if (rect_w - tex_size.x).abs() < 1.0 {
            pixels_per_point
        } else {
            1.0
        };
        let rect_logical = Rect::from_min_max(
            Pos2::new(monitor.rect.min.x / scale_factor, monitor.rect.min.y / scale_factor),
            Pos2::new(monitor.rect.max.x / scale_factor, monitor.rect.max.y / scale_factor),
        );
        let rel_rect = rect_logical.translate(-window_pos.to_vec2());

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
        // 注意：这里的 rect 是基于 egui 窗口坐标的（逻辑坐标）。
        // xcap 截图是基于屏幕绝对坐标的（物理坐标）。

        // 计算绝对坐标
        let abs_pos_logical = window_pos + rect.min.to_vec2();

        let abs_x = (abs_pos_logical.x * pixels_per_point) as i32;
        let abs_y = (abs_pos_logical.y * pixels_per_point) as i32;
        let width = (rect.width() * pixels_per_point) as u32;
        let height = (rect.height() * pixels_per_point) as u32;

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
