mod drag;
mod hover;

use eframe::egui::{Pos2, Rect, Response, Ui};

use crate::feature::screenshot::{
    canvas::{CanvasState, commit_text_shape, shape::ShapeRender},
    capture::{ScreenshotState, ScreenshotTool},
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
            if let Some(shape) = state.edit.shapes.get(selected_idx) {
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
        state.drawing.current_tool = None;
        return;
    }

    // 无工具时：选择窗口或屏幕区域，或取消选中
    if state.drawing.current_tool.is_none() {
        if !is_moving_state {
            canvas_state.selected_shape = None;
        }

        if state.select.selection.is_some() && !state.edit.shapes.is_empty() {
            return;
        }

        if let Some(hovered) = state.select.hovered_window {
            state.set_selection(Some(hovered));
            return;
        } else if let Some(pointer_pos) = response.interact_pointer_pos() {
            let global_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);
            if let Some(cap_phys_rect) =
                find_target_screen_rect(&state.capture.captures, global_phys)
            {
                state.set_selection(Some(cap_phys_rect));
                return;
            }
        }
    }

    // 文本工具点击
    if state.drawing.current_tool == Some(ScreenshotTool::Text) && can_draw {
        if let Some(pos) = response.interact_pointer_pos() {
            let global_phys = global_offset_phys + (pos.to_vec2() * ppp);

            // 点击必须在选区内才允许创建文本框
            if let Some(sel) = state.select.selection {
                if !sel.contains(global_phys) {
                    return;
                }
            }

            if let Some((pos_old, text)) = state.input.active_text_input.take() {
                if !text.trim().is_empty() {
                    commit_text_shape(ui, state, pos_old, text, global_offset_phys, ppp);
                }
            } else {
                state.input.active_text_input = Some((global_phys, String::new()));
            }
        }
    }
}

