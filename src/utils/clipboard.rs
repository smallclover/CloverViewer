use crate::{i18n::lang::get_i18n_text, ui::widgets::toast::ToastManager};
use egui::{Color32, Context};
use std::{borrow::Cow, path::PathBuf, sync::Arc, thread};

pub fn copy_image_to_clipboard_async(
    ctx: &Context,
    pixels_arc: Arc<Vec<Color32>>,
    width: usize,
    height: usize,
    toast_manager: &ToastManager,
) {
    let text = get_i18n_text(ctx);

    toast_manager.loading(text.copying_message);

    let toast_clone = toast_manager.clone();
    let copied_message = text.copied_message;
    let copy_failed_message = text.copy_failed_message;
    // 将 Color32 直传底层 [u8] 数组，极大提升高分辨率图片的剪贴板复制性能
    let raw_bytes: Vec<u8> = bytemuck::cast_slice(&pixels_arc).to_vec();
    thread::spawn(move || {
        let Ok(mut clipboard) = arboard::Clipboard::new() else {
            tracing::error!("{}", copy_failed_message);
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
        tracing::error!("{}", text.copy_failed_message);
        toast_manager.error(text.copy_failed_message.to_string());
        return;
    };
    let _ = clipboard.set_text(path.to_string_lossy().to_string());
    toast_manager.success(text.copied_message);
}
