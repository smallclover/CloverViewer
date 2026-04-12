// src/feature/screenshot/ocr/state.rs
use std::sync::mpsc::Receiver;

#[derive(Default)]
pub struct OcrState {
    pub is_panel_open: bool,
    pub is_processing: bool,
    pub text: Option<String>,
    pub receiver: Option<Receiver<Result<String, String>>>,
}
