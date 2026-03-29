use eframe::egui::{Color32, CursorIcon, Pos2, Rect, Response, Ui, Vec2};

use crate::feature::screenshot::{
    canvas::{
        drag,
        hit_test,
        shape::{clamp_pos_to_rect, ShapeRender},
        ResizeStartState, CanvasState,
    },
    capture::{DrawnShape, HistoryEntry, ScreenshotState, ScreenshotTool},
};
use crate::model::device::find_target_screen_rect;

/// 处理所有画布交互
pub fn handle_interaction(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    canvas_state: &mut CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    toolbar_rect: Option<Rect>,
) {
    let response = ui.interact(
        ui.max_rect(),
        ui.id().with("screenshot_background"),
        eframe::egui::Sense::click_and_drag(),
    );

    let is_pointer_down = ui.ctx().input(|i| i.pointer.primary_down());
    let is_hovering_ui = check_hovering_ui(ui, state, toolbar_rect);

    update_hover_state(
        ui,
        state,
        canvas_state,
        global_offset_phys,
        ppp,
        is_hovering_ui,
        is_pointer_down,
    );

    update_cursor(ui, state, &canvas_state, global_offset_phys, ppp, is_hovering_ui);

    if response.clicked() {
        handle_click(
            ui,
            state,
            canvas_state,
            global_offset_phys,
            ppp,
            toolbar_rect,
            &response,
        );
    }

    if let Some(press_pos) = response.interact_pointer_pos() {
        if !is_hovering_ui {
            let global_phys = global_offset_phys + (press_pos.to_vec2() * ppp);

            if response.drag_started() {
                on_drag_start(ui, state, canvas_state, global_phys, global_offset_phys, ppp, press_pos);
            }
            if response.dragged() {
                on_dragged(ui, state, canvas_state, global_offset_phys, ppp, press_pos);
            }
            if response.drag_stopped() {
                on_drag_stop(state, canvas_state);
            }
        }
    }
}

fn check_hovering_ui(ui: &Ui, state: &ScreenshotState, toolbar_rect: Option<Rect>) -> bool {
    if let Some(pos) = ui.ctx().pointer_latest_pos() {
        let is_clicking_toolbar = toolbar_rect.map_or(false, |r| r.contains(pos));
        let is_interacting_with_picker =
            state.color_picker.is_open && ui.ctx().is_pointer_over_area();
        is_clicking_toolbar || is_interacting_with_picker
    } else {
        false
    }
}

