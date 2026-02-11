use eframe::egui::{self, ColorImage, Rect, TextureHandle, Painter, Color32, Stroke, Pos2, StrokeKind};
use xcap::Monitor;
use image::RgbaImage;

// 1. selection 和 drag_start 从这里移除
pub struct ScreenshotState {
    pub is_active: bool,
    pub captures: Vec<CapturedScreen>,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        Self {
            is_active: false,
            captures: vec![],
        }
    }
}

#[derive(Clone)]
pub struct CapturedScreen {
    pub image: ColorImage,
    pub screen_info: Monitor,
    pub texture: Option<TextureHandle>,
    // 2. 将 selection 和 drag_start 添加到这里
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
}

// 3. 修改函数签名，现在只接收 screen 自身的可变引用
pub fn draw_screenshot_ui(
    ctx: &eframe::egui::Context,
    screen: &mut CapturedScreen,
) -> bool {
    let mut wants_to_close = false;

    let texture: &TextureHandle = screen.texture.get_or_insert_with(|| {
        ctx.load_texture(
            format!("screenshot_{}", screen.screen_info.name().unwrap()),
            screen.image.clone(),
            Default::default(),
        )
    });
    let img_src = (texture.id(), texture.size_vec2());
    egui::CentralPanel::default().show(ctx, |ui| {

        let image_widget = egui::Image::new(img_src).fit_to_exact_size(ui.available_size());
        ui.add(image_widget);

        let painter = ui.painter();
        let viewport_rect = ctx.viewport_rect();
        let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);

        // 4. 所有对 selection 和 drag_start 的访问都改为 screen.xxx
        ui.input(|i| {
            if i.pointer.primary_down() {
                screen.drag_start = i.pointer.press_origin();
            }
            if i.pointer.is_decidedly_dragging() && screen.drag_start.is_some() {
                if let Some(current_pos) = i.pointer.latest_pos() {
                    screen.selection = Some(Rect::from_two_pos(screen.drag_start.unwrap(), current_pos));
                }
            }
            if i.pointer.primary_released() {
                screen.drag_start = None;
            }
        });

        if let Some(sel) = screen.selection {
            let top = Rect::from_min_max(viewport_rect.min, Pos2::new(viewport_rect.max.x, sel.min.y));
            let bottom = Rect::from_min_max(Pos2::new(viewport_rect.min.x, sel.max.y), viewport_rect.max);
            let left = Rect::from_min_max(Pos2::new(viewport_rect.min.x, sel.min.y), Pos2::new(sel.min.x, sel.max.y));
            let right = Rect::from_min_max(Pos2::new(sel.max.x, sel.min.y), Pos2::new(viewport_rect.max.x, sel.max.y));

            painter.rect_filled(top, 0.0, overlay_color);
            painter.rect_filled(bottom, 0.0, overlay_color);
            painter.rect_filled(left, 0.0, overlay_color);
            painter.rect_filled(right, 0.0, overlay_color);

            painter.rect_stroke(sel, 0.0, Stroke::new(2.0, Color32::from_rgb(255, 0, 0)), StrokeKind::Outside);

            let button_pos = Pos2::new(sel.right() + 5.0, sel.bottom() + 5.0);
            let button_rect = Rect::from_min_size(button_pos, egui::vec2(50.0, 25.0));

            let save_button = ui.put(button_rect, egui::Button::new("Save"));
            if save_button.clicked() {
                let screen_image = screen.image.clone();
                let selection = screen.selection;
                let screen_info_name = screen.screen_info.name().unwrap();

                std::thread::spawn(move || {
                    if let Some(sel) = selection {
                        let (x, y, w, h) = (
                            sel.min.x as u32,
                            sel.min.y as u32,
                            sel.width() as u32,
                            sel.height() as u32,
                        );

                        let sub_image = RgbaImage::from_raw(
                            screen_image.width() as u32,
                            screen_image.height() as u32,
                            screen_image.pixels.iter().flat_map(|p| p.to_array()).collect(),
                        ).unwrap();

                        let cropped = image::imageops::crop_imm(&sub_image, x, y, w, h).to_image();

                        if let Ok(profile) = std::env::var("USERPROFILE") {
                            let desktop = format!("{}/Desktop", profile);
                            let path = format!("{}/screenshot_{}.png", desktop, screen_info_name);
                            cropped.save(path).ok();
                        }
                    }
                });

                wants_to_close = true;
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            wants_to_close = true;
        }
    });

    wants_to_close
}
