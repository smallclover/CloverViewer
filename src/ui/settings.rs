use egui::{Align, ComboBox, Context, Layout, Id};
use crate::i18n::{get_text, Language};
use crate::ui::modal::{ModalAction, ModalFrame};

#[derive(PartialEq, Clone, Copy)]
enum SettingsTab {
    General,
    Appearance,
    Advanced,
}

pub fn render_settings_window(
    ctx: &Context,
    open: &mut bool,
    current_lang: &mut Language,
) {
    let text = get_text(*current_lang);

    let tab_id = Id::new("settings_tab_state");
    let mut current_tab = ctx.data(|d| d.get_temp(tab_id)).unwrap_or(SettingsTab::General);

    ModalFrame::show(ctx, open, text.settings_title, |ui| {
        let mut action = ModalAction::None;

        ui.set_min_width(500.0);
        ui.set_min_height(300.0);

        ui.horizontal(|ui| {
            // Left Side: Tabs
            ui.vertical(|ui| {
                ui.set_width(120.0);
                ui.add_space(4.0);

                if ui.selectable_label(current_tab == SettingsTab::General, text.settings_general).clicked() {
                    current_tab = SettingsTab::General;
                }
                if ui.selectable_label(current_tab == SettingsTab::Appearance, text.settings_appearance).clicked() {
                    current_tab = SettingsTab::Appearance;
                }
                if ui.selectable_label(current_tab == SettingsTab::Advanced, text.settings_advanced).clicked() {
                    current_tab = SettingsTab::Advanced;
                }

                ui.add_space(ui.available_height());
            });

            ui.separator();

            // Right Side: Content
            ui.vertical(|ui| {
                ui.set_min_width(300.0);
                ui.add_space(4.0);

                match current_tab {
                    SettingsTab::General => {
                        ui.heading(text.settings_general);
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", text.settings_language));

                            // Fix for ComboBox issue: Ensure unique ID and proper state handling
                            let mut selected = *current_lang;
                            ComboBox::from_id_salt("lang_selector")
                                .selected_text(selected.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut selected, Language::Zh, Language::Zh.as_str());
                                    ui.selectable_value(&mut selected, Language::En, Language::En.as_str());
                                    ui.selectable_value(&mut selected, Language::Ja, Language::Ja.as_str());
                                });

                            // Only update if changed
                            if selected != *current_lang {
                                *current_lang = selected;
                            }
                        });
                    }
                    SettingsTab::Appearance => {
                        ui.heading(text.settings_appearance);
                        ui.add_space(10.0);
                        ui.label("Appearance settings will be here.");
                    }
                    SettingsTab::Advanced => {
                        ui.heading(text.settings_advanced);
                        ui.add_space(10.0);
                        ui.label("Advanced settings will be here.");
                    }
                }

                ui.add_space(20.0);

                // Push close button to the bottom right of the content area
                ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                    if ui.button(text.settings_close).clicked() {
                        action = ModalAction::Close;
                    }
                });
            });
        });

        // Save the tab state
        ctx.data_mut(|d| d.insert_temp(tab_id, current_tab));

        action
    });
}
