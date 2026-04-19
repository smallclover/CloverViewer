use crate::feature::screenshot::capture::ScreenshotState;
use crate::i18n::lang::get_i18n_text;
use eframe::egui::ColorImage;
use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};
use egui::epaint::Vertex;

const MAGNIFIER_GRID_SIZE: i32 = 15;
const MAGNIFIER_PIXEL_SIZE: f32 = 10.0;
const MAGNIFIER_CARD_OFFSET: f32 = 20.0;
const MAGNIFIER_INFO_BAR_HEIGHT: f32 = 64.0;
const MAGNIFIER_CARD_CORNER_RADIUS: f32 = 4.0;
const MAGNIFIER_BORDER_COLOR: Color32 = Color32::from_gray(200);
const MAGNIFIER_GRID_LINE_ALPHA: u8 = 80;
const MAGNIFIER_MESH_RESERVE_CELLS: usize = 961;

struct MagnifierPixelContext<'a> {
    image: &'a ColorImage,
    card_pos: Pos2,
    center_phys_x: isize,
    center_phys_y: isize,
    img_width: isize,
    img_height: isize,
    half_grid: i32,
}

struct MagnifierLayout {
    card_pos: Pos2,
    card_rect: Rect,
    magnifier_rect: Rect,
    info_rect: Rect,
}

struct MagnifierSample {
    center_phys_x: isize,
    center_phys_y: isize,
    img_width: isize,
    img_height: isize,
}

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
    for cap in &state.capture.captures {
        let rect = Rect::from_min_size(
            Pos2::new(cap.screen_info.x as f32, cap.screen_info.y as f32),
            eframe::egui::vec2(cap.screen_info.width as f32, cap.screen_info.height as f32),
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
            ppp,
        );

        // 4. 处理颜色复制 (Ctrl + C 或按钮请求)
        // 优先消费 egui 原生 Copy 事件，其次兼容 Command/Ctrl + C。
        let is_copy_shortcut = ui.input(|i| {
            i.events.iter().any(|e| matches!(e, eframe::egui::Event::Copy))
                || (i.modifiers.command && i.key_pressed(eframe::egui::Key::C))
        });

        if state.input.copy_requested || is_copy_shortcut {
            state.input.copy_requested = false;

            let center_phys_x =
                (global_pointer_phys.x - screen.screen_info.x as f32).round() as isize;
            let center_phys_y =
                (global_pointer_phys.y - screen.screen_info.y as f32).round() as isize;
            let img_width = screen.image.width() as isize;
            let img_height = screen.image.height() as isize;

            if center_phys_x >= 0
                && center_phys_x < img_width
                && center_phys_y >= 0
                && center_phys_y < img_height
            {
                let idx = center_phys_y as usize * screen.image.width() + center_phys_x as usize;
                let color = screen.image.pixels[idx];
                let hex_text = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());

                ui.copy_text(hex_text.clone());
                tracing::info!("Color {} copied to clipboard via egui", hex_text);
            }
        }
    }
}

/// 内部绘制放大镜组件的逻辑
fn draw_magnifier_ui(
    ui: &Ui,
    painter: &Painter,
    image: &ColorImage,
    draw_pos: Pos2,   // 决定 UI 画在哪
    sample_pos: Pos2, // 决定从图片哪里取色
    ppp: f32,
) {
    let text = get_i18n_text(ui);
    let half_grid = MAGNIFIER_GRID_SIZE / 2;
    let magnifier_size = MAGNIFIER_GRID_SIZE as f32 * MAGNIFIER_PIXEL_SIZE;
    let info_bar_height = MAGNIFIER_INFO_BAR_HEIGHT;
    let card_size = Vec2::new(magnifier_size, magnifier_size + info_bar_height);
    let layout = resolve_magnifier_layout(ui, draw_pos, card_size, magnifier_size, info_bar_height);
    let sample = sample_image(image, sample_pos, ppp);

    painter.rect_filled(layout.card_rect, MAGNIFIER_CARD_CORNER_RADIUS, Color32::WHITE);
    paint_pixel_grid(painter, image, &layout, &sample, half_grid);
    paint_crosshair(painter, &layout.magnifier_rect, layout.card_pos, half_grid);
    paint_info_panel(painter, &layout.info_rect, text, image, &sample, info_bar_height);
    paint_card_border(painter, layout.card_rect);
}

fn resolve_magnifier_layout(
    ui: &Ui,
    draw_pos: Pos2,
    card_size: Vec2,
    magnifier_size: f32,
    info_bar_height: f32,
) -> MagnifierLayout {
    let offset = Vec2::new(MAGNIFIER_CARD_OFFSET, MAGNIFIER_CARD_OFFSET);
    let mut card_pos = draw_pos + offset;

    let screen_rect = ui.viewport_rect();
    if card_pos.x + card_size.x > screen_rect.max.x {
        card_pos.x = draw_pos.x - offset.x - card_size.x;
    }
    if card_pos.y + card_size.y > screen_rect.max.y {
        card_pos.y = draw_pos.y - offset.y - card_size.y;
    }

    let card_rect = Rect::from_min_size(card_pos, card_size);
    let magnifier_rect = Rect::from_min_size(card_pos, Vec2::new(magnifier_size, magnifier_size));
    let info_rect = Rect::from_min_max(
        Pos2::new(card_rect.min.x, card_rect.max.y - info_bar_height),
        card_rect.max,
    );

    MagnifierLayout {
        card_pos,
        card_rect,
        magnifier_rect,
        info_rect,
    }
}

