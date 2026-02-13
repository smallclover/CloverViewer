use egui::{Context, TopBottomPanel};
use crate::i18n::lang::get_text;
use crate::model::config::Config;
use crate::model::state::{ViewState, ViewMode};
use crate::ui::components::icons::{draw_icon_button, IconType};

pub fn draw_status_bar(
    ctx: &Context,
    state: &mut ViewState,
    config: &Config,
) {
    let texts = get_text(config.language);
    
    TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(10.0);

            // Grid View Button
            if draw_icon_button(ui, state.view_mode == ViewMode::Grid, IconType::Grid, texts).clicked() {
                state.view_mode = ViewMode::Grid;
            }

            ui.add_space(4.0);

            // Single View Button
            if draw_icon_button(ui, state.view_mode == ViewMode::Single, IconType::Single, texts).clicked() {
                state.view_mode = ViewMode::Single;
            }
        });
    });
}
