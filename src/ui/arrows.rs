use egui::{Rect, Ui};

pub enum Nav {
    Prev,
    Next,
}

pub fn draw_arrows(ui: &mut Ui, rect: Rect) -> Option<Nav> {
    let size = egui::vec2(40.0, 40.0);

    let left = Rect::from_center_size(
        egui::pos2(rect.left() + 30.0, rect.center().y),
        size,
    );
    let right = Rect::from_center_size(
        egui::pos2(rect.right() - 30.0, rect.center().y),
        size,
    );

    if ui.put(left, egui::Button::new("◀")).clicked() {
        return Some(Nav::Prev);
    }
    if ui.put(right, egui::Button::new("▶")).clicked() {
        return Some(Nav::Next);
    }
    None
}
