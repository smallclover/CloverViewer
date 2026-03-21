use eframe::emath::{Pos2, Rect};
use eframe::epaint::{Color32, Stroke, StrokeKind};
use egui::Ui;
use crate::feature::screenshot::capture::{DrawnShape, HistoryEntry, ScreenshotState, ScreenshotTool};
use crate::feature::screenshot::draw::draw_egui_shape;
use crate::utils::screen::find_target_screen_rect;

/// 核心辅助函数：根据指定的坐标检测命中了哪个图形
fn get_hovered_shape_index(
    pos: Pos2,
    state: &ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    painter: &egui::Painter,
) -> Option<usize> {
    for (index, shape) in state.shapes.iter().enumerate().rev() {
        let start_local = Pos2::ZERO + ((shape.start - global_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((shape.end - global_offset_phys) / ppp);
        let shape_rect = Rect::from_two_pos(start_local, end_local);

        let grab_tolerance = (shape.stroke_width / ppp).clamp(4.0, 8.0);

        let is_hovered = match shape.tool {
            ScreenshotTool::Rect => {
                let expanded = shape_rect.expand(grab_tolerance);
                let shrunk = shape_rect.shrink(grab_tolerance);
                expanded.contains(pos) && (!shrunk.is_positive() || !shrunk.contains(pos))
            },
            ScreenshotTool::Circle => {
                let center = shape_rect.center();
                let a = shape_rect.width() / 2.0;
                let b = shape_rect.height() / 2.0;
                let dx = pos.x - center.x;
                let dy = pos.y - center.y;
                let dist = pos.distance(center);

                if dist < 0.1 || a < 0.1 || b < 0.1 {
                    false
                } else {
                    let cos_t = dx / dist;
                    let sin_t = dy / dist;
                    let r_theta = (a * b) / ((b * cos_t).powi(2) + (a * sin_t).powi(2)).sqrt();
                    (dist - r_theta).abs() <= grab_tolerance
                }
            },
            ScreenshotTool::Arrow => {
                dist_to_line_segment(pos, start_local, end_local) <= grab_tolerance
            },
            ScreenshotTool::Text => {
                if let Some(text) = &shape.text {
                    let font_size = 20.0 + (shape.stroke_width * 2.0);
                    let galley = painter.layout_no_wrap(text.clone(), egui::FontId::proportional(font_size), Color32::WHITE);
                    let text_rect = Rect::from_min_size(start_local, galley.size());
                    text_rect.expand(4.0).contains(pos)
                } else {
                    false
                }
            }
        };

        if is_hovered {
            return Some(index);
        }
    }
    None
}

/// 处理与画布的交互（拖拽选区、画图、打字、移动形状）
pub fn handle_interaction(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
    toolbar_rect: Option<Rect>,
) {
    let response = ui.interact(ui.max_rect(), ui.id().with("screenshot_background"), egui::Sense::click_and_drag());

    if response.clicked() {
        if state.current_tool.is_none() {
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
    }

    // ==========================================
    // 1. 引入按下状态锁 (Press State Lock)
    // ==========================================
    let is_pointer_down = ui.ctx().input(|i| i.pointer.primary_down());
    let hover_id = egui::Id::new("hovered_shape_index");

    // 取出上一帧的悬停状态
    let mut visual_hovered_index = ui.data(|d| d.get_temp::<Option<usize>>(hover_id).unwrap_or(None));

    // 【新增】：记录当前鼠标是否悬停在工具栏或颜色拾取器上
    let mut is_hovering_ui = false;

    if let Some(pos) = ui.ctx().pointer_latest_pos() {
        let is_clicking_toolbar = toolbar_rect.map_or(false, |r| r.contains(pos));
        let is_interacting_with_picker = state.color_picker.is_open && ui.ctx().is_pointer_over_area();
        is_hovering_ui = is_clicking_toolbar || is_interacting_with_picker;

        // 只有在鼠标“未按下”时，才实时更新悬停目标
        if !is_pointer_down {
            if !is_hovering_ui {
                visual_hovered_index = get_hovered_shape_index(pos, state, global_offset_phys, ppp, ui.painter());
            } else {
                visual_hovered_index = None;
            }
            ui.data_mut(|d| d.insert_temp(hover_id, visual_hovered_index));
        }
    } else {
        if !is_pointer_down {
            visual_hovered_index = None;
            ui.data_mut(|d| d.insert_temp(hover_id, visual_hovered_index));
        }
    }

    let dragging_id = egui::Id::new("dragging_shape_index");
    let mut dragging_index = ui.data(|d| d.get_temp::<usize>(dragging_id));

    // 只要锁定了某个图形，或正在拖拽中，就进入强制移动状态，严格禁止画图！
    let is_moving_state = visual_hovered_index.is_some() || dragging_index.is_some();
    let can_draw = !is_moving_state;

    // ==========================================
    // 光标反馈优先级
    // ==========================================
    if is_hovering_ui {
        // 如果悬停在工具栏上，强制恢复默认的鼠标箭头模式
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
    } else if is_moving_state && state.current_shape_start.is_none() {
        // 拖拽状态显示十字箭头
        ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
    } else if state.current_tool.is_some() {
        // 画图状态显示准星
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }

    // ==========================================
    // 2. 真实交互处理
    // ==========================================
    if let Some(press_pos) = response.interact_pointer_pos() {
        let is_clicking_toolbar = toolbar_rect.map_or(false, |r| r.contains(press_pos));
        let is_interacting_with_picker = state.color_picker.is_open && ui.ctx().is_pointer_over_area();

        if !is_clicking_toolbar && !is_interacting_with_picker {
            let local_vec_phys = press_pos.to_vec2() * ppp;
            let mut global_phys = global_offset_phys + local_vec_phys;

            // 直接使用上面被锁死的 visual_hovered_index，无论你手速多快拖出几像素，系统只认你按下瞬间的状态！
            let interaction_hovered_index = visual_hovered_index;

            // --- 文本工具的点击事件 ---
            if state.current_tool == Some(ScreenshotTool::Text) {
                if response.clicked() && can_draw {
                    if let Some(sel) = state.selection {
                        global_phys = clamp_pos_to_rect(global_phys, sel);
                    }
                    if let Some((pos, text)) = state.active_text_input.take() {
                        if !text.trim().is_empty() {
                            state.history.push(HistoryEntry { shapes: state.shapes.clone(), selection: state.selection });
                            state.shapes.push(DrawnShape {
                                tool: ScreenshotTool::Text,
                                start: pos,
                                end: pos,
                                color: state.active_color,
                                stroke_width: state.stroke_width,
                                text: Some(text),
                            });
                        }
                    }
                    state.active_text_input = Some((global_phys, String::new()));
                }
            }

            // --- 开始拖拽 ---
            if response.drag_started() {
                if let Some(index) = interaction_hovered_index {
                    // 拖拽已有形状
                    dragging_index = Some(index);
                    ui.data_mut(|d| d.insert_temp(dragging_id, index));
                    state.drag_start = Some(global_phys);
                    state.history.push(HistoryEntry { shapes: state.shapes.clone(), selection: state.selection });
                } else if can_draw && state.current_tool.is_some() {
                    // 开始画新图
                    if let Some(selection) = state.selection {
                        if selection.contains(global_phys) && state.current_tool != Some(ScreenshotTool::Text) {
                            state.current_shape_start = Some(global_phys);
                            state.current_shape_end = Some(global_phys);
                        }
                    }
                } else if can_draw {
                    // 拉动新的截图选区
                    state.drag_start = Some(global_phys);
                    state.toolbar_pos = None;
                    state.color_picker.close();
                }
            }

            // --- 拖拽过程中 ---
            // 获取当前实时坐标
            let current_phys = global_offset_phys + (ui.ctx().pointer_latest_pos().unwrap_or(press_pos).to_vec2() * ppp);

            if response.dragged() {
                if let Some(index) = dragging_index {
                    if let Some(drag_start_phys) = state.drag_start {
                        let delta_phys = current_phys - drag_start_phys;
                        if let Some(shape) = state.shapes.get_mut(index) {
                            let mut dx = delta_phys.x;
                            let mut dy = delta_phys.y;

                            if let Some(sel) = state.selection {
                                let min_x = shape.start.x.min(shape.end.x);
                                let max_x = shape.start.x.max(shape.end.x);
                                let min_y = shape.start.y.min(shape.end.y);
                                let max_y = shape.start.y.max(shape.end.y);

                                if max_x - min_x <= sel.width() {
                                    if min_x + dx < sel.min.x { dx = sel.min.x - min_x; }
                                    if max_x + dx > sel.max.x { dx = sel.max.x - max_x; }
                                }
                                if max_y - min_y <= sel.height() {
                                    if min_y + dy < sel.min.y { dy = sel.min.y - min_y; }
                                    if max_y + dy > sel.max.y { dy = sel.max.y - max_y; }
                                }
                            }

                            let clamped_delta = eframe::emath::Vec2::new(dx, dy);
                            shape.start += clamped_delta;
                            shape.end += clamped_delta;
                            state.drag_start = Some(drag_start_phys + clamped_delta);
                        }
                    }
                } else if let Some(_) = state.current_shape_start {
                    let mut clamped_phys = current_phys;
                    if let Some(sel) = state.selection {
                        clamped_phys = clamp_pos_to_rect(current_phys, sel);
                    }
                    state.current_shape_end = Some(clamped_phys);
                } else if let Some(drag_start_phys) = state.drag_start {
                    let rect = Rect::from_two_pos(drag_start_phys, current_phys);
                    if state.selection.map_or(true, |s| s != rect) {
                        state.selection = Some(rect);
                    }
                }
            }

            // --- 拖拽结束 ---
            if response.drag_stopped() {
                if dragging_index.is_some() {
                    ui.data_mut(|d| d.remove::<usize>(dragging_id));
                    state.drag_start = None;
                } else if let Some(start_pos) = state.current_shape_start {
                    let end_pos = state.current_shape_end.unwrap_or(current_phys);

                    // 防手抖：即使能画图，拖动距离小于 5 像素也不生成微小垃圾图形
                    if start_pos.distance(end_pos) > 5.0 {
                        if let Some(tool) = state.current_tool {
                            state.history.push(HistoryEntry { shapes: state.shapes.clone(), selection: state.selection });
                            state.shapes.push(DrawnShape {
                                tool,
                                start: start_pos,
                                end: end_pos,
                                color: state.active_color,
                                stroke_width: state.stroke_width,
                                text: None,
                            });
                        }
                    }
                    state.current_shape_start = None;
                    state.current_shape_end = None;
                } else if state.drag_start.is_some() {
                    state.drag_start = None;
                    if let Some(sel) = state.selection {
                        if sel.width() > 10.0 && sel.height() > 10.0 {
                            state.history.push(HistoryEntry { shapes: state.shapes.clone(), selection: state.selection });
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

pub fn render_canvas_elements(
    ui: &mut Ui,
    state: &mut ScreenshotState,
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

    let dragging_id = egui::Id::new("dragging_shape_index");
    let dragging_index = ui.data(|d| d.get_temp::<usize>(dragging_id));
    let hover_id = egui::Id::new("hovered_shape_index");
    let hovered_index = ui.data(|d| d.get_temp::<Option<usize>>(hover_id)).unwrap_or(None);

    for (index, shape) in state.shapes.iter().enumerate() {
        let start_local = Pos2::ZERO + ((shape.start - global_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((shape.end - global_offset_phys) / ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        let mut is_visible = viewport_rect.intersects(rect);
        let mut text_rect = rect;

        if shape.tool == ScreenshotTool::Text {
            if let Some(ref text) = shape.text {
                let font_size = 20.0 + (shape.stroke_width * 2.0);
                let galley = painter.layout_no_wrap(text.clone(), egui::FontId::proportional(font_size), Color32::WHITE);
                text_rect = Rect::from_min_size(start_local, galley.size());
                is_visible = viewport_rect.intersects(text_rect);
            }
        }

        if is_visible {
            if (Some(index) == hovered_index || Some(index) == dragging_index) && state.current_shape_start.is_none() {
                let highlight_rect = if shape.tool == ScreenshotTool::Text { text_rect } else { rect };
                painter.rect_stroke(
                    highlight_rect.expand(2.0),
                    2.0,
                    Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 150, 255, 100)),
                    StrokeKind::Outside
                );
            }

            if shape.tool == ScreenshotTool::Text {
                if let Some(ref text) = shape.text {
                    let font_size = 20.0 + (shape.stroke_width * 2.0);
                    painter.text(
                        start_local,
                        egui::Align2::LEFT_TOP,
                        text,
                        egui::FontId::proportional(font_size),
                        shape.color,
                    );
                }
            } else {
                draw_egui_shape(painter, shape.tool, rect, start_local, end_local, shape.stroke_width, shape.color);
            }
        }
    }

    if let (Some(start_phys), Some(end_phys)) = (state.current_shape_start, state.current_shape_end) {
        let start_local = Pos2::ZERO + ((start_phys - global_offset_phys) / ppp);
        let end_local = Pos2::ZERO + ((end_phys - global_offset_phys) / ppp);
        let rect = Rect::from_two_pos(start_local, end_local);

        if viewport_rect.intersects(rect) {
            if let Some(tool) = state.current_tool {
                draw_egui_shape(painter, tool, rect, start_local, end_local, state.stroke_width, state.active_color);
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

                    if let Some(cap_phys_rect) = find_target_screen_rect(&state.captures, global_pointer_phys) {
                        let vec_min = cap_phys_rect.min - global_offset_phys;
                        let vec_max = cap_phys_rect.max - global_offset_phys;

                        let local_logical_rect = Rect::from_min_max(
                            Pos2::ZERO + (vec_min / ppp),
                            Pos2::ZERO + (vec_max / ppp),
                        );

                        paint_style_box(painter, local_logical_rect, 3.0);
                    }
                }
            }
        } else {
            painter.rect_filled(viewport_rect, 0.0, overlay_color);
        }
    }

    if let Some((pos_phys, mut text)) = state.active_text_input.clone() {
        let pos_local = Pos2::ZERO + ((pos_phys - global_offset_phys) / ppp);

        egui::Area::new(egui::Id::new("screenshot_text_input"))
            .fixed_pos(pos_local)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let font_size = 20.0 + (state.stroke_width * 2.0);
                let frame = egui::Frame::default()
                    .fill(Color32::from_black_alpha(150))
                    .stroke(Stroke::new(1.5, state.active_color))
                    .inner_margin(8.0)
                    .corner_radius(4.0);

                frame.show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut text)
                            .font(egui::FontId::proportional(font_size))
                            .text_color(state.active_color)
                            .frame(false)
                            .desired_width(120.0)
                    );
                    response.request_focus();
                    state.active_text_input = Some((pos_phys, text));
                });
            });
    }
}

pub fn paint_style_box(painter: &egui::Painter, rect: Rect, line_width: f32) {
    let anchor_size = 6.0;
    let green = Color32::from_rgb(0, 255, 0);
    let main_stroke = Stroke::new(line_width, green);
    let anchor_stroke = Stroke::new(1.0, green);
    let anchor_fill = green;

    painter.rect_stroke(rect, 0.0, main_stroke, StrokeKind::Inside);

    if rect.width() > anchor_size * 3.0 && rect.height() > anchor_size * 3.0 {
        let inset = anchor_size / 2.0;
        let min = rect.min + egui::vec2(inset, inset);
        let max = rect.max - egui::vec2(inset, inset);
        let center = rect.center();

        let anchors = [
            min, Pos2::new(max.x, min.y), max, Pos2::new(min.x, max.y),
            Pos2::new(center.x, min.y), Pos2::new(center.x, max.y),
            Pos2::new(min.x, center.y), Pos2::new(max.x, center.y),
        ];

        for anchor_pos in anchors {
            let anchor_rect = Rect::from_center_size(anchor_pos, egui::vec2(anchor_size, anchor_size));
            painter.rect_filled(anchor_rect, 0.0, anchor_fill);
            painter.rect_stroke(anchor_rect, 0.0, anchor_stroke, StrokeKind::Inside);
        }
    }
}

fn dist_to_line_segment(p: Pos2, v: Pos2, w: Pos2) -> f32 {
    let l2 = v.distance_sq(w);
    if l2 == 0.0 { return p.distance(v); }
    let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    let projection = Pos2::new(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y));
    p.distance(projection)
}

fn clamp_pos_to_rect(pos: Pos2, rect: Rect) -> Pos2 {
    Pos2::new(
        pos.x.clamp(rect.min.x, rect.max.x),
        pos.y.clamp(rect.min.y, rect.max.y),
    )
}