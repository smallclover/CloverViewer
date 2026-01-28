use eframe::emath::Pos2;
use crate::config::Config;

#[derive(PartialEq, Clone, Debug)]
pub enum UiMode {
    Normal,
    About,
    Settings(Config),
    ContextMenu(Pos2),
}
