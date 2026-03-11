use eframe::egui::{
    Color32, ColorImage, Context,
    Pos2, Rect, Stroke, StrokeKind,
    Ui, ViewportCommand};
use image::{GenericImage, RgbaImage};
use std::{
    borrow::Cow,
    path::PathBuf,
    sync::{
        mpsc::{channel, TryRecvError},
        Arc,
    },
    thread,
    time::Duration
};
use xcap::Monitor;
use arboard::{Clipboard, ImageData};
use eframe::emath::Vec2;
use egui::WindowLevel;
use crate::ui::{
    mode::UiMode,
    screenshot::magnifier::handle_magnifier
};
use crate::model::{
    config::get_context_config,
    device::{DeviceInfo, MonitorInfo},
    state::AppState
};
use crate::os::window::show_window_hide;
use crate::ui::screenshot::draw::{draw_egui_shape, draw_skia_shapes_on_image};
use crate::ui::screenshot::toolbar::{calculate_toolbar_rect, render_toolbar_and_overlays};

// 重新导出 state 模块的类型
pub use crate::ui::screenshot::state::{
    ScreenshotAction, ScreenshotTool, DrawnShape, ScreenshotState,
    HistoryEntry, CapturedScreen, WindowPrevState
};

// capture.rs
pub fn handle_screenshot_system(ctx: &Context, state: &mut AppState) {
    if state.ui_mode != UiMode::Screenshot {
        return;
    }

    // 1. 发起和轮询截图阶段
    if state.screenshot.captures.is_empty() {
        if !state.screenshot.is_capturing {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::ZERO));
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(-20000.0, -20000.0)));
        }
        handle_capture_process(ctx, &mut state.ui_mode, &mut state.screenshot);
        return;
    }

    // 2. 截图完成，全屏展开逻辑 (这部分不变)
    if !state.screenshot.window_configured {
        let (pos, size) = state.common.device_info.global_logical_rect();

        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);

        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
        // 动态赋予窗口巨大的最小尺寸，强制突破操作系统的尺寸截断限制
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(size));

        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
        // 暂时对应，虽然解决了问题但是不够优雅，这个数字似乎要大于1或者1.5
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(size*2.0));

        state.screenshot.window_configured = true;
        ctx.request_repaint();
    }

    // 3. 绘制截图 UI
    let action = draw_screenshot_ui(ctx, &mut state.screenshot, &state.common.device_info);

    // 4. 退出截图模式，使用缓存的正常坐标恢复
    if action != ScreenshotAction::None {
        let screenshot_state_mut = &mut state.screenshot;
        handle_save_action(action, screenshot_state_mut);

        // 恢复窗口的常规属性
        ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(ViewportCommand::Transparent(false));

        // 移回截图前的原始位置和尺寸
        if let Some(pos) = state.common.normal_window_pos {
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
        }
        if let Some(size) = state.common.normal_window_size {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
        }

        match screenshot_state_mut.prev_window_state {
            WindowPrevState::Tray => {
                if let Ok(mut visible) = state.common.window_state.visible.lock() {
                    *visible = false;
                }
                ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                // 调用系统 API 隐藏到托盘
                show_window_hide(state.common.window_state.hwnd_isize);
            }
            WindowPrevState::Minimized => {
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                // 通过 eframe 发送最小化指令
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            WindowPrevState::Normal => {
                // 恢复窗口的常规属性
                ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(ViewportCommand::Transparent(false));

                // 移回截图前的原始位置和尺寸
                if let Some(pos) = state.common.normal_window_pos {
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos));
                }
                if let Some(size) = state.common.normal_window_size {
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
                }

                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(ViewportCommand::Focus);
            }
        }

        state.ui_mode = UiMode::Normal;
        *screenshot_state_mut = ScreenshotState::default();

        ctx.request_repaint();
    }
}

