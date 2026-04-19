mod actions;
mod capture_impl;

use crate::feature::screenshot::canvas::{self, CanvasState};
use crate::feature::screenshot::help_box;
use crate::feature::screenshot::magnifier::handle_magnifier;
use crate::feature::screenshot::toolbar::{calculate_toolbar_rect, render_toolbar_and_overlays};
use crate::model::{config::get_context_config, device::DeviceInfo, state::CommonState};
use crate::os::current_platform;
use eframe::egui::{Color32, Context, Pos2, Rect, Ui, ViewportCommand};
use eframe::emath::Vec2;
use egui::WindowLevel;

const MAX_SURFACE_EXTENT_PHYS: f32 = 8192.0;

// 重新导出 state 模块的类型
pub use crate::feature::screenshot::state::{
    CapturedScreen, DrawnShape, ScreenshotAction, ScreenshotState, ScreenshotTool, WindowPrevState,
};

fn hidden_window_pos() -> Pos2 {
    Pos2::new(-20000.0, -20000.0)
}

fn clamp_viewport_extent(physical_size: Vec2, ppp: f32) -> Vec2 {
    let max_logical_extent = MAX_SURFACE_EXTENT_PHYS / ppp.max(1.0);
    Vec2::new(
        physical_size.x.min(MAX_SURFACE_EXTENT_PHYS) / ppp,
        physical_size.y.min(MAX_SURFACE_EXTENT_PHYS) / ppp,
    )
    .max(Vec2::new(1.0, 1.0))
    .min(Vec2::new(max_logical_extent, max_logical_extent))
}

fn handle_capture_stage(
    ctx: &Context,
    is_active: &mut bool,
    screenshot_state: &mut ScreenshotState,
) -> bool {
    if !screenshot_state.capture.captures.is_empty() {
        return false;
    }

    if !screenshot_state.capture.is_capturing {
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::ZERO));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(hidden_window_pos()));
    }

    let should_exit = capture_impl::handle_capture_process(ctx, screenshot_state);
    if should_exit {
        *is_active = false;
    }

    true
}

fn configure_screenshot_viewport(ctx: &Context, screenshot_state: &mut ScreenshotState) {
    if screenshot_state.runtime.window_configured {
        current_platform().lock_cursor_for_screenshot();
        return;
    }

    let ppp = ctx.pixels_per_point();

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for cap in &screenshot_state.capture.captures {
        let info = &cap.screen_info;
        let phys_x = info.x as f32;
        let phys_y = info.y as f32;
        let phys_w = info.width as f32;
        let phys_h = info.height as f32;

        min_x = min_x.min(phys_x);
        min_y = min_y.min(phys_y);
        max_x = max_x.max(phys_x + phys_w);
        max_y = max_y.max(phys_y + phys_h);
    }

    let total_phys_width = (max_x - min_x + 100.0).max(1.0);
    let total_phys_height = (max_y - min_y + 100.0).max(1.0);
    let exact_logical_pos = Pos2::new(min_x / ppp, min_y / ppp);
    let requested_phys_size = Vec2::new(total_phys_width, total_phys_height);
    let exact_logical_size = clamp_viewport_extent(requested_phys_size, ppp);

    if requested_phys_size.x > MAX_SURFACE_EXTENT_PHYS
        || requested_phys_size.y > MAX_SURFACE_EXTENT_PHYS
    {
        tracing::warn!(
            requested_width = requested_phys_size.x,
            requested_height = requested_phys_size.y,
            clamped_width = exact_logical_size.x * ppp,
            clamped_height = exact_logical_size.y * ppp,
            "Screenshot viewport exceeded wgpu surface extent and was clamped"
        );
    }

    ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
    ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
    ctx.send_viewport_cmd(ViewportCommand::Focus);
    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
    ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(exact_logical_size));
    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(exact_logical_pos));
    ctx.send_viewport_cmd(ViewportCommand::InnerSize(exact_logical_size));

    screenshot_state.runtime.window_configured = true;
    ctx.request_repaint();
}

fn resolve_effective_prev_state(
    ctx: &Context,
    action: ScreenshotAction,
    screenshot_state: &ScreenshotState,
) -> WindowPrevState {
    let config = get_context_config(ctx);
    if config.screenshot_hides_main_window {
        return WindowPrevState::Tray;
    }

    if action == ScreenshotAction::Ocr {
        WindowPrevState::Normal
    } else {
        screenshot_state.runtime.prev_window_state
    }
}

fn restore_window_after_screenshot(
    ctx: &Context,
    common: &CommonState,
    effective_prev_state: WindowPrevState,
) {
    current_platform().unlock_cursor();
    ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(Vec2::ZERO));

    match effective_prev_state {
        WindowPrevState::Tray => {
            if let Ok(mut visible) = common.window_state.visible.lock() {
                *visible = false;
            }
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(hidden_window_pos()));
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::ZERO));
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }
        WindowPrevState::Minimized => {
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        WindowPrevState::Normal => {
            ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
            ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
            let config = get_context_config(ctx);
            if let Some((x, y)) = config.window_pos {
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(x, y)));
            }
            if let Some((w, h)) = config.window_size {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(w, h)));
            }
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }
}

