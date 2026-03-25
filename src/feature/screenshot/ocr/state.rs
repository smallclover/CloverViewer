// src/feature/screenshot/ocr/state.rs
use std::sync::mpsc::Receiver;

pub struct OcrState {
    pub is_panel_open: bool,
    pub is_processing: bool,
    pub text: Option<String>,
    pub receiver: Option<Receiver<Result<String, String>>>,
}

impl Default for OcrState {
    fn default() -> Self {
        Self {
            is_panel_open: false,
            is_processing: false,
            text: None,
            receiver: None,
        }
    }
}