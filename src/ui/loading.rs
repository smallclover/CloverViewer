pub fn loading(ui: &mut egui::Ui){
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            // 稍微深色一点的菊花
            ui.add(egui::Spinner::new().size(32.0).color(egui::Color32::from_gray(150)));
            ui.add_space(12.0);

            // 根据时间生成一点动态感（简单的渐变文字）
            let time = ui.input(|i| i.time);
            let alpha = (((time * 2.0).sin() + 1.0) / 2.0 * 155.0 + 100.0) as u8;

            ui.label(egui::RichText::new("正在解析像素...")
                .color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha))
                .strong());
        });
    });
}