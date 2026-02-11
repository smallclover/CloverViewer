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
    pub scale_factor: f32,
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

    egui::CentralPanel::default().show(ctx, |ui| {
        let image_widget = egui::Image::new(img_src).fit_to_exact_size(ui.available_size());
        ui.add(image_widget);

        let painter = ui.painter();
        let viewport_rect = ctx.viewport_rect();
        let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);

        // --- 核心修复：使用 input 获取视口信息 (egui 0.33 正确写法) ---
        // ctx.input(|i| i.viewport().inner_rect) 返回当前视口在屏幕上的绝对逻辑坐标
        let viewport_inner_rect = ctx.input(|i| i.viewport().inner_rect);

        let viewport_offset = if let Some(rect) = viewport_inner_rect {
            rect.min
        } else {
            // 降级方案：手动计算 (仅在极少数第一帧未就绪时触发)
            let ppp = ctx.pixels_per_point();
            Pos2::new(
                screen.screen_info.x().unwrap_or(0) as f32 / ppp,
                screen.screen_info.y().unwrap_or(0) as f32 / ppp
            )
        };

        // --- 碰撞检测与输入处理 ---
        let mut local_button_rect = None;
        if let Some(global_button_pos) = state.save_button_pos {
            let local_pos = global_button_pos - viewport_offset.to_vec2();
            local_button_rect = Some(Rect::from_min_size(local_pos, egui::vec2(50.0, 25.0)));
        }

        let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::drag());

        if response.drag_started() {
            if let Some(press_pos) = response.interact_pointer_pos() {
                let is_clicking_button = local_button_rect.map_or(false, |r| r.contains(press_pos));

                if !is_clicking_button {
                    let global_pos = press_pos + viewport_offset.to_vec2();
                    state.drag_start = Some(global_pos);
                    state.save_button_pos = None;
                    needs_repaint = true;
                }
            }
        }

        if response.dragged() {
            if let (Some(drag_start_global), Some(current_pos_local)) = (state.drag_start, ui.input(|i| i.pointer.latest_pos())) {
                let current_pos_global = current_pos_local + viewport_offset.to_vec2();
                let rect = Rect::from_two_pos(drag_start_global, current_pos_global);

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
                    if sel.width() > 5.0 && sel.height() > 5.0 {
                        state.save_button_pos = Some(sel.right_bottom() + egui::vec2(5.0, 5.0));
                    } else {
                        state.selection = None;
                        state.save_button_pos = None;
                    }
                    needs_repaint = true;
                }
            }
        }

        // --- 渲染 ---
        if let Some(global_sel) = state.selection {
            let local_sel = global_sel.translate(-viewport_offset.to_vec2());
            let screen_rect_local = Rect::from_min_size(Pos2::ZERO, viewport_rect.size());
            let clipped_local_sel = local_sel.intersect(screen_rect_local);

            if clipped_local_sel.is_positive() {
                let top = Rect::from_min_max(screen_rect_local.min, Pos2::new(screen_rect_local.max.x, clipped_local_sel.min.y));
                let bottom = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.max.y), screen_rect_local.max);
                let left = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.min.y), Pos2::new(clipped_local_sel.min.x, clipped_local_sel.max.y));
                let right = Rect::from_min_max(Pos2::new(clipped_local_sel.max.x, clipped_local_sel.min.y), Pos2::new(screen_rect_local.max.x, clipped_local_sel.max.y));

                painter.rect_filled(top, 0.0, overlay_color);
                painter.rect_filled(bottom, 0.0, overlay_color);
                painter.rect_filled(left, 0.0, overlay_color);
                painter.rect_filled(right, 0.0, overlay_color);

                painter.rect_stroke(clipped_local_sel, 0.0, Stroke::new(2.0, Color32::from_rgb(255, 0, 0)), StrokeKind::Outside);
            } else {
                painter.rect_filled(viewport_rect, 0.0, overlay_color);
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }

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