pub fn draw_screenshot_ui(
    ctx: &Context,
    state: &mut ScreenshotState,
    device_info: &DeviceInfo,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;

    let global_offset_phys = Pos2::new(device_info.phys_min_x as f32, device_info.phys_min_y as f32);
    let ppp = ctx.pixels_per_point();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            let painter = ui.painter();

            for cap in &state.captures {
                if let Some(texture) = state.texture_pool.get(&cap.screen_info.name) {
                    let rect = device_info.screen_logical_rect(&cap.screen_info, ppp);

                    painter.image(
                        texture.id(),
                        rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE
                    );
                }
            }

            state.hovered_window = None;
            let is_hovered = ui.rect_contains_pointer(ui.max_rect());

            if is_hovered && state.selection.is_none() && state.drag_start.is_none() {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);

                    for rect in &state.window_rects {
                        if rect.contains(global_pointer_phys) {
                            let mut is_fullscreen = false;
                            for cap in &state.captures {
                                if (rect.width() - cap.screen_info.width as f32).abs() < 5.0
                                    && (rect.height() - cap.screen_info.height as f32).abs() < 5.0
                                {
                                    is_fullscreen = true;
                                    break;
                                }
                            }
                            if !is_fullscreen {
                                state.hovered_window = Some(*rect);
                            }
                            break;
                        }
                    }
                }
            }

            let local_toolbar_rect = calculate_toolbar_rect(state, global_offset_phys, ppp);

            handle_interaction(ui, state, global_offset_phys, ppp, local_toolbar_rect);

            render_canvas_elements(ui, state, global_offset_phys, ppp, is_hovered);

            if let Some(rect) = local_toolbar_rect {
                if ui.clip_rect().intersects(rect) {
                    let toolbar_act = render_toolbar_and_overlays(ui, state, rect);
                    if toolbar_act != ScreenshotAction::None {
                        action = toolbar_act;
                    }
                }
            }

            let config = get_context_config(ctx);
            if config.magnifier_enabled {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let is_over_toolbar = local_toolbar_rect.map_or(false, |r| r.contains(pointer_pos));
                    let is_interacting_popup = state.color_picker.is_open && ui.ctx().is_pointer_over_area();

                    if !is_over_toolbar && !is_interacting_popup {
                        handle_magnifier(
                            ui,
                            state,
                            global_offset_phys,
                            ppp,
                            pointer_pos
                        );
                    }
                }
            }

            // Ctrl+Z 撤销
            if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
                if let Some(entry) = state.history.pop() {
                    state.shapes = entry.shapes;
                    state.selection = entry.selection;
                    // 更新 toolbar 位置
                    if let Some(sel) = state.selection {
                        state.toolbar_pos = Some(sel.right_bottom());
                    } else {
                        state.toolbar_pos = None;
                    }
                }
            }

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                action = ScreenshotAction::Close;
            }
        });

    ctx.request_repaint();

    action
}

