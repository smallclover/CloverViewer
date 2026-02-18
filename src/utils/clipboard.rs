use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::Duration,
    borrow::Cow
};
use egui::{Color32, Context};
use crate::{
    ui::components::toast::ToastManager,
    i18n::lang::get_i18n_text
};

pub fn copy_image_to_clipboard_async(
    ctx: &Context,
    pixels_arc: Arc<Vec<Color32>>,
    width: usize,
    height: usize,
    toast_manager: &ToastManager,
) {
    let text = get_i18n_text(ctx);

    toast_manager.loading(text.coping_message);

    let toast_clone = toast_manager.clone();
    let copied_message = text.copied_message;
    let copy_failed_message = text.copy_failed_message;
    thread::spawn(move || {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        let bytes: &[u8] = bytemuck::cast_slice(&pixels_arc);
        let img_data = arboard::ImageData {
            width,
            height,
            bytes: Cow::Borrowed(bytes),
        };
        thread::sleep(Duration::from_secs(1));
        if let Err(e) = clipboard.set_image(img_data) {
            toast_clone.error(format!("{}: {}", copy_failed_message, e));
        } else {
            toast_clone.success(copied_message);
        }
    });
}

pub fn copy_image_path_to_clipboard(ctx: &Context,path: PathBuf, toast_manager: &ToastManager,){
    let text = get_i18n_text(ctx);
    let mut clipboard = arboard::Clipboard::new().unwrap();
    let _ = clipboard.set_text(path.to_string_lossy().to_string());

    toast_manager.success(text.copied_message);
}
