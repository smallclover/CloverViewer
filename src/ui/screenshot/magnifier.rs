use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, Ui, Vec2, FontId, Align2, StrokeKind};
use eframe::egui::ColorImage;
use crate::i18n::lang::{get_i18n_text};

/// 绘制放大镜组件
pub fn draw_magnifier(
    ui: &Ui,
    painter: &Painter,
    image: &ColorImage,
    pointer_pos: Pos2,
    ppp: f32,
) {
    let text = get_i18n_text(ui.ctx());
    // --- 1. 参数调整 ---
    let pixel_grid_size = 61;
    let zoom_pixel_size = 3.0;
    let half_grid = pixel_grid_size / 2; // 61 / 2 还是 30

    let magnifier_size = pixel_grid_size as f32 * zoom_pixel_size;
    let info_bar_height = 64.0;
    let card_size = Vec2::new(magnifier_size, magnifier_size + info_bar_height);

    // --- 2. 计算卡片位置 ---
    let offset = Vec2::new(20.0, 20.0);
    let mut card_pos = pointer_pos + offset;

    let screen_rect = ui.ctx().viewport_rect();
    if card_pos.x + card_size.x > screen_rect.max.x {
        card_pos.x = pointer_pos.x - offset.x - card_size.x;
    }
    if card_pos.y + card_size.y > screen_rect.max.y {
        card_pos.y = pointer_pos.y - offset.y - card_size.y;
    }

    let card_rect = Rect::from_min_size(card_pos, card_size);

    // --- 3. 绘制卡片背景和边框 ---
    // [修改] 背景改成纯白
    painter.rect_filled(card_rect, 4.0, Color32::WHITE);
    // [修改] 边框改成一点点灰色 (由深灰改为了浅灰 200)
    painter.rect_stroke(
        card_rect,
        4.0,
        Stroke::new(1.0, Color32::from_gray(200)),
        StrokeKind::Outside
    );

    // --- 4. 绘制上半部分：像素放大镜 ---
    let magnifier_rect = Rect::from_min_size(card_pos, Vec2::new(magnifier_size, magnifier_size));
    let center_phys_x = (pointer_pos.x * ppp).round() as isize;
    let center_phys_y = (pointer_pos.y * ppp).round() as isize;
    let img_width = image.width() as isize;
    let img_height = image.height() as isize;

    // 使用底层 Mesh 一次性渲染所有像素格，避免 3721 次 UI 绘制调用导致的卡顿
    let mut mesh = eframe::egui::Mesh::default();
    // 提前分配内存，防止循环中频繁扩容
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

            // 手动将矩形顶点推入 Mesh，一次 Draw Call 解决几千个方块
            let idx = mesh.vertices.len() as u32;
            mesh.add_triangle(idx, idx + 1, idx + 2);
            mesh.add_triangle(idx, idx + 2, idx + 3);

            use eframe::egui::epaint::Vertex;
            mesh.vertices.push(Vertex { pos: pixel_rect.left_top(), uv: Pos2::ZERO, color });
            mesh.vertices.push(Vertex { pos: pixel_rect.right_top(), uv: Pos2::ZERO, color });
            mesh.vertices.push(Vertex { pos: pixel_rect.right_bottom(), uv: Pos2::ZERO, color });
            mesh.vertices.push(Vertex { pos: pixel_rect.left_bottom(), uv: Pos2::ZERO, color });
        }
    }
    // 将一整个网格添加到画板
    painter.add(egui::Shape::mesh(mesh));

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

    // [修改] 白底上的分割线，改成极浅的灰色
    painter.line_segment(
        [info_rect.left_top(), info_rect.right_top()],
        Stroke::new(1.0, Color32::from_gray(230))
    );

    let coord_text = format!("({}, {})", center_phys_x, center_phys_y);
    let hex_text = format!("#{:02X}{:02X}{:02X}", center_color.r(), center_color.g(), center_color.b());

    // [修改] 字体颜色适配白底
    let text_color = Color32::from_rgb(40, 40, 40); // 深灰色（接近黑）文字
    let hint_color = Color32::from_gray(150);       // 中灰色提示文字
    let font_id = FontId::proportional(12.0);
    let hint_font_id = FontId::proportional(10.0);

    let line_height = info_bar_height / 3.0;

    // 第一行：坐标 (POS)
    painter.text(
        Pos2::new(info_rect.min.x + 8.0, info_rect.min.y + line_height * 0.5 + 2.0),
        Align2::LEFT_CENTER,
        format!("POS: {}", coord_text),
        font_id.clone(),
        text_color,
    );

    // 第二行：HEX值 + 颜色块
    let row2_y = info_rect.min.y + line_height * 1.5 + 2.0;
    let hex_galley = painter.layout_no_wrap(format!("HEX: {}", hex_text), font_id.clone(), text_color);
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
    // [修改] 颜色预览块的边框也改成灰色，防止白色预览块融进白底
    painter.rect_stroke(color_preview_rect, 2.0, Stroke::new(1.0, Color32::from_gray(200)), StrokeKind::Outside);

    // 第三行：提示文字
    painter.text(
        Pos2::new(info_rect.min.x + 8.0, info_rect.min.y + line_height * 2.5),
        Align2::LEFT_CENTER,
        text.tooltip_mouse_copy_color,
        hint_font_id,
        hint_color,
    );
}