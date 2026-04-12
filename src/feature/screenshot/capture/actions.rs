use crate::feature::screenshot::capture::{ScreenshotAction, ScreenshotState};
use crate::feature::screenshot::draw::draw_skia_shapes_on_image;
use arboard::{Clipboard, ImageData};
use eframe::egui::Rect;
use image::{GenericImage, RgbaImage};
use std::borrow::Cow;
use std::path::PathBuf;
use std::thread;

pub(super) fn handle_save_action(
    final_action: ScreenshotAction,
    screenshot_state: &mut ScreenshotState,
) {
    if final_action == ScreenshotAction::SaveAndClose
        || final_action == ScreenshotAction::SaveToClipboard
    {
        // 利用刚写好的函数瞬间切出图片，再丢给线程去写硬盘/剪贴板
        if let Some(final_image) = extract_cropped_image(screenshot_state) {
            thread::spawn(move || {
                if final_action == ScreenshotAction::SaveAndClose {
                    if let Ok(profile) = std::env::var("USERPROFILE") {
                        let desktop = PathBuf::from(profile).join("Desktop");
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let path = desktop.join(format!("screenshot_{}.png", timestamp));
                        if let Err(e) = final_image.save(&path) {
                            tracing::error!("Save failed: {}", e);
                        } else {
                            tracing::info!("Saved to {:?}", path);
                        }
                    }
                } else if final_action == ScreenshotAction::SaveToClipboard
                    && let Ok(mut clipboard) = Clipboard::new()
                {
                    let image_data = ImageData {
                        width: final_image.width() as usize,
                        height: final_image.height() as usize,
                        bytes: Cow::from(final_image.into_raw()),
                    };
                    if let Err(e) = clipboard.set_image(image_data) {
                        tracing::error!("Failed to copy image to clipboard: {}", e);
                    } else {
                        tracing::info!("Copied image to clipboard.");
                    }
                }
            });
        }
    }
}

pub fn extract_cropped_image(screenshot_state: &ScreenshotState) -> Option<RgbaImage> {
    let selection_phys = screenshot_state.select.selection?;
    if !selection_phys.is_positive() {
        return None;
    }

    let captures_data: Vec<_> = screenshot_state
        .capture
        .captures
        .iter()
        .map(|c| {
            (
                c.raw_image.clone(),
                Rect::from_min_size(
                    egui::pos2(c.screen_info.x as f32, c.screen_info.y as f32),
                    egui::vec2(c.screen_info.width as f32, c.screen_info.height as f32),
                ),
            )
        })
        .collect();

    let final_width = selection_phys.width().round() as u32;
    let final_height = selection_phys.height().round() as u32;
    if final_width == 0 || final_height == 0 {
        return None;
    }

    let mut final_image = RgbaImage::new(final_width, final_height);

    for (raw_image, monitor_rect_phys) in captures_data.iter() {
        let intersection = selection_phys.intersect(*monitor_rect_phys);
        if !intersection.is_positive() {
            continue;
        }

        let crop_x = (intersection.min.x - monitor_rect_phys.min.x)
            .max(0.0)
            .round() as u32;
        let crop_y = (intersection.min.y - monitor_rect_phys.min.y)
            .max(0.0)
            .round() as u32;
        let crop_w = intersection.width().round() as u32;
        let crop_h = intersection.height().round() as u32;

        if crop_x + crop_w > raw_image.width() || crop_y + crop_h > raw_image.height() {
            continue;
        }

        let cropped_part =
            image::imageops::crop_imm(&**raw_image, crop_x, crop_y, crop_w, crop_h).to_image();
        let paste_x = (intersection.min.x - selection_phys.min.x).max(0.0).round() as u32;
        let paste_y = (intersection.min.y - selection_phys.min.y).max(0.0).round() as u32;
        let _ = final_image.copy_from(&cropped_part, paste_x, paste_y);
    }

    draw_skia_shapes_on_image(
        &mut final_image,
        &screenshot_state.edit.shapes,
        selection_phys,
    );
    Some(final_image)
}
