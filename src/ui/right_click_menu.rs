use eframe::emath::Pos2;
use egui::{Area, Context, Frame, Id, Order, Sense};
use crate::dev_info;
use crate::i18n::{get_text, Language};

pub fn render_context_menu(
    ctx: &Context,
    pos: &mut Option<Pos2>,
    lang: Language
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
                    if ui.button(text.context_copy_image).clicked() {
                        dev_info!("Copy Image clicked");
                        close_menu = true;
                    }
                    if ui.button(text.context_copy_path).clicked() {
                        dev_info!("Copy Image Path clicked");
                        close_menu = true;
                    }
                });
            });

        if close_menu {
            *pos = None;
        }
    }
}
