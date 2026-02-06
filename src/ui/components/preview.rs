use egui::{Color32, CornerRadius, Rect, TextureHandle, Vec2, Context, Area, Frame, Stroke, Ui, Align2, FontId, UiBuilder, Sense, Id, Image, Shadow, StrokeKind, Spinner};
use crate::core::business::BusinessData;

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

pub fn show_preview_window(
    ctx: &Context,
    data: &mut BusinessData,
) -> bool {
    if !data.list.is_empty() {
        if let Some(new_idx) = draw_picker(ctx, data) {
            if new_idx != data.index {
                data.set_index(new_idx);
                return true;
            }
        }
    }
    false
}

fn draw_picker(
    ctx: &Context,
    data: &mut BusinessData,
) -> Option<usize> {
    let mut new_index = None;
    let item_width = 80.0;
    let visible_items = 5.0;
    let view_width = item_width * visible_items;
    let view_height = 80.0;

    let screen_rect = ctx.content_rect();
    let pos = egui::pos2(
        screen_rect.center().x - view_width / 2.0,
        screen_rect.bottom() - view_height - 60.0,
    );

    Area::new(Id::new("preview_picker"))
        .fixed_pos(pos)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::TRANSPARENT)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(view_width, view_height),
                        Sense::drag().union(Sense::click()),
                    );

                    let id = ui.id().with("picker_state");
                    let mut state = ui.data_mut(|d| d.get_temp::<PickerState>(id)).unwrap_or_default();

                    // 初始化状态
                    if !state.initialized {
                        state.scroll_offset = data.index as f32 * item_width;
                        state.target_offset = state.scroll_offset;
                        state.last_index = data.index;
                        state.initialized = true;
                    }

                    // Sync with external index changes
                    if data.index != state.last_index {
                        state.target_offset = data.index as f32 * item_width;
                        state.last_index = data.index;
                    }

                    // Interaction
                    if response.dragged() {
                        state.scroll_offset -= response.drag_delta().x;
                    } else {
                        if response.drag_stopped() {
                            state.target_offset = (state.scroll_offset / item_width).round() * item_width;
                        } else if response.clicked() {
                            if let Some(mouse_pos) = response.interact_pointer_pos() {
                                let center_x = rect.center().x;
                                let offset_from_center = mouse_pos.x - center_x;
                                let clicked_offset = state.scroll_offset + offset_from_center;
                                let clicked_idx = (clicked_offset / item_width).round();
                                state.target_offset = clicked_idx * item_width;
                            }
                        }

                        // Smooth scroll
                        state.scroll_offset += (state.target_offset - state.scroll_offset) * 0.15;
                    }

                    // Bounds
                    let max_scroll = (data.list.len() as f32 - 1.0) * item_width;
                    state.scroll_offset = state.scroll_offset.clamp(0.0, max_scroll);

                    // Update selection
                    let selected_idx = (state.scroll_offset / item_width).round() as usize;
                    if selected_idx != data.index {
                        new_index = Some(selected_idx);
                        // 更新内部状态，防止下一帧被重置
                        state.last_index = selected_idx;
                    }

                    // Draw center indicator (保留但调淡，作为视觉辅助)
                    let center_indicator_rect = Rect::from_center_size(
                        rect.center(),
                        Vec2::new(item_width + 4.0, 58.0)
                    );
                    // 稍微加一点点背景，让选中的位置不至于完全空荡，但非常淡
                    ui.painter().rect_filled(
                        center_indicator_rect,
                        CornerRadius::same(8),
                        Color32::from_black_alpha(40)
                    );

                    // Draw items
                    let center_x = rect.center().x;

                    // Create a child ui for clipping
                    let mut items_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(rect)
                            .layout(*ui.layout())
                    );
                    items_ui.set_clip_rect(rect);

                    // Calculate visible range
                    let start_idx = ((state.scroll_offset - view_width / 2.0) / item_width).floor() as isize;
                    let end_idx = ((state.scroll_offset + view_width / 2.0) / item_width).ceil() as isize;
                    let start_idx = start_idx.max(0) as usize;
                    let end_idx = end_idx.min(data.list.len() as isize) as usize;

                    let mut to_load = Vec::new();

                    // Split borrows
                    let list = &data.list;
                    let thumb_cache = &mut data.thumb_cache;
                    let failed_thumbs = &data.failed_thumbs;
                    let loading_thumbs = &mut data.loading_thumbs;
                    let loader = &mut data.loader;

                    for i in start_idx..end_idx {
                        if let Some(path) = list.get(i) {
                            let item_center_offset = (i as f32 * item_width) - state.scroll_offset;
                            let item_x = center_x + item_center_offset;

                            if item_x > rect.left() - item_width && item_x < rect.right() + item_width {
                                let dist_from_center = (item_x - center_x).abs();
                                let factor = (1.0 - (dist_from_center / (view_width / 2.0))).max(0.0);

                                let size_factor = 0.6 + 0.4 * factor.powf(2.0);
                                let alpha = factor.powf(1.5);

                                let item_size = Vec2::new(item_width, 50.0) * size_factor;
                                let item_rect = Rect::from_center_size(
                                    egui::pos2(item_x, rect.center().y),
                                    item_size
                                );

                                let thumb_state = if let Some(tex) = thumb_cache.get(path) {
                                    ThumbnailState::Loaded(tex)
                                } else if failed_thumbs.contains(path) {
                                    ThumbnailState::Failed
                                } else {
                                    to_load.push(path.clone());
                                    ThumbnailState::Loading
                                };

                                render_preview_item_custom(&mut items_ui, item_rect, thumb_state, alpha);
                            }
                        }
                    }

                    ui.data_mut(|d| d.insert_temp(id, state));

                    if state.scroll_offset != state.target_offset || response.dragged() {
                        ui.ctx().request_repaint();
                    }

                    // Trigger loads
                    for path in to_load {
                         if !loading_thumbs.contains(&path) {
                             loading_thumbs.insert(path.clone());
                             loader.load_async(ctx.clone(), path, false, Some((160, 120)));
                         }
                    }
                });
        });

    new_index
}

fn render_preview_item_custom(
    ui: &mut Ui,
    rect: Rect,
    state: ThumbnailState,
    alpha: f32,
) {
    match state {
        ThumbnailState::Loaded(tex) => {
            // 给每个图片单独加阴影，增强悬浮感
            if alpha > 0.1 {
                let shadow = Shadow {
                    offset: [0, 2],
                    blur: 6,
                    spread: 0,
                    color: Color32::from_black_alpha((100.0 * alpha) as u8),
                };
                ui.painter().add(shadow.as_shape(rect, CornerRadius::same(6)));
            }

            let image = Image::from_texture(tex)
                .fit_to_exact_size(rect.size())
                .corner_radius(6.0)
                .tint(Color32::WHITE.linear_multiply(alpha));

            ui.put(rect, image);

            // Highlight active item
            if alpha > 0.85 {
                ui.painter().rect_stroke(
                    rect,
                    CornerRadius::same(6),
                    Stroke::new(2.0, Color32::from_white_alpha(200)),
                    StrokeKind::Outside
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


/// 仅负责绘制错误占位
fn paint_error_state(ui: &mut Ui, rect: Rect) {
    ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_rgb(60, 20, 20));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "🚫",
        FontId::proportional(18.0),
        Color32::RED,
    );
}

/// 仅负责绘制加载占位
fn paint_loading_state(ui: &mut Ui, rect: Rect) {
    ui.painter().rect_filled(rect, CornerRadius::same(4), Color32::from_gray(40));
    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(Spinner::new());
        });
    });
}