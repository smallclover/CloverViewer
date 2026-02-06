use eframe::egui;
use egui::{Context, Image, ScrollArea, Ui};
use crate::{
    core::business::BusinessData,
    model::state::{ViewState, ViewMode},
};

pub fn draw_grid_view(
    ctx: &Context,
    ui: &mut Ui,
    data: &mut BusinessData,
    state: &mut ViewState,
) {
    let item_size = egui::vec2(150.0, 150.0);
    let padding = 10.0;
    let available_width = ui.available_width();
    let columns = ((available_width - padding) / (item_size.x + padding)).floor() as usize;
    let columns = columns.max(1);

    // 触发加载缩略图
    data.load_thumbnails(ctx.clone(), data.list.clone());

    let mut clicked_index = None;

    ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("image_grid")
            .spacing(egui::vec2(padding, padding))
            .min_col_width(item_size.x)
            .show(ui, |ui| {
                for (i, path) in data.list.iter().enumerate() {
                    if i > 0 && i % columns == 0 {
                        ui.end_row();
                    }

                    let tex = data.thumb_cache.get(path);
                    let response = if let Some(t) = tex {
                        ui.add(Image::from_texture(t).fit_to_exact_size(item_size))
                    } else {
                        ui.allocate_response(item_size, egui::Sense::click())
                    };

                    if response.clicked() {
                        clicked_index = Some(i);
                    }
                }
            });
    });

    if let Some(index) = clicked_index {
        data.set_index(index);
        state.view_mode = ViewMode::Single;
        data.load_current(ctx.clone());
    }
}
