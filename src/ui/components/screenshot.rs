use eframe::egui::{self, ColorImage, Rect, TextureHandle, Color32, Stroke, Pos2, StrokeKind, ViewportBuilder, ViewportId, ViewportClass, ViewportCommand, Context, Ui};
use image::{RgbaImage, GenericImage};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;
use std::path::PathBuf;
use xcap::Monitor;
use crate::model::state::ViewState;
use crate::ui::components::screenshot_toolbar::draw_screenshot_toolbar;
use crate::ui::components::ui_mode::UiMode;
use arboard::{Clipboard, ImageData};
use std::borrow::Cow;
use crate::ui::components::color_picker::ColorPicker;
use crate::ui::components::magnifier::draw_magnifier;

// --- 类型定义 ---

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

    // Toolbar state
    pub current_tool: Option<ScreenshotTool>,
    pub active_color: Color32,
    pub stroke_width: f32,
    pub color_picker: ColorPicker,
    pub color_picker_anchor: Option<Rect>,


    // Drawing state
    pub shapes: Vec<DrawnShape>,
    pub current_shape_start: Option<Pos2>, // 正在绘制的形状起始点
    pub current_shape_end: Option<Pos2>,

    pub copy_requested: bool,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        let default_color = Color32::from_rgb(204, 0, 0);
        Self {
            captures: Vec::new(),
            selection: None,
            drag_start: None,
            toolbar_pos: None,
            is_capturing: false,
            capture_receiver: None,
            current_tool: None,
            active_color: default_color,
            stroke_width: 2.0,
            color_picker: ColorPicker::new(default_color),
            color_picker_anchor: None,
            shapes: Vec::new(),
            current_shape_start: None,
            current_shape_end: None,
            copy_requested: false,
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

// --- Main System Logic ---

// [注意] 这里增加了 frame 参数，请确保在 app.rs 中调用时传入
pub fn handle_screenshot_system(ctx: &Context, state: &mut ViewState, frame: &eframe::Frame) {
    if state.ui_mode != UiMode::Screenshot {
        return;
    }

    // 1. 初始化捕获
    if state.screenshot_state.captures.is_empty() {
        // 将 frame 传递给捕获流程
        handle_capture_process(ctx, &mut state.ui_mode, &mut state.screenshot_state, frame);
    }

    // 如果还没有捕获到内容，或者捕获失败退出了，直接返回
    if state.screenshot_state.captures.is_empty() {
        if !state.screenshot_state.is_capturing {
            state.ui_mode = UiMode::Normal;
        }
        return;
    }

    let mut final_action = ScreenshotAction::None;
    let mut wants_to_close_viewports = false;

    // 2. 渲染所有视口
    let screens_info: Vec<_> = state.screenshot_state.captures.iter().map(|c| {
        (
            c.screen_info.clone(),
            ViewportId::from_hash_of(format!("screenshot_{}", c.screen_info.name))
        )
    }).collect();

    for (i, (screen_info, viewport_id)) in screens_info.into_iter().enumerate() {
        let phys_x = screen_info.x as f32;
        let phys_y = screen_info.y as f32;
        let self_scale = screen_info.scale_factor;
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
                    let action = draw_screenshot_ui(ctx, &mut state.screenshot_state, i);
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

    // 3. 处理保存与退出
    if wants_to_close_viewports {
        let screenshot_state_mut = &mut state.screenshot_state;
        handle_save_action(final_action, screenshot_state_mut);

        // 4. 重置状态
        state.ui_mode = UiMode::Normal;
        *screenshot_state_mut = ScreenshotState::default();

        // [关键] 退出截图模式时，恢复窗口可被捕获的状态
        // 这样如果用户在正常模式下录屏，窗口是可见的
        set_window_exclude_from_capture(frame, false);

        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }
}

// --- UI Rendering Main Entry ---

pub fn draw_screenshot_ui(
    ctx: &Context,
    state: &mut ScreenshotState,
    screen_index: usize,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;

    // 准备纹理
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
            // 1. 绘制底图
            let image_widget = egui::Image::new(img_src).fit_to_exact_size(ui.available_size());
            ui.add(image_widget);

            // 2. 计算关键坐标参数
            let screen_offset_phys = Pos2::new(screen_info.x as f32, screen_info.y as f32);
            let ppp = ctx.pixels_per_point();

            // 3. 预先计算工具栏位置
            let local_toolbar_rect = calculate_toolbar_rect(state, screen_offset_phys, ppp);

            // 4. 处理交互
            if handle_interaction(ui, state, screen_offset_phys, ppp, local_toolbar_rect) {
                needs_repaint = true;
            }

            // 5. 渲染画布元素
            render_canvas_elements(ui, state, screen_offset_phys, ppp);

            // 6. 渲染工具栏
            if let Some(rect) = local_toolbar_rect {
                if ui.clip_rect().intersects(rect) {
                    let toolbar_act = render_toolbar_and_overlays(ui, state, rect);
                    if toolbar_act != ScreenshotAction::None {
                        action = toolbar_act;
                    }
                }
            }

            // 7. 鼠标放大镜 & 取色
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                let is_over_toolbar = local_toolbar_rect.map_or(false, |r| r.contains(pointer_pos));
                let is_interacting_popup = state.color_picker.is_open && ui.ctx().is_pointer_over_area();

                if !is_over_toolbar && !is_interacting_popup {
                    let screen = &state.captures[screen_index];

                    // 绘制放大镜
                    draw_magnifier(
                        ui,
                        ui.painter(),
                        &screen.image,
                        pointer_pos,
                        ppp
                    );

                    // Ctrl + C 复制颜色
                    if state.copy_requested || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
                        state.copy_requested = false;
                        let center_phys_x = (pointer_pos.x * ppp).round() as isize;
                        let center_phys_y = (pointer_pos.y * ppp).round() as isize;
                        let img_width = screen.image.width() as isize;
                        let img_height = screen.image.height() as isize;

                        if center_phys_x >= 0 && center_phys_x < img_width && center_phys_y >= 0 && center_phys_y < img_height {
                            let idx = center_phys_y as usize * screen.image.width() + center_phys_x as usize;
                            let color = screen.image.pixels[idx];
                            let hex_text = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());

                            if let Ok(mut clipboard) = Clipboard::new() {
                                if let Err(e) = clipboard.set_text(hex_text.clone()) {
                                    eprintln!("[ERROR] Failed to set clipboard text: {}", e);
                                } else {
                                    println!("[SUCCESS] Color {} copied to clipboard", hex_text);
                                }
                            }
                        }
                    }
                }
            }

            // ESC 退出
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                action = ScreenshotAction::Close;
            }
        });

    if needs_repaint {
        ctx.request_repaint();
    }

    action
}

