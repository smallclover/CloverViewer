use eframe::egui::{Color32, Id, Pos2, Stroke, Ui};

use crate::feature::screenshot::capture::ScreenshotState;

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

        eframe::egui::Area::new(Id::new("screenshot_text_input"))
            .fixed_pos(pos_local)
            .order(eframe::egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let font_size = 20.0 + (state.stroke_width * 2.0);
                let font_id = eframe::egui::FontId::proportional(font_size);

                let galley = ui.painter().layout_no_wrap(text.clone(), font_id.clone(), Color32::WHITE);
                let text_width = galley.size().x + 8.0;
                let dynamic_width = text_width.max(10.0).min(max_width);

                let frame = eframe::egui::Frame::default()
                    .fill(Color32::from_black_alpha(150))
                    .inner_margin(8.0)
                    .corner_radius(4.0);

                let frame_response = frame.show(ui, |ui| {
                    ui.set_max_width(max_width);

                    let mut pure_enter_pressed = false;

                    ui.input_mut(|i| {
                        let shift_pressed = i.modifiers.shift;
                        let has_valid_text = i.events.iter().any(|e| {
                            matches!(e, eframe::egui::Event::Text(t) if t != "\n" && t != "\r\n")
                        });

                        i.events.retain(|event| {
                            match event {
                                eframe::egui::Event::Key {
                                    key: eframe::egui::Key::Enter,
                                    pressed: true,
                                    ..
                                } => {
                                    if shift_pressed {
                                        true
                                    } else {
                                        if !has_valid_text {
                                            pure_enter_pressed = true;
                                        }
                                        false
                                    }
                                }
                                eframe::egui::Event::Text(t) if t == "\n" || t == "\r\n" => {
                                    shift_pressed
                                }
                                _ => true,
                            }
                        });
                    });

                    let response = ui.add(
                        eframe::egui::TextEdit::multiline(&mut text)
                            .font(font_id)
                            .text_color(state.active_color)
                            .frame(false)
                            .desired_rows(1)
                            .desired_width(dynamic_width),
                    );

                    response.request_focus();
                    state.active_text_input = Some((pos_phys, text));
                });

                let rect = frame_response.response.rect;
                let stroke = Stroke::new(1.5, Color32::from_gray(200));
                let painter = ui.painter();
                painter.add(eframe::egui::Shape::dashed_line(
                    &[rect.left_top(), rect.right_top()],
                    stroke,
                    5.0,
                    4.0,
                ));
                painter.add(eframe::egui::Shape::dashed_line(
                    &[rect.right_top(), rect.right_bottom()],
                    stroke,
                    5.0,
                    4.0,
                ));
                painter.add(eframe::egui::Shape::dashed_line(
                    &[rect.right_bottom(), rect.left_bottom()],
                    stroke,
                    5.0,
                    4.0,
                ));
                painter.add(eframe::egui::Shape::dashed_line(
                    &[rect.left_bottom(), rect.left_top()],
                    stroke,
                    5.0,
                    4.0,
                ));
            });
    }
}
