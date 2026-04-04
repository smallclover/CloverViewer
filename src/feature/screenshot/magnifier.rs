use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Ui, Vec2, FontId, Align2, StrokeKind};
use eframe::egui::ColorImage;
use arboard::Clipboard;
use egui::epaint::Vertex;
use crate::i18n::lang::get_i18n_text;
use crate::feature::screenshot::capture::ScreenshotState;

/// 处理放大镜和取色器的核心入口
pub fn handle_magnifier(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    pointer_pos: Pos2,
) {
    let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);

    // 1. 寻找鼠标当前所在的具体屏幕
    let mut target_screen = None;
    for cap in &state.captures {
        let rect = Rect::from_min_size(
            Pos2::new(cap.screen_info.x as f32, cap.screen_info.y as f32),
            eframe::egui::vec2(cap.screen_info.width as f32, cap.screen_info.height as f32)
        );
        if rect.contains(global_pointer_phys) {
            target_screen = Some(cap);
            break;
        }
    }

    if let Some(screen) = target_screen {
        // 2. 计算放大镜裁剪所需的局部逻辑坐标
        let screen_local_logical_x = (global_pointer_phys.x - screen.screen_info.x as f32) / ppp;
        let screen_local_logical_y = (global_pointer_phys.y - screen.screen_info.y as f32) / ppp;
        let screen_local_pointer_pos = Pos2::new(screen_local_logical_x, screen_local_logical_y);

        // 3. 绘制放大镜 UI
        draw_magnifier_ui(
            ui,
            ui.painter(),
            &screen.image,
            pointer_pos,
            screen_local_pointer_pos,
            ppp
        );

        // 4. 处理颜色复制 (Ctrl + C 或按钮请求)
        if state.copy_requested || ui.input(|i| i.modifiers.ctrl && i.key_pressed(eframe::egui::Key::C)) {
            state.copy_requested = false;

            let center_phys_x = (global_pointer_phys.x - screen.screen_info.x as f32).round() as isize;
            let center_phys_y = (global_pointer_phys.y - screen.screen_info.y as f32).round() as isize;
            let img_width = screen.image.width() as isize;
            let img_height = screen.image.height() as isize;

            if center_phys_x >= 0 && center_phys_x < img_width && center_phys_y >= 0 && center_phys_y < img_height {
                let idx = center_phys_y as usize * screen.image.width() + center_phys_x as usize;
                let color = screen.image.pixels[idx];
                let hex_text = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());

                if let Ok(mut clipboard) = Clipboard::new() {
                    if let Err(e) = clipboard.set_text(hex_text.clone()) {
                        tracing::error!("Failed to set clipboard text: {}", e);
                    } else {
                        tracing::info!("Color {} copied to clipboard", hex_text);
                    }
                }
            }
        }
    }
}