// --- Sub-functions ---

fn calculate_toolbar_rect(state: &ScreenshotState, screen_offset_phys: Pos2, ppp: f32) -> Option<Rect> {
    if let Some(global_toolbar_pos_phys) = state.toolbar_pos {
        let vec_phys = global_toolbar_pos_phys - screen_offset_phys;
        let local_pos_logical = Pos2::ZERO + (vec_phys / ppp);
        let toolbar_width = 217.0;
        let toolbar_height = 48.0;
        let toolbar_min_pos = Pos2::new(local_pos_logical.x - toolbar_width, local_pos_logical.y + 10.0);
        Some(Rect::from_min_size(toolbar_min_pos, egui::vec2(toolbar_width, toolbar_height)))
    } else {
        None
    }
}

fn handle_interaction(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    screen_offset_phys: Pos2,
    ppp: f32,
    toolbar_rect: Option<Rect>,
) -> bool {
    let mut needs_repaint = false;
    let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::drag());

    if let Some(press_pos) = response.interact_pointer_pos() {
        let is_clicking_toolbar = toolbar_rect.map_or(false, |r| r.contains(press_pos));
        let is_interacting_with_picker = state.color_picker.is_open && ui.ctx().is_pointer_over_area();

        if !is_clicking_toolbar && !is_interacting_with_picker {
            let local_vec_phys = press_pos.to_vec2() * ppp;
            let global_phys = screen_offset_phys + local_vec_phys;

            if response.drag_started() {
                if state.current_tool.is_some() {
                    // 绘图模式
                    if let Some(selection) = state.selection {
                        if selection.contains(global_phys) {
                            state.current_shape_start = Some(global_phys);
                            state.current_shape_end = Some(global_phys);
                            needs_repaint = true;
                        }
                    }
                } else {
                    // 选区模式
                    state.drag_start = Some(global_phys);
                    state.toolbar_pos = None;
                    state.color_picker.close();
                    needs_repaint = true;
                }
            }

            if response.dragged() {
                if let Some(_) = state.current_shape_start {
                    state.current_shape_end = Some(global_phys);
                    needs_repaint = true;
                } else if let Some(drag_start_phys) = state.drag_start {
                    let rect = Rect::from_two_pos(drag_start_phys, global_phys);
                    if state.selection.map_or(true, |s| s != rect) {
                        state.selection = Some(rect);
                    }
                    needs_repaint = true;
                }
            }

            if response.drag_stopped() {
                if let Some(start_pos) = state.current_shape_start {
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
                    state.current_shape_end = None;
                    needs_repaint = true;
                } else if state.drag_start.is_some() {
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
    needs_repaint
}

fn render_canvas_elements(
    ui: &mut Ui,
    state: &ScreenshotState,
    screen_offset_phys: Pos2,
    ppp: f32,
) {
    let painter = ui.painter();
    let viewport_rect = ui.ctx().viewport_rect();
    let overlay_color = Color32::from_rgba_unmultiplied(0, 0, 0, 128);
    let full_rect = ui.max_rect();

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
        let start_local = Pos2::ZERO + ((shape.start - screen_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((shape.end - screen_offset_phys) / ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

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

    if let (Some(start_phys), Some(end_phys)) = (state.current_shape_start, state.current_shape_end) {
        let start_local = Pos2::ZERO + ((start_phys - screen_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((end_phys - screen_offset_phys) / ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        if viewport_rect.intersects(rect) {
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
    }

    if state.selection.is_none() && state.current_shape_start.is_none() {
        let inset_rect = full_rect.shrink(4.0);
        paint_style_box(painter, inset_rect, 3.0);
    }
}

fn render_toolbar_and_overlays(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    toolbar_rect: Rect,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;
    let painter = ui.painter().clone();

    let toolbar_action = draw_screenshot_toolbar(ui, &painter, state, toolbar_rect);
    if toolbar_action != ScreenshotAction::None {
        action = toolbar_action;
    }

    if state.color_picker.show(ui, state.color_picker_anchor, &mut state.stroke_width) {
        state.active_color = state.color_picker.selected_color;
        ui.ctx().request_repaint();
    }

    action
}

fn paint_style_box(painter: &egui::Painter, rect: Rect, line_width: f32) {
    let anchor_size = 6.0;
    let green = Color32::from_rgb(0, 255, 0);
    let main_stroke = Stroke::new(line_width, green);
    let anchor_stroke = Stroke::new(1.0, green);
    let anchor_fill = green;

    painter.rect_stroke(rect, 0.0, main_stroke, StrokeKind::Outside);

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
            painter.rect_stroke(anchor_rect, 0.0, anchor_stroke, StrokeKind::Outside);
        }
    }
}

// --- Helpers for System Logic (Capture, Save) ---

fn handle_capture_process(
    ctx: &Context,
    ui_mode: &mut UiMode,
    screenshot_state: &mut ScreenshotState,
    frame: &eframe::Frame // [关键] 传入 frame 以获取句柄
) {
    if !screenshot_state.is_capturing {
        screenshot_state.is_capturing = true;

        // [关键] 调用 Windows API，设置窗口为“截图透明”
        // 这样窗口可以一直显示，不会挂起，但截不下来
        set_window_exclude_from_capture(frame, true);

        ctx.request_repaint();

        let (tx, rx) = channel();
        screenshot_state.capture_receiver = Some(rx);
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            // 这里只需要极短的等待，甚至不需要 sleep，因为 API 设置是瞬时的
            // 为了保险起见，给一点点 buffer 时间
            thread::sleep(std::time::Duration::from_millis(50));
            println!("[DEBUG] Capturing screens in background...");

            let count = match Monitor::all() {
                Ok(monitors) => monitors.len(),
                Err(e) => { eprintln!("Failed to list monitors: {}", e); return; }
            };

            let captures: Vec<CapturedScreen> = thread::scope(|s| {
                let mut handles = vec![];
                for i in 0..count {
                    handles.push(s.spawn(move || {
                        let monitors = Monitor::all().ok()?;
                        if i >= monitors.len() { return None; }
                        let monitor = &monitors[i];

                        if let (Ok(image), Ok(width)) = (monitor.capture_image(), monitor.width()) {
                            if width == 0 { return None; }
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
                            Some(CapturedScreen { raw_image: Arc::new(image), image: color_image, screen_info: info, texture: None })
                        } else {
                            eprintln!("[ERROR] Failed to capture monitor {}", i);
                            None
                        }
                    }));
                }
                handles.into_iter().filter_map(|h| h.join().unwrap_or(None)).collect()
            });

            let _ = tx.send(captures);
            ctx_clone.request_repaint();
        });
    }

    if let Some(rx) = &screenshot_state.capture_receiver {
        match rx.try_recv() {
            Ok(captures) => {
                screenshot_state.captures = captures;
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;

                // [关键] 截图完成，恢复窗口正常属性
                set_window_exclude_from_capture(frame, false);

                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => { ctx.request_repaint(); }
            Err(TryRecvError::Disconnected) => {
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;
                *ui_mode = UiMode::Normal;

                // 失败也要恢复
                set_window_exclude_from_capture(frame, false);
            }
        }
    }
}

fn handle_save_action(final_action: ScreenshotAction, screenshot_state: &mut ScreenshotState) {
    if final_action == ScreenshotAction::SaveAndClose || final_action == ScreenshotAction::SaveToClipboard {
        println!("[DEBUG] Save action triggered: {:?}", final_action);

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
                                            if y0 + t >= 0 && y0 + t < final_height as i32 { final_image.put_pixel(x as u32, (y0 + t) as u32, color); }
                                            if y1 - t >= 0 && y1 - t < final_height as i32 { final_image.put_pixel(x as u32, (y1 - t) as u32, color); }
                                        }
                                    }
                                }
                                for y in y0..=y1 {
                                    for t in 0..thickness {
                                        if y >= 0 && y < final_height as i32 {
                                            if x0 + t >= 0 && x0 + t < final_width as i32 { final_image.put_pixel((x0 + t) as u32, y as u32, color); }
                                            if x1 - t >= 0 && x1 - t < final_width as i32 { final_image.put_pixel((x1 - t) as u32, y as u32, color); }
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
                                            let dist_in = if a_in > 0.0 && b_in > 0.0 { (dx * dx) / (a_in * a_in) + (dy * dy) / (b_in * b_in) } else { 2.0 };
                                            if dist <= 1.0 && dist_in >= 1.0 {
                                                if x >= 0 && x < final_width as i32 && y >= 0 && y < final_height as i32 { final_image.put_pixel(x as u32, y as u32, color); }
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

// --- Windows API Helper ---

#[cfg(target_os = "windows")]
pub fn set_window_exclude_from_capture(frame: &eframe::Frame, exclude: bool) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE};

    // 1. 获取 WindowHandle
    if let Ok(handle) = frame.window_handle() {
        // 2. [修复] 使用 .as_raw() 获取 RawWindowHandle 枚举 (raw-window-handle 0.6)
        if let RawWindowHandle::Win32(win32) = handle.as_raw() {
            // 3. [修复] 类型转换：win32.hwnd (NonZeroIsize) -> isize -> *mut c_void
            // windows 0.52 的 HWND 包装的是 *mut c_void
            let hwnd_ptr = win32.hwnd.get() as *mut std::ffi::c_void;
            let hwnd = HWND(hwnd_ptr);

            let affinity = if exclude { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
            unsafe {
                let _ = SetWindowDisplayAffinity(hwnd, affinity);
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn set_window_exclude_from_capture(_frame: &eframe::Frame, _exclude: bool) {
    // Non-Windows fallback (不做任何操作，或者使用 Visible 方案回退)
}