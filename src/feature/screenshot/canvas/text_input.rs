use eframe::egui::{Color32, Id, Pos2, Rect, Stroke, Ui};

use crate::feature::screenshot::canvas::commit_text_shape;
use crate::feature::screenshot::capture::ScreenshotState;

const TEXT_EDIT_ID: &str = "screenshot_text_edit";

/// 渲染文本输入框
pub fn render_text_input(
    ui: &mut Ui,
    state: &mut ScreenshotState,
    global_offset_phys: Pos2,
    ppp: f32,
) {
    if let Some((pos_phys, mut text)) = state.input.active_text_input.clone() {
        let font_size = 20.0 + (state.drawing.stroke_width * 2.0);
        let mut pos_local = Pos2::ZERO + ((pos_phys - global_offset_phys) / ppp);

        // 默认文本框宽度（逻辑坐标）：约1字宽 + 内边距16
        let default_box_width = font_size + 16.0;
        // 默认文本框高度（逻辑坐标）：约1行高 + 内边距16 + 行间距余量
        let default_box_height = font_size + 24.0;

        let (max_width, clip_rect) = if let Some(sel) = state.select.selection {
            let sel_min_x_local = Pos2::ZERO.x + ((sel.min.x - global_offset_phys.x) / ppp);
            let sel_max_x_local = Pos2::ZERO.x + ((sel.max.x - global_offset_phys.x) / ppp);
            let sel_min_y_local = Pos2::ZERO.y + ((sel.min.y - global_offset_phys.y) / ppp);
            let sel_max_y_local = Pos2::ZERO.y + ((sel.max.y - global_offset_phys.y) / ppp);

            // 右侧空间不够 → 左移，保留 8px 右边距 ⬅不需要现在以及满足
            if sel_max_x_local - pos_local.x < default_box_width {
                pos_local.x = (sel_max_x_local - default_box_width).max(sel_min_x_local);
            }

            // 下方空间不够 → 上移，保留 8px 下边距
            if sel_max_y_local - pos_local.y < default_box_height {
                pos_local.y = (sel_max_y_local - default_box_height - 8.0).max(sel_min_y_local);
            }

            let adjusted_width = (sel_max_x_local - pos_local.x).max(20.0);
            let sel_clip = Rect::from_min_max(
                Pos2::new(sel_min_x_local, sel_min_y_local),
                Pos2::new(sel_max_x_local, sel_max_y_local),
            );
            (adjusted_width, Some(sel_clip))
        } else {
            (1000.0, None)
        };

        // 将调整后的 pos_local 转回物理坐标，用于提交和状态更新
        let adjusted_pos_phys = global_offset_phys + (pos_local - Pos2::ZERO) * ppp;

        let area_id = Id::new("screenshot_text_input");
        let text_edit_id = area_id.with(TEXT_EDIT_ID);

        egui::Area::new(area_id)
            .fixed_pos(pos_local)
            .order(egui::Order::Foreground)
            .show(ui, |ui| {
                if let Some(rect) = clip_rect {
                    ui.set_clip_rect(rect);
                }
                let font_id = egui::FontId::proportional(font_size);

                let galley =
                    ui.painter()
                        .layout_no_wrap(text.clone(), font_id.clone(), Color32::WHITE);
                let text_width = galley.size().x + 8.0;
                let dynamic_width = text_width.max(10.0).min(max_width);

                let frame = egui::Frame::default()
                    .fill(Color32::from_black_alpha(150))
                    .inner_margin(8.0)
                    .corner_radius(4.0);

                let frame_response = frame.show(ui, |ui| {
                    ui.set_max_width(max_width);

                    ui.input_mut(|i| {
                        let shift_pressed = i.modifiers.shift;

                        i.events.retain(|event| match event {
                            egui::Event::Key {
                                key: egui::Key::Enter,
                                pressed: true,
                                ..
                            } => shift_pressed,
                            egui::Event::Text(t) if t == "\n" || t == "\r\n" => shift_pressed,
                            _ => true,
                        });
                    });

                    let response = ui.add(
                        egui::TextEdit::multiline(&mut text)
                            .id(text_edit_id)
                            .font(font_id)
                            .text_color(state.drawing.active_color)
                            .frame(egui::Frame::NONE)
                            .desired_rows(1)
                            .desired_width(dynamic_width),
                    );

                    // 首次创建时请求焦点（只请求一次）
                    if state.input.active_text_input.is_some()
                        && !response.has_focus()
                        && !response.lost_focus()
                    {
                        response.request_focus();
                    }

                    // 焦点丢失时提交文本（点击工具栏、点击外部区域等）
                    if response.lost_focus() && !text.trim().is_empty() {
                        state.input.active_text_input = None;
                        commit_text_shape(
                            ui,
                            state,
                            adjusted_pos_phys,
                            text,
                            global_offset_phys,
                            ppp,
                        );
                    } else if response.lost_focus() {
                        state.input.active_text_input = None;
                    } else {
                        state.input.active_text_input = Some((adjusted_pos_phys, text));
                    }
                });

                let rect = frame_response.response.rect;
                let stroke = Stroke::new(1.5, Color32::from_gray(200));
                let painter = ui.painter();
                painter.add(egui::Shape::dashed_line(
                    &[rect.left_top(), rect.right_top()],
                    stroke,
                    5.0,
                    4.0,
                ));
                painter.add(egui::Shape::dashed_line(
                    &[rect.right_top(), rect.right_bottom()],
                    stroke,
                    5.0,
                    4.0,
                ));
                painter.add(egui::Shape::dashed_line(
                    &[rect.right_bottom(), rect.left_bottom()],
                    stroke,
                    5.0,
                    4.0,
                ));
                painter.add(egui::Shape::dashed_line(
                    &[rect.left_bottom(), rect.left_top()],
                    stroke,
                    5.0,
                    4.0,
                ));
            });
    }
}