fn handle_interaction(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    toolbar_rect: Option<Rect>,
) {
    let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::click_and_drag());

    if response.clicked() {
        if state.current_tool.is_none() && state.hovered_window.is_some() {
            state.selection = state.hovered_window;
            if let Some(sel) = state.selection {
                state.toolbar_pos = Some(sel.right_bottom());
            }
            return;
        }
    }

    if let Some(press_pos) = response.interact_pointer_pos() {
        let is_clicking_toolbar = toolbar_rect.map_or(false, |r| r.contains(press_pos));
        let is_interacting_with_picker = state.color_picker.is_open && ui.ctx().is_pointer_over_area();

        if !is_clicking_toolbar && !is_interacting_with_picker {
            let local_vec_phys = press_pos.to_vec2() * ppp;
            let global_phys = global_offset_phys + local_vec_phys;

            if response.drag_started() {
                if state.current_tool.is_some() {
                    if let Some(selection) = state.selection {
                        if selection.contains(global_phys) {
                            state.current_shape_start = Some(global_phys);
                            state.current_shape_end = Some(global_phys);
                        }
                    }
                } else {
                    state.drag_start = Some(global_phys);
                    state.toolbar_pos = None;
                    state.color_picker.close();
                }
            }

            if response.dragged() {
                if let Some(_) = state.current_shape_start {
                    state.current_shape_end = Some(global_phys);
                } else if let Some(drag_start_phys) = state.drag_start {
                    let rect = Rect::from_two_pos(drag_start_phys, global_phys);
                    if state.selection.map_or(true, |s| s != rect) {
                        state.selection = Some(rect);
                    }
                }
            }

            if response.drag_stopped() {
                if let Some(start_pos) = state.current_shape_start {
                    if let Some(tool) = state.current_tool {
                        // 保存历史记录
                        state.history.push(HistoryEntry {
                            shapes: state.shapes.clone(),
                            selection: state.selection,
                        });
                        state.shapes.push(DrawnShape {
                            tool,
                            start: start_pos,
                            end: global_phys,
                            color: state.active_color,
                            stroke_width: state.stroke_width,
                        });
                    }
                    state.current_shape_start = None;
                    state.current_shape_end = None;
                } else if state.drag_start.is_some() {
                    state.drag_start = None;
                    if let Some(sel) = state.selection {
                        if sel.width() > 10.0 && sel.height() > 10.0 {
                            // 保存历史记录
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
        }
    }
}

fn render_canvas_elements(
    ui: &mut Ui,
    state: &ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    is_hovered: bool,
) {
    let painter = ui.painter();
    let viewport_rect = ui.ctx().viewport_rect();
    let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);

    if let Some(global_sel_phys) = state.selection {
        let vec_min = global_sel_phys.min - global_offset_phys;
        let vec_max = global_sel_phys.max - global_offset_phys;

        let local_logical_rect = Rect::from_min_max(
            Pos2::ZERO + (vec_min / ppp),
            Pos2::ZERO + (vec_max / ppp),
        );

        let screen_rect_local = Rect::from_min_size(Pos2::ZERO, viewport_rect.size());
        let clipped_local_sel = local_logical_rect.intersect(screen_rect_local);

        if clipped_local_sel.is_positive() {
            let top = Rect::from_min_max(screen_rect_local.min, Pos2::new(screen_rect_local.max.x, clipped_local_sel.min.y));
            let bottom = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.max.y), screen_rect_local.max);
            let left = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.min.y), Pos2::new(clipped_local_sel.min.x, clipped_local_sel.max.y));
            let right = Rect::from_min_max(Pos2::new(clipped_local_sel.max.x, clipped_local_sel.min.y), Pos2::new(screen_rect_local.max.x, clipped_local_sel.max.y));

            painter.rect_filled(top, 0.0, overlay_color);
            painter.rect_filled(bottom, 0.0, overlay_color);
            painter.rect_filled(left, 0.0, overlay_color);
            painter.rect_filled(right, 0.0, overlay_color);

            paint_style_box(painter, clipped_local_sel, 1.0);

            if screen_rect_local.expand(1.0).contains(local_logical_rect.min) {
                let w = global_sel_phys.width().round() as u32;
                let h = global_sel_phys.height().round() as u32;
                let text = format!("{} x {}", w, h);
                let font_id = egui::FontId::proportional(12.0);
                let text_color = Color32::WHITE;

                let galley = painter.layout_no_wrap(text, font_id, text_color);
                let padding = egui::vec2(6.0, 4.0);
                let bg_size = galley.size() + padding * 2.0;

                let mut label_pos = local_logical_rect.min - egui::vec2(0.0, bg_size.y + 5.0);
                if label_pos.y < screen_rect_local.min.y {
                    label_pos = local_logical_rect.min + egui::vec2(5.0, 5.0);
                }

                let label_rect = Rect::from_min_size(label_pos, bg_size);
                painter.rect_filled(label_rect, 4.0, Color32::from_black_alpha(160));
                painter.galley(label_rect.min + padding, galley, text_color);
            }

        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }
    }

    for shape in &state.shapes {
        let start_local = Pos2::ZERO + ((shape.start - global_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((shape.end - global_offset_phys) / ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        if viewport_rect.intersects(rect) {
            draw_egui_shape(painter, shape.tool, rect, shape.stroke_width, shape.color);
        }
    }

    if let (Some(start_phys), Some(end_phys)) = (state.current_shape_start, state.current_shape_end) {
        let start_local = Pos2::ZERO + ((start_phys - global_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((end_phys - global_offset_phys) / ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        if viewport_rect.intersects(rect) {
            if let Some(tool) = state.current_tool {
                draw_egui_shape(painter, tool, rect, state.stroke_width, state.active_color);
            }
        }
    }

    if state.selection.is_none() && state.current_shape_start.is_none() && state.drag_start.is_none() {
        if is_hovered {
            if let Some(hover_phys_rect) = state.hovered_window {
                let vec_min = hover_phys_rect.min - global_offset_phys;
                let vec_max = hover_phys_rect.max - global_offset_phys;

                let local_logical_rect = Rect::from_min_max(
                    Pos2::ZERO + (vec_min / ppp),
                    Pos2::ZERO + (vec_max / ppp),
                );

                let screen_rect_local = Rect::from_min_size(Pos2::ZERO, viewport_rect.size());
                let clipped_local_sel = local_logical_rect.intersect(screen_rect_local);

                if clipped_local_sel.is_positive() {
                    let top = Rect::from_min_max(screen_rect_local.min, Pos2::new(screen_rect_local.max.x, clipped_local_sel.min.y));
                    let bottom = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.max.y), screen_rect_local.max);
                    let left = Rect::from_min_max(Pos2::new(screen_rect_local.min.x, clipped_local_sel.min.y), Pos2::new(clipped_local_sel.min.x, clipped_local_sel.max.y));
                    let right = Rect::from_min_max(Pos2::new(clipped_local_sel.max.x, clipped_local_sel.min.y), Pos2::new(screen_rect_local.max.x, clipped_local_sel.max.y));

                    painter.rect_filled(top, 0.0, overlay_color);
                    painter.rect_filled(bottom, 0.0, overlay_color);
                    painter.rect_filled(left, 0.0, overlay_color);
                    painter.rect_filled(right, 0.0, overlay_color);

                    paint_style_box(painter, clipped_local_sel, 2.0);
                }
            } else {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);

                    for cap in &state.captures {
                        let cap_phys_rect = Rect::from_min_size(
                            Pos2::new(cap.screen_info.x as f32, cap.screen_info.y as f32),
                            egui::vec2(cap.screen_info.width as f32, cap.screen_info.height as f32)
                        );

                        if cap_phys_rect.contains(global_pointer_phys) {
                            let vec_min = cap_phys_rect.min - global_offset_phys;
                            let vec_max = cap_phys_rect.max - global_offset_phys;

                            let local_logical_rect = Rect::from_min_max(
                                Pos2::ZERO + (vec_min / ppp),
                                Pos2::ZERO + (vec_max / ppp),
                            );

                            let inset_rect = local_logical_rect.shrink(4.0);
                            paint_style_box(painter, inset_rect, 3.0);
                            break;
                        }
                    }
                }
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }
    }
}


fn paint_style_box(painter: &egui::Painter, rect: Rect, line_width: f32) {
    let anchor_size = 6.0;
    let green = Color32::from_rgb(0, 255, 0);
    let main_stroke = Stroke::new(line_width, green);
    let anchor_stroke = Stroke::new(1.0, green);
    let anchor_fill = green;

    painter.rect_stroke(rect, 0.0, main_stroke, StrokeKind::Middle);

    if rect.width() > anchor_size * 3.0 && rect.height() > anchor_size * 3.0 {
        let min = rect.min;
        let max = rect.max;
        let center = rect.center();

        let anchors = [
            min, Pos2::new(max.x, min.y), max, Pos2::new(min.x, max.y),
            Pos2::new(center.x, min.y), Pos2::new(center.x, max.y),
            Pos2::new(min.x, center.y), Pos2::new(max.x, center.y),
        ];

        for anchor_pos in anchors {
            let anchor_rect = Rect::from_center_size(anchor_pos, egui::vec2(anchor_size, anchor_size));
            painter.rect_filled(anchor_rect, 0.0, anchor_fill);
            painter.rect_stroke(anchor_rect, 0.0, anchor_stroke, StrokeKind::Middle);
        }
    }
}

fn handle_capture_process(
    ctx: &Context,
    ui_mode: &mut UiMode,
    screenshot_state: &mut ScreenshotState,
) {
    if !screenshot_state.is_capturing {
        screenshot_state.is_capturing = true;

        ctx.request_repaint();

        let (tx, rx) = channel();
        screenshot_state.capture_receiver = Some(rx);
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            println!("[DEBUG] Capturing screens and windows in background...");

            let mut captures = Vec::new();
            if let Ok(monitors) = Monitor::all() {
                for monitor in monitors {
                    if let (Ok(image), Ok(width)) = (monitor.capture_image(), monitor.width()) {
                        if width == 0 { continue; }

                        let color_image = ColorImage::from_rgba_unmultiplied(
                            [image.width() as usize, image.height() as usize],
                            &image,
                        );

                        let info = MonitorInfo {
                            name: monitor.name().unwrap_or_default(),
                            x: monitor.x().unwrap_or(0),
                            y: monitor.y().unwrap_or(0),
                            width: monitor.width().unwrap_or(0),
                            height: monitor.height().unwrap_or(0),
                            scale_factor: monitor.scale_factor().unwrap_or(1.0),
                        };

                        captures.push(CapturedScreen {
                            raw_image: Arc::new(image),
                            image: color_image,
                            screen_info: info,
                        });
                    }
                }
            }

            let mut window_rects = Vec::new();
            if let Ok(windows) = xcap::Window::all() {
                for w in windows {
                    if !w.is_minimized().unwrap_or(true) {
                        let app_name = w.app_name().unwrap_or_default().to_lowercase();
                        if app_name.contains("cloverviewer") || app_name.contains("screenshot") {
                            continue;
                        }

                        let rect = Rect::from_min_size(
                            Pos2::new(w.x().unwrap_or(0) as f32, w.y().unwrap_or(0) as f32),
                            egui::vec2(w.width().unwrap_or(0) as f32, w.height().unwrap_or(0) as f32)
                        );
                        if rect.width() > 50.0 && rect.height() > 50.0 {
                            window_rects.push(rect);
                        }
                    }
                }
            }

            let _ = tx.send((captures, window_rects));
            ctx_clone.request_repaint();
        });
    }

    if let Some(rx) = &screenshot_state.capture_receiver {
        match rx.try_recv() {
            Ok((captures, window_rects)) => {

                for cap in &captures {
                    let monitor_name = &cap.screen_info.name;
                    if let Some(texture) = screenshot_state.texture_pool.get_mut(monitor_name) {
                        texture.set(cap.image.clone(), Default::default());
                    } else {
                        let texture = ctx.load_texture(
                            format!("screenshot_{}", monitor_name),
                            cap.image.clone(),
                            Default::default(),
                        );
                        screenshot_state.texture_pool.insert(monitor_name.clone(), texture);
                    }
                }

                screenshot_state.captures = captures;
                screenshot_state.window_rects = window_rects;
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
            Err(TryRecvError::Disconnected) => {
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;
                *ui_mode = UiMode::Normal;
            }
        }
    }
}

fn handle_save_action(final_action: ScreenshotAction, screenshot_state: &mut ScreenshotState) {
    if final_action == ScreenshotAction::SaveAndClose || final_action == ScreenshotAction::SaveToClipboard {
        if let Some(selection_phys) = screenshot_state.selection {
            if selection_phys.is_positive() {
                let captures_data: Vec<_> = screenshot_state.captures.iter().map(|c| {
                    (
                        c.raw_image.clone(),
                        Rect::from_min_size(
                            egui::pos2(c.screen_info.x as f32, c.screen_info.y as f32),
                            egui::vec2(c.screen_info.width as f32, c.screen_info.height as f32),
                        )
                    )
                }).collect();
                let shapes = screenshot_state.shapes.clone();

                thread::spawn(move || {
                    let final_width = selection_phys.width().round() as u32;
                    let final_height = selection_phys.height().round() as u32;
                    if final_width == 0 || final_height == 0 { return; }

                    let mut final_image = RgbaImage::new(final_width, final_height);

                    for (_, (raw_image, monitor_rect_phys)) in captures_data.iter().enumerate() {
                        let intersection = selection_phys.intersect(*monitor_rect_phys);
                        if !intersection.is_positive() { continue; }

                        let crop_x = (intersection.min.x - monitor_rect_phys.min.x).max(0.0).round() as u32;
                        let crop_y = (intersection.min.y - monitor_rect_phys.min.y).max(0.0).round() as u32;
                        let crop_w = intersection.width().round() as u32;
                        let crop_h = intersection.height().round() as u32;

                        if crop_x + crop_w > raw_image.width() || crop_y + crop_h > raw_image.height() {
                            continue;
                        }

                        let cropped_part = image::imageops::crop_imm(&**raw_image, crop_x, crop_y, crop_w, crop_h).to_image();
                        let paste_x = (intersection.min.x - selection_phys.min.x).max(0.0).round() as u32;
                        let paste_y = (intersection.min.y - selection_phys.min.y).max(0.0).round() as u32;
                        let _ = final_image.copy_from(&cropped_part, paste_x, paste_y);
                    }

                    draw_skia_shapes_on_image(&mut final_image, &shapes, selection_phys);

                    if final_action == ScreenshotAction::SaveAndClose {
                        if let Ok(profile) = std::env::var("USERPROFILE") {
                            let desktop = PathBuf::from(profile).join("Desktop");
                            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                            let path = desktop.join(format!("screenshot_{}.png", timestamp));
                            if let Err(e) = final_image.save(&path) { eprintln!("[ERROR] Save failed: {}", e); } else { println!("[SUCCESS] Saved to {:?}", path); }
                        }
                    } else if final_action == ScreenshotAction::SaveToClipboard {
                        if let Ok(mut clipboard) = Clipboard::new() {
                            let image_data = ImageData { width: final_image.width() as usize, height: final_image.height() as usize, bytes: Cow::from(final_image.into_raw()) };
                            if let Err(e) = clipboard.set_image(image_data) { eprintln!("[ERROR] Failed to copy image to clipboard: {}", e); } else { println!("[SUCCESS] Copied image to clipboard."); }
                        }
                    }
                });
            }
        }
    }
}
