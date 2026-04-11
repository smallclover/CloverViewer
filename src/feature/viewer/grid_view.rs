use crate::{
    core::viewer_state::{ViewMode, ViewerState},
    i18n::lang::get_i18n_text,
};
use eframe::egui;
use egui::{
    Align2, Color32, Context, FontId, Frame, Pos2, Rect, ScrollArea, Sense, Stroke, Ui, Vec2,
};

const GRID_ITEM_SIZE: Vec2 = Vec2::new(150.0, 150.0);
const GRID_FRAME_MARGIN: f32 = 4.0;
const GRID_BORDER_WIDTH: f32 = 2.0;
const GRID_SPACING: f32 = 16.0;
const GRID_PRELOAD_MARGIN: f32 = 500.0;
const GRID_THUMB_SIZE: (u32, u32) = (200, 200);

struct GridLayout {
    columns: usize,
    left_padding: f32,
}

pub fn draw_grid_view(ctx: &Context, ui: &mut Ui, viewer: &mut ViewerState) {
    let text = get_i18n_text(ctx);

    if viewer.list.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(text.viewer_no_images);
        });
        return;
    }

    let layout = compute_grid_layout(ui.available_width());

    let mut clicked_index = None;
    let mut double_clicked_index = None;

    let list = &viewer.list;
    let current_index = viewer.index;
    let thumb_cache = &mut viewer.thumb_cache;
    let loading_thumbs = &mut viewer.loading_thumbs;
    let failed_thumbs = &viewer.failed_thumbs;
    let loader = &mut viewer.loader;

    let visible_rect = ui.clip_rect();
    let preload_rect = visible_rect.expand(GRID_PRELOAD_MARGIN);

    // 3. 开始渲染
    ScrollArea::vertical()
        .auto_shrink([false; 2]) // 强制占满可用高度
        .show(ui, |ui| {
            ui.add_space(GRID_SPACING); // 顶部的呼吸空间

            // 将一维列表按列数切分成一块块的 row
            for (row_idx, row_items) in list.chunks(layout.columns).enumerate() {
                ui.horizontal(|ui| {
                    // 填入计算好的左侧空白，瞬间居中！
                    ui.add_space(layout.left_padding);
                    // 覆盖默认的水平间距，使用我们自定义的间距
                    ui.spacing_mut().item_spacing.x = GRID_SPACING;

                    for (col_idx, path) in row_items.iter().enumerate() {
                        let global_index = row_idx * layout.columns + col_idx;
                        let is_selected = global_index == current_index;
                        render_grid_item(
                            ui,
                            ctx,
                            text.grid_loading,
                            path,
                            is_selected,
                            preload_rect,
                            thumb_cache,
                            failed_thumbs,
                            loading_thumbs,
                            loader,
                            global_index,
                            &mut clicked_index,
                            &mut double_clicked_index,
                        );
                    }
                });

                ui.add_space(GRID_SPACING); // 行与行之间的垂直间距
            }
        });

    // 处理交互状态改变
    if let Some(index) = double_clicked_index {
        viewer.set_index(index);
        viewer.view_mode = ViewMode::Single;
        viewer.load_current(ctx.clone());
    } else if let Some(index) = clicked_index {
        viewer.set_index(index);
    }
}

fn compute_grid_layout(available_width: f32) -> GridLayout {
    let cell_total_width =
        GRID_ITEM_SIZE.x + (GRID_FRAME_MARGIN * 2.0) + (GRID_BORDER_WIDTH * 2.0);
    let mut columns =
        ((available_width + GRID_SPACING) / (cell_total_width + GRID_SPACING)).floor() as usize;
    columns = columns.max(1);

    let content_width = (columns as f32 * cell_total_width) + ((columns as f32 - 1.0) * GRID_SPACING);
    let left_padding = ((available_width - content_width) / 2.0).max(0.0);

    GridLayout {
        columns,
        left_padding,
    }
}

fn render_grid_item(
    ui: &mut Ui,
    ctx: &Context,
    loading_text: &str,
    path: &std::path::PathBuf,
    is_selected: bool,
    preload_rect: Rect,
    thumb_cache: &mut lru::LruCache<std::path::PathBuf, egui::TextureHandle>,
    failed_thumbs: &std::collections::HashSet<std::path::PathBuf>,
    loading_thumbs: &mut std::collections::HashSet<std::path::PathBuf>,
    loader: &mut crate::core::image_loader::ImageLoader,
    global_index: usize,
    clicked_index: &mut Option<usize>,
    double_clicked_index: &mut Option<usize>,
) {
    let (stroke_color, bg_color) = if is_selected {
        (Color32::from_rgb(0, 120, 215), Color32::from_gray(45))
    } else {
        (Color32::from_gray(60), Color32::from_gray(30))
    };

    let frame = Frame::default()
        .fill(bg_color)
        .stroke(Stroke::new(GRID_BORDER_WIDTH, stroke_color))
        .inner_margin(GRID_FRAME_MARGIN)
        .corner_radius(6.0);

    frame.show(ui, |ui| {
        let (rect, response) = ui.allocate_exact_size(GRID_ITEM_SIZE, Sense::click());

        if preload_rect.intersects(rect)
            && !thumb_cache.contains(path)
            && !failed_thumbs.contains(path)
            && !loading_thumbs.contains(path)
        {
            loading_thumbs.insert(path.clone());
            loader.load_async(ctx.clone(), path.clone(), false, Some(GRID_THUMB_SIZE));
        }

        if ui.is_rect_visible(rect) {
            if let Some(t) = thumb_cache.get(path) {
                let tex_size = t.size_vec2();
                let scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y);
                let target_size = tex_size * scale;
                let target_rect = Rect::from_center_size(rect.center(), target_size);

                ui.painter().image(
                    t.id(),
                    target_rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    loading_text,
                    FontId::proportional(14.0),
                    Color32::GRAY,
                );
            }
        }

        if response.double_clicked() {
            *double_clicked_index = Some(global_index);
        } else if response.clicked() {
            *clicked_index = Some(global_index);
        }
    });
}
