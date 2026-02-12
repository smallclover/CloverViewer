use eframe::egui::{self, ColorImage, Rect, TextureHandle, Color32, Stroke, Pos2, StrokeKind, ViewportBuilder, ViewportId, ViewportClass, ViewportCommand};
use image::{RgbaImage, GenericImage};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use std::path::PathBuf;
use xcap::Monitor;

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotAction {
    None,
    Close,
    SaveAndClose,
}

pub struct ScreenshotState {
    pub is_active: bool,
    pub captures: Vec<CapturedScreen>,
    // 全局物理坐标 (Physical Pixels)
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
    pub save_button_pos: Option<Pos2>,

    // Async capture state
    pub is_capturing: bool,
    pub capture_receiver: Option<Receiver<Vec<CapturedScreen>>>,
    pub should_minimize: bool,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        Self {
            is_active: false,
            captures: vec![],
            selection: None,
            drag_start: None,
            save_button_pos: None,
            is_capturing: false,
            capture_receiver: None,
            should_minimize: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
}

#[derive(Clone)]
pub struct CapturedScreen {
    pub raw_image: Arc<RgbaImage>,
    pub image: ColorImage,
    pub screen_info: MonitorInfo,
    pub texture: Option<TextureHandle>,
}

pub fn handle_screenshot_system(ctx: &eframe::egui::Context, state: &mut ScreenshotState) {
    if !state.is_active {
        return;
    }

    // --- 1. 初始化捕获 (异步并行) ---
    if state.captures.is_empty() {
        if !state.is_capturing {
            state.is_capturing = true;

            // 检查窗口是否在前台且未最小化
            let is_focused = ctx.input(|i| i.focused);
            let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));

            state.should_minimize = is_focused && !is_minimized;

            if state.should_minimize {
                // 最小化主窗口
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }

            let (tx, rx) = channel();
            state.capture_receiver = Some(rx);

            // Clone context for the thread to request repaint
            let ctx_clone = ctx.clone();
            let should_minimize = state.should_minimize;

            thread::spawn(move || {
                if should_minimize {
                    // 稍微延迟一下，确保窗口动画完成，避免截取到正在最小化的窗口残影
                    thread::sleep(std::time::Duration::from_millis(300));
                }

                println!("[DEBUG] Capturing screens in background...");

                // 先获取显示器数量，不传递 Monitor 对象
                let count = match Monitor::all() {
                    Ok(monitors) => monitors.len(),
                    Err(e) => {
                        eprintln!("Failed to list monitors: {}", e);
                        return;
                    }
                };

                let captures: Vec<CapturedScreen> = thread::scope(|s| {
                    let mut handles = vec![];
                    for i in 0..count {
                        handles.push(s.spawn(move || {
                            // 在每个线程内部重新获取 Monitor，避免跨线程传递 !Send 的 Monitor
                            let monitors = Monitor::all().ok()?;
                            if i >= monitors.len() { return None; }
                            let monitor = &monitors[i];

                            if let (Ok(image), Ok(width)) = (monitor.capture_image(), monitor.width()) {
                                if width == 0 { return None; }

                                // 在后台线程生成 ColorImage，减少 UI 线程负担
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

                                Some(CapturedScreen {
                                    raw_image: Arc::new(image),
                                    image: color_image,
                                    screen_info: info,
                                    texture: None,
                                })
                            } else {
                                eprintln!("[ERROR] Failed to capture monitor {}", i);
                                None
                            }
                        }));
                    }

                    handles.into_iter()
                        .filter_map(|h| h.join().unwrap_or(None))
                        .collect()
                });

                let _ = tx.send(captures);
                // 截图完成后请求重绘，以便主线程能及时处理结果
                ctx_clone.request_repaint();
            });
        }

