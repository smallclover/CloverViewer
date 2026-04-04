use eframe::egui::{Color32, Id, Pos2, Stroke, Ui};

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
    if let Some((pos_phys, mut text)) = state.active_text_input.clone() {
        let pos_local = Pos2::ZERO + ((pos_phys - global_offset_phys) / ppp);
        let max_width = if let Some(sel) = state.selection {
            let sel_max_x_local = Pos2::ZERO.x + ((sel.max.x - global_offset_phys.x) / ppp);
            (sel_max_x_local - pos_local.x - 16.0).max(20.0)
        } else {
            1000.0
        };

        let area_id = Id::new("screenshot_text_input");
        let text_edit_id = area_id.with(TEXT_EDIT_ID);

        egui::Area::new(area_id)
            .fixed_pos(pos_local)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let font_size = 20.0 + (state.stroke_width * 2.0);
                let font_id = egui::FontId::proportional(font_size);

                let galley = ui.painter().layout_no_wrap(text.clone(), font_id.clone(), Color32::WHITE);
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

                        i.events.retain(|event| {
                            match event {
                                egui::Event::Key {
                                    key: egui::Key::Enter,
                                    pressed: true,
                                    ..
                                } => {
                                    if shift_pressed {
                                        true
                                    } else {
                                        false
                                    }
                                }
                                egui::Event::Text(t) if t == "\n" || t == "\r\n" => {
                                    shift_pressed
                                }
                                _ => true,
                            }
                        });
                    });

                    let response = ui.add(
                        egui::TextEdit::multiline(&mut text)
                            .id(text_edit_id)
                            .font(font_id)
                            .text_color(state.active_color)
                            .frame(false)
                            .desired_rows(1)
                            .desired_width(dynamic_width),
                    );

                    // 首次创建时请求焦点（只请求一次）
                    if state.active_text_input.is_some() && !response.has_focus() && !response.lost_focus() {
                        response.request_focus();
                    }

                    // 焦点丢失时提交文本（点击工具栏、点击外部区域等）
                    if response.lost_focus() && !text.trim().is_empty() {
                        state.active_text_input = None;
                        commit_text_shape(ui, state, pos_phys, text, global_offset_phys, ppp);
                    } else if response.lost_focus() {
                        state.active_text_input = None;
                    } else {
                        state.active_text_input = Some((pos_phys, text));
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
