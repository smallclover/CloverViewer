use eframe::egui::{
    Color32, ColorImage, Context,
    Pos2, Rect, Stroke, StrokeKind, TextureHandle,
    Ui, ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};
use image::{GenericImage, RgbaImage};
use std::{
    borrow::Cow,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, TryRecvError},
        Arc,
    },
    thread,
    time::Duration
};
use std::collections::HashMap;
use xcap::Monitor;
use arboard::{Clipboard, ImageData};
use crate::ui::{
    mode::UiMode,
    screenshot::color_picker::ColorPicker,
    screenshot::magnifier::handle_magnifier

};
use crate::model::{
    config::get_context_config,
    device::{DeviceInfo, MonitorInfo},
    state::ViewState
};
use crate::ui::screenshot::toolbar::{calculate_toolbar_rect, render_toolbar_and_overlays};
// --- 类型定义 ---

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotAction {
    None,
    Close,
    SaveAndClose,
    SaveToClipboard,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ScreenshotTool {
    Rect,
    Circle,
}

#[derive(Clone)]
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

    // 用于窗口自动吸附
    pub window_rects: Vec<Rect>,
    pub hovered_window: Option<Rect>,

    // Async capture state
    pub is_capturing: bool,
    pub capture_receiver: Option<Receiver<(Vec<CapturedScreen>, Vec<Rect>)>>,

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
    pub texture_pool: HashMap<String, TextureHandle>,
}

impl Default for ScreenshotState {
    fn default() -> Self {
        let default_color = Color32::from_rgb(204, 0, 0);
        Self {
            captures: Vec::new(),
            selection: None,
            drag_start: None,
            toolbar_pos: None,
            window_rects: Vec::new(),
            hovered_window: None,
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
            texture_pool: HashMap::new()
        }
    }
}

#[derive(Clone)]
pub struct CapturedScreen {
    pub raw_image: Arc<RgbaImage>,
    pub image: ColorImage,
    pub screen_info: MonitorInfo,
}

// --- Main System Logic ---