/// 获取悬停的控制点索引（如果有选中的 shape）
fn get_hovered_handle(
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

fn update_hover_state(
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
                if let Some(selected_idx) = canvas_state.selected_shape {
                    if let Some(shape) = state.shapes.get(selected_idx) {
                        if shape.supports_resize() {
                            if let Some(_handle) = get_hovered_handle(pos, shape, global_offset_phys, ppp) {
                                // 找到悬停的控制点，不更新 hovered_shape（保持选中状态）
                                // 控制点命中优先，但不需要存储在 hovered_shape 中
                                // 我们会在 update_cursor 和 on_drag_start 中单独处理
                                return;
                            }
                        }
                    }
                }

                // 否则检查 shape body
                canvas_state.hovered_shape = hit_test::get_hovered_shape_index(
                    pos,
                    &state.shapes,
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

fn update_cursor(
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
    if let Some(pos) = ui.ctx().pointer_latest_pos() {
        if let Some(selected_idx) = canvas_state.selected_shape {
            if let Some(shape) = state.shapes.get(selected_idx) {
                if shape.supports_resize() {
                    if let Some(handle) = get_hovered_handle(pos, shape, global_offset_phys, ppp) {
                        // 根据 handle 索引设置对应的光标
                        let cursor = match shape.tool {
                            ScreenshotTool::Arrow => {
                                // 箭头：根据方向显示对应的 resize 光标
                                let dx = (shape.end.x - shape.start.x).abs();
                                let dy = (shape.end.y - shape.start.y).abs();
                                if dx > dy * 2.0 {
                                    // 主要是水平方向
                                    CursorIcon::ResizeHorizontal
                                } else if dy > dx * 2.0 {
                                    // 主要是垂直方向
                                    CursorIcon::ResizeVertical
                                } else {
                                    // 对角线方向，根据斜率判断
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
                                    0 | 2 => CursorIcon::ResizeNwSe, // NW, SE
                                    1 | 3 => CursorIcon::ResizeNeSw, // NE, SW
                                    4 | 6 => CursorIcon::ResizeVertical, // N, S
                                    5 | 7 => CursorIcon::ResizeHorizontal, // E, W
                                    _ => CursorIcon::Crosshair,
                                }
                            }
                        };
                        ui.ctx().set_cursor_icon(cursor);
                        return;
                    }
                }
            }
        }
    }

    let is_moving_state =
        canvas_state.hovered_shape.is_some() || canvas_state.dragging_shape.is_some();

    let mut is_hovering_selection_bg = false;
    if let Some(pos) = ui.ctx().pointer_latest_pos() {
        let global_phys = global_offset_phys + (pos.to_vec2() * ppp);
        if let Some(sel) = state.selection {
            is_hovering_selection_bg = sel.contains(global_phys)
                && canvas_state.hovered_shape.is_none()
                && state.shapes.is_empty();
        }
    }

    // 检测 Alt 键状态
    let is_alt_down = ui.ctx().input(|i| i.modifiers.alt);

    // 更新指针判断逻辑
    let cursor = if canvas_state.hovered_shape.is_some() && is_alt_down {
        CursorIcon::Copy // 悬浮在图形上且按下 Alt，显示复制指针
    } else if (is_moving_state
        && state.current_shape_start.is_none()
        && state.current_pen_points.is_empty())
        || canvas_state.dragging_selection
    {
        CursorIcon::Move
    } else if state.current_tool.is_none() && is_hovering_selection_bg {
        CursorIcon::Move
    } else {
        CursorIcon::Crosshair
    };

    ui.ctx().set_cursor_icon(cursor);
}

fn handle_click(
    ui: &Ui,
    state: &mut ScreenshotState,
    canvas_state: &mut CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    toolbar_rect: Option<Rect>,
    response: &Response,
) {
    let is_hovering_ui = check_hovering_ui(ui, state, toolbar_rect);
    if is_hovering_ui {
        return;
    }

    // 检查是否点击在选中图形的控制点上
    if let Some(pos) = response.interact_pointer_pos() {
        if let Some(selected_idx) = canvas_state.selected_shape {
            if let Some(shape) = state.shapes.get(selected_idx) {
                if shape.supports_resize() {
                    if let Some(_handle) = get_hovered_handle(pos, shape, global_offset_phys, ppp) {
                        // 点击在控制点上，开始 resize 拖拽，不取消选中
                        // 具体的 resize 逻辑在 on_drag_start 中处理
                        return;
                    }
                }
            }
        }
    }

    let is_moving_state =
        canvas_state.hovered_shape.is_some() || canvas_state.dragging_shape.is_some();
    let can_draw = !is_moving_state && !canvas_state.dragging_selection;

    // 第一优先级：点击图形选中它（无论当前工具是什么）
    if let Some(hovered_idx) = canvas_state.hovered_shape {
        canvas_state.selected_shape = Some(hovered_idx);
        // 选中后清空工具，方便后续操作
        state.current_tool = None;
        return;
    }

    // 无工具时：选择窗口或屏幕区域，或取消选中
    if state.current_tool.is_none() {
        // 如果点击在空白处，取消图形选中（但不取消截图区域）
        if !is_moving_state {
            canvas_state.selected_shape = None;
        }

        // 如果已经有截图区域且其中有图形，保持现有选择，不重新选择
        if state.selection.is_some() && !state.shapes.is_empty() {
            return;
        }

        if let Some(hovered) = state.hovered_window {
            state.selection = Some(hovered);
            state.toolbar_pos = Some(hovered.right_bottom());
            return;
        } else if let Some(pointer_pos) = response.interact_pointer_pos() {
            let global_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);
            if let Some(cap_phys_rect) = find_target_screen_rect(&state.captures, global_phys) {
                state.selection = Some(cap_phys_rect);
                state.toolbar_pos = Some(cap_phys_rect.right_bottom());
                return;
            }
        }
    }

    // 文本工具点击
    if state.current_tool == Some(ScreenshotTool::Text) && can_draw {
        if let Some(pos) = response.interact_pointer_pos() {
            let mut global_phys = global_offset_phys + (pos.to_vec2() * ppp);
            if let Some(sel) = state.selection {
                global_phys = clamp_pos_to_rect(global_phys, sel);
            }

            if let Some((pos_old, text)) = state.active_text_input.take() {
                if !text.trim().is_empty() {
                    commit_text_shape(ui, state, pos_old, text, global_offset_phys, ppp);
                }
            } else {
                state.active_text_input = Some((global_phys, String::new()));
            }
        }
    }
}

fn commit_text_shape(
    ui: &Ui,
    state: &mut ScreenshotState,
    pos: Pos2,
    text: String,
    global_offset_phys: Pos2,
    ppp: f32,
) {
    let font_size = 20.0 + (state.stroke_width * 2.0);
    let max_width_logical = if let Some(sel) = state.selection {
        let sel_max_x_local = Pos2::ZERO.x + ((sel.max.x - global_offset_phys.x) / ppp);
        let start_local_x = Pos2::ZERO.x + ((pos.x - global_offset_phys.x) / ppp);
        (sel_max_x_local - start_local_x - 16.0).max(20.0)
    } else {
        1000.0
    };

    let galley = ui.painter().layout(
        text.clone(),
        eframe::egui::FontId::proportional(font_size),
        Color32::WHITE,
        max_width_logical,
    );

    let mut baked_text = String::new();
    let rows_len = galley.rows.len();
    for (i, row) in galley.rows.iter().enumerate() {
        let mut row_str = String::new();
        for glyph in &row.glyphs {
            row_str.push(glyph.chr);
        }
        baked_text.push_str(row_str.trim_end_matches(&['\r', '\n'][..]));
        if i < rows_len - 1 {
            baked_text.push('\n');
        }
    }

    let start_pos_phys = pos + Vec2::new(8.0 * ppp, 8.0 * ppp);
    let text_width_phys = galley.size().x * ppp;
    let end_pos = start_pos_phys + Vec2::new(text_width_phys, 0.0);

    state.history.push(HistoryEntry {
        shapes: state.shapes.clone(),
        selection: state.selection,
    });

    state.shapes.push(DrawnShape {
        tool: ScreenshotTool::Text,
        start: start_pos_phys,
        end: end_pos,
        color: state.active_color,
        stroke_width: state.stroke_width,
        text: Some(baked_text),
        points: None,
        cached_galley: None,
        cached_mosaic: None,
    });
}

fn on_drag_start(
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
        if let Some(shape) = state.shapes.get(selected_idx) {
            if shape.supports_resize() {
                if let Some(handle) = get_hovered_handle(local_pos, shape, global_offset_phys, ppp) {
                    // 开始 resize 拖拽
                    state.history.push(HistoryEntry {
                        shapes: state.shapes.clone(),
                        selection: state.selection,
                    });
                    canvas_state.dragging_shape = Some(selected_idx);
                    canvas_state.dragging_handle = Some(handle);
                    canvas_state.resize_start_state = Some(ResizeStartState {
                        start: shape.start,
                        end: shape.end,
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
    if let Some(sel) = state.selection {
        is_hovering_selection_bg =
            sel.contains(global_phys) && canvas_state.hovered_shape.is_none() && state.shapes.is_empty();
    }

    if let Some(index) = interaction_hovered {
        // 在修改 shapes 之前记录历史，这样撤销时才能正确删除复制出来的图形
        state.history.push(HistoryEntry {
            shapes: state.shapes.clone(),
            selection: state.selection,
        });

        // 检查是否按下了 Alt 键
        if ui.ctx().input(|i| i.modifiers.alt) {
            // 克隆当前图形
            let cloned_shape = state.shapes[index].clone();
            state.shapes.push(cloned_shape);

            // 将拖拽目标和选中目标切换为刚刚生成的新图形
            let new_index = state.shapes.len() - 1;
            canvas_state.dragging_shape = Some(new_index);
            canvas_state.selected_shape = Some(new_index);
        } else {
            // 正常拖拽当前图形
            canvas_state.dragging_shape = Some(index);
            canvas_state.selected_shape = Some(index);
        }

        canvas_state.drag_start_phys = Some(global_phys);
    } else if can_draw && state.current_tool.is_some() {
        if let Some(selection) = state.selection {
            if selection.contains(global_phys) && state.current_tool != Some(ScreenshotTool::Text) {
                if state.current_tool == Some(ScreenshotTool::Pen)
                    || state.current_tool == Some(ScreenshotTool::Mosaic)
                {
                    state.current_pen_points.clear();
                    state.current_pen_points.push(global_phys);
                } else {
                    state.current_shape_start = Some(global_phys);
                    state.current_shape_end = Some(global_phys);
                }
            }
        }
    } else if is_hovering_selection_bg && state.current_tool.is_none() {
        canvas_state.dragging_selection = true;
        canvas_state.drag_start_phys = Some(global_phys);
        state.toolbar_pos = None;
        state.color_picker.close();
    } else if can_draw {
        // 如果已有选择区域且其中有图形，不允许创建新选择区域
        if let Some(sel) = state.selection {
            if sel.contains(global_phys) && !state.shapes.is_empty() {
                // 在选择区域内且有图形，不创建新选择
                return;
            }
        }
        state.drag_start = Some(global_phys);
        state.toolbar_pos = None;
        state.color_picker.close();
    }
}

fn on_dragged(
    ui: &Ui,
    state: &mut ScreenshotState,
    canvas_state: &mut CanvasState,
    global_offset_phys: Pos2,
    ppp: f32,
    _press_pos: Pos2,
) {
    // 获取当前鼠标位置（使用最新的指针位置）
    let current_phys = ui.ctx().pointer_latest_pos().map(|pos| {
        global_offset_phys + (pos.to_vec2() * ppp)
    });

    // 如果没有当前鼠标位置，则跳过本次处理
    let Some(current_phys) = current_phys else { return };

    // 检查是否处于 resize 模式（拖拽控制点）
    if let (Some(shape_idx), Some(handle_idx), Some(start_state)) =
        (canvas_state.dragging_shape, canvas_state.dragging_handle, canvas_state.resize_start_state)
    {
        if let Some(shape) = state.shapes.get_mut(shape_idx) {
            shape.apply_resize(handle_idx, current_phys, &start_state, state.selection);
        }
    } else if let Some(index) = canvas_state.dragging_shape {
        if let Some(drag_start_phys) = canvas_state.drag_start_phys {
            let delta_phys = current_phys - drag_start_phys;
            if let Some(shape) = state.shapes.get_mut(index) {
                let clamped = drag::move_shape(shape, delta_phys, state.selection);
                canvas_state.drag_start_phys = Some(drag_start_phys + clamped);
            }
        }
    } else if canvas_state.dragging_selection {
        if let (Some(drag_start_phys), Some(mut sel)) =
            (canvas_state.drag_start_phys, state.selection)
        {
            let delta_phys = current_phys - drag_start_phys;
            drag::move_selection(&mut sel, delta_phys);
            state.selection = Some(sel);
            canvas_state.drag_start_phys = Some(current_phys);
        }
    } else if (state.current_tool == Some(ScreenshotTool::Pen)
        || state.current_tool == Some(ScreenshotTool::Mosaic))
        && !state.current_pen_points.is_empty()
    {
        let mut clamped_phys = current_phys;
        if let Some(sel) = state.selection {
            clamped_phys = clamp_pos_to_rect(current_phys, sel);
        }
        if let Some(last) = state.current_pen_points.last() {
            if last.distance(clamped_phys) > 2.0 {
                state.current_pen_points.push(clamped_phys);
            }
        }
    } else if state.current_shape_start.is_some() {
        let mut clamped_phys = current_phys;
        if let Some(sel) = state.selection {
            clamped_phys = clamp_pos_to_rect(current_phys, sel);
        }
        state.current_shape_end = Some(clamped_phys);
    } else if let Some(drag_start_phys) = state.drag_start {
        let rect = Rect::from_two_pos(drag_start_phys, current_phys);
        if state.selection.map_or(true, |s| s != rect) {
            state.selection = Some(rect);
        }
    }
}

fn on_drag_stop(state: &mut ScreenshotState, canvas_state: &mut CanvasState) {
    if canvas_state.dragging_shape.is_some() {
        canvas_state.dragging_shape = None;
        canvas_state.drag_start_phys = None;
        // 清理 resize 相关状态
        canvas_state.dragging_handle = None;
        canvas_state.resize_start_state = None;
    } else if canvas_state.dragging_selection {
        canvas_state.dragging_selection = false;
        canvas_state.drag_start_phys = None;
        if let Some(sel) = state.selection {
            state.toolbar_pos = Some(sel.right_bottom());
        }
    } else if !state.current_pen_points.is_empty() {
        if state.current_pen_points.len() > 1 {
            let mut min_pos = state.current_pen_points[0];
            let mut max_pos = state.current_pen_points[0];
            for p in &state.current_pen_points {
                min_pos = min_pos.min(*p);
                max_pos = max_pos.max(*p);
            }

            state.history.push(HistoryEntry {
                shapes: state.shapes.clone(),
                selection: state.selection,
            });

            let tool = state.current_tool.unwrap();
            let used_width = if tool == ScreenshotTool::Mosaic {
                state.mosaic_width
            } else {
                state.stroke_width
            };

            state.shapes.push(DrawnShape {
                tool,
                start: min_pos,
                end: max_pos,
                color: state.active_color,
                stroke_width: used_width,
                text: None,
                points: Some(state.current_pen_points.clone()),
                cached_galley: None,
                cached_mosaic: None,
            });
        }
        state.current_pen_points.clear();
    } else if let Some(start_pos) = state.current_shape_start {
        let end_pos = state.current_shape_end.unwrap_or(start_pos);
        if start_pos.distance(end_pos) > 5.0 {
            if let Some(tool) = state.current_tool {
                state.history.push(HistoryEntry {
                    shapes: state.shapes.clone(),
                    selection: state.selection,
                });
                state.shapes.push(DrawnShape {
                    tool,
                    start: start_pos,
                    end: end_pos,
                    color: state.active_color,
                    stroke_width: state.stroke_width,
                    text: None,
                    points: None,
                    cached_galley: None,
                    cached_mosaic: None,
                });
            }
        }
        state.current_shape_start = None;
        state.current_shape_end = None;
    } else if state.drag_start.take().is_some() {
        // 使用拖拽结束时的当前选择区域（在 on_dragged 中已更新）
        if let Some(sel) = state.selection {
            if sel.width() > 10.0 && sel.height() > 10.0 {
                // 重新选择区域时，清除已有图形
                if !state.shapes.is_empty() {
                    state.history.push(HistoryEntry {
                        shapes: state.shapes.clone(),
                        selection: state.selection,
                    });
                    state.shapes.clear();
                    canvas_state.selected_shape = None;
                }
                state.history.push(HistoryEntry {
                    shapes: state.shapes.clone(),
                    selection: state.selection,
                });
                state.toolbar_pos = Some(sel.right_bottom());
            } else {
                state.selection = None;
                state.toolbar_pos = None;
            }
        }
    }
}
