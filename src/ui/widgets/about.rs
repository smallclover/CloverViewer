use crate::i18n::lang::get_i18n_text;
use crate::ui::widgets::modal::{ModalAction, ModalFrame};
use egui::Context;
///关于窗口
pub fn render_about_window(ctx: &Context, open: &mut bool) {
    let text = get_i18n_text(ctx);

    ModalFrame::show(ctx, open, text.about.title, |ui| {
        let mut action = ModalAction::None;

        ui.vertical_centered(|ui| {
            ui.heading("CloverViewer");
            ui.label(text.about.description);
            ui.add_space(8.0);
            ui.hyperlink_to(
                text.about.github,
                "https://github.com/smallclover/CloverViewer",
            );
            ui.add_space(12.0);
            // 改进后的代码
            let thankful_text =
                format!("{}\n{}", text.about.thankful_head, text.about.thankful_main);

            // 使用 RichText 将字体设置为斜体，并用 weak() 稍微降低一点透明度/对比度
            ui.label(egui::RichText::new(thankful_text).italics().weak());
            ui.add_space(24.0);

            if ui.button(text.about.close).clicked() {
                action = ModalAction::Close;
            }
        });

        action
    });
}
