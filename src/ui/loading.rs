pub fn global_loading(ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("global_loading"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .interactable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Spinner::new()
                        .size(32.0)
                        .color(egui::Color32::from_gray(150)),
                );
                ui.add_space(12.0);

                let time = ui.input(|i| i.time);
                let alpha =
                    (((time * 2.0).sin() + 1.0) / 2.0 * 155.0 + 100.0) as u8;

                ui.label(
                    egui::RichText::new("正在解析像素...")
                        .color(egui::Color32::from_rgba_unmultiplied(
                            200, 200, 200, alpha,
                        ))
                        .strong(),
                );
            });
        });
}

pub fn corner_loading(ui: &mut egui::Ui) {
    let rect = ui.max_rect();

    let size = egui::vec2(24.0, 24.0);
    let pos = egui::pos2(
        rect.right() - size.x - 8.0,
        rect.top() + 8.0,
    );

    let spinner_rect = egui::Rect::from_min_size(pos, size);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(spinner_rect),
        |ui| {
            ui.add(
                egui::Spinner::new()
                    .size(20.0)
                    .color(egui::Color32::from_gray(160)),
            );
        },
    );
}

