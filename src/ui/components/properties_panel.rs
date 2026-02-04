use egui::{Align, Context, Layout, Ui};
use crate::core::business::BusinessData;
use crate::i18n::lang::{get_text, TextBundle};
use crate::model::config::Config;
use crate::model::image_meta::ImageProperties;
use crate::model::state::ViewState;

pub fn render_properties_panel(
    ctx: &Context,
    state: &mut ViewState,
    config: &Config,
) {
    if !state.show_properties_panel {
        return;
    }
    let texts = get_text(config.language);
    let mut my_is_open = state.show_properties_panel;

    egui::SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(texts.img_prop);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("X").clicked() {
                        my_is_open = false;
                    }
                });
            });
            ui.separator();

            if let Some(props) = &state.image_properties {
                render_properties_content(ui, props, texts);
            } else {
                ui.label("No image loaded.");
            }
        });

    state.show_properties_panel = my_is_open;
}

/// 图片属性内容
fn render_properties_content(ui: &mut Ui, properties: &ImageProperties, texts: &TextBundle) {
    egui::Grid::new("properties_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(format!("{}:",texts.img_name));
            ui.label(&properties.name);
            ui.end_row();

            ui.label(format!("{}:",texts.img_date));
            ui.label(&properties.date);
            ui.end_row();

            ui.label(format!("{}:",texts.img_dim));
            ui.label(format!("{}x{} {:.1} MB", properties.width, properties.height, (properties.size as f64) / (1024.0 * 1024.0)));
            ui.end_row();

            ui.label(format!("{}:",texts.img_path));
            ui.label(properties.path.to_string_lossy().to_string());
            ui.end_row();
        });
}


pub fn update_image_properties(data: &BusinessData, state: &mut ViewState) {
    if let Some(properties) = data.current_properties.as_ref() {
        state.image_properties = Some(properties.clone());
    }
}
