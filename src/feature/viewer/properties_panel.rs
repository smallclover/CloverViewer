use super::viewer_state::ViewerState;
use crate::i18n::lang::get_i18n_text;
use crate::model::image_meta::ImageProperties;
use crate::model::mode::PanelMode;
use crate::ui::widgets::icons::{IconType, draw_icon_button};
use egui::{Align, CursorIcon, Grid, Layout, Panel, RichText, Sense, Ui};

pub fn draw_properties_panel_inside(ui: &mut Ui, panel: &mut PanelMode, viewer: &ViewerState) {
    let mut is_open = matches!(panel, PanelMode::Properties);
    if !is_open {
        return;
    }

    let text = get_i18n_text(ui);

    Panel::right("properties_panel")
        .resizable(false)
        .default_size(300.0)
        .min_size(300.0)
        .show_inside(ui, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) {
                ui.set_cursor_icon(CursorIcon::Default);
            }

            ui.horizontal(|ui| {
                ui.heading(text.image.properties);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if draw_icon_button(ui, false, IconType::Cancel, 20.0).clicked() {
                        is_open = false;
                    }
                });
            });
            ui.separator();

            if let Some(props) = &viewer.current.properties {
                render_properties_content(ui, props);
            } else {
                ui.label(text.properties.no_image);
            }
        });

    if !is_open {
        *panel = PanelMode::None;
    }
}

fn render_properties_content(ui: &mut Ui, properties: &ImageProperties) {
    let text = get_i18n_text(ui);

    // 侧边栏基础属性
    Grid::new("basic_properties_grid")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .min_col_width(70.0)
        .show(ui, |ui| {
            ui.label(format!("{}:", text.image.name));
            ui.add(egui::Label::new(&properties.name).wrap())
                .on_hover_text(&properties.name);
            ui.end_row();

            ui.label(format!("{}:", text.image.date));
            ui.label(&properties.date);
            ui.end_row();

            ui.label(format!("{}:", text.image.dimensions));
            ui.label(format!("{}x{}", properties.width, properties.height));
            ui.end_row();

            ui.label(format!("{}:", text.image.file_size));
            ui.label(format!(
                "{:.1} MB",
                (properties.size as f64) / (1024.0 * 1024.0)
            ));
            ui.end_row();
        });

    ui.add_space(10.0);

    // 侧边栏路径
    ui.label(format!("{}:", text.image.path));
    ui.horizontal(|ui| {
        let path_str = properties.path.to_string_lossy().to_string();
        let link_color = ui.visuals().hyperlink_color;
        // 预留复制按钮的空间
        let label_width = ui.available_width() - 30.0;

        ui.allocate_ui_with_layout(
            egui::vec2(label_width, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                let label = egui::Label::new(RichText::new(&path_str).color(link_color))
                    .wrap()
                    .sense(Sense::click());

                let response = ui.add(label).on_hover_text(&path_str);

                if response.hovered() {
                    ui.set_cursor_icon(CursorIcon::PointingHand);
                }

                if response.clicked() {
                    let _ = std::process::Command::new("explorer")
                        .arg("/select,")
                        .arg(&path_str)
                        .spawn();
                }
            },
        );

        // 复制按钮
        if draw_icon_button(ui, false, IconType::Copy, 20.0).clicked() {
            ui.copy_text(path_str);
        }
    });
}
