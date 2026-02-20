use egui::{Align2, Area, Color32, Context, Id, Order, RichText, Sense, Ui, LayerId};
/// 弹窗基类
pub struct ModalFrame;

#[derive(PartialEq, Clone, Copy)]
pub enum ModalAction {
    None,
    Close,
    Apply,
}

impl ModalFrame {
    pub fn show(
        ctx: &Context,
        open: &mut bool,
        title: &str,
        add_contents: impl FnOnce(&mut Ui) -> ModalAction,
    ) {
        if !*open {
            return;
        }

        let window_id = Id::new(title);

        // 1. 绘制遮罩层 (Backdrop)
        let screen_rect = ctx.content_rect();

        // 使用 Middle 层级，这样可以覆盖住位于 Middle 层级的普通窗口
        let mask_layer_id = LayerId::new(Order::Middle, Id::new(title).with("mask"));
        let painter = ctx.layer_painter(mask_layer_id);

        // 绘制半透明黑色背景
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(160));

        // 使用一个不显示任何内容的 Area 来拦截背景点击
        let interceptor_id = Id::new(title).with("interceptor");
        Area::new(interceptor_id)
            .fixed_pos(screen_rect.min)
            .order(Order::Middle) // 也是 Middle
            .interactable(true)
            .show(ctx, |ui| {
                ui.allocate_rect(screen_rect, Sense::click());
            });

        // 确保遮罩在 Middle 层级的最上方 (覆盖其他普通窗口)
        ctx.move_to_top(mask_layer_id);
        ctx.move_to_top(LayerId::new(Order::Middle, interceptor_id));

        // 2. 窗口逻辑 (Window)
        let mut window_open = *open;
        let mut action_from_content = ModalAction::None;
        let mut esc_pressed = false;

        // 将窗口设置为 Foreground 层级，确保它永远在 Middle 层级(遮罩)之上
        let window_response = egui::Window::new(RichText::new(title).strong())
            .id(window_id)
            .open(&mut window_open)
            .collapsible(false)
            .resizable(false)
            .order(Order::Foreground) // 关键修改：提升窗口层级
            .pivot(Align2::CENTER_CENTER)
            .default_pos(screen_rect.center())
            .show(ctx, |ui| {
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    esc_pressed = true;
                }
                action_from_content = add_contents(ui);
                action_from_content
            });

        // 3. 安全地提取返回值
        if let Some(inner_r) = window_response {
            if let Some(action_content) = inner_r.inner{
                action_from_content = action_content;
            }
        }

        // 4. 统一同步状态
        if !window_open || action_from_content == ModalAction::Close || esc_pressed {
            *open = false;
        }

        if action_from_content == ModalAction::Apply {
            *open = false;
        }
    }
}
