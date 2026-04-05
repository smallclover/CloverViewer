use crate::core::business::ViewerState;
use crate::i18n::lang::get_i18n_text;
use crate::model::image_meta::ImageProperties;
use crate::model::mode::OverlayMode;
use crate::ui::widgets::icons::{IconType, draw_icon_button};
use egui::{Align, Color32, Context, CursorIcon, Grid, Layout, RichText, Sense, SidePanel, Ui};

pub fn draw_properties_panel(ctx: &Context, overlay: &mut OverlayMode, viewer: &ViewerState) {
    let mut is_open = matches!(overlay, OverlayMode::Properties);
    if !is_open {
        return;
    }

    let text = get_i18n_text(ctx);

    SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(250.0)
        .min_width(250.0)
        .show(ctx, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) {
                ui.ctx().set_cursor_icon(CursorIcon::Default);
            }

            ui.horizontal(|ui| {
                ui.heading(text.img_prop);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if draw_icon_button(ui, false, IconType::Cancel, 20.0).clicked() {
                        is_open = false;
                    }
                });
            });
            ui.separator();

            if let Some(props) = &viewer.current_properties {
                render_properties_content(ui, props);
            } else {
                ui.label(text.prop_no_image);
            }
        });

    if !is_open {
        *overlay = OverlayMode::None;
    }
}

fn render_properties_content(ui: &mut Ui, properties: &ImageProperties) {
    let text = get_i18n_text(ui.ctx());

    // 侧边栏基础属性
    Grid::new("basic_properties_grid")
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label(format!("{}:", text.img_name));
            ui.add(egui::Label::new(&properties.name).wrap())
                .on_hover_text(&properties.name);
            ui.end_row();

            ui.label(format!("{}:", text.img_date));
            ui.label(&properties.date);
            ui.end_row();

            ui.label(format!("{}:", text.img_dim));
            ui.label(format!(
                "{}x{} {:.1} MB",
                properties.width,
                properties.height,
                (properties.size as f64) / (1024.0 * 1024.0)
            ));
            ui.end_row();
        });

    ui.add_space(10.0);

    // 侧边栏路径
    ui.label(format!("{}:", text.img_path));
    ui.horizontal(|ui| {
        let path_str = properties.path.to_string_lossy().to_string();
        // 预留复制按钮的空间
        let label_width = ui.available_width() - 30.0;

        ui.allocate_ui_with_layout(
            egui::vec2(label_width, 0.0),
            Layout::top_down(Align::Min),
            |ui| {
                let label = egui::Label::new(
                    RichText::new(&path_str).color(Color32::from_rgb(200, 50, 50)),
                )
                .wrap()
                .sense(Sense::click());

                let response = ui.add(label).on_hover_text(&path_str);

                if response.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                }

                if response.clicked() {
                    let _ = std::process::Command::new("explorer")
                        .arg("/select,")
                        .arg(&path_str)
                        .spawn();
                }
            },
        );

        // 水平布局将此按钮垂直居中于路径文本块
        if draw_icon_button(ui, false, IconType::SaveToClipboard, 20.0).clicked() {
            ui.ctx().copy_text(path_str);
        }
    });
}
