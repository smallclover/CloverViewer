use egui::{Context, TopBottomPanel, RichText};
use crate::model::state::{ViewState, ViewMode};

pub fn draw_status_bar(
    ctx: &Context,
    state: &mut ViewState,
) {
    TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(10.0);

            let grid_icon = if state.view_mode == ViewMode::Grid {
                RichText::new("⊞").strong()
            } else {
                RichText::new("⊞")
            };

            let single_icon = if state.view_mode == ViewMode::Single {
                RichText::new("□").strong()
            } else {
                RichText::new("□")
            };

            if ui.button(grid_icon).clicked() {
                state.view_mode = ViewMode::Grid;
            }
            if ui.button(single_icon).clicked() {
                state.view_mode = ViewMode::Single;
            }
        });
    });
}
