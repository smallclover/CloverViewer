use egui::{Align, Context, Layout, Ui, CursorIcon};
use crate::core::business::BusinessData;
use crate::i18n::lang::{get_i18n_text, TextBundle};
use crate::model::image_meta::ImageProperties;
use crate::ui::components::ui_mode::UiMode;
use crate::model::state::ViewState;

pub fn draw_properties_panel(
    ctx: &Context,
    state: &mut ViewState,
    data: &BusinessData,
) {
    // 检查当前 UI 模式是否为 Properties
    let mut is_open = matches!(state.ui_mode, UiMode::Properties);
    if !is_open {
        return;
    }

    let text = get_i18n_text(ctx);

    egui::SidePanel::right("properties_panel")
        .resizable(true)
        .default_width(250.0)
        .show(ctx, |ui| {
            // 强制覆盖光标为默认指针
            if ui.rect_contains_pointer(ui.max_rect()) {
                ui.ctx().set_cursor_icon(CursorIcon::Default);
            }

            ui.horizontal(|ui| {
                ui.heading(text.img_prop);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("X").clicked() {
                        is_open = false;
                    }
                });
            });
            ui.separator();

            // 直接从 data 中获取当前图片的属性
            if let Some(props) = &data.current_properties {
                render_properties_content(ui, props, &text);
            } else {
                ui.label("No image loaded.");
            }
        });

    // 如果面板被关闭，切换回 Normal 模式
    if !is_open {
        state.ui_mode = UiMode::Normal;
    }
}

/// 图片属性内容
fn render_properties_content(ui: &mut Ui, properties: &ImageProperties, text: &TextBundle) {
    egui::Grid::new("properties_grid")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
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
            ui.label(properties.path.to_string_lossy().to_string());
            ui.end_row();
        });
}
