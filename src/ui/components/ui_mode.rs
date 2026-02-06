use egui::Pos2;
use crate::model::config::Config;

#[derive(Clone, PartialEq)]
pub enum UiMode {
    Normal,
    About,
    Settings(Config),
    ContextMenu(Pos2),
    Properties,
}
