use eframe::egui::{self, Color32, ColorImage, Context, Pos2, Rect, Stroke, StrokeKind, TextureHandle, Ui, ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};
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
use xcap::Monitor;
use crate::model::state::ViewState;
use crate::ui::{
    mode::UiMode,
    screenshot::toolbar::draw_screenshot_toolbar
};
use arboard::{Clipboard, ImageData};
use crate::ui::screenshot::color_picker::ColorPicker;
use crate::ui::screenshot::magnifier::draw_magnifier;
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

    // [新增] 用于窗口自动吸附
    pub window_rects: Vec<Rect>,
    pub hovered_window: Option<Rect>,

    // Async capture state
    pub is_capturing: bool,
    // [修改] 管道类型加上 Vec<Rect> 来接收窗口数据
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

            // [新增] 3. 动态计算当前悬停的窗口
            state.hovered_window = None; // 每一帧重置
            let is_hovered = ui.rect_contains_pointer(ui.max_rect());

            if is_hovered && state.selection.is_none() && state.drag_start.is_none() {
                if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                    let global_pointer_phys = screen_offset_phys + (pointer_pos.to_vec2() * ppp);

                    // 遍历所有窗口 (通常是按 Z-order 从顶到底)
                    for rect in &state.window_rects {
                        if rect.contains(global_pointer_phys) {
                            // 检查是否为全屏/桌面 (如果接近显示器大小，就不视为特殊窗口)
                            let is_fullscreen = (rect.width() - screen_info.width as f32).abs() < 5.0
                                && (rect.height() - screen_info.height as f32).abs() < 5.0;
                            if !is_fullscreen {
                                state.hovered_window = Some(*rect);
                            }
                            break; // 找到最顶层包含鼠标的窗口即跳出
                        }
                    }
                }
            }
            if state.selection.is_none() && state.drag_start.is_none() { needs_repaint = true; }

            // 3. 预先计算工具栏位置
            let local_toolbar_rect = calculate_toolbar_rect(state, screen_offset_phys, ppp);



            // 4. 处理交互
            if handle_interaction(ui, state, screen_offset_phys, ppp, local_toolbar_rect) {
                needs_repaint = true;
            }

            // 5. 渲染画布元素
            render_canvas_elements(ui, state, screen_offset_phys, ppp, is_hovered);

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
    let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::click_and_drag());

    // [新增] 处理单击事件：直接将悬停窗口转换为选区
    if response.clicked() {
        if state.current_tool.is_none() && state.hovered_window.is_some() {
            state.selection = state.hovered_window;
            // 将工具栏放在窗口右下角
            if let Some(sel) = state.selection {
                state.toolbar_pos = Some(sel.right_bottom());
            }
            needs_repaint = true;
            return needs_repaint; // 点击后直接返回，不走拖拽逻辑
        }
    }

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
    is_hovered: bool,
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

    // [修改最末尾的部分] 当没有形成选区，也没有正在拖拽绘制时
    if state.selection.is_none() && state.current_shape_start.is_none() && state.drag_start.is_none() {
        if is_hovered {
            if let Some(hover_phys_rect) = state.hovered_window {
                // 鼠标在具体的小窗口上，周围打上灰色遮罩，窗口亮起并加上绿框
                let vec_min = hover_phys_rect.min - screen_offset_phys;
                let vec_max = hover_phys_rect.max - screen_offset_phys;

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
                // 鼠标在桌面上或全屏窗口上：没有遮罩，整个显示器边缘绿框
                let inset_rect = full_rect.shrink(4.0);
                paint_style_box(painter, inset_rect, 3.0);
            }
        } else {
            // 鼠标不在当前屏幕：全屏半透明灰色遮罩
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }
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

            // [新增] 捕获当前所有可见窗口的物理边界
            let mut window_rects = Vec::new();
            if let Ok(windows) = xcap::Window::all() {
                for w in windows {
                    // 只记录可见、未最小化的窗口
                    if !w.is_minimized().unwrap() {
                        let app_name = w.app_name().unwrap_or_default().to_lowercase();

                        // 过滤掉我们自己的程序 "CloverViewer" (避免悬停在自己生成的透明蒙版上)
                        if app_name.contains("cloverviewer") || app_name.contains("screenshot") {
                            continue;
                        }

                        let rect = Rect::from_min_size(
                            Pos2::new(w.x().unwrap() as f32, w.y().unwrap() as f32),
                            egui::vec2(w.width().unwrap() as f32, w.height().unwrap() as f32)
                        );
                        // 忽略尺寸太小的幽灵窗口
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
                screenshot_state.captures = captures;
                screenshot_state.window_rects = window_rects; // 保存窗口边界
                screenshot_state.is_capturing = false;
                screenshot_state.capture_receiver = None;

                // [关键] 截图完成，恢复窗口正常属性
                set_window_exclude_from_capture(frame, false);

                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                // 16ms 定时刷新，减轻GPU压力
                ctx.request_repaint_after(Duration::from_millis(16));
            }
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
                    // 将 image 的底层缓冲区直接包装为 tiny-skia 的画布
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

                            // 确定图形边界，处理反向拖动的情况
                            let x0 = start_x.min(end_x);
                            let y0 = start_y.min(end_y);
                            let width = (start_x - end_x).abs();
                            let height = (start_y - end_y).abs();

                            if width <= 0.0 || height <= 0.0 { continue; }

                            // 配置画笔
                            let mut paint = tiny_skia::Paint::default();
                            // 注意：egui 颜色和 tiny-skia 的接收方式对应
                            paint.set_color_rgba8(shape.color.r(), shape.color.g(), shape.color.b(), shape.color.a());
                            paint.anti_alias = true; // 开启抗锯齿

                            // 配置笔触 (描边粗细)
                            let stroke = tiny_skia::Stroke {
                                width: shape.stroke_width,
                                line_cap: tiny_skia::LineCap::Round,
                                line_join: tiny_skia::LineJoin::Round,
                                ..Default::default()
                            };

                            let transform = tiny_skia::Transform::identity();

                            // 仅当能正确构建有效矩形时才进行绘制
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