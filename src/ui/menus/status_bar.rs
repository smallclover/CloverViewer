use egui::{Context, TopBottomPanel};
use crate::i18n::lang::{get_i18n_text};
use crate::model::state::{AppState};
use crate::core::business::ViewMode;
use crate::ui::widgets::icons::{draw_icon_button, IconType};

pub fn draw_status_bar(
    ctx: &Context,
    state: &mut AppState,
) {
    let text = get_i18n_text(ctx);

    TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(10.0);

            if draw_icon_button(ui, state.viewer.view_mode == ViewMode::Grid, IconType::Grid, &text).clicked() {
                state.viewer.view_mode = ViewMode::Grid;
            }

            ui.add_space(4.0);

            if draw_icon_button(ui, state.viewer.view_mode == ViewMode::Single, IconType::Single, &text).clicked() {
                state.viewer.view_mode = ViewMode::Single;
            }
        });
    });
}
