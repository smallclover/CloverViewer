use crate::core::viewer_state::ViewerState;
use egui::{
    Align2, Area, Color32, Context, CornerRadius, CursorIcon, FontId, Frame, Id, Image, Rect,
    Sense, Shadow, Spinner, Stroke, StrokeKind, TextureHandle, Ui, UiBuilder, Vec2,
};

const PICKER_ITEM_WIDTH: f32 = 80.0;
const PICKER_VISIBLE_ITEMS: f32 = 5.0;
const PICKER_VIEW_HEIGHT: f32 = 80.0;
const PICKER_BOTTOM_MARGIN: f32 = 60.0;
const PICKER_INNER_MARGIN: f32 = 12.0;
const PICKER_SPRING_FACTOR: f32 = 0.15;
const PICKER_SCALE_BASE: f32 = 0.6;
const PICKER_SCALE_RANGE: f32 = 0.4;
const PICKER_ITEM_HEIGHT: f32 = 50.0;

#[derive(Clone, Copy, Default)]
struct PickerState {
    scroll_offset: f32,
    target_offset: f32,
    initialized: bool,
    last_index: usize,
}

enum ThumbnailState<'a> {
    Loaded(&'a TextureHandle),
    Failed,
    Loading,
}

struct PickerLayout {
    pos: egui::Pos2,
    view_width: f32,
    view_height: f32,
}

pub fn show_preview_window(ctx: &Context, viewer: &mut ViewerState) -> bool {
    if !viewer.list.is_empty()
        && let Some(new_idx) = draw_picker(ctx, viewer)
        && new_idx != viewer.index
    {
        viewer.jump_to_index(ctx.clone(), new_idx);
        return true;
    }
    false
}

fn draw_picker(ctx: &Context, viewer: &mut ViewerState) -> Option<usize> {
    let mut new_index = None;
    let layout = calculate_picker_layout(ctx);

    Area::new(Id::new("preview_picker"))
        .fixed_pos(layout.pos)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::TRANSPARENT)
                .inner_margin(PICKER_INNER_MARGIN)
                .show(ui, |ui| {
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(layout.view_width, layout.view_height),
                        Sense::drag().union(Sense::click()),
                    );

                    if ui.rect_contains_pointer(rect) {
                        ui.set_cursor_icon(CursorIcon::Default);
                    }

                    let id = ui.id().with("picker_state");
                    let mut state = load_picker_state(ui, id, viewer.index);
                    update_picker_scroll_state(&mut state, &response, rect, viewer.index);
                    clamp_picker_scroll(&mut state, viewer.list.len());

                    let selected_idx = (state.scroll_offset / PICKER_ITEM_WIDTH).round() as usize;
                    if selected_idx != viewer.index {
                        new_index = Some(selected_idx);
                        state.last_index = selected_idx;
                    }

                    draw_center_indicator(ui, rect);

                    let mut items_ui =
                        ui.new_child(UiBuilder::new().max_rect(rect).layout(*ui.layout()));
                    items_ui.set_clip_rect(rect);

                    let to_load = render_visible_items(&mut items_ui, rect, &state, viewer);

                    ui.data_mut(|d| d.insert_temp(id, state));

                    if state.scroll_offset != state.target_offset || response.dragged() {
                        ui.request_repaint();
                    }

                    queue_thumbnail_loads(ctx, viewer, to_load);
                });
        });

    new_index
}

fn calculate_picker_layout(ctx: &Context) -> PickerLayout {
    let view_width = PICKER_ITEM_WIDTH * PICKER_VISIBLE_ITEMS;
    let screen_rect = ctx.content_rect();

    PickerLayout {
        pos: egui::pos2(
            screen_rect.center().x - view_width / 2.0,
            screen_rect.bottom() - PICKER_VIEW_HEIGHT - PICKER_BOTTOM_MARGIN,
        ),
        view_width,
        view_height: PICKER_VIEW_HEIGHT,
    }
}

fn load_picker_state(ui: &mut Ui, id: Id, current_index: usize) -> PickerState {
    let mut state = ui
        .data_mut(|d| d.get_temp::<PickerState>(id))
        .unwrap_or_default();

    if !state.initialized {
        state.scroll_offset = current_index as f32 * PICKER_ITEM_WIDTH;
        state.target_offset = state.scroll_offset;
        state.last_index = current_index;
        state.initialized = true;
    }

    state
}

fn update_picker_scroll_state(
    state: &mut PickerState,
    response: &egui::Response,
    rect: Rect,
    current_index: usize,
) {
    if current_index != state.last_index {
        state.target_offset = current_index as f32 * PICKER_ITEM_WIDTH;
        state.last_index = current_index;
    }

    if response.dragged() {
        state.scroll_offset -= response.drag_delta().x;
        return;
    }

    if response.drag_stopped() {
        state.target_offset = (state.scroll_offset / PICKER_ITEM_WIDTH).round() * PICKER_ITEM_WIDTH;
    } else if response.clicked()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let offset_from_center = mouse_pos.x - rect.center().x;
        let clicked_offset = state.scroll_offset + offset_from_center;
        let clicked_idx = (clicked_offset / PICKER_ITEM_WIDTH).round();
        state.target_offset = clicked_idx * PICKER_ITEM_WIDTH;
    }

    state.scroll_offset += (state.target_offset - state.scroll_offset) * PICKER_SPRING_FACTOR;
}

