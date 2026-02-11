use eframe::egui::{self, ColorImage, Rect, TextureHandle, Color32, Stroke, Pos2, StrokeKind};
use image::RgbaImage;
use std::sync::Arc;
use xcap::Monitor;

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotAction {
    None,
    Close,
    SaveAndClose,
}

pub struct ScreenshotState {
    pub is_active: bool,
    pub captures: Vec<CapturedScreen>,
    // 全局物理坐标 (Physical Pixels)
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
    pub save_button_pos: Option<Pos2>,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        Self {
            is_active: false,
            captures: vec![],
            selection: None,
            drag_start: None,
            save_button_pos: None,
        }
    }
}

#[derive(Clone)]
pub struct CapturedScreen {
    pub raw_image: Arc<RgbaImage>,
    pub image: ColorImage,
    pub screen_info: Monitor,
    pub texture: Option<TextureHandle>,
}

pub fn draw_screenshot_ui(
    ctx: &eframe::egui::Context,
    state: &mut ScreenshotState,
    screen_index: usize,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let screen = &mut state.captures[screen_index];

    let texture: &TextureHandle = screen.texture.get_or_insert_with(|| {
        ctx.load_texture(
            format!("screenshot_{}", screen.screen_info.name().unwrap_or_default()),
            screen.image.clone(),
            Default::default(),
        )
    });
    let img_src = (texture.id(), texture.size_vec2());

    let mut needs_repaint = false;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(0.0)) // <--- 关键：设置内边距为 0
        .show(ctx, |ui| {
        let image_widget = egui::Image::new(img_src).fit_to_exact_size(ui.available_size());
        ui.add(image_widget);

        let painter = ui.painter();
        let viewport_rect = ctx.viewport_rect();
        let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);
        let full_rect = ui.max_rect();

        // --- 1. 获取物理基准信息 ---
        let screen_x = screen.screen_info.x().unwrap_or(0) as f32;
        let screen_y = screen.screen_info.y().unwrap_or(0) as f32;
        // 当前屏幕的物理起点
        let screen_offset_phys = Pos2::new(screen_x, screen_y);
        // 当前逻辑点距
        let ppp = ctx.pixels_per_point();

        // --- 2. 输入处理：将 局部逻辑坐标 -> 全局物理坐标 ---

        // 计算按钮的局部逻辑区域
        let mut local_button_rect = None;
        if let Some(global_button_pos_phys) = state.save_button_pos {
            // 修复：先计算物理向量差 (Pos2 - Pos2 -> Vec2)，再除以 DPI
            let vec_phys = global_button_pos_phys - screen_offset_phys;
            let local_pos_logical = Pos2::ZERO + (vec_phys / ppp);
            local_button_rect = Some(Rect::from_min_size(local_pos_logical, egui::vec2(50.0, 25.0)));
        }

        let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::drag());

        if response.drag_started() {
            if let Some(press_pos) = response.interact_pointer_pos() { // press_pos 是局部逻辑坐标
                let is_clicking_button = local_button_rect.map_or(false, |r| r.contains(press_pos));

                if !is_clicking_button {
                    // 局部逻辑 -> 物理向量
                    let local_vec_phys = press_pos.to_vec2() * ppp;
                    // 屏幕物理起点 + 物理向量 = 全局物理坐标
                    let global_phys = screen_offset_phys + local_vec_phys;

                    state.drag_start = Some(global_phys);
                    state.save_button_pos = None;
                    needs_repaint = true;
                }
            }
        }

        if response.dragged() {
            if let (Some(drag_start_phys), Some(curr_pos_local)) = (state.drag_start, ui.input(|i| i.pointer.latest_pos())) {
                // 转换当前鼠标位置为全局物理坐标
                let local_vec_phys = curr_pos_local.to_vec2() * ppp;
                let current_pos_phys = screen_offset_phys + local_vec_phys;

                let rect = Rect::from_two_pos(drag_start_phys, current_pos_phys);

                if state.selection.map_or(true, |s| s != rect) {
                    state.selection = Some(rect);
                }
                needs_repaint = true;
            }
        }

        if response.drag_stopped() {
            if state.drag_start.is_some() {
                state.drag_start = None;
                if let Some(sel) = state.selection {
                    // 物理像素大于 10 才显示
                    if sel.width() > 10.0 && sel.height() > 10.0 {
                        // 按钮位置也存为物理坐标
                        // 这里加的 (10.0, 10.0) 是物理像素偏移
                        state.save_button_pos = Some(sel.right_bottom() + egui::vec2(10.0, 10.0));
                    } else {
                        state.selection = None;
                        state.save_button_pos = None;
                    }
                    needs_repaint = true;
                }
            }
        }

        // --- 3. 渲染：将 全局物理坐标 -> 局部逻辑坐标 ---
        if let Some(global_sel_phys) = state.selection {
            // 修复：计算相对于本屏幕物理起点的向量 (Vec2)
            let vec_min = global_sel_phys.min - screen_offset_phys;
            let vec_max = global_sel_phys.max - screen_offset_phys;

            // 转换为逻辑向量并加到局部原点
            let local_logical_rect = Rect::from_min_max(
                Pos2::ZERO + (vec_min / ppp),
                Pos2::ZERO + (vec_max / ppp),
            );

            // 计算交集并绘制
            let screen_rect_local = Rect::from_min_size(Pos2::ZERO, viewport_rect.size());
            let clipped_local_sel = local_logical_rect.intersect(screen_rect_local);

            if clipped_local_sel.is_positive() {
                let top = Rect::from_min_max(screen_rect_local.min, Pos2::new(screen_rect_local.max.x, clipped_local_sel.min.y));
                let bottom = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.max.y), screen_rect_local.max);
                let left = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.min.y), Pos2::new(clipped_local_sel.min.x, clipped_local_sel.max.y));
                let right = Rect::from_min_max(Pos2::new(clipped_local_sel.max.x, clipped_local_sel.min.y), Pos2::new(screen_rect_local.max.x, clipped_local_sel.max.y));

                painter.rect_filled(top, 0.0, overlay_color);
                painter.rect_filled(bottom, 0.0, overlay_color);
                painter.rect_filled(left, 0.0, overlay_color);
                painter.rect_filled(right, 0.0, overlay_color);

                painter.rect_stroke(clipped_local_sel, 0.0, Stroke::new(2.0, Color32::GREEN), StrokeKind::Outside);
            } else {
                painter.rect_filled(viewport_rect, 0.0, overlay_color);
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }

        // --- 在所有内容绘制完毕后，画绿色边框 ---
        // 这样能保证边框浮在图片之上，且不受 Layout 影响
        let border_width = 5.0; // 你想要的宽度
        painter.rect_stroke(
            full_rect,
            0.0,
            Stroke::new(border_width, Color32::GREEN),
            StrokeKind::Inside // <--- 关键：向内描边，保证四边等宽且不被裁剪
        );

        if let Some(button_rect) = local_button_rect {
            if viewport_rect.intersects(button_rect) {
                let save_button = ui.put(button_rect, egui::Button::new("Save"));
                if save_button.clicked() {
                    action = ScreenshotAction::SaveAndClose;
                }
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            action = ScreenshotAction::Close;
        }
    });

    if needs_repaint {
        ctx.request_repaint();
    }

    action
}