        // 检查接收结果
        if let Some(rx) = &state.capture_receiver {
            match rx.try_recv() {
                Ok(captures) => {
                    state.captures = captures;
                    state.is_capturing = false;
                    state.capture_receiver = None;
                    ctx.request_repaint();
                }
                Err(TryRecvError::Empty) => {
                    // 正在截图中，不显示任何 UI，等待后台线程完成
                    // 保持动画以便及时响应
                    ctx.request_repaint();
                    return;
                }
                Err(TryRecvError::Disconnected) => {
                    state.is_capturing = false;
                    state.capture_receiver = None;
                    state.is_active = false; // 失败退出

                    if state.should_minimize {
                        // 失败时也要恢复窗口
                        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    }
                    return;
                }
            }
        }
    }

    if state.captures.is_empty() {
        if !state.is_capturing {
            state.is_active = false;
        }
        return;
    }

    let mut final_action = ScreenshotAction::None;
    let mut wants_to_close_viewports = false;

    // --- 2. 渲染所有视口 ---
    // 先收集需要的信息，避免在闭包中借用 state
    let screens_info: Vec<_> = state.captures.iter().map(|c| {
        (
            c.screen_info.clone(),
            ViewportId::from_hash_of(format!("screenshot_{}", c.screen_info.name))
        )
    }).collect();

    for (i, (screen_info, viewport_id)) in screens_info.into_iter().enumerate() {
        // 获取物理参数
        let phys_x = screen_info.x as f32;
        let phys_y = screen_info.y as f32;
        // 当前屏幕的缩放
        let self_scale = screen_info.scale_factor;
        // 需要将物理坐标除去缩放
        let logic_x = phys_x / self_scale;
        let logic_y = phys_y / self_scale;

        let pos = egui::pos2(logic_x, logic_y);

        ctx.show_viewport_immediate(
            viewport_id,
            ViewportBuilder::default()
                .with_title("Screenshot")
                .with_fullscreen(true)
                .with_decorations(false)
                .with_position(pos),
            |ctx, class| {
                if class == ViewportClass::Immediate {
                    let action = draw_screenshot_ui(ctx, state, i);
                    if action != ScreenshotAction::None {
                        final_action = action;
                        wants_to_close_viewports = true;
                    }
                }
                if wants_to_close_viewports {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            },
        );
    }

    // --- 3. 处理关闭与保存逻辑 ---
    if wants_to_close_viewports {
        if final_action == ScreenshotAction::SaveAndClose {
            println!("[DEBUG] SaveAndClose triggered.");

            // 获取全局物理坐标选区
            if let Some(selection_phys) = state.selection {
                if selection_phys.is_positive() {
                    println!("[DEBUG] Physical Selection: {:?}", selection_phys);

                    // 准备线程所需数据
                    let captures_data: Vec<_> = state.captures.iter().map(|c| {
                        (
                            c.raw_image.clone(),
                            egui::Rect::from_min_size(
                                egui::pos2(c.screen_info.x as f32, c.screen_info.y as f32),
                                egui::vec2(c.screen_info.width as f32, c.screen_info.height as f32),
                            )
                        )
                    }).collect();

                    // 启动保存线程
                    thread::spawn(move || {
                        println!("[THREAD] Starting stitching...");

                        let final_width = selection_phys.width().round() as u32;
                        let final_height = selection_phys.height().round() as u32;

                        if final_width == 0 || final_height == 0 { return; }

                        let mut final_image = RgbaImage::new(final_width, final_height);

                        for (i, (raw_image, monitor_rect_phys)) in captures_data.iter().enumerate() {
                            let intersection = selection_phys.intersect(*monitor_rect_phys);

                            if !intersection.is_positive() {
                                continue;
                            }

                            let crop_x = (intersection.min.x - monitor_rect_phys.min.x).max(0.0).round() as u32;
                            let crop_y = (intersection.min.y - monitor_rect_phys.min.y).max(0.0).round() as u32;
                            let crop_w = intersection.width().round() as u32;
                            let crop_h = intersection.height().round() as u32;

                            if crop_x + crop_w > raw_image.width() || crop_y + crop_h > raw_image.height() {
                                eprintln!("[ERROR] Monitor {} crop out of bounds.", i);
                                continue;
                            }

                            let cropped_part = image::imageops::crop_imm(
                                &**raw_image,
                                crop_x,
                                crop_y,
                                crop_w,
                                crop_h,
                            ).to_image();

                            let paste_x = (intersection.min.x - selection_phys.min.x).max(0.0).round() as u32;
                            let paste_y = (intersection.min.y - selection_phys.min.y).max(0.0).round() as u32;

                            if let Err(e) = final_image.copy_from(&cropped_part, paste_x, paste_y) {
                                eprintln!("[ERROR] Failed to copy part: {}", e);
                            }
                        }

                        if let Ok(profile) = std::env::var("USERPROFILE") {
                            let desktop = PathBuf::from(profile).join("Desktop");
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let path = desktop.join(format!("screenshot_{}.png", timestamp));

                            if let Err(e) = final_image.save(&path) {
                                eprintln!("[ERROR] Save failed: {}", e);
                            } else {
                                println!("[SUCCESS] Saved to {:?}", path);
                            }
                        }
                    });
                }
            }
        }

        // --- 4. 重置状态 ---
        state.is_active = false;
        state.captures.clear();
        state.selection = None;
        state.drag_start = None;
        state.save_button_pos = None;

        if state.should_minimize {
            // 恢复主窗口
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }
}

pub fn draw_screenshot_ui(
    ctx: &eframe::egui::Context,
    state: &mut ScreenshotState,
    screen_index: usize,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let screen = &mut state.captures[screen_index];

    let texture: &TextureHandle = screen.texture.get_or_insert_with(|| {
        ctx.load_texture(
            format!("screenshot_{}", screen.screen_info.name),
            screen.image.clone(),
            Default::default(),
        )
    });
    let img_src = (texture.id(), texture.size_vec2());

    let mut needs_repaint = false;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(0.0))
        .show(ctx, |ui| {
        let image_widget = egui::Image::new(img_src).fit_to_exact_size(ui.available_size());
        ui.add(image_widget);

        let painter = ui.painter();
        let viewport_rect = ctx.viewport_rect();
        let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);
        let full_rect = ui.max_rect();

        // --- 1. 获取物理基准信息 ---
        let screen_x = screen.screen_info.x as f32;
        let screen_y = screen.screen_info.y as f32;
        let screen_offset_phys = Pos2::new(screen_x, screen_y);
        let ppp = ctx.pixels_per_point();

        // --- 2. 输入处理：将 局部逻辑坐标 -> 全局物理坐标 ---

        let mut local_button_rect = None;
        if let Some(global_button_pos_phys) = state.save_button_pos {
            let vec_phys = global_button_pos_phys - screen_offset_phys;
            let local_pos_logical = Pos2::ZERO + (vec_phys / ppp);
            local_button_rect = Some(Rect::from_min_size(local_pos_logical, egui::vec2(50.0, 25.0)));
        }

        let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::drag());

        if response.drag_started() {
            if let Some(press_pos) = response.interact_pointer_pos() {
                let is_clicking_button = local_button_rect.map_or(false, |r| r.contains(press_pos));

                if !is_clicking_button {
                    let local_vec_phys = press_pos.to_vec2() * ppp;
                    let global_phys = screen_offset_phys + local_vec_phys;

                    state.drag_start = Some(global_phys);
                    state.save_button_pos = None;
                    needs_repaint = true;
                }
            }
        }

        if response.dragged() {
            if let (Some(drag_start_phys), Some(curr_pos_local)) = (state.drag_start, ui.input(|i| i.pointer.latest_pos())) {
                let local_vec_phys = curr_pos_local.to_vec2() * ppp;
                let current_pos_phys = screen_offset_phys + local_vec_phys;

                let rect = Rect::from_two_pos(drag_start_phys, current_pos_phys);

                if state.selection.map_or(true, |s| s != rect) {
                    state.selection = Some(rect);
                }
                needs_repaint = true;
            }
        }

        if response.drag_stopped() {
            if state.drag_start.is_some() {
                state.drag_start = None;
                if let Some(sel) = state.selection {
                    if sel.width() > 10.0 && sel.height() > 10.0 {
                        state.save_button_pos = Some(sel.right_bottom() + egui::vec2(10.0, 10.0));
                    } else {
                        state.selection = None;
                        state.save_button_pos = None;
                    }
                    needs_repaint = true;
                }
            }
        }

        // --- 3. 渲染：将 全局物理坐标 -> 局部逻辑坐标 ---
        if let Some(global_sel_phys) = state.selection {
            let vec_min = global_sel_phys.min - screen_offset_phys;
            let vec_max = global_sel_phys.max - screen_offset_phys;

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

                painter.rect_stroke(clipped_local_sel, 0.0, Stroke::new(2.0, Color32::GREEN), StrokeKind::Outside);
            } else {
                painter.rect_filled(viewport_rect, 0.0, overlay_color);
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }

        let border_width = 5.0;
        painter.rect_stroke(
            full_rect,
            0.0,
            Stroke::new(border_width, Color32::GREEN),
            StrokeKind::Inside
        );

        if let Some(button_rect) = local_button_rect {
            if viewport_rect.intersects(button_rect) {
                let save_button = ui.put(button_rect, egui::Button::new("Save"));
                if save_button.clicked() {
                    action = ScreenshotAction::SaveAndClose;
                }
            }
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            action = ScreenshotAction::Close;
        }
    });

    if needs_repaint {
        ctx.request_repaint();
    }

    action
}
