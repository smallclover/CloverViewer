use std::sync::Arc;
use eframe::emath::Pos2;
use egui::{Area, Color32, Context, Frame, Id, Order, Sense};
use crate::core::navigator::Navigator;
use crate::dev_info;
use crate::i18n::{get_text, Language, TextBundle};
use crate::ui::toast::ToastManager;

pub fn render_context_menu(
    ctx: &Context,
    pos: &mut Option<Pos2>,
    lang: Language,
    nav: &Navigator,
    current_texture: Option<&egui::TextureHandle>,
    raw_pixels: Option<Arc<Vec<Color32>>>, // 传入保存的原始数据
    toast_manager: &ToastManager
) {
    if let Some(position) = pos {
        let text = get_text(lang);
        let mut close_menu = false;

        // 1. 绘制一个全屏的透明遮罩层，用于捕获点击并关闭菜单
        // 它的 Order 必须比菜单低，但比主界面高
        // 菜单通常在 Foreground，我们可以把遮罩放在 Middle 或者 Foreground-1
        // 但 egui 的 Order 比较简单。
        // 我们可以先画遮罩，再画菜单。因为它们都是 Area，后画的在上面（如果 Order 相同）。

        // 使用一个覆盖全屏的 Area
        Area::new(Id::new("context_menu_mask"))
            .order(Order::Foreground) // 和菜单同一层级，但先画，所以在下面
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                // 分配整个屏幕的空间
                let screen_rect = ctx.input(|i| i.content_rect());
                let response = ui.allocate_rect(screen_rect, Sense::click());
                if response.clicked_by(egui::PointerButton::Primary) {
                    close_menu = true;
                }
            });

        // 2. 绘制实际的菜单
        Area::new(Id::new("context_menu"))
            .order(Order::Foreground) // 也是 Foreground，后画，所以在遮罩上面
            .fixed_pos(*position)
            .show(ctx, |ui| {
                Frame::menu(ui.style()).show(ui, |ui| {
                    ui.set_min_width(120.0);
                    if ui.button(text.context_menu_copy).clicked() {
                        dev_info!("Copy Image clicked");
                        if let (Some(tex), Some(pixels)) = (current_texture, raw_pixels) {
                            let [w, h] = tex.size();
                            copy_image_to_clipboard_async(pixels, w, h, toast_manager, text);
                        }
                        close_menu = true;
                    }
                    if ui.button(text.context_menu_copy_path).clicked() {
                        dev_info!("Copy Image Path clicked");
                        // ... 剪贴板逻辑 ...
                        if let Some(path) = nav.current() {
                            let mut clipboard = arboard::Clipboard::new().unwrap();
                            let _ = clipboard.set_text(path.to_string_lossy().to_string());
                        }
                        toast_manager.success(text.copied_message);

                        close_menu = true;
                    }
                });
            });

        if close_menu {
            *pos = None;
        }
    }
}

// 使用 bytemuck 实现零拷贝转换
pub fn copy_image_to_clipboard_async(
    pixels_arc: Arc<Vec<Color32>>,
    width: usize,
    height: usize,
    toast_manager: &ToastManager,
    text: &TextBundle,
) {
    // 1. 立即给一个反馈，防止用户觉得卡顿
    toast_manager.info("正在处理复制...");

    let toast_clone = toast_manager.clone();
    let copied_message = text.copied_message;
    let copy_failed_message = text.copy_failed_message;
    std::thread::spawn(move || {
        let mut clipboard = arboard::Clipboard::new().unwrap();

        // 极速转换：&[Color32] -> &[u8]
        let bytes: &[u8] = bytemuck::cast_slice(&pixels_arc);

        let img_data = arboard::ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Borrowed(bytes),
        };

        std::thread::sleep(std::time::Duration::from_secs(1));
        if let Err(e) = clipboard.set_image(img_data) {

            toast_clone.error(format!("{}: {}", copy_failed_message, e));
        }else{
            toast_clone.success(copied_message);
        }
    });
}