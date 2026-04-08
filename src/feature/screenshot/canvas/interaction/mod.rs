mod drag;
mod hover;

use eframe::egui::{Color32, Pos2, Rect, Response, Ui, Vec2};

use crate::feature::screenshot::{
    canvas::{CanvasState, shape::ShapeRender},
    capture::{HistoryEntry, ScreenshotState, ScreenshotTool},
};
use crate::model::device::find_target_screen_rect;
use drag::{on_drag_start, on_drag_stop, on_dragged};
use hover::{check_hovering_ui, update_cursor, update_hover_state};

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

    update_cursor(
        ui,
        state,
        &canvas_state,
        global_offset_phys,
        ppp,
        is_hovering_ui,
    );

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
                on_drag_start(
                    ui,
                    state,
                    canvas_state,
                    global_phys,
                    global_offset_phys,
                    ppp,
                    press_pos,
                );
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
                    if let Some(_handle) =
                        hover::get_hovered_handle(pos, shape, global_offset_phys, ppp)
                    {
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
        state.current_tool = None;
        return;
    }

    // 无工具时：选择窗口或屏幕区域，或取消选中
    if state.current_tool.is_none() {
        if !is_moving_state {
            canvas_state.selected_shape = None;
        }

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
            let global_phys = global_offset_phys + (pos.to_vec2() * ppp);

            // 点击必须在选区内才允许创建文本框
            if let Some(sel) = state.selection {
                if !sel.contains(global_phys) {
                    return;
                }
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

    state
        .shapes
        .push(crate::feature::screenshot::capture::DrawnShape {
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
