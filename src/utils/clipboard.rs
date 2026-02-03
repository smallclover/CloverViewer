use std::sync::Arc;
use egui::Color32;
use crate::{
    i18n::lang::TextBundle,
    ui::components::toast::ToastManager,
};

pub fn copy_image_to_clipboard_async(
    pixels_arc: Arc<Vec<Color32>>,
    width: usize,
    height: usize,
    toast_manager: &ToastManager,
    text: &'static TextBundle,
) {
    toast_manager.loading(text.coping_message);

    let toast_clone = toast_manager.clone();
    let copied_message = text.copied_message;
    let copy_failed_message = text.copy_failed_message;
    std::thread::spawn(move || {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        let bytes: &[u8] = bytemuck::cast_slice(&pixels_arc);
        let img_data = arboard::ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Borrowed(bytes),
        };
        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Err(e) = clipboard.set_image(img_data) {
            toast_clone.error(format!("{}: {}", copy_failed_message, e));
        } else {
            toast_clone.success(copied_message);
        }
    });
}
