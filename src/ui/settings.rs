use egui::{
    Context, Window, Align2, ComboBox,
    Layout, Align
};
use crate::i18n::{get_text, Language};

pub fn render_settings_window(
    ctx: &Context,
    show_settings: &mut bool,
    current_lang: &mut Language,
) {
    if !*show_settings {
        return;
    }

    let text = get_text(*current_lang);
    let mut open = *show_settings;
    let mut should_close = false;

    Window::new(text.settings_title)
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
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

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                if ui.button(text.settings_close).clicked() {
                    should_close = true;
                }
            });
        });

    *show_settings = open && !should_close;
}