fn clamp_picker_scroll(state: &mut PickerState, item_count: usize) {
    let max_scroll = (item_count as f32 - 1.0) * PICKER_ITEM_WIDTH;
    state.scroll_offset = state.scroll_offset.clamp(0.0, max_scroll);
}

fn draw_center_indicator(ui: &mut Ui, rect: Rect) {
    let center_indicator_rect = Rect::from_center_size(
        rect.center(),
        Vec2::new(PICKER_ITEM_WIDTH + 4.0, PICKER_ITEM_HEIGHT + 8.0),
    );
    ui.painter().rect_filled(
        center_indicator_rect,
        CornerRadius::same(8),
        Color32::from_black_alpha(40),
    );
}

fn render_visible_items(
    ui: &mut Ui,
    rect: Rect,
    state: &PickerState,
    viewer: &mut ViewerState,
) -> Vec<std::path::PathBuf> {
    let mut to_load = Vec::new();
    let center_x = rect.center().x;

    for i in visible_item_range(state.scroll_offset, rect.width(), viewer.list.len()) {
        if let Some(path) = viewer.list.get(i) {
            let item_center_offset = (i as f32 * PICKER_ITEM_WIDTH) - state.scroll_offset;
            let item_x = center_x + item_center_offset;

            if item_x <= rect.left() - PICKER_ITEM_WIDTH
                || item_x >= rect.right() + PICKER_ITEM_WIDTH
            {
                continue;
            }

            let factor = visible_factor(item_x, center_x, rect.width());
            let alpha = factor.powf(1.5);
            let item_rect = Rect::from_center_size(
                egui::pos2(item_x, rect.center().y),
                Vec2::new(PICKER_ITEM_WIDTH, PICKER_ITEM_HEIGHT) * preview_scale(factor),
            );

            let thumb_state = if let Some(tex) = viewer.thumb_cache.get(path) {
                ThumbnailState::Loaded(tex)
            } else if viewer.failed_thumbs.contains(path) {
                ThumbnailState::Failed
            } else {
                to_load.push(path.clone());
                ThumbnailState::Loading
            };

            render_preview_item_custom(ui, item_rect, thumb_state, alpha);
        }
    }

    to_load
}

fn visible_item_range(
    scroll_offset: f32,
    view_width: f32,
    item_count: usize,
) -> std::ops::Range<usize> {
    let start_idx = ((scroll_offset - view_width / 2.0) / PICKER_ITEM_WIDTH).floor() as isize;
    let end_idx = ((scroll_offset + view_width / 2.0) / PICKER_ITEM_WIDTH).ceil() as isize;
    let start_idx = start_idx.max(0) as usize;
    let end_idx = end_idx.min(item_count as isize) as usize;
    start_idx..end_idx
}

fn visible_factor(item_x: f32, center_x: f32, view_width: f32) -> f32 {
    (1.0 - ((item_x - center_x).abs() / (view_width / 2.0))).max(0.0)
}

fn preview_scale(factor: f32) -> f32 {
    PICKER_SCALE_BASE + PICKER_SCALE_RANGE * factor.powf(2.0)
}

fn queue_thumbnail_loads(
    ctx: &Context,
    viewer: &mut ViewerState,
    to_load: Vec<std::path::PathBuf>,
) {
    for path in to_load {
        if !viewer.loading_thumbs.contains(&path) {
            viewer.loading_thumbs.insert(path.clone());
            viewer
                .loader
                .load_async(ctx.clone(), path, false, Some((160, 120)));
        }
    }
}

fn render_preview_item_custom(ui: &mut Ui, rect: Rect, state: ThumbnailState, alpha: f32) {
    match state {
        ThumbnailState::Loaded(tex) => {
            if alpha > 0.1 {
                let shadow = Shadow {
                    offset: [0, 2],
                    blur: 6,
                    spread: 0,
                    color: Color32::from_black_alpha((100.0 * alpha) as u8),
                };
                ui.painter()
                    .add(shadow.as_shape(rect, CornerRadius::same(6)));
            }

            let image = Image::from_texture(tex)
                .fit_to_exact_size(rect.size())
                .corner_radius(6.0)
                .tint(Color32::WHITE.linear_multiply(alpha));

            ui.put(rect, image);

            if alpha > 0.85 {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(6),
                    Stroke::new(2.0, Color32::from_white_alpha(200)),
                    StrokeKind::Outside,
                );
            }
        }
        ThumbnailState::Failed => {
            paint_error_state(ui, rect);
        }
        ThumbnailState::Loading => {
            paint_loading_state(ui, rect);
        }
    }
}

fn paint_error_state(ui: &mut Ui, rect: Rect) {
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(60, 20, 20));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "🚫",
        FontId::proportional(18.0),
        Color32::RED,
    );
}

fn paint_loading_state(ui: &mut Ui, rect: Rect) {
    ui.painter()
        .rect_filled(rect, CornerRadius::same(4), Color32::from_gray(40));
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(Spinner::new());
        });
    });
}