pub fn handle_screenshot_system(ctx: &Context, state: &mut ViewState) {
    if state.ui_mode != UiMode::Screenshot {
        return;
    }

    // 1. 初始化捕获
    if state.screenshot_state.captures.is_empty() {
        handle_capture_process(ctx, &mut state.ui_mode, &mut state.screenshot_state);
    }

    if state.screenshot_state.captures.is_empty() {
        if !state.screenshot_state.is_capturing {
            state.ui_mode = UiMode::Normal;
        }
        return;
    }

    let mut final_action = ScreenshotAction::None;
    let mut wants_to_close_viewports = false;
    // 拿到启动画布需要的坐标和尺寸
    let (pos, size) = state.device_info.global_logical_rect(1.0);

    let viewport_id = ViewportId::from_hash_of("screenshot_global_canvas");

    ctx.show_viewport_immediate(
        viewport_id,
        ViewportBuilder::default()
            .with_title("Global Screenshot Canvas")
            .with_position(pos)
            .with_min_inner_size(size)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(),
        |ctx, class| {
            if class == ViewportClass::Immediate {
                // 整个大画布的绘制逻辑入口，不再传递 screen_index
                let action = draw_screenshot_ui(ctx, &mut state.screenshot_state, &state.device_info);
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

    // 3. 处理保存与退出
    if wants_to_close_viewports {
        let screenshot_state_mut = &mut state.screenshot_state;
        handle_save_action(final_action, screenshot_state_mut);

        state.ui_mode = UiMode::Normal;
        *screenshot_state_mut = ScreenshotState::default();

        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }
}

// --- UI Rendering Main Entry ---

pub fn draw_screenshot_ui(
    ctx: &Context,
    state: &mut ScreenshotState,
    device_info: &DeviceInfo,
) -> ScreenshotAction {
    let mut action = ScreenshotAction::None;

    // 大画布全局左上角的物理坐标
    let global_offset_phys = Pos2::new(device_info.phys_min_x as f32, device_info.phys_min_y as f32);
    let ppp = ctx.pixels_per_point();

    egui::CentralPanel::default()
        // 确保底图透明
        .frame(egui::Frame::NONE.fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            let painter = ui.painter();

            // 1. 绘制所有底图纹理 (基于物理偏移除以缩放率映射)
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

            // 2. 动态计算当前悬停的窗口
            state.hovered_window = None;
            let is_hovered = ui.rect_contains_pointer(ui.max_rect());

            if is_hovered && state.selection.is_none() && state.drag_start.is_none() {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);

                    for rect in &state.window_rects {
                        if rect.contains(global_pointer_phys) {
                            // 判断是否为全屏窗口 (允许存在一点点误差)
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

            // 3. 预先计算工具栏位置
            let local_toolbar_rect = calculate_toolbar_rect(state, global_offset_phys, ppp);

            // 4. 处理交互
            handle_interaction(ui, state, global_offset_phys, ppp, local_toolbar_rect);

            // 5. 渲染画布元素 (遮罩、边框、绘制的图形)
            render_canvas_elements(ui, state, global_offset_phys, ppp, is_hovered);

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
            let config = get_context_config(ctx);
            if config.magnifier_enabled {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let is_over_toolbar = local_toolbar_rect.map_or(false, |r| r.contains(pointer_pos));
                    let is_interacting_popup = state.color_picker.is_open && ui.ctx().is_pointer_over_area();

                    if !is_over_toolbar && !is_interacting_popup {
                        // 所有计算和复制逻辑都已被提取到 magnifier.rs 内
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

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                action = ScreenshotAction::Close;
            }
        });

    // 强制每一帧都重绘，确保极致丝滑
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
        let start_local = Pos2::ZERO + ((start_phys - global_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((end_phys - global_offset_phys) / ppp);
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
                // 我们通过鼠标的全局物理坐标，找出它当前所在的那个具体显示器
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let global_pointer_phys = global_offset_phys + (pointer_pos.to_vec2() * ppp);

                    for cap in &state.captures {
                        let cap_phys_rect = Rect::from_min_size(
                            Pos2::new(cap.screen_info.x as f32, cap.screen_info.y as f32),
                            egui::vec2(cap.screen_info.width as f32, cap.screen_info.height as f32)
                        );

                        if cap_phys_rect.contains(global_pointer_phys) {
                            // 找到了鼠标当前所在的显示器，将它的物理边界映射回逻辑边界
                            let vec_min = cap_phys_rect.min - global_offset_phys;
                            let vec_max = cap_phys_rect.max - global_offset_phys;

                            let local_logical_rect = Rect::from_min_max(
                                Pos2::ZERO + (vec_min / ppp),
                                Pos2::ZERO + (vec_max / ppp),
                            );

                            // 往内缩 4 个像素画绿框，完美贴合单块屏幕边缘
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
) {
    if !screenshot_state.is_capturing {
        screenshot_state.is_capturing = true;

        ctx.request_repaint();

        let (tx, rx) = channel();
        screenshot_state.capture_receiver = Some(rx);
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
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

                // 纹理处理
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
        // 基于绝对的物理坐标来运算的，
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

                    // 1. 拼接底层截图图像
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

                    // 2. 绘制标注图形 (使用 tiny-skia 进行完美抗锯齿渲染)
                    if let Some(mut pixmap) = tiny_skia::PixmapMut::from_bytes(
                        &mut final_image,
                        final_width,
                        final_height,
                    ) {
                        for shape in shapes {
                            let start_x = shape.start.x - selection_phys.min.x;
                            let start_y = shape.start.y - selection_phys.min.y;
                            let end_x = shape.end.x - selection_phys.min.x;
                            let end_y = shape.end.y - selection_phys.min.y;

                            let x0 = start_x.min(end_x);
                            let y0 = start_y.min(end_y);
                            let width = (start_x - end_x).abs();
                            let height = (start_y - end_y).abs();

                            if width <= 0.0 || height <= 0.0 { continue; }

                            let mut paint = tiny_skia::Paint::default();
                            paint.set_color_rgba8(shape.color.r(), shape.color.g(), shape.color.b(), shape.color.a());
                            paint.anti_alias = true;

                            let stroke = tiny_skia::Stroke {
                                width: shape.stroke_width,
                                line_cap: tiny_skia::LineCap::Round,
                                line_join: tiny_skia::LineJoin::Round,
                                ..Default::default()
                            };

                            let transform = tiny_skia::Transform::identity();

                            if let Some(rect) = tiny_skia::Rect::from_xywh(x0, y0, width, height) {
                                match shape.tool {
                                    ScreenshotTool::Rect => {
                                        let path = tiny_skia::PathBuilder::from_rect(rect);
                                        pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                                    }
                                    ScreenshotTool::Circle => {
                                        let path = tiny_skia::PathBuilder::from_oval(rect).unwrap();
                                        pixmap.stroke_path(&path, &paint, &stroke, transform, None);
                                    }
                                }
                            }
                        }
                    }

                    // 3. 执行保存或复制动作
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