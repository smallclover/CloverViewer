use std::path::PathBuf;
use egui::{Align, Context, Layout, Ui};
use crate::core::business::BusinessData;
use crate::model::state::ViewState;

pub struct ImageProperties {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size: u64,
}

pub fn render_properties_panel(
    ctx: &Context,
    state: &mut ViewState,
) {
    if !state.show_properties_panel {
        return;
    }

    let mut my_is_open = state.show_properties_panel;

    egui::SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Properties");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("x").clicked() {
                        my_is_open = false;
                    }
                });
            });
            ui.separator();

            if let Some(props) = &state.image_properties {
                render_properties_content(ui, props);
            } else {
                ui.label("No image loaded.");
            }
        });

    state.show_properties_panel = my_is_open;
}

fn render_properties_content(ui: &mut Ui, properties: &ImageProperties) {
    egui::Grid::new("properties_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Path:");
            ui.label(properties.path.to_string_lossy().to_string());
            ui.end_row();

            ui.label("Dimensions:");
            ui.label(format!("{} x {}", properties.width, properties.height));
            ui.end_row();

            ui.label("Size:");
            ui.label(format!("{} bytes", properties.size));
            ui.end_row();
        });
}


pub fn update_image_properties(data: &BusinessData, state: &mut ViewState) {
    if let (Some(path), Some(texture)) = (data.current(), data.current_texture.as_ref()) {
        if let Ok(metadata) = std::fs::metadata(&path) {
            let [width, height] = texture.size();
            state.image_properties = Some(ImageProperties {
                path: path.to_path_buf(),
                width: width as u32,
                height: height as u32,
                size: metadata.len(),
            });
        }
    }
}
