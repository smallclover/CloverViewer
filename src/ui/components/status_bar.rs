use egui::{Context, TopBottomPanel};
use crate::model::state::{ViewState, ViewMode};
use crate::ui::components::icons::{draw_icon_button, IconType};

pub fn draw_status_bar(
    ctx: &Context,
    state: &mut ViewState,
    screenshot_active: &mut bool,
) {
    TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(10.0);

            // Screenshot button
            if ui.button("Screenshot").clicked() {
                *screenshot_active = true;
            }

            // Grid View Button
            if draw_icon_button(ui, state.view_mode == ViewMode::Grid, IconType::Grid).on_hover_text("Grid View").clicked() {
                state.view_mode = ViewMode::Grid;
            }

            ui.add_space(4.0);

            // Single View Button
            if draw_icon_button(ui, state.view_mode == ViewMode::Single, IconType::Single).on_hover_text("Single View").clicked() {
                state.view_mode = ViewMode::Single;
            }
        });
    });
}