/// 内部绘制放大镜组件的逻辑
fn draw_magnifier_ui(
    ui: &Ui,
    painter: &Painter,
    image: &ColorImage,
    draw_pos: Pos2, // 决定 UI 画在哪
    sample_pos: Pos2, // 决定从图片哪里取色
    ppp: f32,
) {
    let text = get_i18n_text(ui.ctx());
    // --- 1. 参数调整 ---
    let pixel_grid_size = 15;
    let zoom_pixel_size = 10.0;
    let half_grid = pixel_grid_size / 2;

    let magnifier_size = pixel_grid_size as f32 * zoom_pixel_size;
    let info_bar_height = 64.0;
    let card_size = Vec2::new(magnifier_size, magnifier_size + info_bar_height);

    // --- 2. 计算卡片位置 ---
    let offset = Vec2::new(20.0, 20.0);
    let mut card_pos = draw_pos + offset;

    let screen_rect = ui.ctx().viewport_rect();
    if card_pos.x + card_size.x > screen_rect.max.x {
        card_pos.x = draw_pos.x - offset.x - card_size.x;
    }
    if card_pos.y + card_size.y > screen_rect.max.y {
        card_pos.y = draw_pos.y - offset.y - card_size.y;
    }

    let card_rect = Rect::from_min_size(card_pos, card_size);

    // --- 3. 绘制卡片背景和边框 ---
    painter.rect_filled(card_rect, 4.0, Color32::WHITE);
    // --- 4. 绘制上半部分：像素放大镜 ---
    let magnifier_rect = Rect::from_min_size(card_pos, Vec2::new(magnifier_size, magnifier_size));
    let center_phys_x = (sample_pos.x * ppp).round() as isize;
    let center_phys_y = (sample_pos.y * ppp).round() as isize;
    let img_width = image.width() as isize;
    let img_height = image.height() as isize;

    let mut mesh = eframe::egui::Mesh::default();
    mesh.reserve_triangles(3721 * 2);
    mesh.reserve_vertices(3721 * 4);

    for dy in -half_grid..=half_grid {
        for dx in -half_grid..=half_grid {
            let src_x = center_phys_x + dx;
            let src_y = center_phys_y + dy;
            let color = if src_x >= 0 && src_x < img_width && src_y >= 0 && src_y < img_height {
                let idx = src_y as usize * image.width() + src_x as usize;
                image.pixels[idx]
            } else {
                Color32::BLACK
            };

            let grid_x = (dx + half_grid) as f32;
            let grid_y = (dy + half_grid) as f32;
            let pixel_rect = Rect::from_min_size(
                card_pos + Vec2::new(grid_x * zoom_pixel_size, grid_y * zoom_pixel_size),
                Vec2::new(zoom_pixel_size, zoom_pixel_size)
            );

            // 使用圆角而非粗暴切角，完美贴合卡片的 4.0 弧度
            if dy == -half_grid && dx == -half_grid {
                // 左上角像素：单独绘制，应用 4.0 的左上圆角 (nw: North-West)
                painter.rect_filled(
                    pixel_rect,
                    egui::CornerRadius { nw: 4, ne: 0, sw: 0, se: 0 },
                    color
                );
            } else if dy == -half_grid && dx == half_grid {
                // 右上角像素：单独绘制，应用 4.0 的右上圆角 (ne: North-East)
                painter.rect_filled(
                    pixel_rect,
                    egui::CornerRadius { nw: 0, ne: 4, sw: 0, se: 0 },
                    color
                );
            } else {
                // 其他正常的像素：画完整的正方形 (两个三角形) 合并入 mesh 以提高性能
                let idx = mesh.vertices.len() as u32;
                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx, idx + 2, idx + 3);
                mesh.vertices.push(Vertex { pos: pixel_rect.left_top(), uv: Pos2::ZERO, color });
                mesh.vertices.push(Vertex { pos: pixel_rect.right_top(), uv: Pos2::ZERO, color });
                mesh.vertices.push(Vertex { pos: pixel_rect.right_bottom(), uv: Pos2::ZERO, color });
                mesh.vertices.push(Vertex { pos: pixel_rect.left_bottom(), uv: Pos2::ZERO, color });
            }
        }
    }
    // 将普通像素统一绘制
    painter.add(eframe::egui::Shape::mesh(mesh));

    // --- 4.5 绘制像素格子 ---
    let grid_line_color = Color32::from_rgba_unmultiplied(0, 0, 0, 80);
    let grid_line_width = 1.0;
    for dy in 0..pixel_grid_size {
        for dx in 0..pixel_grid_size {
            let x = card_pos.x + dx as f32 * zoom_pixel_size;
            let y = card_pos.y + dy as f32 * zoom_pixel_size;
            // 竖线（向右延伸一格），最后一列不画避免与边框重合
            if dx < pixel_grid_size - 1 {
                painter.line_segment(
                    [Pos2::new(x + zoom_pixel_size, y), Pos2::new(x + zoom_pixel_size, y + zoom_pixel_size)],
                    Stroke::new(grid_line_width, grid_line_color)
                );
            }
            // 横线（向下一格），最后一行不画避免与边框重合
            if dy < pixel_grid_size - 1 {
                painter.line_segment(
                    [Pos2::new(x, y + zoom_pixel_size), Pos2::new(x + zoom_pixel_size, y + zoom_pixel_size)],
                    Stroke::new(grid_line_width, grid_line_color)
                );
            }
        }
    }

    // --- 4.6 绘制卡片边框 (在画完网格之后画，让边框压在最上面，确保完美闭合) ---
    painter.rect_stroke(
        card_rect,
        4.0,
        Stroke::new(1.0, Color32::from_gray(200)),
        StrokeKind::Inside // Inside 向内绘制
    );
    // --- 5. 绘制十字准星 ---
    let center_grid_idx = half_grid as f32;
    let center_pixel_rect = Rect::from_min_size(
        card_pos + Vec2::new(center_grid_idx * zoom_pixel_size, center_grid_idx * zoom_pixel_size),
        Vec2::new(zoom_pixel_size, zoom_pixel_size)
    );
    painter.rect_stroke(
        center_pixel_rect,
        0.0,
        Stroke::new(1.5, Color32::from_rgb(0, 255, 255)),
        StrokeKind::Outside
    );
    let center_line_color = Color32::from_rgba_unmultiplied(0, 255, 255, 100);
    painter.line_segment([magnifier_rect.center_top(), magnifier_rect.center_bottom()], Stroke::new(1.0, center_line_color));
    painter.line_segment([magnifier_rect.left_center(), magnifier_rect.right_center()], Stroke::new(1.0, center_line_color));

    // --- 6. 绘制下半部分：信息文本 ---
    let center_idx = if center_phys_x >= 0 && center_phys_x < img_width && center_phys_y >= 0 && center_phys_y < img_height {
        center_phys_y as usize * image.width() + center_phys_x as usize
    } else {
        0
    };
    let center_color = if center_phys_x >= 0 && center_phys_x < img_width { image.pixels[center_idx] } else { Color32::BLACK };

    let info_rect = Rect::from_min_max(
        Pos2::new(card_rect.min.x, card_rect.max.y - info_bar_height),
        card_rect.max
    );

    painter.line_segment(
        [info_rect.left_top(), info_rect.right_top()],
        Stroke::new(1.0, Color32::from_gray(230))
    );

    let coord_text = format!("({}, {})", center_phys_x, center_phys_y);
    let hex_text = format!("#{:02X}{:02X}{:02X}", center_color.r(), center_color.g(), center_color.b());

    let text_color = Color32::from_rgb(40, 40, 40);
    let hint_color = Color32::from_gray(150);
    let font_id = FontId::proportional(12.0);
    let hint_font_id = FontId::proportional(10.0);

    let line_height = info_bar_height / 3.0;

    painter.text(
        Pos2::new(info_rect.min.x + 8.0, info_rect.min.y + line_height * 0.5 + 2.0),
        Align2::LEFT_CENTER,
        format!("{}{}", text.magnifier_pos, coord_text),
        font_id.clone(),
        text_color,
    );

    let row2_y = info_rect.min.y + line_height * 1.5 + 2.0;
    let hex_galley = painter.layout_no_wrap(format!("{}{}", text.magnifier_hex, hex_text), font_id.clone(), text_color);
    let hex_text_width = hex_galley.size().x;
    painter.galley(
        Pos2::new(info_rect.min.x + 8.0, row2_y - hex_galley.size().y / 2.0),
        hex_galley,
        text_color
    );

    let color_preview_size = 12.0;
    let color_preview_pos = Pos2::new(
        info_rect.min.x + 8.0 + hex_text_width + 8.0,
        row2_y - color_preview_size / 2.0
    );
    let color_preview_rect = Rect::from_min_size(
        color_preview_pos,
        Vec2::new(color_preview_size, color_preview_size)
    );
    painter.rect_filled(color_preview_rect, 2.0, center_color);
    painter.rect_stroke(color_preview_rect, 2.0, Stroke::new(1.0, Color32::from_gray(200)), StrokeKind::Outside);

    painter.text(
        Pos2::new(info_rect.min.x + 8.0, info_rect.min.y + line_height * 2.5),
        Align2::LEFT_CENTER,
        text.tooltip_mouse_copy_color,
        hint_font_id,
        hint_color,
    );
}