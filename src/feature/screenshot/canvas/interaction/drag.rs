use crate::feature::screenshot::{
    canvas::{
        CanvasState, ResizeStartState, drag,
        shape::{ShapeRender, clamp_pos_to_rect},
    },
    capture::{DrawnShape, ScreenshotState, ScreenshotTool},
};
use eframe::egui::{Pos2, Rect, Ui};

use super::hover::get_hovered_handle;

pub(super) fn on_drag_start(
    ui: &Ui,
    state: &mut ScreenshotState,
    canvas_state: &mut CanvasState,
    global_phys: Pos2,
    global_offset_phys: Pos2,
    ppp: f32,
    local_pos: Pos2,
) {
    // ========== 第一优先级：选中图形的控制点拖拽 ==========
    // 只要有选中的图形，优先检查是否命中控制点
    if let Some(selected_idx) = canvas_state.selected_shape {
        if let Some(shape) = state.edit.shapes.get(selected_idx) {
            if shape.supports_resize() {
                if let Some(handle) = get_hovered_handle(local_pos, shape, global_offset_phys, ppp)
                {
                    let start = shape.start;
                    let end = shape.end;
                    // 开始 resize 拖拽
                    state.push_history_snapshot();
                    canvas_state.dragging_shape = Some(selected_idx);
                    canvas_state.dragging_handle = Some(handle);
                    canvas_state.resize_start_state = Some(ResizeStartState {
                        start,
                        end,
                    });
                    return;
                }
            }
        }
    }

    let interaction_hovered = canvas_state.hovered_shape;
    let is_moving_state =
        canvas_state.hovered_shape.is_some() || canvas_state.dragging_shape.is_some();
    let can_draw = !is_moving_state && !canvas_state.dragging_selection;

    let mut is_hovering_selection_bg = false;
    if let Some(sel) = state.select.selection {
        is_hovering_selection_bg = sel.contains(global_phys)
            && canvas_state.hovered_shape.is_none()
            && state.edit.shapes.is_empty();
    }

    if let Some(index) = interaction_hovered {
        // 在修改 shapes 之前记录历史，这样撤销时才能正确删除复制出来的图形
        state.push_history_snapshot();

        // 检查是否按下了 Alt 键
        if ui.ctx().input(|i| i.modifiers.alt) {
            // 克隆当前图形
            let cloned_shape = state.edit.shapes[index].clone();
            state.edit.shapes.push(cloned_shape);

            // 将拖拽目标和选中目标切换为刚刚生成的新图形
            let new_index = state.edit.shapes.len() - 1;
            canvas_state.dragging_shape = Some(new_index);
            canvas_state.selected_shape = Some(new_index);
        } else {
            // 正常拖拽当前图形
            canvas_state.dragging_shape = Some(index);
            canvas_state.selected_shape = Some(index);
        }

        canvas_state.drag_start_phys = Some(global_phys);
    } else if can_draw && state.drawing.current_tool.is_some() {
        if let Some(selection) = state.select.selection {
            if selection.contains(global_phys)
                && state.drawing.current_tool != Some(ScreenshotTool::Text)
            {
                if state.drawing.current_tool == Some(ScreenshotTool::Pen)
                    || state.drawing.current_tool == Some(ScreenshotTool::Mosaic)
                {
                    state.input.current_pen_points.clear();
                    state.input.current_pen_points.push(global_phys);
                } else {
                    state.input.current_shape_start = Some(global_phys);
                    state.input.current_shape_end = Some(global_phys);
                }
            }
        }
    } else if is_hovering_selection_bg && state.drawing.current_tool.is_none() {
        canvas_state.dragging_selection = true;
        canvas_state.drag_start_phys = Some(global_phys);
        state.clear_toolbar();
        state.drawing.color_picker.close();
    } else if can_draw {
        // 如果已有选择区域且其中有图形，不允许创建新选择区域
        if let Some(sel) = state.select.selection {
            if sel.contains(global_phys) && !state.edit.shapes.is_empty() {
                return;
            }
        }
        state.select.drag_start = Some(global_phys);
        state.clear_toolbar();
        state.drawing.color_picker.close();
    }
}

