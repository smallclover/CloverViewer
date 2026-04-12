use eframe::egui::{CursorIcon, Pos2, Rect, Ui};

use crate::feature::screenshot::{
    canvas::{CanvasState, hit_test, shape::ShapeRender},
    capture::{DrawnShape, ScreenshotState, ScreenshotTool},
};

pub(super) fn check_hovering_ui(
    ui: &Ui,
    state: &ScreenshotState,
    toolbar_rect: Option<Rect>,
) -> bool {
    if let Some(pos) = ui.ctx().pointer_latest_pos() {
        let is_clicking_toolbar = toolbar_rect.is_some_and(|r| r.contains(pos));
        let is_interacting_with_picker =
            state.drawing.color_picker.is_open && ui.ctx().is_pointer_over_area();
        is_clicking_toolbar || is_interacting_with_picker
    } else {
        false
    }
}

/// 获取悬停的控制点索引（如果有选中的 shape）
pub(super) fn get_hovered_handle(
    local_pos: Pos2,
    shape: &DrawnShape,
    global_offset_phys: Pos2,
    ppp: f32,
) -> Option<usize> {
    let handles = shape.resize_handles(global_offset_phys, ppp);
    for (index, (handle_pos, hit_radius)) in handles.iter().enumerate() {
        if local_pos.distance(*handle_pos) <= *hit_radius {
            return Some(index);
        }
    }
    None
}

pub(super) fn update_hover_state(
    ui: &Ui,
    state: &ScreenshotState,
    canvas_state: &mut CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    is_hovering_ui: bool,
    is_pointer_down: bool,
) {
    if !is_pointer_down {
        if let Some(pos) = ui.ctx().pointer_latest_pos() {
            if !is_hovering_ui {
                // 先检查是否悬停在选中图形的控制点上
                if let Some(selected_idx) = canvas_state.selected_shape
                    && let Some(shape) = state.edit.shapes.get(selected_idx)
                    && shape.supports_resize()
                    && let Some(_handle) = get_hovered_handle(pos, shape, global_offset_phys, ppp)
                {
                    // 找到悬停的控制点，不更新 hovered_shape（保持选中状态）
                    return;
                }

                // 否则检查 shape body
                canvas_state.hovered_shape = hit_test::get_hovered_shape_index(
                    pos,
                    &state.edit.shapes,
                    global_offset_phys,
                    ppp,
                    ui.painter(),
                );
            } else {
                canvas_state.hovered_shape = None;
            }
        } else {
            canvas_state.hovered_shape = None;
        }
    }
}

pub(super) fn update_cursor(
    ui: &Ui,
    state: &ScreenshotState,
    canvas_state: &CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    is_hovering_ui: bool,
) {
    if is_hovering_ui {
        ui.ctx().set_cursor_icon(CursorIcon::Default);
        return;
    }

    // 检查是否悬停在选中图形的控制点上
    if let Some(pos) = ui.ctx().pointer_latest_pos()
        && let Some(selected_idx) = canvas_state.selected_shape
        && let Some(shape) = state.edit.shapes.get(selected_idx)
        && shape.supports_resize()
        && let Some(handle) = get_hovered_handle(pos, shape, global_offset_phys, ppp)
    {
        // 根据 handle 索引设置对应的光标
        let cursor = match shape.tool {
            ScreenshotTool::Arrow => {
                // 箭头：根据方向显示对应的 resize 光标
                let dx = (shape.end.x - shape.start.x).abs();
                let dy = (shape.end.y - shape.start.y).abs();
                if dx > dy * 2.0 {
                    CursorIcon::ResizeHorizontal
                } else if dy > dx * 2.0 {
                    CursorIcon::ResizeVertical
                } else {
                    let is_same_direction =
                        (shape.end.x - shape.start.x) * (shape.end.y - shape.start.y) >= 0.0;
                    if is_same_direction {
                        CursorIcon::ResizeNwSe
                    } else {
                        CursorIcon::ResizeNeSw
                    }
                }
            }
            _ => {
                // Rect/Circle/Text: 8 控制点
                match handle {
                    0 | 2 => CursorIcon::ResizeNwSe,       // NW, SE
                    1 | 3 => CursorIcon::ResizeNeSw,       // NE, SW
                    4 | 6 => CursorIcon::ResizeVertical,   // N, S
                    5 | 7 => CursorIcon::ResizeHorizontal, // E, W
                    _ => CursorIcon::Crosshair,
                }
            }
        };
        ui.ctx().set_cursor_icon(cursor);
        return;
    }

    let is_moving_state =
        canvas_state.hovered_shape.is_some() || canvas_state.dragging_shape.is_some();

    let mut is_hovering_selection_bg = false;
    if let Some(pos) = ui.ctx().pointer_latest_pos() {
        let global_phys = global_offset_phys + (pos.to_vec2() * ppp);
        if let Some(sel) = state.select.selection {
            is_hovering_selection_bg = sel.contains(global_phys)
                && canvas_state.hovered_shape.is_none()
                && state.edit.shapes.is_empty();
        }
    }

    // 检测 Alt 键状态
    let is_alt_down = ui.ctx().input(|i| i.modifiers.alt);

    // 更新指针判断逻辑
    let cursor = if canvas_state.hovered_shape.is_some() && is_alt_down {
        CursorIcon::Copy // 悬浮在图形上且按下 Alt，显示复制指针
    } else if (is_moving_state
        && state.input.current_shape_start.is_none()
        && state.input.current_pen_points.is_empty())
        || canvas_state.dragging_selection
        || (state.drawing.current_tool.is_none() && is_hovering_selection_bg)
    {
        CursorIcon::Move
    } else {
        CursorIcon::Crosshair
    };

    ui.ctx().set_cursor_icon(cursor);
}
