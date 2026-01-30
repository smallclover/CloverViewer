use egui::{
    Context,Area,Id,Order,Align2,Color32,
    Vec2,Spinner,RichText
};

pub fn global_loading(ctx: &Context, content: String) {
    Area::new(Id::new("global_loading"))
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .interactable(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    Spinner::new()
                        .size(32.0)
                        .color(Color32::from_gray(150)),
                );
                ui.add_space(12.0);

                let time = ui.input(|i| i.time);
                let alpha =
                    (((time * 2.0).sin() + 1.0) / 2.0 * 155.0 + 100.0) as u8;

                ui.label(
                    RichText::new(content)
                        .color(Color32::from_rgba_unmultiplied(
                            200, 200, 200, alpha,
                        ))
                        .strong(),
                );
            });
        });
}
