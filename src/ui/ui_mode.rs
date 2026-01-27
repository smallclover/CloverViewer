use eframe::emath::Pos2;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum UiMode {
    Normal,
    About,
    Settings,
    ContextMenu(Pos2),
}