pub(super) fn on_dragged(
    ui: &Ui,
    state: &mut ScreenshotState,
    canvas_state: &mut CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    _press_pos: Pos2,
) {
    // 获取当前鼠标位置（使用最新的指针位置）
    let current_phys = ui
        .ctx()
        .pointer_latest_pos()
        .map(|pos| global_offset_phys + (pos.to_vec2() * ppp));

    // 如果没有当前鼠标位置，则跳过本次处理
    let Some(current_phys) = current_phys else {
        return;
    };

    // 检查是否处于 resize 模式（拖拽控制点）
    if let (Some(shape_idx), Some(handle_idx), Some(start_state)) = (
        canvas_state.dragging_shape,
        canvas_state.dragging_handle,
        canvas_state.resize_start_state,
    ) {
        if let Some(shape) = state.edit.shapes.get_mut(shape_idx) {
            shape.apply_resize(handle_idx, current_phys, &start_state, state.select.selection);
        }
    } else if let Some(index) = canvas_state.dragging_shape {
        if let Some(drag_start_phys) = canvas_state.drag_start_phys {
            let delta_phys = current_phys - drag_start_phys;
            if let Some(shape) = state.edit.shapes.get_mut(index) {
                let clamped = drag::move_shape(shape, delta_phys, state.select.selection);
                canvas_state.drag_start_phys = Some(drag_start_phys + clamped);
            }
        }
    } else if canvas_state.dragging_selection {
        if let (Some(drag_start_phys), Some(mut sel)) =
            (canvas_state.drag_start_phys, state.select.selection)
        {
            let delta_phys = current_phys - drag_start_phys;
            drag::move_selection(&mut sel, delta_phys);
            state.update_selection_only(Some(sel));
            canvas_state.drag_start_phys = Some(current_phys);
        }
    } else if (state.drawing.current_tool == Some(ScreenshotTool::Pen)
        || state.drawing.current_tool == Some(ScreenshotTool::Mosaic))
        && !state.input.current_pen_points.is_empty()
    {
        let mut clamped_phys = current_phys;
        if let Some(sel) = state.select.selection {
            clamped_phys = clamp_pos_to_rect(current_phys, sel);
        }
        if let Some(last) = state.input.current_pen_points.last() {
            if last.distance(clamped_phys) > 2.0 {
                state.input.current_pen_points.push(clamped_phys);
            }
        }
    } else if state.input.current_shape_start.is_some() {
        let mut clamped_phys = current_phys;
        if let Some(sel) = state.select.selection {
            clamped_phys = clamp_pos_to_rect(current_phys, sel);
        }
        state.input.current_shape_end = Some(clamped_phys);
    } else if let Some(drag_start_phys) = state.select.drag_start {
        let rect = Rect::from_two_pos(drag_start_phys, current_phys);
        if state.select.selection.map_or(true, |s| s != rect) {
            state.update_selection_only(Some(rect));
        }
    }
}

pub(super) fn on_drag_stop(state: &mut ScreenshotState, canvas_state: &mut CanvasState) {
    if canvas_state.dragging_shape.is_some() {
        canvas_state.dragging_shape = None;
        canvas_state.drag_start_phys = None;
        // 清理 resize 相关状态
        canvas_state.dragging_handle = None;
        canvas_state.resize_start_state = None;
    } else if canvas_state.dragging_selection {
        canvas_state.dragging_selection = false;
        canvas_state.drag_start_phys = None;
        state.sync_toolbar_to_selection();
    } else if !state.input.current_pen_points.is_empty() {
        if state.input.current_pen_points.len() > 1 {
            let mut min_pos = state.input.current_pen_points[0];
            let mut max_pos = state.input.current_pen_points[0];
            for p in &state.input.current_pen_points {
                min_pos = min_pos.min(*p);
                max_pos = max_pos.max(*p);
            }

            state.push_history_snapshot();

            let Some(tool) = state.drawing.current_tool else {
                return;
            };
            let used_width = if tool == ScreenshotTool::Mosaic {
                state.drawing.mosaic_width
            } else {
                state.drawing.stroke_width
            };

            state.edit.shapes.push(DrawnShape {
                tool,
                start: min_pos,
                end: max_pos,
                color: state.drawing.active_color,
                stroke_width: used_width,
                text: None,
                points: Some(state.input.current_pen_points.clone()),
                cached_galley: None,
                cached_mosaic: None,
            });
        }
        state.input.current_pen_points.clear();
    } else if let Some(start_pos) = state.input.current_shape_start {
        let end_pos = state.input.current_shape_end.unwrap_or(start_pos);
        if start_pos.distance(end_pos) > 5.0 {
            if let Some(tool) = state.drawing.current_tool {
                state.push_history_snapshot();
                state.edit.shapes.push(DrawnShape {
                    tool,
                    start: start_pos,
                    end: end_pos,
                    color: state.drawing.active_color,
                    stroke_width: state.drawing.stroke_width,
                    text: None,
                    points: None,
                    cached_galley: None,
                    cached_mosaic: None,
                });
            }
        }
        state.input.current_shape_start = None;
        state.input.current_shape_end = None;
    } else if state.select.drag_start.take().is_some() {
        if let Some(sel) = state.select.selection {
            if sel.width() > 10.0 && sel.height() > 10.0 {
                // 重新选择区域时，清除已有图形
                if !state.edit.shapes.is_empty() {
                    state.push_history_snapshot();
                    state.edit.shapes.clear();
                    canvas_state.selected_shape = None;
                }
                state.push_history_snapshot();
                state.sync_toolbar_to_selection();
            } else {
                state.set_selection(None);
            }
        }
    }
}
