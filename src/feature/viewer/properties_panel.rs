use egui::{Align, Context, Layout, Ui, CursorIcon, SidePanel, Grid};
use crate::core::business::ViewerState;
use crate::i18n::lang::{get_i18n_text};
use crate::model::image_meta::ImageProperties;
use crate::model::mode::OverlayMode;
use crate::ui::widgets::icons::{draw_icon_button, IconType};

pub fn draw_properties_panel(
    ctx: &Context,
    overlay: &mut OverlayMode,
    viewer: &ViewerState,
) {
    let mut is_open = matches!(overlay, OverlayMode::Properties);
    if !is_open {
        return;
    }

    let text = get_i18n_text(ctx);

    SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) {
                ui.ctx().set_cursor_icon(CursorIcon::Default);
            }

            ui.horizontal(|ui| {
                ui.heading(text.img_prop);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if draw_icon_button(ui, false, IconType::Cancel,20.0).clicked() {
                        is_open = false;
                    }
                });
            });
            ui.separator();

            if let Some(props) = &viewer.current_properties {
                render_properties_content(ui, props);
            } else {
                ui.label("No image loaded.");
            }
        });

    if !is_open {
        *overlay = OverlayMode::None;
    }
}

fn render_properties_content(ui: &mut Ui, properties: &ImageProperties) {

    let text = get_i18n_text(ui.ctx());

    Grid::new("properties_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .show(ui, |ui| {
            ui.label(format!("{}:",text.img_name));
            ui.label(&properties.name);
            ui.end_row();

            ui.label(format!("{}:",text.img_date));
            ui.label(&properties.date);
            ui.end_row();

            ui.label(format!("{}:",text.img_dim));
            ui.label(format!("{}x{} {:.1} MB", properties.width, properties.height, (properties.size as f64) / (1024.0 * 1024.0)));
            ui.end_row();

            ui.label(format!("{}:",text.img_path));
            ui.horizontal(|ui| {
                let path_str = properties.path.to_string_lossy().to_string();
                ui.add(egui::Label::new(&path_str).wrap());

                if draw_icon_button(ui, false, IconType::SaveToClipboard, 20.0).clicked() {
                    ui.ctx().copy_text(path_str);
                }
            });
            ui.end_row();
        });
}