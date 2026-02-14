use eframe::egui::{self, ColorImage, Rect, TextureHandle, Color32, Stroke, Pos2, StrokeKind, ViewportBuilder, ViewportId, ViewportClass, ViewportCommand, Context};
use image::{RgbaImage, GenericImage};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use std::path::PathBuf;
use xcap::Monitor;
use crate::model::config::Config;
use crate::model::state::ViewState;
use crate::ui::components::screenshot_toolbar::draw_screenshot_toolbar;
use crate::ui::components::ui_mode::UiMode;
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use crate::ui::components::color_picker::ColorPicker;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ScreenshotAction {
    None,
    Close,
    SaveAndClose,
    SaveToClipboard,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ScreenshotTool {
    Rect,
    Circle,
}

#[derive(Clone, Debug)]
pub struct DrawnShape {
    pub tool: ScreenshotTool,
    pub start: Pos2, // 全局物理坐标
    pub end: Pos2,   // 全局物理坐标
    pub color: Color32,
    pub stroke_width: f32,
}

pub struct ScreenshotState {
    pub captures: Vec<CapturedScreen>,
    // 全局物理坐标 (Physical Pixels)
    pub selection: Option<Rect>,
    pub drag_start: Option<Pos2>,
    pub toolbar_pos: Option<Pos2>,

    // Async capture state
    pub is_capturing: bool,
    pub capture_receiver: Option<Receiver<Vec<CapturedScreen>>>,
    pub should_minimize: bool,

    // Toolbar state
    pub current_tool: Option<ScreenshotTool>,
    pub active_color: Color32,
    pub stroke_width: f32,
    pub color_picker: ColorPicker,
    pub color_picker_position: Option<Pos2>,


    // Drawing state
    pub shapes: Vec<DrawnShape>,
    pub current_shape_start: Option<Pos2>, // 正在绘制的形状起始点（全局物理坐标）
}

impl Default for ScreenshotState {
    fn default() -> Self {
        let default_color = Color32::from_rgb(255, 0, 0);
        Self {
            captures: Vec::new(),
            selection: None,
            drag_start: None,
            toolbar_pos: None,
            is_capturing: false,
            capture_receiver: None,
            should_minimize: false,
            current_tool: None,
            active_color: default_color,
            stroke_width: 2.0,
            color_picker: ColorPicker::new(default_color),
            color_picker_position: None,
            shapes: Vec::new(),
            current_shape_start: None,
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

pub fn handle_screenshot_system(ctx: &Context, state: &mut ViewState, config: &Config) {
    if state.ui_mode != UiMode::Screenshot {
        return;
    }

    let screenshot_state = &mut state.screenshot_state;

    // --- 1. 初始化捕获 (异步并行) ---
    if screenshot_state.captures.is_empty() {
        if !screenshot_state.is_capturing {
            screenshot_state.is_capturing = true;

            // 检查窗口是否在前台且未最小化
            let is_focused = ctx.input(|i| i.focused);
            let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));

            screenshot_state.should_minimize = is_focused && !is_minimized;

            if screenshot_state.should_minimize {
                // 最小化主窗口
                ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
            }

            let (tx, rx) = channel();
            screenshot_state.capture_receiver = Some(rx);

            // Clone context for the thread to request repaint
            let ctx_clone = ctx.clone();
            let should_minimize = screenshot_state.should_minimize;

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
        if let Some(rx) = &screenshot_state.capture_receiver {
            match rx.try_recv() {
                Ok(captures) => {
                    screenshot_state.captures = captures;
                    screenshot_state.is_capturing = false;
                    screenshot_state.capture_receiver = None;
                    ctx.request_repaint();
                }
                Err(TryRecvError::Empty) => {
                    // 正在截图中，不显示任何 UI，等待后台线程完成
                    // 保持动画以便及时响应
                    ctx.request_repaint();
                    return;
                }
                Err(TryRecvError::Disconnected) => {
                    screenshot_state.is_capturing = false;
                    screenshot_state.capture_receiver = None;
                    state.ui_mode = UiMode::Normal; // 失败退出

                    if screenshot_state.should_minimize {
                        // 失败时也要恢复窗口
                        ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    }
                    return;
                }
            }
        }
    }

    if screenshot_state.captures.is_empty() {
        if !screenshot_state.is_capturing {
            state.ui_mode = UiMode::Normal;
        }
        return;
    }

    let mut final_action = ScreenshotAction::None;
    let mut wants_to_close_viewports = false;

    // --- 2. 渲染所有视口 ---
    // 先收集需要的信息，避免在闭包中借用 state
    let screens_info: Vec<_> = screenshot_state.captures.iter().map(|c| {
        (
            c.screen_info.clone(),
            ViewportId::from_hash_of(format!("screenshot_{}", c.screen_info.name))
        )
    }).collect();

    let screenshot_state_mut = &mut state.screenshot_state;

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
                    let action = draw_screenshot_ui(ctx, screenshot_state_mut, i, config);
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
        if final_action == ScreenshotAction::SaveAndClose || final_action == ScreenshotAction::SaveToClipboard {
            println!("[DEBUG] Save action triggered: {:?}", final_action);

            // 获取全局物理坐标选区
            if let Some(selection_phys) = screenshot_state_mut.selection {
                if selection_phys.is_positive() {
                    println!("[DEBUG] Physical Selection: {:?}", selection_phys);

                    // 准备线程所需数据
                    let captures_data: Vec<_> = screenshot_state_mut.captures.iter().map(|c| {
                        (
                            c.raw_image.clone(),
                            Rect::from_min_size(
                                egui::pos2(c.screen_info.x as f32, c.screen_info.y as f32),
                                egui::vec2(c.screen_info.width as f32, c.screen_info.height as f32),
                            )
                        )
                    }).collect();

                    // 收集绘制的形状
                    let shapes = screenshot_state_mut.shapes.clone();

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

                        // --- 绘制形状到最终图片上 ---
                        for shape in shapes {
                             let start_x = shape.start.x - selection_phys.min.x;
                             let start_y = shape.start.y - selection_phys.min.y;
                             let end_x = shape.end.x - selection_phys.min.x;
                             let end_y = shape.end.y - selection_phys.min.y;
                             let rect = Rect::from_two_pos(Pos2::new(start_x, start_y), Pos2::new(end_x, end_y));
                             let x0 = rect.min.x.round() as i32;
                             let y0 = rect.min.y.round() as i32;
                             let x1 = rect.max.x.round() as i32;
                             let y1 = rect.max.y.round() as i32;
                             let color = image::Rgba([shape.color.r(), shape.color.g(), shape.color.b(), shape.color.a()]);
                             let thickness = shape.stroke_width.round() as i32;

                             match shape.tool {
                                 ScreenshotTool::Rect => {
                                     for x in x0..=x1 {
                                         for t in 0..thickness {
                                             if x >= 0 && x < final_width as i32 {
                                                 if y0 + t >= 0 && y0 + t < final_height as i32 {
                                                     final_image.put_pixel(x as u32, (y0 + t) as u32, color);
                                                 }
                                                 if y1 - t >= 0 && y1 - t < final_height as i32 {
                                                     final_image.put_pixel(x as u32, (y1 - t) as u32, color);
                                                 }
                                             }
                                         }
                                     }
                                     for y in y0..=y1 {
                                         for t in 0..thickness {
                                             if y >= 0 && y < final_height as i32 {
                                                 if x0 + t >= 0 && x0 + t < final_width as i32 {
                                                     final_image.put_pixel((x0 + t) as u32, y as u32, color);
                                                 }
                                                 if x1 - t >= 0 && x1 - t < final_width as i32 {
                                                     final_image.put_pixel((x1 - t) as u32, y as u32, color);
                                                 }
                                             }
                                         }
                                     }
                                 }
                                 ScreenshotTool::Circle => {
                                     let center_x = (x0 + x1) as f32 / 2.0;
                                     let center_y = (y0 + y1) as f32 / 2.0;
                                     let a = (x1 - x0).abs() as f32 / 2.0;
                                     let b = (y1 - y0).abs() as f32 / 2.0;
                                     if a > 0.0 && b > 0.0 {
                                         for x in x0..=x1 {
                                             for y in y0..=y1 {
                                                 let dx = x as f32 - center_x;
                                                 let dy = y as f32 - center_y;
                                                 let dist = (dx * dx) / (a * a) + (dy * dy) / (b * b);
                                                 let a_in = a - thickness as f32;
                                                 let b_in = b - thickness as f32;
                                                 let dist_in = if a_in > 0.0 && b_in > 0.0 {
                                                     (dx * dx) / (a_in * a_in) + (dy * dy) / (b_in * b_in)
                                                 } else {
                                                     2.0
                                                 };
                                                 if dist <= 1.0 && dist_in >= 1.0 {
                                                     if x >= 0 && x < final_width as i32 && y >= 0 && y < final_height as i32 {
                                                         final_image.put_pixel(x as u32, y as u32, color);
                                                     }
                                                 }
                                             }
                                         }
                                     }
                                 }
                             }
                        }

                        if final_action == ScreenshotAction::SaveAndClose {
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
                        } else if final_action == ScreenshotAction::SaveToClipboard {
                            if let Ok(mut clipboard) = Clipboard::new() {
                                let image_data = ImageData {
                                    width: final_image.width() as usize,
                                    height: final_image.height() as usize,
                                    bytes: Cow::from(final_image.into_raw()),
                                };
                                if let Err(e) = clipboard.set_image(image_data) {
                                    eprintln!("[ERROR] Failed to copy image to clipboard: {}", e);
                                } else {
                                    println!("[SUCCESS] Copied image to clipboard.");
                                }
                            }
                        }
                    });
                }
            }
        }

        // --- 4. 重置状态 ---
        state.ui_mode = UiMode::Normal;
        *screenshot_state_mut = ScreenshotState::default();

        if screenshot_state_mut.should_minimize {
            // 恢复主窗口
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }
}

pub fn draw_screenshot_ui(
    ctx: &Context,
    state: &mut ScreenshotState,
    screen_index: usize,
    config: &Config,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;

    // 解决借用冲突：先获取 texture 和 screen_info，然后释放 state 的借用
    let (img_src, screen_info) = {
        let screen = &mut state.captures[screen_index];
        let texture = screen.texture.get_or_insert_with(|| {
            ctx.load_texture(
                format!("screenshot_{}", screen.screen_info.name),
                screen.image.clone(),
                Default::default(),
            )
        });
        ((texture.id(), texture.size_vec2()), screen.screen_info.clone())
    };

    let mut needs_repaint = false;

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.inner_margin(0.0))
        .show(ctx, |ui| {
        let image_widget = egui::Image::new(img_src).fit_to_exact_size(ui.available_size());
        ui.add(image_widget);

        // // --- Color Picker ---
        // if state.color_picker.show(ui, state.color_picker_position) {
        //     state.active_color = state.color_picker.selected_color;
        //     needs_repaint = true;
        // }

        let painter = ui.painter().clone(); // Clone painter to avoid borrowing ui later
        let viewport_rect = ctx.viewport_rect();
        let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);
        let full_rect = ui.max_rect();

        // --- 1. 获取物理基准信息 ---
        let screen_x = screen_info.x as f32;
        let screen_y = screen_info.y as f32;
        let screen_offset_phys = Pos2::new(screen_x, screen_y);
        let ppp = ctx.pixels_per_point();

        // --- 2. 输入处理：将 局部逻辑坐标 -> 全局物理坐标 ---

        let mut local_toolbar_rect = None;
        if let Some(global_toolbar_pos_phys) = state.toolbar_pos {
            let vec_phys = global_toolbar_pos_phys - screen_offset_phys;
            let local_pos_logical = Pos2::ZERO + (vec_phys / ppp);

            // 精确计算的宽度：
            // 5个按钮(160) + 5个间距(40) + 1个分隔符(1) + 2个Padding(16) = 217.0
            let toolbar_width = 217.0;
            let toolbar_height = 48.0;
            // 调整工具栏位置右对齐
            let toolbar_min_pos = Pos2::new(local_pos_logical.x - toolbar_width, local_pos_logical.y + 10.0);

            local_toolbar_rect = Some(Rect::from_min_size(toolbar_min_pos, egui::vec2(toolbar_width, toolbar_height)));
        }

        let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::drag());

        // --- 交互逻辑 ---
        if let Some(press_pos) = response.interact_pointer_pos() {
            let is_clicking_toolbar = local_toolbar_rect.map_or(false, |r| r.contains(press_pos));

            let is_interacting_with_picker = state.color_picker.is_open &&
                ui.ctx().is_pointer_over_area(); // 简单的检查，如果鼠标悬浮在任何 egui 窗口上（包括 picker）

            if !is_clicking_toolbar && !is_interacting_with_picker {
                let local_vec_phys = press_pos.to_vec2() * ppp;
                let global_phys = screen_offset_phys + local_vec_phys;

                if response.drag_started() {
                    if state.current_tool.is_some() {
                        // 绘图模式
                        if let Some(selection) = state.selection {
                            if selection.contains(global_phys) {
                                state.current_shape_start = Some(global_phys);
                                needs_repaint = true;
                            }
                        }
                    } else {
                        // 选区模式
                        state.drag_start = Some(global_phys);
                        state.toolbar_pos = None;
                        state.color_picker.close();//重新开始拖拽选区时，强制关闭颜色选择器
                        needs_repaint = true;
                    }
                }

                if response.dragged() {
                    if let Some(_) = state.current_shape_start {
                        // 正在绘图，只请求重绘，不修改数据，直到松开
                        needs_repaint = true;
                    } else if let Some(drag_start_phys) = state.drag_start {
                        // 正在拖拽选区
                        let rect = Rect::from_two_pos(drag_start_phys, global_phys);
                        if state.selection.map_or(true, |s| s != rect) {
                            state.selection = Some(rect);
                        }
                        needs_repaint = true;
                    }
                }

                if response.drag_stopped() {
                    if let Some(start_pos) = state.current_shape_start {
                        // 完成绘图
                        if let Some(tool) = state.current_tool {
                            state.shapes.push(DrawnShape {
                                tool,
                                start: start_pos,
                                end: global_phys,
                                color: state.active_color,
                                stroke_width: state.stroke_width,
                            });
                        }
                        state.current_shape_start = None;
                        needs_repaint = true;
                    } else if state.drag_start.is_some() {
                        // 完成选区
                        state.drag_start = None;
                        if let Some(sel) = state.selection {
                            if sel.width() > 10.0 && sel.height() > 10.0 {
                                state.toolbar_pos = Some(sel.right_bottom());
                            } else {
                                state.selection = None;
                                state.toolbar_pos = None;
                            }
                            needs_repaint = true;
                        }
                    }
                }
            }
        }

        // --- 3. 渲染：将 全局物理坐标 -> 局部逻辑坐标 ---

        // 3.1 渲染选区背景遮罩
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

        // 3.2 渲染已绘制的形状
        for shape in &state.shapes {
             // 转换坐标
             let start_local = Pos2::ZERO + ((shape.start - screen_offset_phys) / ppp);
             let end_local = Pos2::ZERO + ((shape.end - screen_offset_phys) / ppp);
             let rect = Rect::from_two_pos(start_local, end_local);

             // 裁剪到当前屏幕
             if viewport_rect.intersects(rect) {
                 match shape.tool {
                     ScreenshotTool::Rect => {
                         painter.rect_stroke(rect, 0.0, Stroke::new(shape.stroke_width, shape.color), StrokeKind::Outside);
                     }
                     ScreenshotTool::Circle => {
                         painter.add(egui::Shape::ellipse_stroke(rect.center(), rect.size() / 2.0, Stroke::new(shape.stroke_width, shape.color)));
                     }
                 }
             }
        }

        // 3.3 渲染正在绘制的形状
        if let (Some(start_phys), Some(curr_pos_local)) = (state.current_shape_start, ui.input(|i| i.pointer.latest_pos())) {
             let start_local = Pos2::ZERO + ((start_phys - screen_offset_phys) / ppp);
             let rect = Rect::from_two_pos(start_local, curr_pos_local);

             if let Some(tool) = state.current_tool {
                 match tool {
                     ScreenshotTool::Rect => {
                         painter.rect_stroke(rect, 0.0, Stroke::new(state.stroke_width, state.active_color), StrokeKind::Outside);
                     }
                     ScreenshotTool::Circle => {
                         painter.add(egui::Shape::ellipse_stroke(rect.center(), rect.size() / 2.0, Stroke::new(state.stroke_width, state.active_color)));
                     }
                 }
             }
        }

        let border_width = 5.0;
        painter.rect_stroke(
            full_rect,
            0.0,
            Stroke::new(border_width, Color32::GREEN),
            StrokeKind::Inside
        );

        if let Some(toolbar_rect) = local_toolbar_rect {
            if viewport_rect.intersects(toolbar_rect) {
                let toolbar_action = draw_screenshot_toolbar(ui, &painter, state, toolbar_rect, config);
                if toolbar_action != ScreenshotAction::None {
                    action = toolbar_action;
                }

                // [修复核心] 将 Color Picker 放在这里渲染！
                // 这样只有拥有工具栏的那个屏幕才会尝试绘制颜色选择器。
                // 此时 state.color_picker_position 是基于工具栏计算出的局部坐标，
                // 对于“拥有工具栏”的这个屏幕来说，这个坐标是正确的。
                // 对于其他屏幕，因为 local_toolbar_rect 通常不会相交（或者位置完全不对），不会进入这里，或者位置正确但不会绘制。
                // 最重要的是，我们利用工具栏的可见性来约束颜色选择器的可见性。
                if state.color_picker.show(ui, state.color_picker_position) {
                    state.active_color = state.color_picker.selected_color;
                    needs_repaint = true;
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
