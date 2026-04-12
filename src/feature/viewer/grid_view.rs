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

#[derive(Default)]
struct GridInteraction {
    clicked_index: Option<usize>,
    double_clicked_index: Option<usize>,
}

struct GridRenderContext<'a> {
    ctx: &'a Context,
    loading_text: &'a str,
    current_index: usize,
    preload_rect: Rect,
    thumb_cache: &'a mut lru::LruCache<std::path::PathBuf, egui::TextureHandle>,
    failed_thumbs: &'a std::collections::HashSet<std::path::PathBuf>,
    loading_thumbs: &'a mut std::collections::HashSet<std::path::PathBuf>,
    loader: &'a mut crate::core::image_loader::ImageLoader,
    interaction: &'a mut GridInteraction,
}

pub fn draw_grid_view(ctx: &Context, ui: &mut Ui, viewer: &mut ViewerState) {
    let text = get_i18n_text(ctx);

    if viewer.list.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(text.viewer_no_images);
        });
        return;
    }

    let layout = calculate_grid_layout(ui.available_width());
    let mut interaction = GridInteraction::default();

    let list = &viewer.list;
    let current_index = viewer.index;
    let thumb_cache = &mut viewer.thumb_cache;
    let loading_thumbs = &mut viewer.loading_thumbs;
    let failed_thumbs = &viewer.failed_thumbs;
    let loader = &mut viewer.loader;

    let visible_rect = ui.clip_rect();
    let preload_rect = visible_rect.expand(GRID_PRELOAD_MARGIN);
    let mut render_context = GridRenderContext {
        ctx,
        loading_text: text.grid_loading,
        current_index,
        preload_rect,
        thumb_cache,
        failed_thumbs,
        loading_thumbs,
        loader,
        interaction: &mut interaction,
    };

    ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            render_grid_rows(ui, list, &layout, &mut render_context);
        });

    apply_grid_interaction(ctx, viewer, interaction);
}

fn calculate_grid_layout(available_width: f32) -> GridLayout {
    let cell_total_width = GRID_ITEM_SIZE.x + (GRID_FRAME_MARGIN * 2.0) + (GRID_BORDER_WIDTH * 2.0);
    let mut columns =
        ((available_width + GRID_SPACING) / (cell_total_width + GRID_SPACING)).floor() as usize;
    columns = columns.max(1);

    let content_width =
        (columns as f32 * cell_total_width) + ((columns as f32 - 1.0) * GRID_SPACING);
    let left_padding = ((available_width - content_width) / 2.0).max(0.0);

    GridLayout {
        columns,
        left_padding,
    }
}

fn render_grid_rows(
    ui: &mut Ui,
    list: &[std::path::PathBuf],
    layout: &GridLayout,
    render: &mut GridRenderContext<'_>,
) {
    ui.add_space(GRID_SPACING);

    for (row_idx, row_items) in list.chunks(layout.columns).enumerate() {
        ui.horizontal(|ui| {
            ui.add_space(layout.left_padding);
            ui.spacing_mut().item_spacing.x = GRID_SPACING;

            for (col_idx, path) in row_items.iter().enumerate() {
                let global_index = row_idx * layout.columns + col_idx;
                let is_selected = global_index == render.current_index;
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

                    if render.preload_rect.intersects(rect)
                        && !render.thumb_cache.contains(path)
                        && !render.failed_thumbs.contains(path)
                        && !render.loading_thumbs.contains(path)
                    {
                        render.loading_thumbs.insert(path.clone());
                        render.loader.load_async(
                            render.ctx.clone(),
                            path.clone(),
                            false,
                            Some(GRID_THUMB_SIZE),
                        );
                    }

                    if ui.is_rect_visible(rect) {
                        if let Some(texture) = render.thumb_cache.get(path) {
                            let tex_size = texture.size_vec2();
                            let scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y);
                            let target_size = tex_size * scale;
                            let target_rect = Rect::from_center_size(rect.center(), target_size);

                            ui.painter().image(
                                texture.id(),
                                target_rect,
                                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                                Color32::WHITE,
                            );
                        } else {
                            ui.painter().text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                render.loading_text,
                                FontId::proportional(14.0),
                                Color32::GRAY,
                            );
                        }
                    }

                    if response.double_clicked() {
                        render.interaction.double_clicked_index = Some(global_index);
                    } else if response.clicked() {
                        render.interaction.clicked_index = Some(global_index);
                    }
                });
            }
        });

        ui.add_space(GRID_SPACING);
    }
}

fn apply_grid_interaction(ctx: &Context, viewer: &mut ViewerState, interaction: GridInteraction) {
    if let Some(index) = interaction.double_clicked_index {
        viewer.set_index(index);
        viewer.view_mode = ViewMode::Single;
        viewer.load_current(ctx.clone());
    } else if let Some(index) = interaction.clicked_index {
        viewer.set_index(index);
    }
}