fn handle_completion_action(
    ctx: &Context,
    screenshot_state: &mut ScreenshotState,
    common: &CommonState,
    action: ScreenshotAction,
) -> Option<image::DynamicImage> {
    let ocr_result_image = if action == ScreenshotAction::Ocr {
        actions::extract_cropped_image(screenshot_state).map(image::DynamicImage::ImageRgba8)
    } else {
        actions::handle_save_action(action, screenshot_state);
        None
    };

    let effective_prev_state = resolve_effective_prev_state(ctx, action, screenshot_state);
    restore_window_after_screenshot(ctx, common, effective_prev_state);
    *screenshot_state = ScreenshotState::default();
    ctx.request_repaint();

    ocr_result_image
}

pub fn prepare_screenshot_frame(
    ctx: &Context,
    is_active: &mut bool,
    screenshot_state: &mut ScreenshotState,
    _common: &CommonState,
) -> bool {
    if !*is_active {
        return false;
    }

    if handle_capture_stage(ctx, is_active, screenshot_state) {
        return false;
    }

    configure_screenshot_viewport(ctx, screenshot_state);

    true
}

pub fn finalize_screenshot_action(
    ctx: &Context,
    screenshot_state: &mut ScreenshotState,
    common: &CommonState,
    action: ScreenshotAction,
) -> Option<image::DynamicImage> {
    handle_completion_action(ctx, screenshot_state, common, action)
}

pub fn draw_screenshot_ui_inside(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    device_info: &DeviceInfo,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let ctx = ui.ctx().clone();

    let global_offset_phys =
        Pos2::new(device_info.phys_min_x as f32, device_info.phys_min_y as f32);
    let ppp = ctx.pixels_per_point();

    let painter = ui.painter();

    for cap in &state.capture.captures {
        if let Some(texture) = state.capture.texture_pool.get(&cap.screen_info.name) {
            let rect = device_info.screen_logical_rect(&cap.screen_info, ppp);

            painter.image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
    }

    state.select.hovered_window = None;
    let is_hovered = ui.rect_contains_pointer(ui.max_rect());

    if is_hovered
        && state.select.selection.is_none()
        && state.select.drag_start.is_none()
        && let Some(pointer_pos) = ui.pointer_latest_pos()
    {
        let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);

        for rect in &state.capture.window_rects {
            if rect.contains(global_pointer_phys) {
                let mut is_fullscreen = false;
                for cap in &state.capture.captures {
                    if (rect.width() - cap.screen_info.width as f32).abs() < 5.0
                        && (rect.height() - cap.screen_info.height as f32).abs() < 5.0
                    {
                        is_fullscreen = true;
                        break;
                    }
                }
                if !is_fullscreen {
                    state.select.hovered_window = Some(*rect);
                }
                break;
            }
        }
    }

    let local_toolbar_rect = calculate_toolbar_rect(state, global_offset_phys, ppp);

    let mut canvas_state = CanvasState::load_from_ui(ui);
    let interaction_action = canvas::interaction::handle_interaction(
        ui,
        state,
        &mut canvas_state,
        global_offset_phys,
        ppp,
        local_toolbar_rect,
    );
    canvas::render::render_canvas_elements(
        ui,
        state,
        &canvas_state,
        global_offset_phys,
        ppp,
        is_hovered,
    );
    canvas_state.save_to_ui(ui);

    if interaction_action != ScreenshotAction::None {
        action = interaction_action;
    }

    // [新增] 绘制左下角快捷键与工具栏帮助说明框
    help_box::render_help_box(ui, state, global_offset_phys, ppp);

    if let Some(rect) = local_toolbar_rect
        && ui.clip_rect().intersects(rect)
    {
        let toolbar_act = render_toolbar_and_overlays(ui, state, rect);
        if toolbar_act != ScreenshotAction::None {
            action = toolbar_act;
        }
    }

    let config = get_context_config(&ctx);
    if config.magnifier_enabled
        && let Some(pointer_pos) = ui.pointer_latest_pos()
    {
        let is_over_toolbar = local_toolbar_rect.is_some_and(|r| r.contains(pointer_pos));
        let is_interacting_popup = state.drawing.color_picker.is_open && ui.is_pointer_over_egui();

        if !is_over_toolbar && !is_interacting_popup {
            handle_magnifier(ui, state, global_offset_phys, ppp, pointer_pos);
        }
    }

    let undo_requested = ui.input(|i| i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::Z));
    let redo_requested = ui.input(|i| {
        i.modifiers.ctrl
            && (i.key_pressed(egui::Key::Y)
                || (i.modifiers.shift && i.key_pressed(egui::Key::Z)))
    });

    if undo_requested {
        state.undo_last();
    }

    if redo_requested {
        state.redo_last();
    }

    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let can_save_to_clipboard = state.has_positive_selection();

        if can_save_to_clipboard && state.input.active_text_input.is_none() {
            action = ScreenshotAction::SaveToClipboard;
        }
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        action = ScreenshotAction::Close;
    }

    ctx.request_repaint();

    action
}
