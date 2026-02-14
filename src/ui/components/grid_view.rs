use eframe::egui;
use egui::{Context, ScrollArea, Ui, Color32, Frame, Stroke, Sense, Align2, FontId, Id};
use std::sync::Arc;
use crate::{
    core::business::BusinessData,
    i18n::lang::get_text,
    model::{config::Config, state::{ViewState, ViewMode}},
};

pub fn draw_grid_view(
    ctx: &Context,
    ui: &mut Ui,
    data: &mut BusinessData,
    state: &mut ViewState,
) {
    let config = ctx.data(|d| d.get_temp::<Arc<Config>>(Id::new("config")).unwrap());
    if data.list.is_empty() {
        let texts = get_text(config.language);
        ui.centered_and_justified(|ui| {
            ui.label(texts.viewer_no_images);
        });
        return;
    }

    let item_size = egui::vec2(150.0, 150.0);
    let padding = 10.0;
    let frame_margin = 4.0;
    // 统一边框宽度，避免选中时布局抖动
    let border_width = 2.0;
    let cell_width = item_size.x + frame_margin * 2.0;

    let available_width = ui.available_width();
    let columns = ((available_width - padding) / (cell_width + padding)).floor() as usize;
    let columns = columns.max(1);

    let mut clicked_index = None;
    let mut double_clicked_index = None;

    // 分离借用
    let list = &data.list;
    let current_index = data.index;
    let thumb_cache = &mut data.thumb_cache;
    let loading_thumbs = &mut data.loading_thumbs;
    let failed_thumbs = &data.failed_thumbs;
    let loader = &mut data.loader;

    // 获取可见区域并扩大用于预加载
    let visible_rect = ui.clip_rect();
    let preload_rect = visible_rect.expand(500.0);

    ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("image_grid")
            .spacing(egui::vec2(padding, padding))
            .min_col_width(cell_width)
            .show(ui, |ui| {
                for (i, path) in list.iter().enumerate() {
                    if i > 0 && i % columns == 0 {
                        ui.end_row();
                    }

                    let is_selected = i == current_index;

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
                        let (rect, response) = ui.allocate_exact_size(item_size, Sense::click());

                        // 预加载逻辑：在预加载范围内且未加载过
                        // 注意：visible_rect 包含在 preload_rect 中，所以可见元素也会触发这里的加载检查
                        if preload_rect.intersects(rect) {
                             if !thumb_cache.contains(path) && !failed_thumbs.contains(path) && !loading_thumbs.contains(path) {
                                loading_thumbs.insert(path.clone());
                                loader.load_async(ctx.clone(), path.clone(), false, Some((200, 200)));
                            }
                        }

                        // 绘制逻辑：仅在严格可见范围内
                        if ui.is_rect_visible(rect) {
                            // 仅当可见时才调用 get，这会更新 LRU 缓存的顺序，保证可见元素不被淘汰
                            if let Some(t) = thumb_cache.get(path) {
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
    }
}
