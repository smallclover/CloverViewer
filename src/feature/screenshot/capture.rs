use eframe::egui::{
    Color32, ColorImage, Context,
    Pos2, Rect,ViewportCommand};
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
use crate::feature::screenshot::canvas::{handle_interaction, render_canvas_elements};
use crate::model::{
    config::get_context_config,
    device::{DeviceInfo, MonitorInfo},
    state::CommonState
};
use crate::os::window::{get_taskbar_rects, lock_cursor_for_screenshot, unlock_cursor};
use crate::feature::screenshot::draw::{draw_skia_shapes_on_image};
use crate::feature::screenshot::help_box;
use crate::feature::screenshot::toolbar::{calculate_toolbar_rect, render_toolbar_and_overlays};
use crate::feature::screenshot::magnifier::handle_magnifier;

// 重新导出 state 模块的类型
pub use crate::feature::screenshot::state::{
    ScreenshotAction, ScreenshotTool, DrawnShape, ScreenshotState,
    HistoryEntry, CapturedScreen, WindowPrevState
};

// capture.rs
/// 处理截图系统的更新
/// `is_active` - 是否处于截图模式，函数内部可将其设为 false 以退出截图模式
pub fn handle_screenshot_system(ctx: &Context, is_active: &mut bool, screenshot_state: &mut ScreenshotState, common: &CommonState) {
    if !*is_active {
        return;
    }

    // 1. 发起和轮询截图阶段
    if screenshot_state.captures.is_empty() {
        if !screenshot_state.is_capturing {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::ZERO));
            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(-20000.0, -20000.0)));
        }
        let should_exit = handle_capture_process(ctx, screenshot_state);
        if should_exit {
            *is_active = false;
        }
        return;
    }

    // 2. 截图完成，全屏展开逻辑
    if !screenshot_state.window_configured {
        let ppp = ctx.pixels_per_point();

        // 1. 获取包含所有显示器的【绝对物理边界】
        // 算出总物理宽高
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for cap in &screenshot_state.captures {
            let info = &cap.screen_info;
            let phys_x = info.x as f32 * info.scale_factor;
            let phys_y = info.y as f32 * info.scale_factor;
            let phys_w = info.width as f32 * info.scale_factor;
            let phys_h = info.height as f32 * info.scale_factor;

            min_x = min_x.min(phys_x);
            min_y = min_y.min(phys_y);
            max_x = max_x.max(phys_x + phys_w);
            max_y = max_y.max(phys_y + phys_h);
        }

        // 2. 计算出物理总宽高，并加一点冗余防裁剪 (比如加 100 物理像素)
        let total_phys_width = max_x - min_x + 100.0;
        let total_phys_height = max_y - min_y + 100.0;

        // 3. 将物理坐标除以当前的 ppp，换算成当下操作系统能听懂的逻辑坐标
        let exact_logical_pos = Pos2::new(min_x / ppp, min_y / ppp);
        let exact_logical_size = Vec2::new(total_phys_width / ppp, total_phys_height / ppp);

        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));

        // 4. 使用精准计算出的逻辑尺寸，替代之前的 (pos, size*2.0)
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(exact_logical_size));
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(exact_logical_pos));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(exact_logical_size));

        screenshot_state.window_configured = true;
        ctx.request_repaint();
    }else {
        lock_cursor_for_screenshot();
    }

    // 3. 绘制截图 UI
    let action = draw_screenshot_ui(ctx, screenshot_state, &common.device_info);

    // 4. 退出截图模式，使用缓存的正常坐标恢复
    if action != ScreenshotAction::None {
        handle_save_action(action, screenshot_state);
        //开启截图模式之后，将不再显示住窗口
        let config = get_context_config(ctx);
        let force_hide_to_tray = config.screenshot_hides_main_window;

        let effective_prev_state = if force_hide_to_tray {
            WindowPrevState::Tray
        } else {
            screenshot_state.prev_window_state
        };
        unlock_cursor();

        // 恢复默认的最小，否则截图完成时无法手动改变窗口大小
        ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(Vec2::ZERO));

        match effective_prev_state {
            WindowPrevState::Tray => {
                if let Ok(mut visible) = common.window_state.visible.lock() {
                    *visible = false;
                }
                // 让托盘式无感
                ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(-20000.0, -20000.0)));
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::ZERO));
                ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                // 调用系统 API 隐藏到托盘,似乎没用，暂时注释掉
                // show_window_hide(common.window_state.hwnd_isize);
            }
            WindowPrevState::Minimized => {
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                // 发送最小化指令
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }
            WindowPrevState::Normal => {
                // 恢复窗口的常规属性
                ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
                let config = get_context_config(ctx);
                // 移回截图前的原始位置和尺寸
                if let Some((x, y)) = config.window_pos {
                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::new(x, y)));
                }
                if let Some((w, h)) = config.window_size {
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(w, h)));
                }

                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(ViewportCommand::Focus);
            }
        }

        *is_active = false;
        *screenshot_state = ScreenshotState::default();

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

            // [新增] 绘制左下角快捷键与工具栏帮助说明框
            help_box::render_help_box(ui, state, global_offset_phys, ppp);

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

/// 处理截图捕获过程
/// 返回 true 表示应该退出截图模式
fn handle_capture_process(
    ctx: &Context,
    screenshot_state: &mut ScreenshotState,
) -> bool {
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
            // 使用win底层API强制捕获副屏的任务栏
            let taskbars = get_taskbar_rects();
            window_rects.extend(taskbars);
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
                return true; // 表示应该退出截图模式
            }
        }
    }
    false // 不需要退出
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