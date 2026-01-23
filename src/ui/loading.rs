use crate::i18n::{get_text, Language};

use egui::{
    Context,Area,Id,Order,Align2,Color32,
    Vec2,Spinner,RichText,UiBuilder,Rect,Ui
};

pub fn global_loading(ctx: &Context, lang: Language) {
    let text = get_text(lang);
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
                    RichText::new(text.loading_parsing)
                        .color(Color32::from_rgba_unmultiplied(
                            200, 200, 200, alpha,
                        ))
                        .strong(),
                );
            });
        });
}

pub fn corner_loading(ui: &mut Ui) {
    let rect = ui.max_rect();

    let size = egui::vec2(24.0, 24.0);
    let pos = egui::pos2(
        rect.right() - size.x - 8.0,
        rect.top() + 8.0,
    );

    let spinner_rect = Rect::from_min_size(pos, size);

    ui.scope_builder(
        UiBuilder::new().max_rect(spinner_rect),
        |ui| {
            ui.add(
                Spinner::new()
                    .size(20.0)
                    .color(Color32::from_gray(160)),
            );
        },
    );
}
