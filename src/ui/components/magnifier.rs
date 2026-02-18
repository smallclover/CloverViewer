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
    // 下面的参数可以调节放大倍率和显示状态
    //>>
    let pixel_grid_size = 60;
    //<<
    let zoom_pixel_size = 3.0;
    let half_grid = pixel_grid_size / 2;

    let magnifier_size = pixel_grid_size as f32 * zoom_pixel_size;

    // [修改点 1] 增加信息栏高度，以容纳提示文字 (46.0 -> 64.0)
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
    painter.rect_filled(card_rect, 4.0, Color32::from_rgb(30, 30, 30));
    painter.rect_stroke(
        card_rect,
        4.0,
        Stroke::new(1.0, Color32::from_gray(100)),
        StrokeKind::Inside
    );

    // --- 4. 绘制上半部分：像素放大镜 ---
    // ... (这部分像素绘制逻辑保持不变，为了节省篇幅省略，请保留原有的网格遍历代码) ...
    let magnifier_rect = Rect::from_min_size(card_pos, Vec2::new(magnifier_size, magnifier_size));
    let center_phys_x = (pointer_pos.x * ppp).round() as isize;
    let center_phys_y = (pointer_pos.y * ppp).round() as isize;
    let img_width = image.width() as isize;
    let img_height = image.height() as isize;

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
            painter.rect_filled(pixel_rect, 0.0, color);
        }
    }

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
        Stroke::new(1.0, Color32::from_gray(60))
    );

    let coord_text = format!("({}, {})", center_phys_x, center_phys_y);
    let hex_text = format!("#{:02X}{:02X}{:02X}", center_color.r(), center_color.g(), center_color.b());

    let text_color = Color32::WHITE;
    let hint_color = Color32::from_gray(180); // 提示文字用灰色
    let font_id = FontId::proportional(12.0);
    // 提示文字稍微小一点
    let hint_font_id = FontId::proportional(10.0);

    // 布局计算：将信息栏分为三行 (大概)
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
    painter.rect_stroke(color_preview_rect, 2.0, Stroke::new(1.0, Color32::WHITE), StrokeKind::Inside);

    // [修改点 2] 第三行：提示文字
    painter.text(
        Pos2::new(info_rect.min.x + 8.0, info_rect.min.y + line_height * 2.5),
        Align2::LEFT_CENTER,
        text.tooltip_mouse_copy_color,
        hint_font_id,
        hint_color,
    );
}