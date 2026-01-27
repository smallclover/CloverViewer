use egui::containers::modal::Modal;
use egui::{Align, Context, Frame, Id, Layout, RichText, Ui};

pub struct ModalFrame;
pub enum ModalAction {
    None,
    Close,
}

/// modal基类
impl ModalFrame {
    pub fn show(
        ctx: &Context,
        open: &mut bool,
        title: &str,
        add_contents: impl FnOnce(&mut Ui) -> ModalAction,
    ) {
        if !*open {
            return;
        }

        // 用title做唯一Id
        Modal::new(Id::new(title)).show(ctx, |ui| {
            Frame::window(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).strong());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(egui::RichText::new("✖")).clicked() {
                            *open = false;
                        }
                    });
                });

                ui.separator();

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    *open = false;
                }

                match add_contents(ui) {
                    ModalAction::Close => *open = false,
                    ModalAction::None => {}
                }
            });
        });
    }

}