fn sample_image(image: &ColorImage, sample_pos: Pos2, ppp: f32) -> MagnifierSample {
    MagnifierSample {
        center_phys_x: (sample_pos.x * ppp).round() as isize,
        center_phys_y: (sample_pos.y * ppp).round() as isize,
        img_width: image.width() as isize,
        img_height: image.height() as isize,
    }
}

fn paint_pixel_grid(
    painter: &Painter,
    image: &ColorImage,
    layout: &MagnifierLayout,
    sample: &MagnifierSample,
    half_grid: i32,
) {
    let mut mesh = eframe::egui::Mesh::default();
    mesh.reserve_triangles(MAGNIFIER_MESH_RESERVE_CELLS * 2);
    mesh.reserve_vertices(MAGNIFIER_MESH_RESERVE_CELLS * 4);

    let pixel_context = MagnifierPixelContext {
        image,
        card_pos: layout.card_pos,
        center_phys_x: sample.center_phys_x,
        center_phys_y: sample.center_phys_y,
        img_width: sample.img_width,
        img_height: sample.img_height,
        half_grid,
    };
    paint_magnifier_pixels(painter, &pixel_context, &mut mesh);
    painter.add(eframe::egui::Shape::mesh(mesh));
    paint_grid_lines(painter, layout.card_pos);
}

fn paint_grid_lines(painter: &Painter, card_pos: Pos2) {
    let grid_line_color = Color32::from_rgba_unmultiplied(0, 0, 0, MAGNIFIER_GRID_LINE_ALPHA);
    let grid_line_width = 1.0;

    for dy in 0..MAGNIFIER_GRID_SIZE {
        for dx in 0..MAGNIFIER_GRID_SIZE {
            let x = card_pos.x + dx as f32 * MAGNIFIER_PIXEL_SIZE;
            let y = card_pos.y + dy as f32 * MAGNIFIER_PIXEL_SIZE;

            if dx < MAGNIFIER_GRID_SIZE - 1 {
                painter.line_segment(
                    [
                        Pos2::new(x + MAGNIFIER_PIXEL_SIZE, y),
                        Pos2::new(x + MAGNIFIER_PIXEL_SIZE, y + MAGNIFIER_PIXEL_SIZE),
                    ],
                    Stroke::new(grid_line_width, grid_line_color),
                );
            }

            if dy < MAGNIFIER_GRID_SIZE - 1 {
                painter.line_segment(
                    [
                        Pos2::new(x, y + MAGNIFIER_PIXEL_SIZE),
                        Pos2::new(x + MAGNIFIER_PIXEL_SIZE, y + MAGNIFIER_PIXEL_SIZE),
                    ],
                    Stroke::new(grid_line_width, grid_line_color),
                );
            }
        }
    }
}

fn paint_crosshair(painter: &Painter, magnifier_rect: &Rect, card_pos: Pos2, half_grid: i32) {
    let center_grid_idx = half_grid as f32;
    let center_pixel_rect = Rect::from_min_size(
        card_pos
            + Vec2::new(
                center_grid_idx * MAGNIFIER_PIXEL_SIZE,
                center_grid_idx * MAGNIFIER_PIXEL_SIZE,
            ),
        Vec2::new(MAGNIFIER_PIXEL_SIZE, MAGNIFIER_PIXEL_SIZE),
    );
    painter.rect_stroke(
        center_pixel_rect,
        0.0,
        Stroke::new(1.5, Color32::from_rgb(0, 255, 255)),
        StrokeKind::Outside,
    );

    let center_line_color = Color32::from_rgba_unmultiplied(0, 255, 255, 100);
    painter.line_segment(
        [magnifier_rect.center_top(), magnifier_rect.center_bottom()],
        Stroke::new(1.0, center_line_color),
    );
    painter.line_segment(
        [magnifier_rect.left_center(), magnifier_rect.right_center()],
        Stroke::new(1.0, center_line_color),
    );
}

