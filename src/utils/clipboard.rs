use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    borrow::Cow
};
use egui::{Color32, Context};
use crate::{
    ui::widgets::toast::ToastManager,
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
    // 将 Color32 转为独立的 Vec<u8>，避免跨线程借用 Arc 数据
    let raw_bytes: Vec<u8> = pixels_arc.iter().flat_map(|c| [c.r(), c.g(), c.b(), c.a()]).collect();
    thread::spawn(move || {
        let Ok(mut clipboard) = arboard::Clipboard::new() else {
            eprintln!("[ERROR] 无法初始化剪贴板");
            toast_clone.error(copy_failed_message.to_string());
            return;
        };
        let img_data = arboard::ImageData {
            width,
            height,
            bytes: Cow::Owned(raw_bytes),
        };
        if let Err(e) = clipboard.set_image(img_data) {
            toast_clone.error(format!("{}: {}", copy_failed_message, e));
        } else {
            toast_clone.success(copied_message);
        }
    });
}

pub fn copy_image_path_to_clipboard(ctx: &Context, path: PathBuf, toast_manager: &ToastManager) {
    let text = get_i18n_text(ctx);
    let Ok(mut clipboard) = arboard::Clipboard::new() else {
        eprintln!("[ERROR] 无法初始化剪贴板");
        toast_manager.error(text.copy_failed_message.to_string());
        return;
    };
    let _ = clipboard.set_text(path.to_string_lossy().to_string());
    toast_manager.success(text.copied_message);
}
