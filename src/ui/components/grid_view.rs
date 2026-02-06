use eframe::egui;
use egui::{Context, ScrollArea, Ui, Color32, Frame, Stroke, Sense, Align2, FontId};
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
    let frame_margin = 4.0;
    // 统一边框宽度，避免选中时布局抖动
    let border_width = 2.0;
    let cell_width = item_size.x + frame_margin * 2.0;

    let available_width = ui.available_width();
    let columns = ((available_width - padding) / (cell_width + padding)).floor() as usize;
    let columns = columns.max(1);

    // 触发加载缩略图
    data.load_thumbnails(ctx.clone(), data.list.clone());

    let mut clicked_index = None;
    let mut double_clicked_index = None;

    ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("image_grid")
            .spacing(egui::vec2(padding, padding))
            .min_col_width(cell_width)
            .show(ui, |ui| {
                for (i, path) in data.list.iter().enumerate() {
                    if i > 0 && i % columns == 0 {
                        ui.end_row();
                    }

                    let tex = data.thumb_cache.get(path);
                    let is_selected = i == data.index;

                    // 统一使用相同的边框宽度，只改变颜色
                    let (stroke_color, bg_color) = if is_selected {
                        (Color32::LIGHT_BLUE, Color32::from_gray(45))
                    } else {
                        (Color32::from_gray(60), Color32::from_gray(35))
                    };

                    let frame = Frame::default()
                        .fill(bg_color)
                        .stroke(Stroke::new(border_width, stroke_color))
                        .inner_margin(frame_margin)
                        .corner_radius(4.0);

                    frame.show(ui, |ui| {
                        // 强制分配固定大小的空间
                        let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());

                        if let Some(t) = tex {
                            // 计算保持比例的尺寸，居中显示
                            let tex_size = t.size_vec2();
                            let scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y);
                            let target_size = tex_size * scale;
                            let target_rect = egui::Rect::from_center_size(rect.center(), target_size);

                            ui.painter().image(
                                t.id(),
                                target_rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                Color32::WHITE
                            );
                        } else {
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                "Loading...",
                                FontId::proportional(14.0),
                                Color32::GRAY,
                            );
                        }

                        if response.double_clicked() {
                            double_clicked_index = Some(i);
                        } else if response.clicked() {
                            clicked_index = Some(i);
                        }
                    });
                }
            });
    });

    if let Some(index) = double_clicked_index {
        data.set_index(index);
        state.view_mode = ViewMode::Single;
        data.load_current(ctx.clone());
    } else if let Some(index) = clicked_index {
        data.set_index(index);
        // 单击仅选中
    }
}