fn paint_info_panel(
    painter: &Painter,
    info_rect: &Rect,
    text: &crate::i18n::lang::TextBundle,
    image: &ColorImage,
    sample: &MagnifierSample,
    info_bar_height: f32,
) {
    let center_color = sampled_center_color(image, sample);
    let coord_text = format!("({}, {})", sample.center_phys_x, sample.center_phys_y);
    let hex_text = format!(
        "#{:02X}{:02X}{:02X}",
        center_color.r(),
        center_color.g(),
        center_color.b()
    );

    painter.line_segment(
        [info_rect.left_top(), info_rect.right_top()],
        Stroke::new(1.0, Color32::from_gray(230)),
    );

    let text_color = Color32::from_rgb(40, 40, 40);
    let hint_color = Color32::from_gray(150);
    let font_id = FontId::proportional(12.0);
    let hint_font_id = FontId::proportional(10.0);
    let line_height = info_bar_height / 3.0;

    painter.text(
        Pos2::new(
            info_rect.min.x + 8.0,
            info_rect.min.y + line_height * 0.5 + 2.0,
        ),
        Align2::LEFT_CENTER,
        format!("{}{}", text.magnifier.pos, coord_text),
        font_id.clone(),
        text_color,
    );

    let row2_y = info_rect.min.y + line_height * 1.5 + 2.0;
    let hex_galley = painter.layout_no_wrap(
        format!("{}{}", text.magnifier.hex, hex_text),
        font_id.clone(),
        text_color,
    );
    let hex_text_width = hex_galley.size().x;
    painter.galley(
        Pos2::new(info_rect.min.x + 8.0, row2_y - hex_galley.size().y / 2.0),
        hex_galley,
        text_color,
    );

    let color_preview_size = 12.0;
    let color_preview_pos = Pos2::new(
        info_rect.min.x + 8.0 + hex_text_width + 8.0,
        row2_y - color_preview_size / 2.0,
    );
    let color_preview_rect = Rect::from_min_size(
        color_preview_pos,
        Vec2::new(color_preview_size, color_preview_size),
    );
    painter.rect_filled(color_preview_rect, 2.0, center_color);
    painter.rect_stroke(
        color_preview_rect,
        2.0,
        Stroke::new(1.0, Color32::from_gray(200)),
        StrokeKind::Outside,
    );

    painter.text(
        Pos2::new(info_rect.min.x + 8.0, info_rect.min.y + line_height * 2.5),
        Align2::LEFT_CENTER,
        text.tooltip.mouse_copy_color,
        hint_font_id,
        hint_color,
    );
}

fn sampled_center_color(image: &ColorImage, sample: &MagnifierSample) -> Color32 {
    if sample.center_phys_x < 0
        || sample.center_phys_x >= sample.img_width
        || sample.center_phys_y < 0
        || sample.center_phys_y >= sample.img_height
    {
        return Color32::BLACK;
    }

    let center_idx = sample.center_phys_y as usize * image.width() + sample.center_phys_x as usize;
    image.pixels[center_idx]
}

fn paint_card_border(painter: &Painter, card_rect: Rect) {
    painter.rect_stroke(
        card_rect,
        MAGNIFIER_CARD_CORNER_RADIUS,
        Stroke::new(1.0, MAGNIFIER_BORDER_COLOR),
        StrokeKind::Inside,
    );
}

fn paint_magnifier_pixels(
    painter: &Painter,
    context: &MagnifierPixelContext<'_>,
    mesh: &mut eframe::egui::Mesh,
) {
    for dy in -context.half_grid..=context.half_grid {
        for dx in -context.half_grid..=context.half_grid {
            let src_x = context.center_phys_x + dx as isize;
            let src_y = context.center_phys_y + dy as isize;
            let color = if src_x >= 0
                && src_x < context.img_width
                && src_y >= 0
                && src_y < context.img_height
            {
                let idx = src_y as usize * context.image.width() + src_x as usize;
                context.image.pixels[idx]
            } else {
                Color32::BLACK
            };

            let grid_x = (dx + context.half_grid) as f32;
            let grid_y = (dy + context.half_grid) as f32;
            let pixel_rect = Rect::from_min_size(
                context.card_pos
                    + Vec2::new(grid_x * MAGNIFIER_PIXEL_SIZE, grid_y * MAGNIFIER_PIXEL_SIZE),
                Vec2::new(MAGNIFIER_PIXEL_SIZE, MAGNIFIER_PIXEL_SIZE),
            );

            if dy == -context.half_grid && dx == -context.half_grid {
                painter.rect_filled(
                    pixel_rect,
                    egui::CornerRadius {
                        nw: MAGNIFIER_CARD_CORNER_RADIUS as u8,
                        ne: 0,
                        sw: 0,
                        se: 0,
                    },
                    color,
                );
            } else if dy == -context.half_grid && dx == context.half_grid {
                painter.rect_filled(
                    pixel_rect,
                    egui::CornerRadius {
                        nw: 0,
                        ne: MAGNIFIER_CARD_CORNER_RADIUS as u8,
                        sw: 0,
                        se: 0,
                    },
                    color,
                );
            } else {
                let idx = mesh.vertices.len() as u32;
                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx, idx + 2, idx + 3);
                mesh.vertices.push(Vertex {
                    pos: pixel_rect.left_top(),
                    uv: Pos2::ZERO,
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: pixel_rect.right_top(),
                    uv: Pos2::ZERO,
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: pixel_rect.right_bottom(),
                    uv: Pos2::ZERO,
                    color,
                });
                mesh.vertices.push(Vertex {
                    pos: pixel_rect.left_bottom(),
                    uv: Pos2::ZERO,
                    color,
                });
            }
        }
    }
}
