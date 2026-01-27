use egui::{Align, ComboBox, Context, Layout};
use crate::i18n::{get_text, Language};
use crate::ui::modal::{ModalAction, ModalFrame};

pub fn render_settings_window(
    ctx: &Context,
    open: &mut bool,
    current_lang: &mut Language,
) {
    let text = get_text(*current_lang);

    ModalFrame::show(ctx, open, text.settings_title, |ui| {
        let mut action = ModalAction::None;

        ui.set_min_width(300.0);

        ui.horizontal(|ui| {
            ui.label(format!("{}:", text.settings_language));

            ComboBox::from_id_salt("lang_selector")
                .selected_text(current_lang.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(current_lang, Language::Zh, Language::Zh.as_str());
                    ui.selectable_value(current_lang, Language::En, Language::En.as_str());
                    ui.selectable_value(current_lang, Language::Ja, Language::Ja.as_str());
                });
        });

        ui.add_space(20.0);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button(text.settings_close).clicked() {
                action = ModalAction::Close;
            }
        });

        action
    });

}
