use egui::Context;
use crate::i18n::lang::get_i18n_text;
use crate::ui::components::modal::{ModalAction, ModalFrame};
///关于窗口
pub fn render_about_window(ctx: &Context, open: &mut bool) {

    let text = get_i18n_text(ctx);

    ModalFrame::show(ctx, open, text.about_title, |ui| {
        let mut action = ModalAction::None;

        ui.vertical_centered(|ui| {
            ui.heading("CloverViewer");
            ui.label(text.about_desc);
            ui.add_space(8.0);
            ui.hyperlink_to(
                text.about_github,
                "https://github.com/smallclover/CloverViewer",
            );
            ui.add_space(12.0);

            if ui.button(text.about_close).clicked() {
                action = ModalAction::Close;
            }
        });

        action
    });

}
